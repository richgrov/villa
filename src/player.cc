#include "player.h"

#include <cstring>
#include <iostream>

#include "net/networking.h"
#include "protocol/packets.h"
#include "protocol/types.h"

using namespace simulo;

namespace {

int username_len(const char *username) {
   for (int i = 0; i < 16; ++i) {
      if (username[i] == '\0') {
         return i;
      }
   }
   return 16;
}

} // namespace

void simulo::player_init(Player *player, Connection *conn, const char *username) {
   player->conn = conn;
   memcpy(player->username, username, MAX_USERNAME_LEN);
   std::cout << std::string(player->username, username_len(player->username)) << "\n";
}
