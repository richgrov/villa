#ifndef SIMULO_NET_WIN_NETWORKING_H_
#define SIMULO_NET_WIN_NETWORKING_H_

#include <WinSock2.h>
#include <cstdint>

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

   void accept();

   HANDLE completion_port_;
   SOCKET listen_socket_;
   SOCKET accepted_socket_;
   unsigned char accept_buf_[kAddressLen * 2];
   WSAOVERLAPPED overlapped_;
};

} // namespace simulo

#endif // !SIMULO_NET_WIN_NETWORKING_H_