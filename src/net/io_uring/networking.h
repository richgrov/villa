#include <liburing.h>
#include <netinet/in.h>

typedef struct {
} Connection;

typedef struct {
   Connection *conn;
   // Will be null-terminated if username length is <16. Otherwise, full buffer is used.
   char username[16];
} IncomingConnection;

typedef struct {
   struct io_uring ring;
   struct sockaddr_in address;
   socklen_t address_size;
   int fd;
} Networking;

bool net_init(Networking *net, uint16_t port, IncomingConnection *accepted_connections);

void net_deinit(Networking *net);

bool net_listen(Networking *net);

int net_poll(Networking *net);
