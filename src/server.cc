#include "server.h"

#include <chrono>
#include <thread>

#include "net/win/networking.h"

using namespace simulo;

Server::Server() : networking_(25565) {}

void Server::run() {
   networking_.listen();

   while (true) {
      networking_.poll();
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
   }
}
