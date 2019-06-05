//
// Created by davide on 6/5/19.
//

#include "disassembler.hpp"
#include <cstring>

Disassembler::Disassembler(const char* binary):exec_arch(UNKNOWN)
{
    Disassembler::binary = strdup(binary);
}

Architecture Disassembler::get_arch() const
{
    return exec_arch;
}
