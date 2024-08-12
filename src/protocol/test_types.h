#include "protocol/types.h"
#include "util/os_detect.h"
#include "util/test_assert.h"

void test_read_protocol_types() {
   unsigned char short_buf[] = {0x13, 0x07};
   assert_eq_i16(4871, read_mc_short(short_buf));

   unsigned char int_buf[] = {0x44, 0xE1, 0x11, 0xA7};
   assert_eq_i32(1155600807, read_mc_int(int_buf));

   unsigned char long_buf[] = {0xFF, 0x10, 0x7C, 0x99, 0x00, 0x65, 0x9A, 0x0D};
#ifdef SIMULO_WINDOWS
   int64_t expected_long = -67416997832058355LL;
#else
   int64_t expected_long = -67416997832058355L;
#endif
   assert_eq_i64(expected_long, read_mc_long(long_buf));
}
