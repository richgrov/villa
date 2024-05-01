#include "server.h"

#include <chrono>
#include <cstdint>
#include <thread>

#include "net/win/networking.h"

using namespace simulo;

namespace {

constexpr std::size_t kAcceptQueueCapacity = 8;

}; // namespace

Server::Server() : accepted_connections_(), networking_(25565, accepted_connections_) {
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
      const net::Connection conn = std::move(accepted_connections_.back());
      accepted_connections_.pop_back();
      std::cout << conn.handshake_packet.username_len << "\n";
   }
}
