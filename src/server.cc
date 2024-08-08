#include "server.h"

#include <chrono>
#include <cstdint>
#include <memory>
#include <thread>

#include "net/win/networking.h"
#include "player.h"

using namespace simulo;

namespace {}; // namespace

Server::Server() : accepted_connections_() {
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
      int key = players_.alloc_zeroed();
      Player &player = players_.get(key);
      player_init(&player, incoming.conn, incoming.username);
   }
}
