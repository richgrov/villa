#ifndef SIMULO_NET_WIN_NETWORKING_H_
#define SIMULO_NET_WIN_NETWORKING_H_

#include <cstdint>
#include <memory>

#include <WinSock2.h>
#include <minwinbase.h>
// clang-format off
#include <MSWSock.h>
// clang-format on

#include "protocol/packets.h"
#include "util/slab.h"

namespace simulo::net {

enum class Operation : unsigned char {
   kRead,
   kWrite,
};

struct OverlappedWithOp : OVERLAPPED {
   Operation op;
};

enum class LoginReadStage : unsigned char {
   kHandshake,
   kLogin,
};

struct Connection {
   SOCKET socket;
   OverlappedWithOp overlapped;
   LoginReadStage read_stage;
   packet::Handshake handshake_packet;
   unsigned char buf[packet::Login::kMaxSize + 1]; // +1 for packet id
   unsigned char buf_used;
   unsigned char target_buf_len;

   explicit Connection(SOCKET socket);
   ~Connection();

   void prep_read();
};

class Networking {
public:
   explicit Networking(std::uint16_t port);
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
   static void write(Connection &conn, const unsigned char *buf, unsigned int len);
   void handle_write(bool op_success, int connection_key, DWORD len) const;

   using ConnectionSlab = Slab<Connection, 256>;
   std::unique_ptr<ConnectionSlab> connections_;
   // Used to resolve AcceptEx dynamically instead of using the one provided by mswsock.lib. See
   // https://stackoverflow.com/a/6800704. Additionally, it slightly reduces memory usage
   LPFN_ACCEPTEX accept_ex_;
   HANDLE root_completion_port_;
   SOCKET listen_socket_;
   SOCKET accepted_socket_;
   unsigned char accept_buf_[kAddressLen * 2]; // *2 to hold the local and remote address
   WSAOVERLAPPED overlapped_;
};

} // namespace simulo::net

#endif // !SIMULO_NET_WIN_NETWORKING_H_
