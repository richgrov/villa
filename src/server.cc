#include "server.h"

#include <chrono>
#include <thread>

#include "net/win/networking.h"
#include "player.h"
#include "util/arrays.h"

using namespace simulo;

namespace {}; // namespace

Server::Server() : accepted_connections_() {
   slab_init(players_, ARRAY_LEN(players_), sizeof(Player));
   net_init(&networking_, 25565, accepted_connections_);
}

void Server::run() {
   net_listen(&networking_);

   while (true) {
      tick();
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
   }
}

void Server::tick() {
   int num_accepted = net_poll(&networking_);

   for (int i = 0; i < num_accepted; ++i) {
      IncomingConnection &incoming = accepted_connections_[i];

      if (next_avail_player_ == SIMULO_INVALID_SLAB_KEY) {
         break; // todo
      }

      int key = next_avail_player_;
      Player &player = players_[key];
      next_avail_player_ = slab_get_next_id(&player);

      player_init(&player, incoming.conn, incoming.username);
   }
}
