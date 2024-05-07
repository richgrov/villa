#include "player.h"

#include <iostream>

#include "net/networking.h"

using namespace simulo;

Player::Player(net::Connection &conn) : conn_(conn) {
   std::cout << conn_.username_len() << "\n";
}
