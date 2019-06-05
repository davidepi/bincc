CMAKE_MINIMUM_REQUIRED(VERSION 3.10 FATAL_ERROR)
PROJECT(BCC)
SET(CMAKE_CXX_STANDARD 11)

#MAIN SOURCES
SET(bcc_main_sources
    ${PROJECT_SOURCE_DIR}/src/disassembler/disassembler.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_disassembler.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_func.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_info.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_pipe.cpp
    ${PROJECT_SOURCE_DIR}/src/disassembler/radare2/r2_stmt.cpp
    )
#TEST SOURCES
SET(bcc_test_sources
    ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_disassembler_tests.cpp
    ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_pipe_tests.cpp
    ${PROJECT_SOURCE_DIR}/test/disassembler/radare2/r2_response_tests.cpp
    )

INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/src)

ADD_EXECUTABLE(analyzer
               ${CMAKE_CURRENT_SOURCE_DIR}/src/main.cpp
               ${bcc_main_sources})

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
                   ${bcc_main_sources}
                   ${bcc_test_sources})
    TARGET_COMPILE_DEFINITIONS(runTests PUBLIC TESTS_DIR="${PROJECT_SOURCE_DIR}/test/")
    TARGET_COMPILE_DEFINITIONS(runTests PUBLIC ${DISASM})
    LIST(APPEND TEST_REQUIRED_LIBS ${GTEST_BOTH_LIBRARIES})
    LIST(APPEND TEST_REQUIRED_LIBS ${CMAKE_THREAD_LIBS_INIT})
    TARGET_LINK_LIBRARIES(runTests ${TEST_REQUIRED_LIBS})
    GTEST_DISCOVER_TESTS(runTests AUTO)
ENDIF()
