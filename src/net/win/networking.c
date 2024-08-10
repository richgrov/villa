#include "networking.h"

#include <stddef.h>

#include <MSWSock.h>
#include <WinSock2.h>

#include "config.h"
#include "protocol/packets.h"
#include "util/arrays.h"
#include "util/debug_assert.h"
#include "util/slab.h"

static void close_or_log_error(SOCKET socket) {
   if (closesocket(socket) != SOCKET_ERROR) {
      return;
   }

   int err = WSAGetLastError();
   SIMULO_PANIC("Failed to close %llu: %d", socket, err);
}

static void load_accept_ex(SOCKET listener, LPFN_ACCEPTEX *fn) {
   GUID accept_ex_guid = WSAID_ACCEPTEX;
   DWORD unused;
   int load_result = WSAIoctl(
      listener, SIO_GET_EXTENSION_FUNCTION_POINTER, &accept_ex_guid, sizeof(accept_ex_guid), fn,
      sizeof(LPFN_ACCEPTEX), &unused, NULL, NULL
   );

   if (load_result == SOCKET_ERROR) {
      SIMULO_PANIC("fatal: couldn't load AcceptEx: WSAIoctl returned %d", WSAGetLastError());
   }
}

#define LISTENER_COMPLETION_KEY -1

bool net_init(Networking *net, const uint16_t port, IncomingConnection *accepted_connections) {
   slab_init(net->connections_, ARRAY_LEN(net->connections_), sizeof(Connection));

   memset(net, 0, sizeof(Networking));
   net->accepted_socket_ = INVALID_SOCKET;
   net->accepted_connections_ = accepted_connections;

   struct WSAData wsa_data;
   int startup_res = WSAStartup(MAKEWORD(2, 2), &wsa_data);
   if (startup_res != 0) {
      fprintf(stderr, "error: WSAStartup returned %d", startup_res);
      return false;
   }

   net->root_completion_port_ = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0);
   if (net->root_completion_port_ == NULL) {
      fprintf(stderr, "error: CreateIOCompletionPort returned %lu", GetLastError());
      return false;
   }

   net->listen_socket_ = socket(AF_INET, SOCK_STREAM, 0);
   if (net->listen_socket_ == INVALID_SOCKET) {
      fprintf(stderr, "error: socket returned %d", WSAGetLastError());
      return false;
   }

   load_accept_ex(net->listen_socket_, &net->accept_ex_);

   SOCKADDR_IN bind_addr;
   bind_addr.sin_family = AF_INET;
   bind_addr.sin_addr.s_addr = INADDR_ANY;
   bind_addr.sin_port = htons(port);
   if (bind(net->listen_socket_, (struct sockaddr *)&bind_addr, sizeof(bind_addr)) ==
       SOCKET_ERROR) {
      fprintf(stderr, "error: bind returned %d", WSAGetLastError());
      return false;
   }

   return true;
}

static void release_connection(Networking *net, int connection_key) {
   SIMULO_DEBUG_ASSERT(
      connection_key >= 0 && connection_key < ARRAY_LEN(net->connections_),
      "tried to release connection %d", connection_key
   );

   Connection *conn = &net->connections_[connection_key];
   if (conn->socket != INVALID_SOCKET) {
      close_or_log_error(conn->socket);
   }

   slab_reclaim(
      net->connections_, sizeof(Connection), connection_key, &net->next_avail_connection_
   );
}

void net_deinit(Networking *net) {
   bool unallocated_connections[ARRAY_LEN(net->connections_)];
   memset(unallocated_connections, false, sizeof(unallocated_connections));

   int next = net->next_avail_connection_;
   while (next != SIMULO_INVALID_SLAB_KEY) {
      unallocated_connections[next] = true;
      next = slab_get_next_id(&net->connections_[next]);
   }

   for (int i = 0; i < ARRAY_LEN(net->connections_); ++i) {
      if (!unallocated_connections[i]) {
         release_connection(net, i);
      }
   }

   closesocket(net->listen_socket_);
}

static void net_accept(Networking *net) {
   net->accepted_socket_ = socket(AF_INET, SOCK_STREAM, 0);
   BOOL success = net->accept_ex_(
      net->listen_socket_, net->accepted_socket_, net->accept_buf_, 0, SIMULO_NET_ADDRESS_LEN,
      SIMULO_NET_ADDRESS_LEN, NULL, &net->overlapped_
   );

   if (!success) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "Abnormal error from AcceptEx: %d", err);
   }
}

