cmake_minimum_required(VERSION 3.17)

project(parser)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

include_directories(generated)
include_directories(runtime)

add_library(parser STATIC
    parser.cpp
)

install(TARGETS parser DESTINATION .)
