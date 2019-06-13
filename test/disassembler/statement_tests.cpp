//
// Created by davide on 6/13/19.
//

#include "disassembler/statement.hpp"
#include <gtest/gtest.h>

TEST(Statement, default_ctor)
{
    Statement stmt;
    EXPECT_EQ(stmt.get_offset(), 0);
    EXPECT_STREQ(stmt.get_command().c_str(), "");
    EXPECT_STREQ(stmt.get_mnemonic().c_str(), "");
    EXPECT_STREQ(stmt.get_args().c_str(), "");
}

TEST(Statement, no_args)
{
    Statement stmt(1552, "ret");
    EXPECT_EQ(stmt.get_offset(), 0x610);
    EXPECT_STREQ(stmt.get_command().c_str(), "ret");
    EXPECT_STREQ(stmt.get_mnemonic().c_str(), "ret");
    EXPECT_STREQ(stmt.get_args().c_str(), "");
}

TEST(Statement, multi_args)
{
    Statement stmt(0x5341A5, "mov r9d, dword [rsp + r10 + 0x20]");
    EXPECT_EQ(stmt.get_offset(), 5456293);
    EXPECT_STREQ(stmt.get_command().c_str(),
                 "mov r9d, dword [rsp + r10 + 0x20]");
    EXPECT_STREQ(stmt.get_mnemonic().c_str(), "mov");
    EXPECT_STREQ(stmt.get_args().c_str(), "r9d, dword [rsp + r10 + 0x20]");
}

TEST(Statement, to_lowercase)
{
    Statement stmt(0x5667, "CMP RAX, r8");
    EXPECT_EQ(stmt.get_offset(), 0x5667);
    EXPECT_STREQ(stmt.get_command().c_str(), "cmp rax, r8");
    EXPECT_STREQ(stmt.get_mnemonic().c_str(), "cmp");
    EXPECT_STREQ(stmt.get_args().c_str(), "rax, r8");
}
