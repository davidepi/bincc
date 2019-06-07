#.rst:
# FindBAP
# --------
#
# Find the BinaryAnalysisPlatform disassembler (BAP)
#
# IMPORTED Targets
# ^^^^^^^^^^^^^^^^
#
# This module defines the :prop_tgt:`IMPORTED` target ``BAP::BAP``,
# if BAP has been found.
#
# Result Variables
# ^^^^^^^^^^^^^^^^
#
# This module defines the following variables:
#
# ::
#
#   BAP_INCLUDE_DIRS - include directories for BAP
#   BAP_LIBRARIES - libraries to link against BAP
#   BAP_FOUND - true if BAP has been found and can be used

#=============================================================================
# Copyright 2012 Benjamin Eikel
# Copyright 2016 Ryan Pavlik
# Copyright 2019 Davide Pizzolotto
#
# Distributed under the OSI-approved BSD License (the "License");
# see below.
#
# This software is distributed WITHOUT ANY WARRANTY; without even the
# implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
# See the License for more information.
#=============================================================================
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
# * Redistributions of source code must retain the above copyright
#   notice, this list of conditions and the following disclaimer.
#
# * Redistributions in binary form must reproduce the above copyright
#   notice, this list of conditions and the following disclaimer in the
#   documentation and/or other materials provided with the distribution.
#
# * Neither the names of Kitware, Inc., the Insight Software Consortium,
#   nor the names of their contributors may be used to endorse or promote
#   products derived from this software without specific prior written
#   permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#=============================================================================

find_path(BAP_INCLUDE_DIR bap.h)
if(WIN32)
    if(CMAKE_SIZEOF_VOID_P EQUAL 4)
        set(BAP_ARCH Win32)
    else()
        set(BAP_ARCH x64)
    endif()
    set(BAP_EXTRA_SUFFIXES lib/Release/${BAP_ARCH} bin/Release/${BAP_ARCH})
endif()

if(WIN32 AND BAP_INCLUDE_DIR)
    get_filename_component(BAP_LIB_ROOT_CANDIDATE "${BAP_INCLUDE_DIR}/.." ABSOLUTE)
endif()

find_library(BAP_LIBRARY
             NAMES bap
             PATH_SUFFIXES lib64 ${BAP_EXTRA_SUFFIXES}
             HINTS "${BAP_LIB_ROOT_CANDIDATE}")

if(WIN32 AND BAP_LIBRARY AND NOT BAP_LIBRARY MATCHES ".*s.lib")
    get_filename_component(BAP_LIB_DIR "${BAP_LIBRARY}" DIRECTORY)
    get_filename_component(BAP_BIN_ROOT_CANDIDATE1 "${BAP_LIB_DIR}/.." ABSOLUTE)
    get_filename_component(BAP_BIN_ROOT_CANDIDATE2 "${BAP_LIB_DIR}/../../.." ABSOLUTE)
    find_file(BAP_RUNTIME_LIBRARY
              NAMES bap32.dll
              PATH_SUFFIXES bin ${BAP_EXTRA_SUFFIXES}
              HINTS
              "${BAP_BIN_ROOT_CANDIDATE1}"
              "${BAP_BIN_ROOT_CANDIDATE2}")
endif()

set(BAP_INCLUDE_DIRS ${BAP_INCLUDE_DIR})
set(BAP_LIBRARIES ${BAP_LIBRARY})

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(BAP
                                  REQUIRED_VARS BAP_INCLUDE_DIR BAP_LIBRARY)

if(BAP_FOUND AND NOT TARGET BAP::BAP)
    if(WIN32 AND BAP_LIBRARY MATCHES ".*s.lib")
        # Windows, known static library.
        add_library(BAP::BAP STATIC IMPORTED)
        set_target_properties(BAP::BAP PROPERTIES
                              IMPORTED_LOCATION "${BAP_LIBRARY}"
                              PROPERTY INTERFACE_COMPILE_DEFINITIONS BAP_STATIC)

    elseif(WIN32 AND BAP_RUNTIME_LIBRARY)
        # Windows, known dynamic library and we have both pieces
        # TODO might be different for mingw
        add_library(BAP::BAP SHARED IMPORTED)
        set_target_properties(BAP::BAP PROPERTIES
                              IMPORTED_LOCATION "${BAP_RUNTIME_LIBRARY}"
                              IMPORTED_IMPLIB "${BAP_LIBRARY}")
    else()

        # Anything else - previous behavior.
        add_library(BAP::BAP UNKNOWN IMPORTED)
        set_target_properties(BAP::BAP PROPERTIES
                              IMPORTED_LOCATION "${BAP_LIBRARY}")
    endif()

    set_target_properties(BAP::BAP PROPERTIES
                          INTERFACE_INCLUDE_DIRECTORIES "${BAP_INCLUDE_DIRS}")

endif()

mark_as_advanced(BAP_INCLUDE_DIR BAP_LIBRARY BAP_RUNTIME_LIBRARY)
