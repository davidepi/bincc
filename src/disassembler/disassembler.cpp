//
// Created by davide on 6/5/19.
//

#include "disassembler.hpp"
#include <cstring>

Disassembler::Disassembler(const char* bin_path) : exec_arch(UNKNOWN)
{
    Disassembler::binary = bin_path;
}

Architecture Disassembler::get_arch() const
{
    return exec_arch;
}

std::set<std::string> Disassembler::get_function_names() const
{
    return function_names;
}

void Disassembler::set_binary(const char* bin_path)
{
    exec_arch = UNKNOWN;
    function_names.clear();
    Disassembler::binary = bin_path;
}
