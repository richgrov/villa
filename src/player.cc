#include "player.h"

#include <iostream>

#include "net/networking.h"
#include "protocol/types.h"

using namespace simulo;

namespace {

int username_len(const std::array<char, 16> &username) {
   for (int i = 0; i < 16; ++i) {
      if (username[i] == '\0') {
         return i;
      }
   }
   return 16;
}

} // namespace

Player::Player(net::Connection &conn, std::array<char, 16> username)
    : conn_(conn), username_(username) {
   std::cout << std::string(username_.data(), username_len(username)) << "\n";
}
