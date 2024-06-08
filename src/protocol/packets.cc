#include "packets.h"

#include "types.h"
#include "util/debug_assert.h"

using namespace simulo;
using namespace simulo::packet;

int Handshake::read(const unsigned char *buf, const std::size_t len) {
   if (len < Handshake::kMinSize + 1) {
      return Handshake::kMinSize + 1 - len;
   }

   if (buf[0] != kId) {
      return -1;
   }

   username_len = read_mc_short(&buf[1]);
   if (username_len < 1 || username_len > 16) {
      return -1;
   }

   std::size_t expected_size = 1 + MC_STRING_SIZE(username_len);
   return expected_size - len;
}

bool Login::process(const unsigned char *buf, const std::size_t len) {
   const unsigned char *cursor = buf;

   if (*cursor != kId) {
      return false;
   }

   cursor = buf + 1;

   protocol_version = read_mc_int(cursor);

   cursor += sizeof(protocol_version);
   username_len = read_mc_short(cursor);
   if (username_len < 1 || Login::required_size(username_len) > len) {
      return false;
   }

   cursor = buf + 1 + sizeof(protocol_version) + sizeof(username_len);

   if (!read_mc_string(cursor, username_len, username)) {
      return false;
   }

   cursor += username_len * sizeof(McChar);
   map_seed = read_mc_long(cursor);

   cursor += sizeof(map_seed);
   dimension = *cursor;
   return true;
}
