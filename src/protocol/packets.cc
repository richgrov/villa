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

ReadResult Login::read(unsigned char *buf, std::size_t len, int progress) {
   ReadResult result = {};
   unsigned char *cursor = buf;

   switch (progress) {
   case 0:
      if (*cursor != kId) {
         result.min_remaining_bytes = -1;
         return result;
      }

      result.progress = 1;

   case 1:
      if (len < kMinSize + 1) { // +1 for packet id
         result.min_remaining_bytes = kMinSize - len;
         return result;
      }

      result.progress = 2;

   case 2:
      cursor = buf + 1;

      protocol_version = read_int(cursor);

      cursor += sizeof(protocol_version);
      username_len = read_string_header(cursor);
      if (username_len < 1 || username_len > 16) {
         result.min_remaining_bytes = -1;
         return result;
      }

      result.progress = 2;

   case 3:
      // clang-format off
      {
         // clang-format on
         std::size_t expected_size = 1 + required_size(username_len);
         if (len < expected_size) {
            result.min_remaining_bytes = expected_size - len;
            return result;
         }
      }

      result.progress = 4;

   case 4:
      cursor = buf + 1 + sizeof(protocol_version) + sizeof(username_len);

      if (!read_string_data(cursor, username_len, username)) {
         result.min_remaining_bytes = -1;
         return result;
      }

      cursor += username_len * kCharSize;
      map_seed = read_int(cursor);

      cursor += sizeof(map_seed);
      dimension = *cursor;

      return result;

   default:
      SIMULO_PANIC("progress = {}", result.progress);
   }
}
