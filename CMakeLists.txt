CMAKE_MINIMUM_REQUIRED(VERSION 3.10 FATAL_ERROR)
IF (NOT CMAKE_BUILD_TYPE)
    SET(CMAKE_BUILD_TYPE "Release")
ENDIF ()
MESSAGE("Build: ${CMAKE_BUILD_TYPE}")
PROJECT(BCC VERSION 1.0 LANGUAGES CXX)
SET(CMAKE_CXX_STANDARD 11)
SET(CMAKE_CXX_STANDARD_REQUIRED ON)
SET(CMAKE_VERBOSE_MAKEFILE ON)
SET(CMAKE_CXX_OUTPUT_EXTENSION_REPLACE ON)
SET(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_SOURCE_DIR}/cmake/Modules/")

#MAIN SOURCES
SET(BCC_MAIN_SOURCES
        ${PROJECT_SOURCE_DIR}/src/disassembler/disassembler.cpp
        ${PROJECT_SOURCE_DIR}/src/disassembler/function.cpp
        ${PROJECT_SOURCE_DIR}/src/disassembler/info.cpp
        ${PROJECT_SOURCE_DIR}/src/disassembler/statement.cpp
        ${PROJECT_SOURCE_DIR}/src/architectures/architecture_x86.cpp
        ${PROJECT_SOURCE_DIR}/src/architectures/architecture_arm.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/analysis.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/abstract_block.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/acyclic_block.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/basic_block.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/cyclic_block.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/cfg.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/cfs.cpp
        ${PROJECT_SOURCE_DIR}/src/analysis/comparison.cpp
        )

#TEST SOURCES
SET(BCC_TEST_SOURCES
        ${PROJECT_SOURCE_DIR}/test/disassembler/statement_tests.cpp
        ${PROJECT_SOURCE_DIR}/test/analysis/analysis_tests.cpp
        ${PROJECT_SOURCE_DIR}/test/analysis/blocks_tests.cpp
        ${PROJECT_SOURCE_DIR}/test/analysis/architecture_tests.cpp
        ${PROJECT_SOURCE_DIR}/test/analysis/cfg_tests.cpp
        ${PROJECT_SOURCE_DIR}/test/analysis/cfs_tests.cpp
        ${PROJECT_SOURCE_DIR}/test/analysis/comparison_tests.cpp
        )

LIST(APPEND ALL_REQUIRED_INCLUDES ${PROJECT_SOURCE_DIR}/src)

#COMPILER FLAGS
SET(DEBUG_FLAGS -O0 -g -Wall -Wextra -Wpedantic -Werror -Wno-unused-function)
SET(RELEASE_FLAGS -w -O3 -march=native)

#FIND DISASSEMBLERS AND POPULATE THE DISASM LIST
FIND_PROGRAM(RADARE2 NAMES "r2" "radare2")
IF (RADARE2)
    LIST(APPEND ALL_REQUIRED_DEFS RADARE2_PATH="${RADARE2}")
    LIST(APPEND DISASM_MAIN_SOURCES
            ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_json_parser.cpp
            ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_pipe.cpp
            ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_disassembler.cpp
            )
    LIST(APPEND DISASM_TEST_SOURCES
            ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_disassembler_tests.cpp
            ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_pipe_tests.cpp
            ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_response_tests.cpp
            )
ENDIF ()

FIND_PACKAGE(Threads REQUIRED)
IF (Threads_FOUND)
    LIST(APPEND ALL_REQUIRED_LIBS ${CMAKE_THREAD_LIBS_INIT})
ENDIF ()

#CHECK IF SOME DISASSEMBLERS HAVE BEEN FOUND OR DIE
IF (NOT DISASM_MAIN_SOURCES)
    MESSAGE(FATAL_ERROR "Not a single supported disassembler could be found")
ENDIF ()

#JSON LIB CONFIGURATION
IF (NOT EXISTS ${PROJECT_SOURCE_DIR}/libs/json/CMakeLists.txt)
    MESSAGE(FATAL_ERROR "The libs folder is empty. Please run `git submodule init` and `git submodule update`")
ELSE ()
    SET(JSON_BuildTests OFF CACHE INTERNAL "")
    SET(JSON_MultipleHeaders ON CACHE INTERNAL "")
    ADD_SUBDIRECTORY(${PROJECT_SOURCE_DIR}/libs/json)
    LIST(APPEND ALL_REQUIRED_LIBS nlohmann_json::nlohmann_json)
ENDIF ()

FIND_PROGRAM(GCOVR NAMES "gcovr")
IF (GCOVR)
    #ALSO GATHER COVERAGE IF IN DEBUG MODE
    LIST(APPEND ALL_REQUIRED_LIBS $<$<CONFIG:Debug>:gcov>)
    SET(DEBUG_FLAGS ${DEBUG_FLAGS} $<$<CONFIG:Debug>:--coverage>)
ENDIF ()

#LINK ALL LIBRARIES TO TESTS, IF TESTS ARE ENABLED
FIND_PACKAGE(GTest)
IF (GTEST_FOUND)
    SET(BUILD_TESTS TRUE)
    LIST(APPEND TEST_REQUIRED_LIBS ${GTEST_BOTH_LIBRARIES})
    LIST(APPEND TEST_REQUIRED_LIBS ${CMAKE_THREAD_LIBS_INIT})
    LIST(APPEND TEST_REQUIRED_INCLUDES ${GTEST_INCLUDE_DIRS})
