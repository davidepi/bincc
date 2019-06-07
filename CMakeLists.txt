CMAKE_MINIMUM_REQUIRED(VERSION 3.10 FATAL_ERROR)
PROJECT(BCC VERSION 1.0 LANGUAGES CXX)
SET(CMAKE_CXX_STANDARD 11)
SET(CMAKE_VERBOSE_MAKEFILE ON)

#MAIN SOURCES
SET(BCC_MAIN_SOURCES
    ${PROJECT_SOURCE_DIR}/src/disassembler/disassembler.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_disassembler.cpp
    src/disassembler/function.cpp
    src/disassembler/info.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_pipe.cpp
    src/disassembler/statement.cpp
    src/disassembler/radare2/r2_json_parser.cpp src/disassembler/radare2/r2_json_parser.hpp)
#TEST SOURCES
SET(BCC_TEST_SOURCES
    ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_disassembler_tests.cpp
    ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_pipe_tests.cpp
    ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_response_tests.cpp
    )

INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/src)

ADD_EXECUTABLE(analyzer
               ${CMAKE_CURRENT_SOURCE_DIR}/src/main.cpp
               ${BCC_MAIN_SOURCES})

#COMPILER FLAGS
SET(DEBUG_FLAGS
    -Wall
    -Wpedantic
    -Wsuggest-attribute=pure
    -Wsuggest-attribute=const
    -Wsuggest-attribute=noreturn
    -Werror
    -Wfatal-errors
    )
SET(RELEASE_FLAGS
    -w
    -O3
    -flto
    )

TARGET_COMPILE_OPTIONS(analyzer PUBLIC $<$<CONFIG:Debug>:${DEBUG_FLAGS}>)
TARGET_COMPILE_OPTIONS(analyzer PUBLIC $<$<CONFIG:Release>:${RELEASE_FLAGS}>)

#FIND DISASSEMBLERS AND POPULATE THE DISASM LIST
LIST(APPEND DISASM "")
FIND_PROGRAM(RADARE2 NAMES "r2" "radare2")
IF(RADARE2)
    LIST(APPEND DISASM RADARE2_PATH="${RADARE2}")
ENDIF()

#PASS THE DISASSEMBLERS TO THE MAIN PROGRAM OR DIE
IF(NOT DISASM)
    MESSAGE(FATAL_ERROR "Not a single supported disassembler could be found")
ENDIF()

#JSON LIB CONFIGURATION
IF(NOT EXISTS ${PROJECT_SOURCE_DIR}/lib/json)
    MESSAGE(FATAL_ERROR "The lib folder is empty. Please run `git submodule init` and `git submodule update`")
ELSE()
    SET(JSON_BuildTests OFF CACHE INTERNAL "")
    SET(JSON_MultipleHeaders ON CACHE INTERNAL "")
    ADD_SUBDIRECTORY(${PROJECT_SOURCE_DIR}/lib/json)
    LIST(APPEND MAIN_REQUIRED_LIBS nlohmann_json::nlohmann_json)
    LIST(APPEND TEST_REQUIRED_LIBS nlohmann_json::nlohmann_json)
ENDIF()

#LINK ALL LIBRARIES
TARGET_LINK_LIBRARIES(analyzer ${MAIN_REQUIRED_LIBS})
TARGET_COMPILE_DEFINITIONS(analyzer PUBLIC ${DISASM})

#LINK ALL LIBRARIES TO TESTS, IF TESTS ARE ENABLED
FIND_PACKAGE(GTest)
IF(GTEST_FOUND)
    ENABLE_TESTING()
    INCLUDE_DIRECTORIES(${GTEST_INCLUDE_DIRS})
    ADD_EXECUTABLE(runTests EXCLUDE_FROM_ALL
                   ${PROJECT_SOURCE_DIR}/test/main_test.cpp
                   ${BCC_MAIN_SOURCES}
                   ${BCC_TEST_SOURCES})
    TARGET_COMPILE_DEFINITIONS(runTests PUBLIC
                               TESTS_DIR="${PROJECT_SOURCE_DIR}/test/")
    TARGET_COMPILE_DEFINITIONS(runTests PUBLIC ${DISASM})
    LIST(APPEND TEST_REQUIRED_LIBS ${GTEST_BOTH_LIBRARIES})
    LIST(APPEND TEST_REQUIRED_LIBS ${CMAKE_THREAD_LIBS_INIT})
    TARGET_LINK_LIBRARIES(runTests ${TEST_REQUIRED_LIBS})
    GTEST_DISCOVER_TESTS(runTests AUTO)
ENDIF()
