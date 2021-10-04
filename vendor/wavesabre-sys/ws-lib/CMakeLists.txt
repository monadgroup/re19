project("WaveSabre")

cmake_minimum_required(VERSION 3.11)
set_property(GLOBAL PROPERTY USE_FOLDERS ON)

if(MSVC)
	# disable exceptions globally (will be added back for VSTs)
	string(REPLACE "/EHsc" "" CMAKE_CXX_FLAGS ${CMAKE_CXX_FLAGS})
endif()

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /GS- /Gs- /GR-")

# shared code
add_subdirectory(WaveSabreCore)
add_subdirectory(WaveSabrePlayerLib)
add_subdirectory(WaveSabreC)
