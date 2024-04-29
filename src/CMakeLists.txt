if (WIN32)
   set(OS_SPECIFIC_FILES
      "net/win/networking.h"
      "net/win/networking.cc"
   )
endif()

set(SOURCE_FILES
   "net/networking.h"

   "protocol/packets.cc"
   "protocol/packets.h"
   "protocol/types.cc"
   "protocol/types.h"

   "util/slab.h"

   "server.cc"
   "server.h"

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

if (CMAKE_VERSION VERSION_GREATER 3.12)
   set_property(TARGET simulo PROPERTY CXX_STANDARD 20)
endif()

if (BUILD_TESTING)
   add_executable(simulo_test
      "test_main.cc"
      "util/slab_test.cc"
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
