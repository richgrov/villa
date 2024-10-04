#ifndef SIMULO_NET_NETWORKING_H_
#define SIMULO_NET_NETWORKING_H_

#include "util/os_detect.h"

#if defined(SIMULO_WINDOWS)

#include "win/networking.h"

#elif defined(SIMULO_LINUX)

#include "io_uring/networking.h"

#else // windows

#error "platform not supported"

#endif // non-windows

#endif // !SIMULO_NET_NETWORKING_H_
