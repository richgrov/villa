#ifndef SIMULO_PLAYER_H_
#define SIMULO_PLAYER_H_

#include "net/networking.h"

typedef struct {
   Connection *conn;
   char username[16];
} Player;

// Username is expected to be at least 16 chars
void player_init(Player *player, Connection *conn, const char *username);

#endif // !SIMULO_PLAYER_H_
