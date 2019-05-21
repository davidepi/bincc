CMAKE_MINIMUM_REQUIRED(VERSION 3.10 FATAL_ERROR)
PROJECT(BCC)
SET(CMAKE_CXX_STANDARD 11)

FILE(GLOB bcc_main_sources src/disassembler/*.cpp)
FILE(GLOB bcc_main_headers src/disassembler/*.hpp)

INCLUDE_DIRECTORIES(${CMAKE_SOURCE_DIR}/lib)

ADD_EXECUTABLE(analyzer
               ${CMAKE_CURRENT_SOURCE_DIR}/src/main.cpp
               ${bcc_main_sources}
               ${bcc_main_headers})

FIND_PACKAGE(GTest)
IF(GTEST_FOUND)
    ENABLE_TESTING()
    INCLUDE_DIRECTORIES(${GTEST_INCLUDE_DIRS})
    ADD_EXECUTABLE(runTests EXCLUDE_FROM_ALL
                   ${CMAKE_CURRENT_SOURCE_DIR}/test/main_test.cpp
                   ${bcc_main_sources})
    TARGET_LINK_LIBRARIES(runTests
                          ${GTEST_BOTH_LIBRARIES}
                          ${CMAKE_THREAD_LIBS_INIT})
    GTEST_DISCOVER_TESTS(runTests AUTO)
ENDIF()
