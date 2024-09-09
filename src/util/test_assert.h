#ifndef SIMULO_UTIL_TEST_ASSERT_H_
#define SIMULO_UTIL_TEST_ASSERT_H_

#include <stdint.h>

#include "util/debug_assert.h"
#include "util/os_detect.h"

static void assert_eq_i16(int16_t expected, int16_t actual) {
   ASSERT(expected == actual, "%d != %d", expected, actual);
}

static void assert_eq_i32(int32_t expected, int32_t actual) {
   ASSERT(expected == actual, "%d != %d", expected, actual);
}

static void assert_eq_i64(int64_t expected, int64_t actual) {
#ifdef SIMULO_WINDOWS
   ASSERT(expected == actual, "%lld != %lld", expected, actual);
#else
   ASSERT(expected == actual, "%ld != %ld", expected, actual);
#endif
}

static void assert_eq_bytes(
   unsigned char *expected, size_t expected_len, unsigned char *actual, size_t actual_len
) {
#ifdef SIMULO_WINDOWS
   ASSERT(expected_len == actual_len, "expected %lld bytes but got %lld", expected_len, actual_len);
#else
   ASSERT(expected_len == actual_len, "expected %ld bytes but got %ld", expected_len, actual_len);
#endif

   for (size_t i = 0; i < expected_len; ++i) {
      unsigned char exp = expected[i];
      unsigned char act = actual[i];

#ifdef SIMULO_WINDOWS
      ASSERT(exp == act, "expected byte %lld to be %d but got %d", i, exp, act);
#else
      ASSERT(ex == ac, "expected byte %ld to be %d but got %d", i, ex, ac);
#endif
   }
}

#endif // !SIMULO_UTIL_TEST_ASSERT_H_
