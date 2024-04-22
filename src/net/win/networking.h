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

namespace simulo {

class Networking {
public:
   explicit Networking(std::uint16_t port);

   void listen();
   void poll();

private:
   // AcceptEx requires length of address to be at least 16 bytes more than its
   // true size
   static constexpr DWORD kAddressLen = sizeof(sockaddr_in) + 16;

   enum Operation {
      kRead,
      kWrite,
   };

   struct OverlappedWithOp : OVERLAPPED {
      Operation op;
   };

   enum LoginReadStage {
      kHandshake,
   };

   struct Connection {
      SOCKET socket;
      OverlappedWithOp overlapped;
      LoginReadStage read_stage;
      int packet_read_state;
      union {
         packet::Handshake handshake_packet;
         packet::Login login_packet;
      };
      unsigned char buf[packet::Login::kMaxSize + 1]; // +1 for packet id
      union {
         unsigned int used;
         unsigned int expected_write_amount;
      };
      unsigned int target_buf_len;

      Connection(SOCKET socket);
      ~Connection();
   };

   void accept();
   void handle_accept(bool success);
   static void read(Connection &conn);
   void handle_read(bool op_success, int connection_key, DWORD len);
   static void write(Connection &conn, const unsigned char *buf, unsigned int len);
   void handle_write(bool op_success, int connection_key, DWORD len) const;

   using ConnectionSlab = Slab<Connection, 256>;
   std::unique_ptr<Slab<Connection, 256>> connections_;
   LPFN_ACCEPTEX accept_ex_;
   HANDLE root_completion_port_;
   SOCKET listen_socket_;
   SOCKET accepted_socket_;
   unsigned char accept_buf_[kAddressLen * 2];
   WSAOVERLAPPED overlapped_;
};

} // namespace simulo

#endif // !SIMULO_NET_WIN_NETWORKING_H_
