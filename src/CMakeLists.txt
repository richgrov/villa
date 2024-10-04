if (WIN32)
   set(OS_SPECIFIC_FILES
      "net/win/networking.c"
      "net/win/networking.h"
   )
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
   set(OS_SPECIFIC_FILES
      "net/io_uring/networking.c"
      "net/io_uring/networking.h"
   )
endif()

set(SOURCE_FILES
   "net/networking.h"

   "protocol/packets.c"
   "protocol/packets.h"
   "protocol/types.c"
   "protocol/types.h"

   "player.c"
   "player.h"

   "util/crossplatform_time.h"

   ${OS_SPECIFIC_FILES}
)

add_executable(simulo 
   "main.c"
   ${SOURCE_FILES}
)

target_include_directories(simulo PRIVATE "${PROJECT_SOURCE_DIR}/src")
target_include_directories(simulo PRIVATE "${PROJECT_SOURCE_DIR}/thirdparty")

if (WIN32)
   target_link_libraries(simulo Ws2_32.lib)
endif()

set_property(TARGET simulo PROPERTY C_STANDARD 11)

if (BUILD_TESTING)
   add_executable(simulo_test
      test_main.c
      ${SOURCE_FILES}
   )

   target_include_directories(simulo_test PRIVATE "${PROJECT_SOURCE_DIR}/src")
   target_include_directories(simulo_test PRIVATE "${PROJECT_SOURCE_DIR}/thirdparty")

   if (WIN32)
      target_link_libraries(simulo_test Ws2_32.lib)
   endif()

   set_property(TARGET simulo PROPERTY C_STANDARD 11)
endif()
