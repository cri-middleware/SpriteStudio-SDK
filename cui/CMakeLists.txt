include(${CMAKE_CURRENT_SOURCE_DIR}/../cmake/initialize.cmake)
CMAKE_MINIMUM_REQUIRED(VERSION 3.25)
project(SsConverter3Cui)

INITIALIZE(SsConverter3Cui_EXTEND_SRCS)

PREPARE()

message("start fetching dependency libraries")
include(FetchContent)

message("fetch argparse")
FetchContent_Declare(
        argparse
        GIT_REPOSITORY https://github.com/p-ranav/argparse.git
        GIT_TAG        v3.1
)
FetchContent_MakeAvailable(argparse)
include_directories(${argparse_SOURCE_DIR}/include)

if(NOT TARGET SsConverter3Lib)
    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../lib ${PROJECT_NAME}_lib)
endif()

set(SsConverter3Cui_SRCS
        SsConverter3.cpp
)

set(SsConverter3Cui_HEADERS
)

include_directories(${CMAKE_CURRENT_SOURCE_DIR})
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../lib)

add_executable(SsConverter3 ${SsConverter3Cui_SRCS} ${SsConverter3Cui_HEADERS} ${SsConverter3Cui_EXTEND_SRCS})
target_link_libraries(SsConverter3 PRIVATE SsConverter3Lib)
