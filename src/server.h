#ifndef SIMULO_SERVER_H_
#define SIMULO_SERVER_H_

#include "config.h"
#include "net/networking.h"
#include "net/win/networking.h"
#include "player.h"
#include "util/slab.h"

namespace simulo {

class Server {
public:
   explicit Server();

   inline ~Server() {
      net_deinit(&networking_);
   }

   void run();

private:
   void tick();

   IncomingConnection accepted_connections_[SIMULO_JOIN_QUEUE_CAPACITY];
   Player players_[256];
   int next_avail_player_;
   Networking networking_;
};

} // namespace simulo

#endif // !SIMULO_SERVER_H_
