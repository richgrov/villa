#include "types.h"

bool read_mc_string(const unsigned char *buf, int16_t num_code_units, McChar *dest) {
   for (int i = 0; i < num_code_units; i++) {
      McChar code_point = (buf[i * 2] << 8) | buf[i * 2 + 1];
      // All displayable characters in Beta 1.7.3 are in the Basic Multilingual Plane, meaning
      // there is no need to worry about creating surrogate pairs.
      if (code_point >= 0xD800 && code_point <= 0xDBFF) {
         return false;
      }

      dest[i] = code_point;
   }

   return true;
}
