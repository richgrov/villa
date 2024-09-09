#include "protocol/types.h"
#include "util/os_detect.h"
#include "util/test_assert.h"

void test_protocol_short() {
   unsigned char short_buf[] = {0x13, 0x07};
   assert_eq_i16(4871, read_mc_short(short_buf));
}

void test_protocol_long() {
   unsigned char long_buf[] = {0xFF, 0x10, 0x7C, 0x99, 0x00, 0x65, 0x9A, 0x0D};
   unsigned char write_buf[8];

#ifdef SIMULO_WINDOWS
   assert_eq_i64(-67416997832058355LL, read_mc_long(long_buf));
   write_mc_long(write_buf, -67416997832058355LL);
#else
   assert_eq_i64(-67416997832058355L, read_mc_long(long_buf));
   write_mc_long(write_buf, -67416997832058355L);
#endif

   assert_eq_bytes(long_buf, sizeof(long_buf), write_buf, sizeof(write_buf));
}

void test_protocol_int() {
   unsigned char int_buf[] = {0x44, 0xE1, 0x11, 0xA7};
   unsigned char write_buf[4];

   assert_eq_i32(1155600807, read_mc_int(int_buf));
   write_mc_int(write_buf, 1155600807);
   assert_eq_bytes(int_buf, sizeof(int_buf), write_buf, sizeof(write_buf));
}