ELSEIF (NOT GTEST_FOUND AND EXISTS ${PROJECT_SOURCE_DIR}/libs/googletest/CMakeLists.txt)
    MESSAGE("-- Found GTest submodule. Building from sources")
    ADD_SUBDIRECTORY(${PROJECT_SOURCE_DIR}/libs/googletest)
    SET(BUILD_TESTS TRUE)
    LIST(APPEND TEST_REQUIRED_LIBS gtest)
    LIST(APPEND TEST_REQUIRED_INCLUDES ${PROJECT_SOURCE_DIR}/libs/googletest/googletest/include)
ENDIF ()
IF (BUILD_TESTS)
    ENABLE_TESTING()
    INCLUDE_DIRECTORIES(${GTEST_INCLUDE_DIRS})
    ADD_EXECUTABLE(runtests EXCLUDE_FROM_ALL
            ${PROJECT_SOURCE_DIR}/test/main_test.cpp
            ${BCC_MAIN_SOURCES}
            ${BCC_TEST_SOURCES}
            ${DISASM_MAIN_SOURCES}
            ${DISASM_TEST_SOURCES})
    TARGET_COMPILE_DEFINITIONS(runtests PUBLIC TESTS_DIR="${PROJECT_SOURCE_DIR}/test/")
    TARGET_INCLUDE_DIRECTORIES(runtests PUBLIC ${ALL_REQUIRED_INCLUDES})
    TARGET_COMPILE_DEFINITIONS(runtests PUBLIC ${ALL_REQUIRED_DEFS})
    TARGET_INCLUDE_DIRECTORIES(runtests PUBLIC ${TEST_REQUIRED_INCLUDES})
    TARGET_COMPILE_OPTIONS(runtests PUBLIC $<$<CONFIG:Debug>:${DEBUG_FLAGS}>)
    TARGET_LINK_LIBRARIES(runtests ${ALL_REQUIRED_LIBS} ${TEST_REQUIRED_LIBS})
    GTEST_DISCOVER_TESTS(runtests AUTO)
ENDIF ()

#CONFIGURE MEMCHECK (BOTH DEBUG AND RELEASE)
FIND_PROGRAM(VALGRIND NAMES "valgrind")
IF (VALGRIND AND BUILD_TESTS)
    SET(VALGRIND_FLAGS --trace-children=no --leak-check=full --show-leak-kinds=all --error-exitcode=1 --track-origins=yes)
    ADD_CUSTOM_TARGET(memcheck
            COMMAND ${VALGRIND} ${VALGRIND_FLAGS} ./runtests --gtest_filter=*-Analyser.max_time)
    ADD_DEPENDENCIES(memcheck runtests)
ENDIF ()

#CONFIGURE COVERAGE (TARGET EXISTS IN BOTH BUT WORKS PROPERLY ONLY IN DEBUG)
IF (GCOVR AND BUILD_TESTS)
    ADD_CUSTOM_COMMAND(OUTPUT run_gcovr DEPENDS runtests
            POST_BUILD
            COMMAND ${GCOVR} -r ${PROJECT_SOURCE_DIR}/src --object-directory=${PROJECT_BINARY_DIR}
            WORKING_DIRECTORY ${PROJECT_BINARY_DIR})
    ADD_CUSTOM_COMMAND(OUTPUT gcovr_html DEPENDS runtests
            POST_BUILD
            COMMAND mkdir -p ${PROJECT_BINARY_DIR}/coverage/
            COMMAND ${GCOVR} -r ${PROJECT_SOURCE_DIR}/src --object-directory=${PROJECT_BINARY_DIR} --branches --exclude-unreachable-branches --html --html-details -o ${PROJECT_BINARY_DIR}/coverage/coverage.html
            WORKING_DIRECTORY ${PROJECT_BINARY_DIR})
    ADD_CUSTOM_TARGET(coverage DEPENDS run_gcovr)
    ADD_CUSTOM_TARGET(report DEPENDS gcovr_html)
ENDIF ()

#MAIN PROGRAM
ADD_EXECUTABLE(analyzer
        ${CMAKE_CURRENT_SOURCE_DIR}/src/main.cpp
        ${BCC_MAIN_SOURCES}
        ${DISASM_MAIN_SOURCES}
        )
TARGET_COMPILE_OPTIONS(analyzer PUBLIC $<$<CONFIG:Debug>:${DEBUG_FLAGS}>)
TARGET_COMPILE_OPTIONS(analyzer PUBLIC $<$<CONFIG:Release>:${RELEASE_FLAGS}>)
TARGET_INCLUDE_DIRECTORIES(analyzer PUBLIC ${ALL_REQUIRED_INCLUDES})
TARGET_LINK_LIBRARIES(analyzer PUBLIC ${ALL_REQUIRED_LIBS})
TARGET_COMPILE_DEFINITIONS(analyzer PUBLIC ${ALL_REQUIRED_DEFS})
