//
// Created by davide on 6/13/19.
//

#include "analysis/analysis_x86.hpp"
#include <gtest/gtest.h>

TEST(Analysis, X86_is_jump)
{
    AnalysisX86 anal(nullptr);

   std::string mne = "jo";
   EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnl";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "mul";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "jnbe";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jl";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jcxz";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnc";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jb";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jno";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jp";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jg";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "div";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "jge";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jng";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jns";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnz";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jpe";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jle";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jmp";
    EXPECT_EQ(anal.is_jump(mne), JumpType::UNCONDITIONAL);
    mne = "jna";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jne";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "min";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "jnae";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnp";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "rsqrt";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "je";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "ja";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnle";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jnb";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jc";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jae";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jpo";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "max";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "jnge";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jbe";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jecxz";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "sqrt";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "sub";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "js";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "jz";
    EXPECT_EQ(anal.is_jump(mne), JumpType::CONDITIONAL);
    mne = "rcp";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
    mne = "add";
    EXPECT_EQ(anal.is_jump(mne), JumpType::NONE);
}
