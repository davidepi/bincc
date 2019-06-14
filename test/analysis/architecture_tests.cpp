//
// Created by davide on 6/13/19.
//
#include "architectures/architecture_x86.hpp"
#include <gtest/gtest.h>


TEST(Architecture, UNK_is_jump)
{
    ArchitectureUNK arch;

    std::string mne = "b";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jmp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
}

TEST(Architecture, UNK_is_return)
{
    ArchitectureUNK arch;

    std::string mne = "ret";
    EXPECT_FALSE(arch.is_return(mne));
    mne = "bx lr";
    EXPECT_FALSE(arch.is_return(mne));
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
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnl";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "mul";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jnbe";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jl";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jcxz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnc";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jb";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jno";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jg";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "div";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jge";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jng";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jns";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jpe";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jle";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jmp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::UNCONDITIONAL);
    mne = "jna";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jne";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "min";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jnae";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "rsqrt";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "je";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "ja";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnle";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnb";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jc";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jae";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jpo";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "max";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "jnge";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jbe";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jecxz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "sqrt";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "sub";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "js";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jz";
    EXPECT_EQ(arch.is_jump(mne), JumpType::CONDITIONAL);
    mne = "rcp";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
    mne = "add";
    EXPECT_EQ(arch.is_jump(mne), JumpType::NONE);
}

TEST(Architecture, X86_is_return)
{
    ArchitectureX86 arch;

    std::string mne = "bx lr";
    EXPECT_FALSE(arch.is_return(mne));
    mne = "ret";
    EXPECT_TRUE(arch.is_return(mne));
}

TEST(Architecture, X86_get_name)
{
    ArchitectureX86 arch;

    EXPECT_STREQ(arch.get_name().c_str(), "x86");
}
