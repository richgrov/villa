#include "networking.h"

#include <array>
#include <iostream>
#include <string>

#include <MSWSock.h>
#include <WinSock2.h>
#include <vector>

#include "config.h"
#include "protocol/packets.h"
#include "util/debug_assert.h"
#include "util/slab.h"

using namespace simulo::net;

namespace {

template <typename T>
std::runtime_error create_func_error(const std::string &func_name, T err_code) {
   auto err_msg = func_name + " failed: " + std::to_string(err_code);
   return std::runtime_error(err_msg);
}

void close_or_log_error(SOCKET socket) {
   if (closesocket(socket) != SOCKET_ERROR) {
      return;
   }

   int err = WSAGetLastError();
   SIMULO_PANIC("Failed to close %llu: %d", socket, err);
}

void load_accept_ex(SOCKET listener, LPFN_ACCEPTEX *fn) {
   GUID accept_ex_guid = WSAID_ACCEPTEX;
   DWORD unused;
   int load_result = WSAIoctl(
      listener, SIO_GET_EXTENSION_FUNCTION_POINTER, &accept_ex_guid, sizeof(accept_ex_guid), fn,
      sizeof(LPFN_ACCEPTEX), &unused, nullptr, nullptr
   );

   if (load_result == SOCKET_ERROR) {
      throw create_func_error("WSAIoctl", WSAGetLastError());
   }
}

constexpr ULONG_PTR kListenerCompletionKey = -1;

} // namespace

Networking::Networking(const std::uint16_t port, IncomingConnection *accepted_connections)
    : accepted_socket_(INVALID_SOCKET), overlapped_{}, accepted_connections_(accepted_connections),
      num_accepted_(0) {

   WSAData wsa_data;
   int startup_res = WSAStartup(MAKEWORD(2, 2), &wsa_data);
   if (startup_res != 0) {
      throw create_func_error("WSAStartup", startup_res);
   }

   root_completion_port_ = CreateIoCompletionPort(INVALID_HANDLE_VALUE, nullptr, 0, 0);
   if (root_completion_port_ == nullptr) {
      throw create_func_error("CreateIOCompletionPort", GetLastError());
   }

   listen_socket_ = socket(AF_INET, SOCK_STREAM, 0);
   if (listen_socket_ == INVALID_SOCKET) {
      throw create_func_error("socket", WSAGetLastError());
   }

   load_accept_ex(listen_socket_, &accept_ex_);

   SOCKADDR_IN bind_addr;
   bind_addr.sin_family = AF_INET;
   bind_addr.sin_addr.s_addr = INADDR_ANY;
   bind_addr.sin_port = htons(port);
   if (bind(listen_socket_, reinterpret_cast<sockaddr *>(&bind_addr), sizeof(bind_addr)) ==
       SOCKET_ERROR) {
      throw create_func_error("bind", WSAGetLastError());
   }
}

Networking::~Networking() {
   closesocket(listen_socket_);
}

void Networking::listen() {
   if (::listen(listen_socket_, 16) == SOCKET_ERROR) {
      throw create_func_error("listen", WSAGetLastError());
   }

   HANDLE listen_port = CreateIoCompletionPort(
      reinterpret_cast<HANDLE>(listen_socket_), root_completion_port_, kListenerCompletionKey, 0
   );

   if (listen_port == nullptr) {
      throw create_func_error("CreateIOCompletionPort", GetLastError());
   }

   accept();
}

int Networking::poll() {
   num_accepted_ = 0;

   DWORD len;
   ULONG_PTR completion_key;
   WSAOVERLAPPED *overlapped;

   while (true) {
      BOOL op_success =
         GetQueuedCompletionStatus(root_completion_port_, &len, &completion_key, &overlapped, 0);

      bool no_more_completions = overlapped == nullptr;
      if (no_more_completions) {
         break;
      }

      bool accepted_new_connection = completion_key == kListenerCompletionKey;
      if (accepted_new_connection) {
         handle_accept(op_success);
      } else {
         auto *with_op = reinterpret_cast<OverlappedWithOp *>(overlapped);
         int conn_key = static_cast<int>(completion_key);

         switch (with_op->operation) {
         case OpReadHandshake:
         case OpReadLogin:
            handle_read(op_success, conn_key, len);
            break;

         case OpWriteHandshake:
            handle_write(op_success, conn_key, len);
            break;

         default:
            SIMULO_PANIC("op = %d", static_cast<int>(with_op->operation));
         }
      }
   }

   return num_accepted_;
}

