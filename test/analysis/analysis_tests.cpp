#include "analysis/analysis_x86.hpp"
#include <gtest/gtest.h>
//
// Created by davide on 6/13/19.
//
TEST(Analysis, Analysis_constructor_empty_string)
{
    // empty string
    std::string func;
    AnalysisX86 anal(func);
    Statement ins = anal[0];
    EXPECT_EQ(ins.get_offset(), 0x0);
    EXPECT_STREQ(ins.get_command().c_str(), "");
}

TEST(Analysis, Analysis_constructor_null_vector)
{
    AnalysisX86 anal(nullptr);
    Statement ins = anal[0];
    EXPECT_EQ(ins.get_offset(), 0x0);
    EXPECT_STREQ(ins.get_command().c_str(), "");
}

TEST(Analysis, Analysis_constructor_string)
{
    // string
    std::string func = "sym.if_and\n"
                       "0x610 test edi, edi\n"
                       "0x612 je 0x620\n"
                       "0x614 test esi, esi\n"
                       "0x616 mov eax, 5\n"
                       "0x61b je 0x620\n"
                       "0x61d ret\n"
                       "0x620 mov eax, 6\n"
                       "0x625 ret\n";
    AnalysisX86 anal(func);
    // first ins
    Statement ins = anal[0];
    EXPECT_EQ(ins.get_offset(), 0x610);
    EXPECT_STREQ(ins.get_command().c_str(), "test edi, edi");
    // last ins
    ins = anal[7];
    EXPECT_EQ(ins.get_offset(), 0x625);
    EXPECT_STREQ(ins.get_command().c_str(), "ret");
    // out of bounds, beginning
    ins = anal[-1];
    EXPECT_EQ(ins.get_offset(), 0x0);
    EXPECT_STREQ(ins.get_command().c_str(), "");
    // out of bounds, end
    ins = anal[8];
    EXPECT_EQ(ins.get_offset(), 0x0);
    EXPECT_STREQ(ins.get_command().c_str(), "");
}

TEST(Analysis, Analysis_constructor_vector)
{
    // array
    std::vector<Statement> stmts;
    stmts.emplace_back(0x610, "test edi, edi");
    stmts.emplace_back(0x620, "je 0x620");
    stmts.emplace_back(0x614, "test esi, esi");
    stmts.emplace_back(0x616, "mov eax, 5");
    stmts.emplace_back(0x61b, "je 0x620");
    stmts.emplace_back(0x61d, "ret");
    stmts.emplace_back(0x620, "mov eax, 6");
    stmts.emplace_back(0x625, "ret");

    AnalysisX86 anal(&stmts);
    // first ins
    Statement ins = anal[0];
    EXPECT_EQ(ins.get_offset(), 0x610);
    EXPECT_STREQ(ins.get_command().c_str(), "test edi, edi");
    // last ins
    ins = anal[7];
    EXPECT_EQ(ins.get_offset(), 0x625);
    EXPECT_STREQ(ins.get_command().c_str(), "ret");
    // out of bounds, beginning
    ins = anal[-1];
    EXPECT_EQ(ins.get_offset(), 0x0);
    EXPECT_STREQ(ins.get_command().c_str(), "");
    // out of bounds, end
    ins = anal[8];
    EXPECT_EQ(ins.get_offset(), 0x0);
    EXPECT_STREQ(ins.get_command().c_str(), "");
}
