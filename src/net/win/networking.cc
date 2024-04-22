#include "networking.h"

#include <array>
#include <iostream>
#include <string>

#include <MSWSock.h>
#include <WinSock2.h>

#include "protocol/packets.h"
#include "util/debug_assert.h"
#include "util/slab.h"

using namespace simulo;

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

#ifdef NDEBUG
   std::cerr << "Failed to close socket " << socket << ": " << WSAGetLastError() << "\n";
#else
   SIMULO_PANIC("Failed to close socket {}: {}", socket, WSAGetLastError());
#endif
}

void load_accept_ex(SOCKET listener, LPFN_ACCEPTEX *fn) {
   GUID accept_ex_guid = WSAID_ACCEPTEX;
   DWORD unused;
   int load_result =
       WSAIoctl(listener, SIO_GET_EXTENSION_FUNCTION_POINTER, &accept_ex_guid,
                sizeof(accept_ex_guid), fn, sizeof(LPFN_ACCEPTEX), &unused, nullptr, nullptr);

   if (load_result == SOCKET_ERROR) {
      throw create_func_error("WSAIoctl", WSAGetLastError());
   }
}

constexpr ULONG_PTR kListenerCompletionKey = -1;

} // namespace

Networking::Connection::Connection(const SOCKET s)
    : socket(s), overlapped{}, read_stage(kHandshake), packet_read_progress(0), handshake_packet(),
      used(0), target_buf_len(1) {}

Networking::Connection::~Connection() {
   close_or_log_error(socket);
}

void Networking::Connection::prep_read() {
   overlapped.op = kRead;
   packet_read_progress = 0;
   used = 0;
   target_buf_len = 1;
}

Networking::Networking(const std::uint16_t port)
    : connections_(std::make_unique<ConnectionSlab>()), accepted_socket_(INVALID_SOCKET),
      overlapped_{} {

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

void Networking::listen() {
   if (::listen(listen_socket_, 16) == SOCKET_ERROR) {
      throw create_func_error("listen", WSAGetLastError());
   }

   HANDLE listen_port = CreateIoCompletionPort(reinterpret_cast<HANDLE>(listen_socket_),
                                               root_completion_port_, kListenerCompletionKey, 0);

   if (listen_port == nullptr) {
      throw create_func_error("CreateIOCompletionPort", GetLastError());
   }

   accept();
}

void Networking::poll() {
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

         switch (with_op->op) {
         case kRead:
            handle_read(op_success, conn_key, len);
            break;

         case kWrite:
            handle_write(op_success, conn_key, len);
            break;

         default:
            SIMULO_PANIC("op = {}", enum_ordinal(with_op->op));
         }
      }
   }
}

void Networking::accept() {
   accepted_socket_ = socket(AF_INET, SOCK_STREAM, 0);
   BOOL success = accept_ex_(listen_socket_, accepted_socket_, accept_buf_, 0, kAddressLen,
                             kAddressLen, nullptr, &overlapped_);

   if (!success) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "Abnormal error from AcceptEx: {}", err);
   }
}

void Networking::handle_accept(const bool success) {
   if (!success) {
      std::cerr << "Failed to accept " << accepted_socket_ << ": " << GetLastError() << "\n";
      close_or_log_error(accepted_socket_);
      return;
   }

   int key;
   {
      key = connections_->emplace(accepted_socket_);
      if (key == kInvalidSlabKey) {
         std::cerr << "Out of connection objects for " << accepted_socket_ << "\n";
         close_or_log_error(accepted_socket_);
         return;
      }

      accepted_socket_ = INVALID_SOCKET;
   }

   Connection &conn = connections_->get(key);

   HANDLE client_completion_port =
       CreateIoCompletionPort(reinterpret_cast<HANDLE>(conn.socket), root_completion_port_,
                              static_cast<ULONG_PTR>(key), 0);

   if (client_completion_port == nullptr) {
      std::cerr << "Failed to create completion port for " << conn.socket << ": " << GetLastError()
                << "\n";

      connections_->release(key);
      return;
   }

   read(conn);
   accept();
}

