#ifndef SIMULO_PLAYER_H_
#define SIMULO_PLAYER_H_

#include "net/networking.h"

typedef union {
   struct {
      Connection *conn;
      char username[16];
   };
   int next; // used for slab allocation
} Player;

// Username is expected to be at least 16 chars
void player_init(Player *player, Connection *conn);

#endif // !SIMULO_PLAYER_H_
