//
// Created by davide on 6/5/19.
//

#include "disassembler.hpp"
#include <cstring>
#include <sstream>

Disassembler::Disassembler(const char* bin_path) : exec_arch(UNKNOWN)
{
    Disassembler::binary = bin_path;
}

Architecture Disassembler::get_arch() const
{
    return exec_arch;
}

std::set<Function> Disassembler::get_function_names() const
{
    return function_names;
}

void Disassembler::set_binary(const char* bin_path)
{
    exec_arch = UNKNOWN;
    function_names.clear();
    function_bodies.clear();
    Disassembler::binary = bin_path;
}

std::string Disassembler::get_function_as_string(const std::string& name) const
{
    std::stringstream sstr;
    std::unordered_map<std::string, std::vector<Statement>>::const_iterator
        got = function_bodies.find(name);

    if(got != function_bodies.end())
    {
        sstr << name;
        const std::vector<Statement> stmts = got->second;
        for(const Statement& stmt : stmts)
        {
            sstr << stmt.get_offset() << " " << stmt.get_mnemonic() << '\n';
        }
    }
    else
    {
        sstr << "";
    }
    return sstr.str();
}

const std::vector<Statement>*
Disassembler::get_function_body(const std::string& name) const
{
    std::unordered_map<std::string, std::vector<Statement>>::const_iterator
        got = function_bodies.find(name);

    if(got != function_bodies.end())
    {
        return &(got->second);
    }
    return nullptr;
}

std::ostream& operator<<(std::ostream& stream, const Disassembler& disasm)
{
    // std::endl also flushes the stream
    std::string endline("\n");
    std::string tab("\t");

    stream << "--- " << disasm.binary << " ---" << endline;
    for(const Function& fn : disasm.function_names)
    {
        const std::string& fn_name = fn.get_name();
        stream << fn_name << endline;
        std::unordered_map<std::string, std::vector<Statement>>::const_iterator
            got = disasm.function_bodies.find(fn_name);
        if(got != disasm.function_bodies.end())
        {
            const std::vector<Statement>* stmts = &(got->second);
            for(const Statement& stmt : *stmts)
            {
                stream << "|0x" << std::hex << std::uppercase
                       << stmt.get_offset() << std::nouppercase << tab
                       << stmt.get_mnemonic() << endline;
            }
        }
        stream << ';' << endline << endline;
    }
    stream << "----";
    for(unsigned int i = 0; i < disasm.binary.length(); i++)
    {
        stream << '-';
    }
    stream << "----";
    return stream;
}
