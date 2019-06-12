#include "disassembler/disassembler.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include <gtest/gtest.h>

TEST(Disassembler, R2_arch)
{
    DisassemblerR2 disasm_x86(TESTS_DIR "resources/ls_unstripped_x86");
    EXPECT_EQ(disasm_x86.get_arch(), Architecture::UNKNOWN);
    disasm_x86.analyse();
    EXPECT_EQ(disasm_x86.get_arch(), Architecture::X86);

    DisassemblerR2 disasm_arm(TESTS_DIR "resources/ls_unstripped_arm");
    EXPECT_EQ(disasm_arm.get_arch(), Architecture::UNKNOWN);
    disasm_arm.analyse();
    EXPECT_EQ(disasm_arm.get_arch(), Architecture::ARM);
}

TEST(Disassembler, R2_functions)
{
    std::set<Function> functions;
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped_x86");
    functions = disasm.get_function_names();
    EXPECT_EQ(functions.size(), 0);
    disasm.analyse();
    functions = disasm.get_function_names();
    EXPECT_GT(functions.size(), 0);
}

TEST(Disassembler, R2_function_bodies)
{
    std::string body;
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped_x86");
    body = disasm.get_function_as_string("sym.is_colored");
    EXPECT_EQ(body.length(), 0);
    disasm.analyse();
    body = disasm.get_function_as_string("sym.is_colored");
    EXPECT_GT(body.length(), 0);
}
