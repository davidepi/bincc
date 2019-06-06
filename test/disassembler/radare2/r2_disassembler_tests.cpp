#include "disassembler/disassembler.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include <gtest/gtest.h>

TEST(Disassembler, R2_arch)
{
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped_x86");
    EXPECT_EQ(disasm.get_arch(), Architecture::UNKNOWN);
    disasm.analyse();
    EXPECT_EQ(disasm.get_arch(), Architecture::X86);
}

TEST(Disassembler, R2_functions)
{
    std::set<std::string> functions;
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped_x86");
    functions = disasm.get_function_names();
    EXPECT_EQ(functions.size(), 0);
    disasm.analyse();
    functions = disasm.get_function_names();
    EXPECT_GT(functions.size(), 0);
}

TEST(Disassembler, R2_function_bodies)
{
    std::vector<std::string> body;
    DisassemblerR2 disasm(TESTS_DIR "resources/ls_unstripped_x86");
    body = disasm.get_function_body("sym.is_colored");
    EXPECT_EQ(body.size(), 0);
    disasm.analyse();
    body = disasm.get_function_body("sym.is_colored");
    EXPECT_GT(body.size(), 0);
    for(const std::string& str : body)
    {
        std::cout<<str<<std::endl;
    }
}
