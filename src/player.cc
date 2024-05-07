#include "player.h"

#include "net/networking.h"
#include <iostream>

using namespace simulo;

Player::Player(net::Connection &conn) : conn_(conn) {
   std::cout << conn_.username_len() << "\n";
}
