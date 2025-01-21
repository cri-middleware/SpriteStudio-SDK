include(${CMAKE_CURRENT_SOURCE_DIR}/../cmake/initialize.cmake)
CMAKE_MINIMUM_REQUIRED(VERSION 3.25)
project(SsConverter3Lib)

INITIALIZE(SsConverter3Lib_EXTEND_SRCS)

PREPARE()

CCACHE()

# pugixml
set(PugiXml_SRCS
        src/pugixml/pugixml.cpp
)
set(PugiXml_HEADERS
        src/pugixml/pugixml.hpp
        src/pugixml/pugiconfig.hpp
)

# flatbuffers
set(flatbuffers_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src/flatbuffers)
set(flatbuffers_SRCS
        ${flatbuffers_SOURCE_DIR}/src/idl_parser.cpp
        ${flatbuffers_SOURCE_DIR}/src/idl_gen_text.cpp
        ${flatbuffers_SOURCE_DIR}/src/reflection.cpp
        ${flatbuffers_SOURCE_DIR}/src/util.cpp
)
set(flatbuffers_HEADERS
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/allocator.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/array.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/base.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/buffer.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/buffer_ref.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/code_generator.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/code_generators.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/default_allocator.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/detached_buffer.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/flatbuffer_builder.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/flatbuffers.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/flatc.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/flex_flat_util.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/flexbuffers.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/grpc.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/hash.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/idl.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/minireflect.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/pch/flatc_pch.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/pch/pch.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/reflection.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/reflection_generated.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/registry.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/stl_emulation.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/string.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/struct.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/table.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/util.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/vector.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/vector_downward.h
        ${flatbuffers_SOURCE_DIR}/include/flatbuffers/verifier.h
)
include_directories(${flatbuffers_SOURCE_DIR}/include)

set(PROJECTDEPS_SRCS
        ${PugiXml_SRCS}
        ${flatbuffers_SRCS}
)

set(PROJECTDEPS_HEADERS
        ${PugiXml_HEADERS}
        ${flatbuffers_HEADERS}
)

set(SsConverter3Lib_SRCS
        src/SSConverter3Lib.cpp
        src/sspj.cpp
        src/ssse.cpp
        src/fnt.cpp
        src/ssce.cpp
        src/ssae.cpp
        src/ssee.cpp
        src/ssqe.cpp
        src/util.cpp
        src/util.h
        src/error.cpp
        src/error.h
        src/simpleSemaphore.cpp
        src/simpleSemaphore.h
        src/phase.cpp
        src/phase.h

        src/util/Generator.h
        src/util/XorShift32.cpp
        src/util/XorShift32.h
)

set(SsConverter3Lib_HEADERS
        include/SSConverter3Lib/SSConverter3Lib.h
        include/SSConverter3Lib/version.h

        include/SSConverter3Lib/sspj.h
        include/SSConverter3Lib/ssae.h
        include/SSConverter3Lib/ssce.h
        include/SSConverter3Lib/ssee.h
        include/SSConverter3Lib/ssqe.h
        include/SSConverter3Lib/ssse.h
        include/SSConverter3Lib/fnt.h

        include/SSConverter3Lib/ssfb2/type_base_generated.h
        include/SSConverter3Lib/ssfb2/Data/common_ssfb2_generated.h
        include/SSConverter3Lib/ssfb2/Data/ssc2_generated.h
        include/SSConverter3Lib/ssfb2/Data/ssf2_generated.h
        include/SSConverter3Lib/ssfb2/Data/ssq2_generated.h
        include/SSConverter3Lib/ssfb2/Data/ssa2_generated.h
        include/SSConverter3Lib/ssfb2/Data/sse2_generated.h
        include/SSConverter3Lib/ssfb2/Data/ssfb2_generated.h
        include/SSConverter3Lib/ssfb2/Data/sss2_generated.h
)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)
add_library(${PROJECT_NAME} ${SsConverter3Lib_SRCS} ${SsConverter3Lib_HEADERS} ${SsConverter3Lib_EXTEND_SRCS} ${PROJECTDEPS_SRCS} ${PROJECTDEPS_HEADERS})
target_include_directories(${PROJECT_NAME} PUBLIC include)
target_include_directories(${PROJECT_NAME} PUBLIC ${flatbuffers_SOURCE_DIR}/include)

if (ENABLE_CTEST)
    INITIALIZE_TEST()

    enable_testing()

    # Create an executable target for your unit test
    add_executable(lib_tests tests/lib_tests.cpp)

    # Link your library and Catch2 to the test executable
    target_link_libraries(lib_tests PRIVATE Catch2::Catch2WithMain)
    target_link_libraries(lib_tests PRIVATE ${PROJECT_NAME})

    # Define the test and specify the command to run
    add_test(NAME converterTests COMMAND lib_tests)
endif()
