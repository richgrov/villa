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
   static constexpr std::int32_t kProtocolVersion = 14;
   static constexpr unsigned char kId = 1;

   std::int32_t protocol_version;
   int16_t username_len;
   McChar username[16];
   std::int64_t map_seed;
   unsigned char dimension;

   static constexpr std::size_t required_size(std::size_t username_code_points) {
      return sizeof(protocol_version) + MC_STRING_SIZE(username_code_points) + sizeof(map_seed) +
             sizeof(dimension);
   }

   static const std::size_t kMaxSize;

   bool process(const unsigned char *buf, std::size_t expected_username_len);
};

inline constexpr std::size_t Login::kMaxSize = Login::required_size(16);

// The username sent in the handshake packet is ignored by this implementation. We only care about
// its length to know the size of the following Login packet.
struct Handshake {
   static constexpr unsigned char kId = 2;
   static constexpr unsigned char kOfflineModeResponse[] = {kId, 0, 1, 0, '-'};

   int16_t username_len;

   int read(const unsigned char *buf, std::size_t len);

   static const std::size_t kMinSize = sizeof(username_len) + MC_STRING_SIZE(1);
};

} // namespace packet
} // namespace simulo

#endif // !SIMULO_PROTOCOL_PACKETS_H_
