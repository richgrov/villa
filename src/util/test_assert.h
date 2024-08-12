#ifndef SIMULO_UTIL_TEST_ASSERT_H_
#define SIMULO_UTIL_TEST_ASSERT_H_

#include "util/debug_assert.h"

static void assert_eq_short(short expected, short actual) {
   ASSERT(expected == actual, "%d != %d", expected, actual);
}

#endif // !SIMULO_UTIL_TEST_ASSERT_H_
