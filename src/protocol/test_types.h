#include "protocol/types.h"
#include "util/test_assert.h"

void test_read_protocol_types() {
   unsigned char short_buf[] = {0x13, 0x07};
   assert_eq_short(4871, read_mc_short(short_buf));
}
