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
    std::cout << *disasm << std::endl;
}
