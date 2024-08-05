#ifndef SIMULO_NET_WIN_NETWORKING_H_
#define SIMULO_NET_WIN_NETWORKING_H_

#include <array>
#include <cstdint>
#include <vector>

#include <WinSock2.h>
#include <minwinbase.h>
// clang-format off
#include <MSWSock.h>
// clang-format on

#include "protocol/packets.h"
#include "protocol/types.h"
#include "util/slab.h"

namespace simulo::net {

enum Operation {
   OpReadHandshake,
   OpReadLogin,
   OpWriteHandshake,
};

typedef struct {
   OVERLAPPED overlapped;
   unsigned char operation;
} OverlappedWithOp;

typedef struct {
   SOCKET socket;
   OverlappedWithOp overlapped;
   unsigned char buf[LOGIN_PACKET_SIZE(MAX_USERNAME_LEN)];
   unsigned char buf_used;
   unsigned char target_buf_len;
} Connection;

typedef struct {
   Connection *conn;
   // Will be null-terminated if username length is <16. Otherwise, full buffer is used.
   char username[16];
} IncomingConnection;

class Networking {
public:
   explicit Networking(std::uint16_t port, std::vector<IncomingConnection> &accepted_connections);
   ~Networking();

   void listen();
   void poll();

private:
   // AcceptEx requires length of address to be at least 16 bytes more than its
   // true size
   static constexpr DWORD kAddressLen = sizeof(sockaddr_in) + 16;

   void accept();
   void handle_accept(bool success);
   static void read(Connection &conn);
   void handle_read(bool op_success, int connection_key, DWORD len);
   void handle_read_handshake(int connection_key, Connection &conn);
   void handle_read_login(int connection_key, Connection &conn);
   /// `conn.overlapped_.op` MUST be set to a writing value before calling this
   static void write(Connection &conn, const unsigned char *buf, unsigned int len);
   void handle_write(bool op_success, int connection_key, DWORD len);

   void release_connection(int connection_key);

   using ConnectionSlab = Slab<Connection, 256>;
   ConnectionSlab connections_;
   // Used to resolve AcceptEx dynamically instead of using the one provided by mswsock.lib. See
   // https://stackoverflow.com/a/6800704. Additionally, it slightly reduces memory usage
   LPFN_ACCEPTEX accept_ex_;
   HANDLE root_completion_port_;
   SOCKET listen_socket_;
   SOCKET accepted_socket_;
   unsigned char accept_buf_[kAddressLen * 2]; // *2 to hold the local and remote address
   WSAOVERLAPPED overlapped_;

   std::vector<IncomingConnection> &accepted_connections_;
};

} // namespace simulo::net

#endif // !SIMULO_NET_WIN_NETWORKING_H_
