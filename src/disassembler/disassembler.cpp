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

std::vector<std::string>
Disassembler::get_function_body(const std::string& name) const
{
    std::unordered_map<std::string, std::vector<std::string>>::const_iterator
        got = function_bodies.find(name);

    if(got != function_bodies.end())
    {
        return got->second;
    }
    return std::vector<std::string>();
}
