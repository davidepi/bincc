CMAKE_MINIMUM_REQUIRED(VERSION 3.10 FATAL_ERROR)
PROJECT(BCC)
SET(CMAKE_CXX_STANDARD 11)

FILE(GLOB bcc_main_sources
     ${PROJECT_SOURCE_DIR}/src/**/*.cpp
     )
FILE(GLOB bcc_main_headers
     ${PROJECT_SOURCE_DIR}/src/**/*.hpp
     )
FILE(GLOB bcc_test_sources
     ${PROJECT_SOURCE_DIR}/test/**/*.cpp
     )

INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/lib)
INCLUDE_DIRECTORIES(${PROJECT_SOURCE_DIR}/src)

ADD_EXECUTABLE(analyzer
               ${CMAKE_CURRENT_SOURCE_DIR}/src/main.cpp
               ${bcc_main_sources}
               ${bcc_main_headers})

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

#LINK ALL LIBRARIES TO TESTS, IF TESTS ARE ENABLED
FIND_PACKAGE(GTest)
IF(GTEST_FOUND)
    ENABLE_TESTING()
    INCLUDE_DIRECTORIES(${GTEST_INCLUDE_DIRS})
    ADD_EXECUTABLE(runTests EXCLUDE_FROM_ALL
                   ${CMAKE_CURRENT_SOURCE_DIR}/test/main_test.cpp
                   ${bcc_main_sources}
                   ${bcc_test_sources})
    LIST(APPEND TEST_REQUIRED_LIBS ${GTEST_BOTH_LIBRARIES})
    LIST(APPEND TEST_REQUIRED_LIBS ${CMAKE_THREAD_LIBS_INIT})
    TARGET_LINK_LIBRARIES(runTests ${TEST_REQUIRED_LIBS})
    GTEST_DISCOVER_TESTS(runTests AUTO)
ENDIF()

