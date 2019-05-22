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
    set(JSON_BuildTests OFF CACHE INTERNAL "")
    ADD_SUBDIRECTORY(${PROJECT_SOURCE_DIR}/lib/json)
    TARGET_LINK_LIBRARIES(analyzer PRIVATE nlohmann_json::nlohmann_json)
ENDIF()

FIND_PACKAGE(GTest)
IF(GTEST_FOUND)
    ENABLE_TESTING()
    INCLUDE_DIRECTORIES(${GTEST_INCLUDE_DIRS})
    ADD_EXECUTABLE(runTests EXCLUDE_FROM_ALL
                   ${CMAKE_CURRENT_SOURCE_DIR}/test/main_test.cpp
                   ${bcc_main_sources}
                   ${bcc_test_sources})
    TARGET_LINK_LIBRARIES(runTests
                          ${GTEST_BOTH_LIBRARIES}
                          ${CMAKE_THREAD_LIBS_INIT})
    GTEST_DISCOVER_TESTS(runTests AUTO)
ENDIF()
