#ifndef SIMULO_UTIL_CROSSPLATFORM_TIME_H_
#define SIMULO_UTIL_CROSSPLATFORM_TIME_H_

#include "util/os_detect.h"

#ifdef SIMULO_WINDOWS

#include <Windows.h>

static inline void crossplatform_sleep_ms(int ms) {
   Sleep((DWORD)ms);
}

#endif

#endif // !SIMULO_UTIL_CROSSPLATFORM_TIME_H_
