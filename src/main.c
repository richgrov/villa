#include "config.h"
#include "net/networking.h" // IWYU pragma: export
#include "player.h"
#include "util/arrays.h"
#include "util/crossplatform_time.h"

#define OUT_OF_PLAYERS -1

Player players[128];
int next_avail_player;
Networking networking;
ConnectionId join_queue[SIMULO_JOIN_QUEUE_CAPACITY];

void tick() {
   int num_accepted = net_poll(&networking);

   for (int i = 0; i < num_accepted; ++i) {
      ConnectionId conn_id = join_queue[i];

      if (next_avail_player == OUT_OF_PLAYERS) {
         break; // todo
      }

      Player *player = &players[next_avail_player];
      next_avail_player = player->next;
      player_init(player, &networking.connections[conn_id]);
   }
}

int main() {
   for (int i = 0; i < ARRAY_LEN(players); ++i) {
      int next;
      if (i == ARRAY_LEN(players) - 1) {
         next = OUT_OF_PLAYERS;
      } else {
         next = i + 1;
      }
      players[i].next = next;
   }
   next_avail_player = 0;

   if (!net_init(&networking, 25565, join_queue)) {
      return -1;
   }

   if (!net_listen(&networking)) {
      return -1;
   }

   while (true) {
      tick();
      crossplatform_sleep_ms(20);
   }

   net_deinit(&networking);
   return 0;
}
