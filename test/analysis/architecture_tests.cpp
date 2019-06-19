//
// Created by davide on 6/13/19.
//
#include "architectures/architecture_x86.hpp"
#include "architectures/architecture_arm.hpp"
#include <gtest/gtest.h>


TEST(Architecture, UNK_is_jump)
{
    ArchitectureUNK arch;

    std::string mne = "b";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jmp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
}

TEST(Architecture, UNK_get_name)
{
    ArchitectureUNK arch;

    EXPECT_STREQ(arch.get_name().c_str(), "unknown");
}


TEST(Architecture, X86_is_jump)
{
    ArchitectureX86 arch;

    std::string mne = "jo";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jnl";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "mul";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jnbe";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jl";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jcxz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jnc";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jb";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jno";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jg";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "div";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jge";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jng";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jns";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jnz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jpe";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jle";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jmp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_UNCONDITIONAL);
    mne = "jna";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jne";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "min";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jnae";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jnp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "rsqrt";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "je";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "ja";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jnle";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jnb";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jc";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jae";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jpo";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "max";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jnge";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jbe";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jecxz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "sqrt";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "sub";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "js";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "jz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "rcp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "add";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "bx lr";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "ret";
    EXPECT_EQ(arch.is_jump(mne), JumpType::RET_UNCONDITIONAL);
}

TEST(Architecture, X86_get_name)
{
    ArchitectureX86 arch;

    EXPECT_STREQ(arch.get_name().c_str(), "x86");
}

TEST(Architecture, ARM_is_jump)
{
    ArchitectureARM arch;

    std::string mne;
    mne = "beq";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bne";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bcs";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bhs";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bcc";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "blo";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bmi";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bpl";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bvs";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bvc";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bhi";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bls";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bge";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "bgt";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "blt";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "ble";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_CONDITIONAL);
    mne = "b";
    EXPECT_EQ(arch.is_jump(mne), JumpType::JUMP_UNCONDITIONAL);
    mne = "bl";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "bxle";
    EXPECT_EQ(arch.is_jump(mne), JumpType::RET_CONDITIONAL);
    mne = "bx";
    EXPECT_EQ(arch.is_jump(mne), JumpType::RET_UNCONDITIONAL);
    mne = "ret";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
}

TEST(Architecture, ARM_get_name)
{
    ArchitectureARM arch;

    EXPECT_STREQ(arch.get_name().c_str(), "arm");
}
