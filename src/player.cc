#include "player.h"

#include <cstring>
#include <iostream>

#include "net/networking.h"
#include "protocol/packets.h"
#include "protocol/types.h"

using namespace simulo;

namespace {

int username_len(const char *username) {
   for (int i = 0; i < 16; ++i) {
      if (username[i] == '\0') {
         return i;
      }
   }
   return 16;
}

} // namespace

Player::Player(Connection &conn, const char *username) : conn_(conn) {
   memcpy(username_, username, MAX_USERNAME_LEN);
   std::cout << std::string(username_, username_len(username_)) << "\n";
}
