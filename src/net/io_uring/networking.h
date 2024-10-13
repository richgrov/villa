#include <liburing.h>
#include <netinet/in.h>

#include "protocol/types.h"

#define LOGIN_PACKET_SIZE                                                                          \
   sizeof(char) + sizeof(char) + sizeof(McString) + sizeof(McString) + sizeof(char)

typedef union {
   unsigned char next_unallocated;
   struct {
      int fd;
      unsigned char buf[LOGIN_PACKET_SIZE];
   };
} Connection;

typedef struct {
   Connection *conn;
   // Will be null-terminated if username length is <16. Otherwise, full buffer is used.
   char username[16];
} IncomingConnection;

#define NETWORKING_NUM_CONNECTIONS 128

typedef struct {
   struct io_uring ring;
   struct sockaddr_in address;
   socklen_t address_size;
   int fd;
   unsigned char next_unallocated_conn;
   Connection connections[NETWORKING_NUM_CONNECTIONS];
} Networking;

bool net_init(Networking *net, uint16_t port, IncomingConnection *accepted_connections);

void net_deinit(Networking *net);

bool net_listen(Networking *net);

int net_poll(Networking *net);
