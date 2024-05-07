#ifndef SIMULO_SERVER_H_
#define SIMULO_SERVER_H_

#include <functional>
#include <vector>

#include "net/networking.h"
namespace simulo {

class Server {
public:
   explicit Server();

   void run();

private:
   void tick();

   std::vector<std::reference_wrapper<net::Connection>> accepted_connections_;
   net::Networking networking_;
};

} // namespace simulo

#endif // !SIMULO_SERVER_H_
