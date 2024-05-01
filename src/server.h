#ifndef SIMULO_SERVER_H_
#define SIMULO_SERVER_H_

#include "net/networking.h"
#include "net/win/networking.h"

#include <vector>

namespace simulo {

class Server {
public:
   explicit Server();

   void run();

private:
   void tick();

   std::vector<net::Connection> accepted_connections_;
   net::Networking networking_;
};

} // namespace simulo

#endif // !SIMULO_SERVER_H_
