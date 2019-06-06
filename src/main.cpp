#include "disassembler/radare2/r2_disassembler.hpp"
#include "unistd.h"
#include <iostream>

static void fatal(const char* message)
{
    fprintf(stderr, "%s\n", message);
    exit(EXIT_FAILURE);
}

int main(int argc, const char* argv[])
{
    if(argc != 2)
    {
        fatal("Usage: ./analyze <path_to_binary>");
    }
    if(access(argv[1], R_OK) == -1)
    {
        fatal("Input file does not exists or is not readable");
    }
    Disassembler* disasm = new DisassemblerR2(argv[1]);
    disasm->analyse();
    std::set<std::string> functions = disasm->get_function_names();
    for(const std::string& name : functions)
    {
        std::cout << name << "\n";
        std::vector<std::string> stmts = disasm->get_function_body(name);
        for(const std::string& stmt : stmts)
        {
            std::cout << "    " << stmt << "\n";
        }
        std::cout << std::endl;
    }
}
