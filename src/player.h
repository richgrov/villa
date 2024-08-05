#ifndef SIMULO_PLAYER_H_
#define SIMULO_PLAYER_H_

#include "net/networking.h"

namespace simulo {

class Player {
public:
   // Username is expected to be at least 16 chars
   Player(net::Connection &conn, const char *username);

private:
   net::Connection &conn_;
   std::array<char, 16> username_;
};

} // namespace simulo

#endif // !SIMULO_PLAYER_H_
