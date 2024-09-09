#include <stdio.h>

#include "protocol/test_types.h"

int main(int argc, char **argv) {
   test_read_protocol_types();
   test_write_protocol_types();
   puts("OK");
   return 0;
}
