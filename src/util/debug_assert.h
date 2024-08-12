#ifndef SIMULO_UTIL_DEBUG_ASSERT_H_
#define SIMULO_UTIL_DEBUG_ASSERT_H_

#include <stdio.h>
#include <stdlib.h>

#define ASSERT(cond, fmt, ...)                                                                     \
   if (!(cond)) {                                                                                  \
      fprintf(stderr, "%s:%d: " fmt, __FILE__, __LINE__, __VA_ARGS__);                             \
      abort();                                                                                     \
   }

#ifdef NDEBUG

#define SIMULO_DEBUG_ASSERT(cond, fmt, ...)
#define SIMULO_DEBUG_LOG(fmt, ...)

#else // NDEBUG

#define SIMULO_DEBUG_ASSERT(cond, fmt, ...)                                                        \
   {                                                                                               \
      if (!(cond)) {                                                                               \
         fprintf(stderr, "%s:%d: " fmt, __FILE__, __LINE__, __VA_ARGS__);                          \
         abort();                                                                                  \
      }                                                                                            \
   }

#define SIMULO_DEBUG_LOG(fmt, ...) fprintf(stderr, "%s:%d: " fmt, __FILE__, __LINE__, __VA_ARGS__)

#endif // !NDEBUG

#define SIMULO_PANIC(fmt, ...)                                                                     \
   fprintf(stderr, "%s:%d: " fmt, __FILE__, __LINE__, __VA_ARGS__);                                \
   abort()

#endif // !SIMULO_UTIL_DEBUG_ASSERT_H_
