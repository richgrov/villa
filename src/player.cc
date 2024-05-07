#include "player.h"

#include <iostream>

#include "net/networking.h"
#include "protocol/types.h"

using namespace simulo;

Player::Player(net::Connection &conn, StringSize username_len) : conn_(conn) {
   std::cout << username_len << "\n";
}
