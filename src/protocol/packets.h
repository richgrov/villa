#ifndef SIMULO_PROTOCOL_PACKETS_H_
#define SIMULO_PROTOCOL_PACKETS_H_

#include <cstddef>
#include <cstdint>

#include "types.h"

namespace simulo {

struct ReadResult {
   int min_remaining_bytes;
   int progress;
};

namespace packet {

struct Login {
   static constexpr UByte kId = 1;

   std::int32_t protocol_version;
   StringSize username_len;
   char16_t username[16]; // null-terminated
   std::int64_t map_seed;
   UByte dimension;

   static constexpr std::size_t kMaxSize =
       sizeof(protocol_version) + string_size(16) + sizeof(map_seed) + sizeof(dimension);
};

struct Handshake {
   static constexpr UByte kId = 2;

   StringSize username_len;
   char16_t username[16];

   ReadResult read(unsigned char *buf, std::size_t len, int stage);
};

} // namespace packet
} // namespace simulo

#endif // !SIMULO_PROTOCOL_PACKETS_H_
