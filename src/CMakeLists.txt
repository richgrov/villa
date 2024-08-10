if (WIN32)
   set(OS_SPECIFIC_FILES
      "net/win/networking.c"
      "net/win/networking.h"
   )
endif()

set(SOURCE_FILES
   "net/networking.h"

   "protocol/packets.c"
   "protocol/packets.h"
   "protocol/types.c"
   "protocol/types.h"

   "player.cc"
   "player.h"

   ${OS_SPECIFIC_FILES}
)

add_executable(simulo 
   "main.cc"
   ${SOURCE_FILES}
)

target_include_directories(simulo PRIVATE "${PROJECT_SOURCE_DIR}/src")
target_include_directories(simulo PRIVATE "${PROJECT_SOURCE_DIR}/thirdparty")

if (WIN32)
   target_link_libraries(simulo Ws2_32.lib)
endif()

set_property(TARGET simulo PROPERTY C_STANDARD 11)

if (CMAKE_VERSION VERSION_GREATER 3.12)
   set_property(TARGET simulo PROPERTY CXX_STANDARD 20)
endif()

if (BUILD_TESTING)
   add_executable(simulo_test
      "test_main.cc"
      ${SOURCE_FILES}
   )

   target_include_directories(simulo_test PRIVATE "${PROJECT_SOURCE_DIR}/src")
   target_include_directories(simulo_test PRIVATE "${PROJECT_SOURCE_DIR}/thirdparty")

   if (WIN32)
      target_link_libraries(simulo_test Ws2_32.lib)
   endif()

   if (CMAKE_VERSION VERSION_GREATER 3.12)
      set_property(TARGET simulo_test PROPERTY CXX_STANDARD 20)
   endif()
endif()
