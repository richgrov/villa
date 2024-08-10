#ifndef SIMULO_UTIL_CROSSPLATFORM_TIME_H_
#define SIMULO_UTIL_CROSSPLATFORM_TIME_H_

#if defined(WIN32) || defined(_WIN32) || defined(__WIN32) && !defined(__CYGWIN__)

#include <Windows.h>

static inline void crossplatform_sleep_ms(int ms) {
   Sleep((DWORD)ms);
}

#endif

#endif // !SIMULO_UTIL_CROSSPLATFORM_TIME_H_
