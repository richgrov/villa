if (WIN32)
   set(OS_SPECIFIC_FILES
      "net/win/networking.h"
      "net/win/networking.cc"
   )
endif()

add_executable(simulo
   "main.cc"
    
   "net/networking.h"

   "protocol/types.cc"
   "protocol/types.h"

   "util/slab.h"
   ${OS_SPECIFIC_FILES}
)

target_include_directories(simulo PRIVATE "${PROJECT_SOURCE_DIR}/src")

if (WIN32)
   target_link_libraries(simulo Ws2_32.lib Mswsock.lib)
endif()

if (CMAKE_VERSION VERSION_GREATER 3.12)
   set_property(TARGET simulo PROPERTY CXX_STANDARD 20)
endif()
