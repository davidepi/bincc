#include "disassembler/disassembler.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include <gtest/gtest.h>

TEST(Disassembler, R2_arch)
{
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped");
    EXPECT_EQ(disasm.get_arch(), Architecture::UNKNOWN);
    disasm.analyse();
    EXPECT_EQ(disasm.get_arch(), Architecture::X86);
}

TEST(Disassembler, R2_functions)
{
    std::set<std::string> functions;
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped");
    functions = disasm.get_function_names();
    EXPECT_EQ(functions.size(), 0);
    disasm.analyse();
    functions = disasm.get_function_names();
    EXPECT_GT(functions.size(), 0);
}
