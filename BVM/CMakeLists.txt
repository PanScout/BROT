cmake_minimum_required(VERSION 3.16)
project(BVM LANGUAGES CXX)

set(OUTPUT_FILE "bvm")


set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/bin)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Debug)
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Release")
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION ON)
endif()

add_executable("${OUTPUT_FILE}"
  src/main.cpp
)

target_compile_options("${OUTPUT_FILE}" PRIVATE -Werror -Wall -Wextra -Wpedantic -Wsign-conversion -Wfloat-conversion -Wnarrowing -Wshadow -Wold-style-cast -Wcast-qual -Wconversion -fstandalone-debug  -glldb )
