#ifndef SIMULO_NET_NETWORKING_H_
#define SIMULO_NET_NETWORKING_H_

#if defined(WIN32) || defined(_WIN32) || defined(__WIN32) && !defined(__CYGWIN__)

#include "win/networking.h"

#else // windows

#error "platform not supported"

#endif // non-windows

#endif // SIMULO_NET_NETWORKING_H_