bool net_listen(Networking *net) {
   if (listen(net->listen_socket_, 16) == SOCKET_ERROR) {
      fprintf(stderr, "error: listen returned %d", WSAGetLastError());
      return false;
   }

   HANDLE listen_port = CreateIoCompletionPort(
      (HANDLE)net->listen_socket_, net->root_completion_port_, LISTENER_COMPLETION_KEY, 0
   );

   if (listen_port == NULL) {
      fprintf(stderr, "error: CreateIOCompletionPort returned %lu", GetLastError());
      return false;
   }

   net_accept(net);
   return true;
}

static void net_read(Networking *net, Connection *conn) {
   WSABUF buf;
   buf.buf = (CHAR *)&conn->buf[conn->buf_used];
   buf.len = sizeof(conn->buf) - conn->buf_used;

   DWORD flags = 0;
   int result = WSARecv(conn->socket, &buf, 1, NULL, &flags, &conn->overlapped.overlapped, NULL);

   if (result == SOCKET_ERROR) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "err = %d", err);
   }
}

// `conn.overlapped_.op` MUST be set to a writing value before calling this
static void
net_write(Networking *net, Connection *conn, const unsigned char *data, const unsigned int len) {
   SIMULO_DEBUG_ASSERT(
      conn->overlapped.operation == OpWriteHandshake, "expected writing op but got %d",
      (int)conn->overlapped.operation
   );

   WSABUF buf;
   // Buffer is read-only- safe to cast away const
   buf.buf = (CHAR *)data;
   buf.len = len;

   conn->buf_used = len;

   int result = WSASend(conn->socket, &buf, 1, NULL, 0, &conn->overlapped.overlapped, NULL);
   if (result == SOCKET_ERROR) {
      int err = WSAGetLastError();
      SIMULO_DEBUG_ASSERT(err == ERROR_IO_PENDING, "err = %d", err);
   }
}

static void handle_accept(Networking *net, const bool success) {
   if (!success) {
      SIMULO_DEBUG_LOG("Failed to accept %llu: %lu", net->accepted_socket_, GetLastError());
      close_or_log_error(net->accepted_socket_);
      return;
   }

   if (net->next_avail_connection_ == SIMULO_INVALID_SLAB_KEY) {
      SIMULO_DEBUG_LOG("Out of connection objects for %llu", net->accepted_socket_);
      close_or_log_error(net->accepted_socket_);
      return;
   }

   int key = net->next_avail_connection_;
   Connection *conn = &net->connections_[key];
   net->next_avail_connection_ = slab_get_next_id(&conn);

   memset(conn, 0, sizeof(Connection));
   conn->socket = net->accepted_socket_;
   conn->target_buf_len = 1;
   net->accepted_socket_ = INVALID_SOCKET;

   HANDLE client_completion_port =
      CreateIoCompletionPort((HANDLE)conn->socket, net->root_completion_port_, (ULONG_PTR)key, 0);

   if (client_completion_port == NULL) {
      SIMULO_DEBUG_LOG(
         "Failed to create completion port for %llu: %lu", conn->socket, GetLastError()
      );

      release_connection(net, key);
      return;
   }

   net_read(net, conn);
   net_accept(net);
}

static void handle_read_handshake(Networking *net, int connection_key, Connection *conn) {
   Handshake handshake = {};
   int min_remaining_bytes = remaining_handshake_bytes(conn->buf, conn->buf_used, &handshake);
   switch (min_remaining_bytes) {
   case -1:
      SIMULO_DEBUG_LOG("Couldn't read handshake from %llu", conn->socket);
      release_connection(net, connection_key);
      break;

   case 0:
      SIMULO_DEBUG_ASSERT(
         handshake.username_len > 0 && handshake.username_len <= 16, "username len = %d",
         handshake.username_len
      );
      conn->target_buf_len = LOGIN_PACKET_SIZE(handshake.username_len);

      conn->overlapped.operation = OpWriteHandshake;
      net_write(net, conn, OFFLINE_MODE_RESPONSE, sizeof(OFFLINE_MODE_RESPONSE));
      break;

   default:
      SIMULO_DEBUG_ASSERT(
         min_remaining_bytes > 0 && min_remaining_bytes <= sizeof(conn->buf), "remaining = %d",
         min_remaining_bytes
      );

      conn->target_buf_len += (unsigned int)min_remaining_bytes;
      SIMULO_DEBUG_ASSERT(
         conn->target_buf_len <= sizeof(conn->buf), "target=%d", conn->target_buf_len
      );

      net_read(net, conn);
      break;
   }
}

