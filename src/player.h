#ifndef SIMULO_PLAYER_H_
#define SIMULO_PLAYER_H_

#include "net/networking.h"

namespace simulo {

class Player {
public:
   Player(net::Connection &conn);

private:
   net::Connection &conn_;
};

} // namespace simulo

#endif // !SIMULO_PLAYER_H_
