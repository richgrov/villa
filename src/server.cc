#include "server.h"

#include <chrono>
#include <thread>

#include "net/win/networking.h"
#include "player.h"
#include "util/arrays.h"

using namespace simulo;

#define OUT_OF_PLAYERS -1

Server::Server() : accepted_connections_() {
   for (int i = 0; i < ARRAY_LEN(players_); ++i) {
      int next;
      if (i == ARRAY_LEN(players_) - 1) {
         next = OUT_OF_PLAYERS;
      } else {
         next = i + 1;
      }
      players_[i].next = next;
   }

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

      if (next_avail_player_ == OUT_OF_PLAYERS) {
         break; // todo
      }

      Player &player = players_[next_avail_player_];
      next_avail_player_ = player.next;
      player_init(&player, incoming.conn, incoming.username);
   }
}
