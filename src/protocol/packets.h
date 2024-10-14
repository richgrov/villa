#ifndef SIMULO_PROTOCOL_PACKETS_H_
#define SIMULO_PROTOCOL_PACKETS_H_

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "types.h"

#define MAX_USERNAME_LEN 16
#define CLASSIC_PROTOCOL_VER 7

#define PLAYER_IDENTIFICATION_ID 0
#define PLAYER_IDENTIFICATION_PKT_SIZE                                                             \
   (sizeof(unsigned char) + sizeof(McString) + sizeof(McString) + sizeof(unsigned char))

typedef struct {
   unsigned char protocol_version;
   const char *username;
   int username_len;
   const char *verification_key;
   int verification_key_len;
   unsigned char padding;
} PlayerIdentification;

bool read_player_identification_pkt(const unsigned char *buf, PlayerIdentification *pkt);

typedef enum {
   USER_TYPE_REGULAR,
   USER_TYPE_OPERATOR,
} UserType;

typedef struct {
   unsigned char protocol_version;
   const char *server_name;
   const char *server_motd;
   UserType user_type;
} ServerIdentification;

void write_server_identification_pkt(unsigned char *buf, ServerIdentification *pkt);

#define LOGIN_ID 1
typedef struct {
   int32_t protocol_version;
   int16_t username_len;
   McChar username[16];
   int64_t map_seed;
   uint8_t dimension;
} Login;

bool read_login_pkt(const unsigned char *buf, size_t len, Login *pkt);

#define LOGIN_PACKET_SIZE(username_len)                                                            \
   (1 +                            /* packet id */                                                 \
    4 +                            /* protocol version */                                          \
    MC_STRING_SIZE(username_len) + /* username */                                                  \
    8 +                            /* seed */                                                      \
    1)                             /* dimension */

#define HANDSHAKE_ID 2
// The username sent in the handshake packet is ignored by this implementation. We only care
// about its length to know the size of the following Login packet.
typedef struct {
   int16_t username_len;
} Handshake;

static const unsigned char OFFLINE_MODE_RESPONSE[] = {
   HANDSHAKE_ID, // packet id
   0,            // username length high byte
   1,            // username legth low byte
   0,            // first char high byte
   '-'           // first char low byte
};

int remaining_handshake_bytes(const unsigned char *buf, size_t len, Handshake *pkt);

#endif // !SIMULO_PROTOCOL_PACKETS_H_
