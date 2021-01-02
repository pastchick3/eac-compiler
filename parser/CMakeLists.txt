cmake_minimum_required(VERSION 3.17)

project(parser)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)
set(CXXFLAGS "${CXXFLAGS} /EHsc")

include_directories(generated)
include_directories(runtime)

add_subdirectory(generated)

add_library(parser STATIC
    parser.cpp
)

install(TARGETS CBaseListener DESTINATION .)
install(TARGETS CLexer DESTINATION .)
install(TARGETS CListener DESTINATION .)
install(TARGETS CParser DESTINATION .)
install(TARGETS parser DESTINATION .)
