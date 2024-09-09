#include <stdio.h>

#include "protocol/test_types.h"

int main(int argc, char **argv) {
   test_protocol_short();
   test_protocol_int();
   test_protocol_long();
   puts("OK");
   return 0;
}
