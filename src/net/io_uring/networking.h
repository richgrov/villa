#include <liburing.h>
#include <netinet/in.h>

#include "protocol/packets.h"

typedef unsigned char ConnectionId;

typedef union {
   unsigned char next_unallocated;
   struct {
      int fd;
      unsigned char buf[PLAYER_IDENTIFICATION_PKT_SIZE + 1]; // +1 for packet id
      int buf_used;
      // Will be null-terminated if username length is <16. Otherwise, full buffer is used.
      char username[16];
   };
} Connection;

typedef struct {
   struct io_uring ring;
   struct sockaddr_in address;
   socklen_t address_size;
   int fd;
   ConnectionId next_unallocated_conn;
   Connection connections[128];

   ConnectionId *join_queue;
   int join_queue_len;
} Networking;

bool net_init(Networking *net, uint16_t port, ConnectionId *join_queue);

void net_deinit(Networking *net);

bool net_listen(Networking *net);

int net_poll(Networking *net);
