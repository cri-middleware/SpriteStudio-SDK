include(${CMAKE_CURRENT_SOURCE_DIR}/../cmake/initialize.cmake)
CMAKE_MINIMUM_REQUIRED(VERSION 3.25)
project(SsConverter2)

INITIALIZE(SsConverter2_EXTEND_SRCS)

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

if(NOT TARGET SsConverter2Lib)
    add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../lib ${PROJECT_NAME}_lib)
endif()

set(SsConverter2_SRCS
        SsConverter2.cpp
)

set(SsConverter2_HEADERS
)

include_directories(${CMAKE_CURRENT_SOURCE_DIR})
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../lib)

add_executable(${PROJECT_NAME} ${SsConverter2_SRCS} ${SsConverter2_HEADERS} ${SsConverter2_EXTEND_SRCS})
target_link_libraries(${PROJECT_NAME} PRIVATE SsConverter2Lib)
