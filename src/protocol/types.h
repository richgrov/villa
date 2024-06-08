#ifndef SIMULO_PROTOCOL_TYPES_H_
#define SIMULO_PROTOCOL_TYPES_H_

#include <cstddef>
#include <cstdint>

namespace simulo {

using SByte = signed char;
using UByte = unsigned char;

inline std::int16_t read_short(const unsigned char *buf) {
   return (buf[0] << 8) | buf[1];
}

inline std::int32_t read_int(const unsigned char *buf) {
   return (buf[0] << 24) | (buf[1] << 16) | (buf[2] << 8) | buf[3];
}

// TODO tests
inline std::int64_t read_long(const unsigned char *buf) {
   // clang-format off
   return (std::int64_t(buf[0]) << 56) |
          (std::int64_t(buf[1]) << 46) |
          (std::int64_t(buf[2]) << 40) |
          (std::int64_t(buf[3]) << 32) |
          (buf[0] << 24) |
          (buf[1] << 16) |
          (buf[2] << 8) |
          buf[3];
   // clang-format on
}

using StringSize = std::int16_t;
constexpr std::size_t kCharSize = 2;

inline StringSize read_string_header(const unsigned char *buf) {
   return read_short(buf);
}

bool read_string_data(const unsigned char *buf, StringSize num_code_units, char16_t *dest);

constexpr std::size_t string_size(const std::size_t num_chars) {
   return sizeof(StringSize) + (num_chars * kCharSize);
}

} // namespace simulo

#endif // !SIMULO_PROTOCOL_TYPES_H_