void Networking::read(Connection &conn) {
   WSABUF buf;
   buf.buf = reinterpret_cast<CHAR *>(&conn.buf[conn.used]);
   buf.len = sizeof(conn.buf) - conn.used;

   DWORD flags = 0;
   int result = WSARecv(conn.socket, &buf, 1, nullptr, &flags, &conn.overlapped, nullptr);

   if (result == SOCKET_ERROR) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "err = {}", err);
   }
}

void Networking::handle_read(const bool op_success, const int connection_key, const DWORD len) {
   Connection &conn = connections_->get(connection_key);

   if (!op_success) {
      std::cerr << "Read failed for " << conn.socket << ": " << GetLastError() << "\n";
      connections_->release(connection_key);
   }

   if (len < 1) {
      std::cerr << "EOF for " << conn.socket << "\n";
      connections_->release(connection_key);
      return;
   }

   conn.used += len;
   if (conn.used < conn.target_buf_len) {
      read(conn);
      return;
   }

   switch (conn.read_stage) {
   case kHandshake:
      handle_read_handshake(connection_key, conn);
      break;

   case kLogin:
      handle_read_login(connection_key, conn);
      break;
   }
}

void Networking::handle_read_handshake(int connection_key, Connection &conn) {
   ReadResult result = conn.handshake_packet.read(conn.buf, conn.used, conn.read_stage);
   switch (result.min_remaining_bytes) {
   case -1:
      std::cerr << "Failed to read handshake for " << conn.socket << "\n";
      connections_->release(connection_key);
      break;

   case 0:
      write(conn, packet::Handshake::kOfflineModeResponse,
            sizeof(packet::Handshake::kOfflineModeResponse));
      break;

   default:
      SIMULO_DEBUG_ASSERT(result.min_remaining_bytes > 0, "remaining = {}, stage = {}",
                          result.min_remaining_bytes, result.progress);
      conn.target_buf_len = static_cast<unsigned int>(result.min_remaining_bytes);
      conn.packet_read_progress = result.progress;
      read(conn);
      break;
   }
}

void Networking::handle_read_login(int connection_key, Connection &conn) {
   ReadResult result = conn.login_packet.read(conn.buf, conn.used, conn.read_stage);
   switch (result.min_remaining_bytes) {
   case -1:
      std::cerr << "Failed to read login for " << conn.socket << "\n";
      connections_->release(connection_key);
      break;

   case 0:
      std::cout << "UL: " << conn.login_packet.username_len
                << ", PV: " << conn.login_packet.protocol_version << "\n";
      connections_->release(connection_key);
      break;

   default:
      SIMULO_DEBUG_ASSERT(result.min_remaining_bytes > 0, "remaining = {}, stage = {}",
                          result.min_remaining_bytes, result.progress);

      conn.target_buf_len = static_cast<unsigned int>(result.min_remaining_bytes);
      conn.packet_read_progress = result.progress;
      read(conn);
      break;
   }
}

void Networking::write(Connection &conn, const unsigned char *data, const unsigned int len) {
   WSABUF buf;
   // Buffer is read-only- safe to const_cast
   buf.buf = const_cast<CHAR *>(reinterpret_cast<const CHAR *>(data));
   buf.len = len;

   conn.overlapped.op = kWrite;
   conn.expected_write_amount = len;

   int result = WSASend(conn.socket, &buf, 1, nullptr, 0, &conn.overlapped, nullptr);
   if (result == SOCKET_ERROR) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "err = {}", err);
   }
}

void Networking::handle_write(const bool op_success, const int connection_key,
                              const DWORD len) const {
   Connection &conn = connections_->get(connection_key);

   if (!op_success) {
      std::cerr << "Write failed for " << conn.socket << ": " << GetLastError() << "\n";
      connections_->release(connection_key);
   }

   // Although not official, WSASend has never been observed to partially complete unless the socket
   // loses connection. Keep things simple by asserting that the operation should fully complete.
   if (len < conn.expected_write_amount) {
      std::cerr << "Only wrote " << len << " bytes to " << conn.socket << " instead of "
                << conn.expected_write_amount << "\n";
      connections_->release(connection_key);
      return;
   }

   std::cout << "Username is " << conn.handshake_packet.username_len << " characters long.\n";

   conn.prep_read();
   conn.read_stage = kLogin;
   conn.login_packet = {};
   conn.target_buf_len = packet::Login::kMinSize + 1; // +1 for packet id
   read(conn);
}
