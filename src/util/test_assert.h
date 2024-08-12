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

#endif // !SIMULO_UTIL_TEST_ASSERT_H_
