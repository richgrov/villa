#include "player.h"

#include <stdio.h>
#include <string.h>

#include "net/networking.h" // IWYU pragma: export
#include "protocol/packets.h"

static int username_len(const char *username) {
   for (int i = 0; i < 16; ++i) {
      if (username[i] == '\0') {
         return i;
      }
   }
   return 16;
}

void player_init(Player *player, Connection *conn, const char *username) {
   player->conn = conn;
   memcpy(player->username, username, MAX_USERNAME_LEN);
   printf("%.*s\n", username_len(player->username), player->username);
}
