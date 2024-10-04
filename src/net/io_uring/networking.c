#include "networking.h"

#include <errno.h>
#include <netinet/in.h>
#include <stdbool.h>
#include <stdio.h>
#include <sys/socket.h>

static inline bool enable_reuseaddr(int fd) {
   int value = 1;
   int res = setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &value, sizeof(value));
   return res == 0;
}

bool net_init(Networking *net, uint16_t port, IncomingConnection *accepted_connections) {
   struct sockaddr_in address = {
      .sin_family = AF_INET,
      .sin_port = htons(port),
      .sin_addr.s_addr = INADDR_ANY,
   };
   net->address = address;
   net->address_size = sizeof(address);

   net->fd = socket(AF_INET, SOCK_STREAM, 0);
   if (net->fd == -1) {
      fprintf(stderr, "socket returned -1: %d", errno);
      return false;
   }

   if (!enable_reuseaddr(net->fd)) {
      fprintf(stderr, "couldn't reuseaddr on %d: %d", net->fd, errno);
      return false;
   }

   if (bind(net->fd, (struct sockaddr *)&net->address, net->address_size) == -1) {
      fprintf(stderr, "couldn't bind %d: %d", net->fd, errno);
      return false;
   }

   return true;
}

void net_deinit(Networking *net) {}

bool net_listen(Networking *net) {
   return false;
}

int net_poll(Networking *net) {
   return 0;
}
