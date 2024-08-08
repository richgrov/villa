#ifndef SIMULO_PLAYER_H_
#define SIMULO_PLAYER_H_

#include "net/networking.h"

namespace simulo {

class Player {
public:
   // Username is expected to be at least 16 chars
   Player(Connection *conn, const char *username);

private:
   Connection *conn_;
   char username_[16];
};

} // namespace simulo

#endif // !SIMULO_PLAYER_H_
