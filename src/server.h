#ifndef SIMULO_SERVER_H_
#define SIMULO_SERVER_H_

#include "net/networking.h"

namespace simulo {

class Server {
public:
   Server();

   void run();

private:
   Networking networking_;
};

} // namespace simulo

#endif // !SIMULO_SERVER_H_
