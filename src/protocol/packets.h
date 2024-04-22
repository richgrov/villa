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
   char16_t username[16];
   std::int64_t map_seed;
   UByte dimension;

   static constexpr std::size_t required_size(std::size_t username_code_points) {
      return sizeof(protocol_version) + string_size(username_code_points) + sizeof(map_seed) +
             sizeof(dimension);
   }

   static const std::size_t kMinSize;
   static const std::size_t kMaxSize;

   ReadResult read(unsigned char *buf, std::size_t len, int progress);
};

inline constexpr std::size_t Login::kMinSize = Login::required_size(1);
inline constexpr std::size_t Login::kMaxSize = Login::required_size(16);

struct Handshake {
   static constexpr UByte kId = 2;
   static constexpr unsigned char kOfflineModeResponse[] = {kId, 0, 1, 0, '-'};

   StringSize username_len;
   char16_t username[16];

   ReadResult read(unsigned char *buf, std::size_t len, int progress);
};

} // namespace packet
} // namespace simulo

#endif // !SIMULO_PROTOCOL_PACKETS_H_
