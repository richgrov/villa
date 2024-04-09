cmake_minimum_required (VERSION 3.8)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Enable Hot Reload for MSVC compilers if supported.
if (POLICY CMP0141)
  cmake_policy(SET CMP0141 NEW)
  set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<IF:$<AND:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>,$<$<CONFIG:Debug,RelWithDebInfo>:EditAndContinue>,$<$<CONFIG:Debug,RelWithDebInfo>:ProgramDatabase>>")
endif()

project ("simulo")

if (WIN32)
	set(OS_SPECIFIC_FILES
		"net/win/networking.h"
		"net/win/networking.cc"
	)
endif()

add_executable(simulo
	"main.cc"
	
	"net/networking.h"
	${OS_SPECIFIC_FILES}
)

if (WIN32)
	target_link_libraries(simulo Ws2_32.lib Mswsock.lib)
endif()

if (CMAKE_VERSION VERSION_GREATER 3.12)
  set_property(TARGET simulo PROPERTY CXX_STANDARD 20)
endif()
