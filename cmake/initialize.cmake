include_guard()

set(MACROS_DIR ${CMAKE_CURRENT_LIST_DIR})

macro(OPTIONS)
    # ===================================================
    option(ENABLE_CCACHE "Enable ccache?" ON)
    option(ENABLE_CTEST "Enable ctest?" OFF)

    if(MSVC)
        set(DEFAULT_STATIC_OPTION YES)
    else()
        set(DEFAULT_STATIC_OPTION NO)
    endif()
    option(ENABLE_STATIC "static link" ${DEFAULT_STATIC_OPTION})

    if(WIN32 OR WIN64)
        if(ENABLE_CTEST)
            set(ENABLE_STATIC ON)
        endif()
    endif()
endmacro()

macro(PREPARE)
    if(MSVC)
        add_compile_options("$<$<C_COMPILER_ID:MSVC>:/utf-8>")
        add_compile_options("$<$<CXX_COMPILER_ID:MSVC>:/utf-8>")
    elseif (APPLE)
        if((CMAKE_BUILD_TYPE STREQUAL "Release") AND (NOT CMAKE_OSX_ARCHITECTURES))
            set(CMAKE_OSX_ARCHITECTURES "arm64;x86_64" CACHE STRING "" FORCE)
        endif()
        set(CMAKE_OSX_DEPLOYMENT_TARGET "10.15")
    elseif(CMAKE_COMPILER_IS_GNUCXX)
        add_definitions(-std=gnu++14)
    endif()
endmacro()

macro(CCACHE)
    find_program(CCACHE_EXE sccache)
    if(CCACHE_EXE)
        if(ENABLE_CCACHE AND (NOT SPRITESTUDIO_SDK_CCACHE_ALREADY_SET_FLAG))
            message(STATUS "Enable ccache")
            if(CMAKE_C_COMPILER_LAUNCHER)
                set(CMAKE_C_COMPILER_LAUNCHER "${CMAKE_C_COMPILER_LAUNCHER}" "${CCACHE_EXE}")
            else()
                set(CMAKE_C_COMPILER_LAUNCHER "${CCACHE_EXE}")
            endif()
            if(CMAKE_CXX_COMPILER_LAUNCHER)
                set(CMAKE_CXX_COMPILER_LAUNCHER "${CMAKE_CXX_COMPILER_LAUNCHER}" "${CCACHE_EXE}")
            else()
                set(CMAKE_CXX_COMPILER_LAUNCHER "${CCACHE_EXE}")
            endif()

            if (MSVC)
                set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT Embedded)
                cmake_policy(SET CMP0141 NEW)
            endif()

            set(SPRITESTUDIO_SDK_CCACHE_ALREADY_SET_FLAG ON)
        endif()
    endif()
endmacro()

macro(INITIALIZE EXTEND_SOURCE_VAR)
    set(CMAKE_CXX_STANDARD 17)
    set(CMAKE_CXX_STANDARD_REQUIRED ON)
    set(CMAKE_CXX_EXTENSIONS OFF)

    set(EXTEND_SOURCE_VAR )
    # Platforms
    if(APPLE)
        if(CMAKE_BUILD_TYPE STREQUAL "Release")
            set(CMAKE_OSX_ARCHITECTURES "arm64;x86_64")
        endif()
    elseif(WIN32 OR WIN64)
        #if (NOT ENABLE_STATIC)
        #    list(APPEND EXTEND_SOURCE_VAR ../utils/src/dllmain.c)
        #endif()
    endif()

endmacro()

macro(INITIALIZE_TEST)
    include(FetchContent)
    FetchContent_Declare(
            Catch2
            GIT_REPOSITORY https://github.com/catchorg/Catch2.git
            GIT_TAG        v3.7.0
    )
    FetchContent_MakeAvailable(Catch2)

    # test data directory setting

    get_filename_component(TEST_ROOT_DIR ${MACROS_DIR} DIRECTORY)
    set(TEST_DATA_DIR "${TEST_ROOT_DIR}/test_data")

    file(GLOB_RECURSE SSPJ_FOUND_FILES "${TEST_DATA_DIR}/*.sspj")
    set(SSPJ_FILE_LIST "")
    foreach(FILE_PATH ${SSPJ_FOUND_FILES})
        file(RELATIVE_PATH RELATIVE_PATH "${TEST_DATA_DIR}" "${FILE_PATH}")
        set(SSPJ_FILE_LIST "${SSPJ_FILE_LIST}\"${RELATIVE_PATH}\",\n")
    endforeach()

    configure_file(${MACROS_DIR}/test_config.h.in tests/test_config.h @ONLY)
    include_directories(${CMAKE_CURRENT_BINARY_DIR}/tests)
    include_directories(${MACROS_DIR})
endmacro()
