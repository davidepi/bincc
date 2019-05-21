CMAKE_MINIMUM_REQUIRED(VERSION 3.10 FATAL_ERROR)
PROJECT(BCC)
SET(CMAKE_CXX_STANDARD 11)

FILE(GLOB bcc_main_sources src/disassembler/*.cpp)
FILE(GLOB bcc_main_headers src/disassembler/*.hpp)

INCLUDE_DIRECTORIES(${CMAKE_SOURCE_DIR}/lib)

ADD_EXECUTABLE(analyzer ${bcc_main_sources} ${bcc_main_headers})

FIND_PACKAGE(GTest)
IF(GTestFOUND)
ENDIF()
