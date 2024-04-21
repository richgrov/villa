#include "packets.h"

#include "types.h"
#include "util/debug_assert.h"

using namespace simulo;
using namespace simulo::packet;

ReadResult Handshake::read(unsigned char *buf, std::size_t len, int progress) {
   ReadResult result = {};

   switch (progress) {
   case 0:
      if (buf[0] != kId) {
         result.min_remaining_bytes = -1;
         return result;
      }

      result.progress = 1;

   case 1:
      if (len - 1 < sizeof(StringSize)) {
         result.min_remaining_bytes = sizeof(StringSize) - (len - 1);
         return result;
      }

      result.progress = 2;

   case 2:
      username_len = read_string_header(&buf[1]);
      if (username_len < 1 || username_len > 16) {
         result.min_remaining_bytes = -1;
         return result;
      }

      result.progress = 3;

   case 3:
      // clang-format off
      {
         // clang-format on
         std::size_t expected_username_len = kCharSize * username_len;
         std::size_t current_username_len = len - 1 - sizeof(StringSize);
         if (current_username_len < expected_username_len) {
            result.min_remaining_bytes = expected_username_len - current_username_len;
            return result;
         }
      }

      result.progress = 4;

   case 4:
      if (!read_string_data(&buf[1 + sizeof(StringSize)], username_len, username)) {
         result.min_remaining_bytes = -1;
      }

      return result;

   default:
      SIMULO_PANIC("unreachable {}", progress);
   }
}
