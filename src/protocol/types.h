#ifndef SIMULO_PROTOCOL_TYPES_H_
#define SIMULO_PROTOCOL_TYPES_H_

#include <stdbool.h>
#include <stdint.h>

typedef char McString[64];

static inline int16_t read_mc_short(const unsigned char *buf) {
   return buf[0] << 8 | buf[1];
}

static inline int32_t read_mc_int(const unsigned char *buf) {
   return buf[0] << 24 | buf[1] << 16 | buf[2] << 8 | buf[3];
}

static inline unsigned char *write_mc_int(unsigned char *buf, int32_t i) {
   buf[0] = i >> 24;
   buf[1] = i >> 16;
   buf[2] = i >> 8;
   buf[3] = i;
   return buf + sizeof(int32_t);
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

static inline unsigned char *write_mc_long(unsigned char *buf, int64_t i) {
   buf[0] = i >> 56;
   buf[1] = i >> 48;
   buf[2] = i >> 40;
   buf[3] = i >> 32;
   buf[4] = i >> 24;
   buf[5] = i >> 16;
   buf[6] = i >> 8;
   buf[7] = i;
   return buf + sizeof(int64_t);
}

typedef uint16_t McChar;

bool read_mc_string(const unsigned char *buf, int16_t num_code_units, McChar *dest);

int mc_string_len(const McString str);

#define MC_STRING_SIZE(n_chars) (sizeof(int16_t) + (n_chars) * sizeof(McChar))

#endif // !SIMULO_PROTOCOL_TYPES_H_
