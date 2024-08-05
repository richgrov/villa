#include "server.h"

#include <chrono>
#include <cstdint>
#include <memory>
#include <thread>

#include "net/win/networking.h"

using namespace simulo;

namespace {

constexpr std::size_t kAcceptQueueCapacity = 8;

}; // namespace

Server::Server()
    : accepted_connections_(), players_(std::make_unique<PlayerSlab>()),
      networking_(25565, accepted_connections_) {

   accepted_connections_.reserve(kAcceptQueueCapacity);
}

void Server::run() {
   networking_.listen();

   while (true) {
      tick();
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
   }
}

void Server::tick() {
   networking_.poll();

   while (!accepted_connections_.empty()) {
      net::IncomingConnection &incoming = accepted_connections_.back();
      accepted_connections_.pop_back();
      players_->emplace(*incoming.conn, incoming.username);
   }
}
