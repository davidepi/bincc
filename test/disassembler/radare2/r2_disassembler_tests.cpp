#include "disassembler/disassembler.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include <gtest/gtest.h>

TEST(Disassembler, R2_arch)
{
    DisassemblerR2 disasm_x86(TESTS_DIR "resources/add_x86");
    EXPECT_STREQ(disasm_x86.get_arch()->get_name().c_str(), "unknown");
    disasm_x86.analyse();
    EXPECT_STREQ(disasm_x86.get_arch()->get_name().c_str(), "x86");

    DisassemblerR2 disasm_arm(TESTS_DIR "resources/add_arm");
    EXPECT_STREQ(disasm_arm.get_arch()->get_name().c_str(), "unknown");
    disasm_arm.analyse();
    EXPECT_STREQ(disasm_arm.get_arch()->get_name().c_str(), "arm");
}

TEST(Disassembler, R2_functions)
{
    std::set<Function> functions;
    DisassemblerR2 disasm(TESTS_DIR "resources/add_x86");
    functions = disasm.get_function_names();
    EXPECT_EQ(functions.size(), 0);
    disasm.analyse();
    functions = disasm.get_function_names();
    EXPECT_GT(functions.size(), 0);
}

TEST(Disassembler, R2_function_bodies)
{
    std::string body;
    DisassemblerR2 disasm(TESTS_DIR "resources/add_x86");
    body = disasm.get_function_as_string("sym.add_multiple");
    EXPECT_EQ(body.length(), 0);
    disasm.analyse();
    body = disasm.get_function_as_string("sym.add_multiple");
    EXPECT_GT(body.length(), 0);
}

TEST(Disassembler, change_binary)
{
    std::string new_name = TESTS_DIR "resources/add_arm";
    DisassemblerR2 disasm(TESTS_DIR "resources/add_x86");
    disasm.set_binary(new_name.c_str());
    EXPECT_STREQ(disasm.get_binary_name().c_str(), new_name.c_str());
}
