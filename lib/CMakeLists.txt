include(${CMAKE_CURRENT_SOURCE_DIR}/../cmake/initialize.cmake)
CMAKE_MINIMUM_REQUIRED(VERSION 3.25)
project(SsConverter2Lib)

INITIALIZE(SsConverter2Lib_EXTEND_SRCS)

PREPARE()

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

set(SsConverter2Lib_SRCS
        src/SSConverter2Lib.cpp
        src/sspj.cpp
)

set(SsConverter2Lib_HEADERS
        include/SSConverter2Lib.h
        include/sspj.h
        include/ssae.h
        include/ssce.h
        include/ssee.h
        include/ssqe.h
        include/ssse.h
        include/fnt.h
)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)
add_library(${PROJECT_NAME} ${SsConverter2Lib_SRCS} ${SsConverter2Lib_HEADERS} ${SsConverter2Lib_EXTEND_SRCS} ${PROJECTDEPS_SRCS} ${PROJECTDEPS_HEADERS})
target_include_directories(${PROJECT_NAME} PUBLIC include)
target_include_directories(${PROJECT_NAME} PUBLIC ${flatbuffers_SOURCE_DIR}/include)