static void handle_read_login(Networking *net, int connection_key, Connection *conn) {
   Login login_packet = {};
   bool ok = read_login_pkt(conn->buf, conn->buf_used, &login_packet);
   if (!ok) {
      SIMULO_DEBUG_LOG("Couldn't read login from %llu", conn->socket);
      release_connection(net, connection_key);
      return;
   }

   if (login_packet.protocol_version != BETA173_PROTOCOL_VER) {
      SIMULO_DEBUG_LOG(
         "Invalid protocol version from %llu: %d", conn->socket, login_packet.protocol_version
      );
      release_connection(net, connection_key);
      return;
   }

   if (net->num_accepted_ >= SIMULO_JOIN_QUEUE_CAPACITY) {
      SIMULO_DEBUG_LOG("Couldn't accept %llu because join queue is full", conn->socket);
      release_connection(net, connection_key);
      return;
   }

   char username[16];
   for (int i = 0; i < login_packet.username_len; ++i) {
      username[i] = (char)login_packet.username[i];
   }

   if (login_packet.username_len < 16) {
      username[login_packet.username_len] = '\0';
   }

   IncomingConnection *inc = &net->accepted_connections_[net->num_accepted_++];
   inc->conn = conn;
   memcpy(inc->username, username, sizeof(username));
}

static void
handle_read(Networking *net, const bool op_success, const int connection_key, const DWORD len) {
   Connection *conn = &net->connections_[connection_key];

   if (!op_success) {
      SIMULO_DEBUG_LOG("Read failed for %lld: %lu", conn->socket, GetLastError());
      release_connection(net, connection_key);
   }

   if (len < 1) {
      SIMULO_DEBUG_LOG("EOF from %lld", conn->socket);
      release_connection(net, connection_key);
      return;
   }

   SIMULO_DEBUG_ASSERT(
      len + (DWORD)conn->buf_used <= sizeof(conn->buf), "conn=%d, len=%lu, used=%d", connection_key,
      len, conn->buf_used
   );

   conn->buf_used += len;
   if (conn->buf_used < conn->target_buf_len) {
      net_read(net, conn);
      return;
   }

   switch (conn->overlapped.operation) {
   case OpReadHandshake:
      handle_read_handshake(net, connection_key, conn);
      break;

   case OpReadLogin:
      handle_read_login(net, connection_key, conn);
      break;

   default:
      SIMULO_PANIC("invalid op %d", (int)conn->overlapped.operation);
   }
}

static void
handle_write(Networking *net, const bool op_success, const int connection_key, const DWORD len) {
   Connection *conn = &net->connections_[connection_key];

   if (!op_success) {
      SIMULO_DEBUG_LOG("Write failed for %llu: %lu", conn->socket, GetLastError());
      release_connection(net, connection_key);
   }

   // Although not official, WSASend has never been observed to partially complete unless the socket
   // loses connection. Keep things simple by asserting that the operation should fully complete.
   if (len < conn->buf_used) {
      SIMULO_DEBUG_LOG(
         "Only wrote %lu bytes to %llu instead of %d", len, conn->socket, conn->buf_used
      );
      release_connection(net, connection_key);
      return;
   }

   conn->overlapped.operation = OpReadLogin;
   conn->buf_used = 0;
   net_read(net, conn);
}

int net_poll(Networking *net) {
   net->num_accepted_ = 0;

   DWORD len;
   ULONG_PTR completion_key;
   WSAOVERLAPPED *overlapped;

   while (true) {
      BOOL op_success = GetQueuedCompletionStatus(
         net->root_completion_port_, &len, &completion_key, &overlapped, 0
      );

      bool no_more_completions = overlapped == NULL;
      if (no_more_completions) {
         break;
      }

      bool accepted_new_connection = completion_key == LISTENER_COMPLETION_KEY;
      if (accepted_new_connection) {
         handle_accept(net, op_success);
      } else {
         OverlappedWithOp *with_op = (OverlappedWithOp *)overlapped;
         int conn_key = (int)completion_key;

         switch (with_op->operation) {
         case OpReadHandshake:
         case OpReadLogin:
            handle_read(net, op_success, conn_key, len);
            break;

         case OpWriteHandshake:
            handle_write(net, op_success, conn_key, len);
            break;

         default:
            SIMULO_PANIC("op = %d", (int)with_op->operation);
         }
      }
   }

   return net->num_accepted_;
}
