#ifndef SIMULO_UTIL_OS_DETECT_H_
#define SIMULO_UTIL_OS_DETECT_H_

#if defined(WIN32) || defined(_WIN32) || defined(__WIN32) && !defined(__CYGWIN__)
#define SIMULO_WINDOWS
#endif

#endif // !SIMULO_UTIL_OS_DETECT_H_