void Networking::accept() {
   accepted_socket_ = socket(AF_INET, SOCK_STREAM, 0);
   BOOL success = accept_ex_(
      listen_socket_, accepted_socket_, accept_buf_, 0, kAddressLen, kAddressLen, nullptr,
      &overlapped_
   );

   if (!success) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "Abnormal error from AcceptEx: %d", err);
   }
}

void Networking::handle_accept(const bool success) {
   if (!success) {
      SIMULO_DEBUG_LOG("Failed to accept %llu: %lu", accepted_socket_, GetLastError());
      close_or_log_error(accepted_socket_);
      return;
   }

   int key = connections_.emplace();
   if (key == kInvalidSlabKey) {
      SIMULO_DEBUG_LOG("Out of connection objects for %llu", accepted_socket_);
      close_or_log_error(accepted_socket_);
      return;
   }

   Connection &conn = connections_.get(key);
   memset(&conn, 0, sizeof(Connection));
   conn.socket = accepted_socket_;
   conn.target_buf_len = 1;
   accepted_socket_ = INVALID_SOCKET;

   HANDLE client_completion_port = CreateIoCompletionPort(
      reinterpret_cast<HANDLE>(conn.socket), root_completion_port_, static_cast<ULONG_PTR>(key), 0
   );

   if (client_completion_port == nullptr) {
      SIMULO_DEBUG_LOG(
         "Failed to create completion port for %llu: %lu", conn.socket, GetLastError()
      );

      release_connection(key);
      return;
   }

   read(conn);
   accept();
}

void Networking::read(Connection &conn) {
   WSABUF buf;
   buf.buf = reinterpret_cast<CHAR *>(&conn.buf[conn.buf_used]);
   buf.len = sizeof(conn.buf) - conn.buf_used;

   DWORD flags = 0;
   int result =
      WSARecv(conn.socket, &buf, 1, nullptr, &flags, &conn.overlapped.overlapped, nullptr);

   if (result == SOCKET_ERROR) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "err = %d", err);
   }
}

void Networking::handle_read(const bool op_success, const int connection_key, const DWORD len) {
   Connection &conn = connections_.get(connection_key);

   if (!op_success) {
      SIMULO_DEBUG_LOG("Read failed for %lld: %lu", conn.socket, GetLastError());
      release_connection(connection_key);
   }

   if (len < 1) {
      SIMULO_DEBUG_LOG("EOF from %lld", conn.socket);
      release_connection(connection_key);
      return;
   }

   SIMULO_DEBUG_ASSERT(
      len + static_cast<DWORD>(conn.buf_used) <= sizeof(conn.buf), "conn=%d, len=%lu, used=%d",
      connection_key, len, conn.buf_used
   );

   conn.buf_used += len;
   if (conn.buf_used < conn.target_buf_len) {
      read(conn);
      return;
   }

   switch (conn.overlapped.operation) {
   case OpReadHandshake:
      handle_read_handshake(connection_key, conn);
      break;

   case OpReadLogin:
      handle_read_login(connection_key, conn);
      break;

   default:
      SIMULO_PANIC("invalid op %d", static_cast<int>(conn.overlapped.operation));
   }
}

