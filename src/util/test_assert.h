#ifndef SIMULO_UTIL_TEST_ASSERT_H_
#define SIMULO_UTIL_TEST_ASSERT_H_

#include <stdint.h>

#include "util/debug_assert.h"

static void assert_eq_i16(int16_t expected, int16_t actual) {
   ASSERT(expected == actual, "%d != %d", expected, actual);
}

#endif // !SIMULO_UTIL_TEST_ASSERT_H_
