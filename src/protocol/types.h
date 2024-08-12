#ifndef SIMULO_PROTOCOL_TYPES_H_
#define SIMULO_PROTOCOL_TYPES_H_

#include <stdbool.h>
#include <stdint.h>

static inline int16_t read_mc_short(const unsigned char *buf) {
   return buf[0] << 8 | buf[1];
}

static inline int32_t read_mc_int(const unsigned char *buf) {
   return buf[0] << 24 | buf[1] << 16 | buf[2] << 8 | buf[3];
}

// TODO tests
static inline int64_t read_mc_long(const unsigned char *buf) {
   // clang-format off
   return (int64_t) buf[0] << 56 |
          (int64_t) buf[1] << 48 |
          (int64_t) buf[2] << 40 |
          (int64_t) buf[3] << 32 |
          buf[4] << 24 |
          buf[5] << 16 |
          buf[6] << 8 |
          buf[7];
   // clang-format on
}

typedef uint16_t McChar;

bool read_mc_string(const unsigned char *buf, int16_t num_code_units, McChar *dest);

#define MC_STRING_SIZE(n_chars) (sizeof(int16_t) + (n_chars) * sizeof(McChar))

#endif // !SIMULO_PROTOCOL_TYPES_H_
