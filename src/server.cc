#include "server.h"

#include <chrono>
#include <cstdint>
#include <memory>
#include <thread>

#include "net/win/networking.h"

using namespace simulo;

namespace {}; // namespace

Server::Server()
    : accepted_connections_(), players_(std::make_unique<PlayerSlab>()),
      networking_(25565, accepted_connections_) {}

void Server::run() {
   networking_.listen();

   while (true) {
      tick();
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
   }
}

void Server::tick() {
   int num_accepted = networking_.poll();

   for (int i = 0; i < num_accepted; ++i) {
      net::IncomingConnection &incoming = accepted_connections_[i];
      players_->emplace(*incoming.conn, incoming.username);
   }
}