void Networking::handle_read_handshake(int connection_key, Connection &conn) {
   Handshake handshake{};
   int min_remaining_bytes = remaining_handshake_bytes(conn.buf, conn.buf_used, &handshake);
   switch (min_remaining_bytes) {
   case -1:
      SIMULO_DEBUG_LOG("Couldn't read handshake from %llu", conn.socket);
      release_connection(connection_key);
      break;

   case 0:
      SIMULO_DEBUG_ASSERT(
         handshake.username_len > 0 && handshake.username_len <= 16, "username len = %d",
         handshake.username_len
      );
      conn.target_buf_len = LOGIN_PACKET_SIZE(handshake.username_len);

      conn.overlapped.operation = OpWriteHandshake;
      write(conn, OFFLINE_MODE_RESPONSE, sizeof(OFFLINE_MODE_RESPONSE));
      break;

   default:
      SIMULO_DEBUG_ASSERT(
         min_remaining_bytes > 0 && min_remaining_bytes <= sizeof(conn.buf), "remaining = %d",
         min_remaining_bytes
      );

      conn.target_buf_len += static_cast<unsigned int>(min_remaining_bytes);
      SIMULO_DEBUG_ASSERT(
         conn.target_buf_len <= sizeof(conn.buf), "target=%d", conn.target_buf_len
      );

      read(conn);
      break;
   }
}

void Networking::handle_read_login(int connection_key, Connection &conn) {
   Login login_packet{};
   bool ok = read_login_pkt(conn.buf, conn.buf_used, &login_packet);
   if (!ok) {
      SIMULO_DEBUG_LOG("Couldn't read login from %llu", conn.socket);
      release_connection(connection_key);
      return;
   }

   if (login_packet.protocol_version != BETA173_PROTOCOL_VER) {
      SIMULO_DEBUG_LOG(
         "Invalid protocol version from %llu: %d", conn.socket, login_packet.protocol_version
      );
      release_connection(connection_key);
      return;
   }

   if (num_accepted_ >= SIMULO_JOIN_QUEUE_CAPACITY) {
      SIMULO_DEBUG_LOG("Couldn't accept %llu because join queue is full", conn.socket);
      release_connection(connection_key);
      return;
   }

   std::array<char, 16> username;
   for (int i = 0; i < login_packet.username_len; ++i) {
      username[i] = static_cast<char>(login_packet.username[i]);
   }

   if (login_packet.username_len < 16) {
      username[login_packet.username_len] = '\0';
   }

   IncomingConnection *inc = &accepted_connections_[num_accepted_++];
   inc->conn = &conn;
   memcpy(inc->username, username.data(), username.size());
}

void Networking::write(Connection &conn, const unsigned char *data, const unsigned int len) {
   SIMULO_DEBUG_ASSERT(
      conn.overlapped.operation == OpWriteHandshake, "expected writing op but got %d",
      static_cast<int>(conn.overlapped.operation)
   );

   WSABUF buf;
   // Buffer is read-only- safe to const_cast
   buf.buf = const_cast<CHAR *>(reinterpret_cast<const CHAR *>(data));
   buf.len = len;

   conn.buf_used = len;

   int result = WSASend(conn.socket, &buf, 1, nullptr, 0, &conn.overlapped.overlapped, nullptr);
   if (result == SOCKET_ERROR) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "err = %d", err);
   }
}

void Networking::handle_write(const bool op_success, const int connection_key, const DWORD len) {
   Connection &conn = connections_.get(connection_key);

   if (!op_success) {
      SIMULO_DEBUG_LOG("Write failed for %llu: %lu", conn.socket, GetLastError());
      release_connection(connection_key);
   }

   // Although not official, WSASend has never been observed to partially complete unless the socket
   // loses connection. Keep things simple by asserting that the operation should fully complete.
   if (len < conn.buf_used) {
      SIMULO_DEBUG_LOG(
         "Only wrote %lu bytes to %llu instead of %d", len, conn.socket, conn.buf_used
      );
      release_connection(connection_key);
      return;
   }

   conn.overlapped.operation = OpReadLogin;
   conn.buf_used = 0;
   read(conn);
}

void Networking::release_connection(int connection_key) {
   Connection &conn = connections_.get(connection_key);
   if (conn.socket != INVALID_SOCKET) {
      close_or_log_error(conn.socket);
   }
   connections_.release(connection_key);
}
