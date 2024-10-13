#include "packets.h"

#include <stddef.h>

#include "types.h"

bool read_player_identification_pkt(const unsigned char *buf, PlayerIdentification *pkt) {
   const unsigned char *cursor = buf;

   pkt->protocol_version = *cursor++;
   if (pkt->protocol_version != 7) {
      return false;
   }

   pkt->username = (const char *)cursor;
   pkt->username_len = mc_string_len(pkt->username);
   cursor += sizeof(McString);
   if (pkt->username_len < 1 || pkt->username_len > 16) {
      return false;
   }

   pkt->verification_key = (const char *)cursor;
   pkt->verification_key_len = mc_string_len(pkt->verification_key);
   cursor += sizeof(McString);

   pkt->padding = *cursor++;

   return true;
}

bool read_login_pkt(const unsigned char *buf, const size_t len, Login *pkt) {
   const unsigned char *cursor = buf;

   if (*cursor != LOGIN_ID) {
      return false;
   }
   ++cursor;

   pkt->protocol_version = read_mc_int(cursor);
   cursor += sizeof(pkt->protocol_version);

   pkt->username_len = read_mc_short(cursor);
   if (pkt->username_len < 1 || LOGIN_PACKET_SIZE(pkt->username_len) > len) {
      return false;
   }
   cursor += sizeof(pkt->username_len);

   if (!read_mc_string(cursor, pkt->username_len, pkt->username)) {
      return false;
   }

   cursor += pkt->username_len * sizeof(McChar);

   pkt->map_seed = read_mc_long(cursor);
   cursor += sizeof(pkt->map_seed);

   pkt->dimension = *cursor;
   return true;
}

int required_handshake_size(int16_t username_len) {
   return 1 +                           // packet id
          MC_STRING_SIZE(username_len); // username
}

int remaining_handshake_bytes(const unsigned char *buf, const size_t len, Handshake *pkt) {
   if (len < required_handshake_size(1)) {
      return required_handshake_size(1) - len;
   }

   if (buf[0] != HANDSHAKE_ID) {
      return -1;
   }

   pkt->username_len = read_mc_short(&buf[1]);
   if (pkt->username_len < 1 || pkt->username_len > 16) {
      return -1;
   }

   return required_handshake_size(pkt->username_len) - len;
}
