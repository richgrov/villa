#ifndef SIMULO_UTIL_CROSSPLATFORM_TIME_H_
#define SIMULO_UTIL_CROSSPLATFORM_TIME_H_

#include "util/os_detect.h"

#if defined(SIMULO_WINDOWS)

#include <Windows.h>

static inline void crossplatform_sleep_ms(int ms) {
   Sleep((DWORD)ms);
}

#elif defined(SIMULO_LINUX)

#include <errno.h>
#include <time.h>

#include "debug_assert.h"

static inline void crossplatform_sleep_ms(int ms) {
   struct timespec time = {
      .tv_sec = ms / 1000,
      .tv_nsec = (ms % 1000) * 1000 * 1000,
   };

   int status = nanosleep(&time, NULL);
   SIMULO_DEBUG_ASSERT(status == 0, "nanosleep returned %d: errno = %d", status, errno);
}

#endif

#endif // !SIMULO_UTIL_CROSSPLATFORM_TIME_H_
