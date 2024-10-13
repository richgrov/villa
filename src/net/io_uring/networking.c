#include "networking.h"

#include <errno.h>
#include <liburing.h>
#include <netinet/in.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#include "util/arrays.h"

#define ACCEPT_CQE_ID 0xFFFFFFFFFFFFFFFF
#define CONNECTION_ID_MASK 0xFF
#define CONN_READ_FLAG (1 << 8)
#define CONN_WRITE_FLAG (1 << 9)

static inline bool enable_reuseaddr(int fd) {
   int value = 1;
   int res = setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &value, sizeof(value));
   return res == 0;
}

static inline void queue_accept(Networking *net) {
   struct io_uring_sqe *sqe = io_uring_get_sqe(&net->ring);
   io_uring_prep_multishot_accept(
      sqe, net->fd, (struct sockaddr *)&net->address, &net->address_size, 0
   );
   sqe->user_data = ACCEPT_CQE_ID;
}

static inline void queue_read(Networking *net, int conn_id, Connection *conn) {
   struct io_uring_sqe *sqe = io_uring_get_sqe(&net->ring);
   io_uring_prep_recv(sqe, conn->fd, conn->buf, sizeof(conn->buf), 0);
   sqe->user_data = conn_id | CONN_READ_FLAG;
}

bool net_init(Networking *net, uint16_t port, IncomingConnection *accepted_connections) {
   for (int i = 0; i < ARRAY_LEN(net->connections); ++i) {
      net->connections[i].next_unallocated = i + 1;
   }

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

   struct io_uring_params params = {};
   // TODO: is 2048 a good amount?
   int res = io_uring_queue_init_params(2048, &net->ring, &params);
   if (res != 0) {
      fprintf(stderr, "couldn't init uring params: %d", -res);
      return false;
   }

   if (!(params.features & IORING_FEAT_FAST_POLL)) {
      fprintf(stderr, "fast poll isn't supported");
      return false;
   }

   queue_accept(net);

   return true;
}

void net_deinit(Networking *net) {
   shutdown(net->fd, SHUT_RDWR);
   close(net->fd);
}

bool net_listen(Networking *net) {
   bool ok = listen(net->fd, 16) == 0;
   if (!ok) {
      fprintf(stderr, "couldn't listen on %d: %d", net->fd, errno);
   }
   return ok;
}

static inline void handle_accept(Networking *net, struct io_uring_cqe *cqe) {
   if (!(cqe->flags & IORING_CQE_F_MORE)) {
      queue_accept(net);
   }

   if (cqe->res < 0) {
      // TODO: Don't print certain client-causable errors
      fprintf(stderr, "accept error: %d\n", -cqe->res);
      return;
   }

   bool out_of_connections = net->next_unallocated_conn >= ARRAY_LEN(net->connections);
   if (out_of_connections) {
      // TODO: Send disconnect packet
      return;
   }

   int conn_id = net->next_unallocated_conn;
   Connection *conn = &net->connections[conn_id];
   net->next_unallocated_conn = conn->next_unallocated;
   memset(conn, 0, sizeof(*conn));

   conn->fd = cqe->res;
   queue_read(net, conn_id, conn);
}

static void handle_read(Networking *net, struct io_uring_cqe *cqe) {
   printf("read %d\n", cqe->res);
}

int net_poll(Networking *net) {
   io_uring_submit_and_wait(&net->ring, 1);

   unsigned int head;
   struct io_uring_cqe *cqe;
   unsigned int count = 0;

   io_uring_for_each_cqe(&net->ring, head, cqe) {
      ++count;
      if (cqe->user_data == ACCEPT_CQE_ID) {
         handle_accept(net, cqe);
         continue;
      }

      if (cqe->user_data & CONN_READ_FLAG) {
         handle_read(net, cqe);
      }
   }

   io_uring_cq_advance(&net->ring, count);
   return 0;
}
