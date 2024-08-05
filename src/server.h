#ifndef SIMULO_SERVER_H_
#define SIMULO_SERVER_H_

#include <array>
#include <memory>

#include "config.h"
#include "net/networking.h"
#include "player.h"
#include "util/slab.h"

namespace simulo {

class Server {
public:
   explicit Server();

   void run();

private:
   void tick();

   net::IncomingConnection accepted_connections_[SIMULO_JOIN_QUEUE_CAPACITY];
   using PlayerSlab = Slab<Player, 256>;
   std::unique_ptr<PlayerSlab> players_;
   net::Networking networking_;
};

} // namespace simulo

#endif // !SIMULO_SERVER_H_
