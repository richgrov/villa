#include "protocol/types.h"
#include "util/test_assert.h"

void test_read_protocol_types() {
   unsigned char short_buf[] = {0x13, 0x07};
   assert_eq_i16(4871, read_mc_short(short_buf));

   unsigned char int_buf[] = {0x44, 0xE1, 0x11, 0xA7};
   assert_eq_i32(1155600807, read_mc_int(int_buf));
}
