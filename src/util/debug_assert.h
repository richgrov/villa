#ifndef SIMULO_UTIL_DEBUG_ASSERT_H_
#define SIMULO_UTIL_DEBUG_ASSERT_H_

#include <type_traits>
#ifdef NDEBUG

#define SIMULO_DEBUG_ASSERT(cond, fmt, ...)

#else // NDEBUG

#include <cstdlib>
#include <format>
#include <iostream>
#include <utility>

#define SIMULO_DEBUG_ASSERT(cond, fmt, ...)                                                        \
   {                                                                                               \
      if (!(cond)) {                                                                               \
         std::cerr << __FILE__ << ":" << __LINE__ << ": " << std::format(fmt, __VA_ARGS__)         \
                   << "\n";                                                                        \
         std::exit(1);                                                                             \
      }                                                                                            \
   }

#endif // !NDEBUG

#define SIMULO_PANIC(fmt, ...)                                                                     \
   std::cerr << __FILE__ << ":" << __LINE__ << ": " << std::format(fmt, __VA_ARGS__) << "\n";      \
   std::exit(1)

template <class T>
   requires std::is_enum_v<T>
std::underlying_type_t<T> enum_ordinal(T enum_value) {
   return static_cast<std::underlying_type_t<T>>(enum_value);
}

#endif // !SIMULO_UTIL_DEBUG_ASSERT_H_
