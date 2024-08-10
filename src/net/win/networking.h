#ifndef SIMULO_NET_WIN_NETWORKING_H_
#define SIMULO_NET_WIN_NETWORKING_H_

#include <stdint.h>

#include <WinSock2.h>
#include <minwinbase.h>
// clang-format off
#include <MSWSock.h>
// clang-format on

#include "protocol/packets.h"

enum Operation {
   OpReadHandshake,
   OpReadLogin,
   OpWriteHandshake,
};

typedef struct {
   OVERLAPPED overlapped;
   unsigned char operation;
} OverlappedWithOp;

typedef union {
   struct {
      SOCKET socket;
      OverlappedWithOp overlapped;
      unsigned char buf[LOGIN_PACKET_SIZE(MAX_USERNAME_LEN)];
      unsigned char buf_used;
      unsigned char target_buf_len;
   };
   int next;
} Connection;

typedef struct {
   Connection *conn;
   // Will be null-terminated if username length is <16. Otherwise, full buffer is used.
   char username[16];
} IncomingConnection;

// AcceptEx requires length of address to be at least 16 bytes more than its
// true size
#define SIMULO_NET_ADDRESS_LEN (sizeof(struct sockaddr_in) + 16)

typedef struct {
   Connection connections[256];
   int next_avail_connection;

   // Used to resolve AcceptEx dynamically instead of using the one provided by mswsock.lib. See
   // https://stackoverflow.com/a/6800704. Additionally, it slightly reduces memory usage
   LPFN_ACCEPTEX accept_ex;
   HANDLE root_completion_port;
   SOCKET listen_socket;
   SOCKET accepted_socket;
   unsigned char accept_buf[SIMULO_NET_ADDRESS_LEN * 2]; // *2 to hold the local and remote address
   WSAOVERLAPPED overlapped;

   IncomingConnection *accepted_connections;
   int num_accepted;
} Networking;

bool net_init(Networking *net, uint16_t port, IncomingConnection *accepted_connections);

void net_deinit(Networking *net);

bool net_listen(Networking *net);

int net_poll(Networking *net);

#endif // !SIMULO_NET_WIN_NETWORKING_H_
