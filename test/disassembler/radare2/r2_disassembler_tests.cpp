#include "disassembler/disassembler.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include <gtest/gtest.h>

TEST(Disassembler, R2_arch)
{
    DisassemblerR2 disasm(TESTS_DIR "/resources/ls");
    EXPECT_EQ(disasm.get_arch(), Architecture::UNKNOWN);
    disasm.analyze();
    EXPECT_EQ(disasm.get_arch(), Architecture::X86);
}
