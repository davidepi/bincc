//
// Created by davide on 6/13/19.
//
#include "analysis/basic_block.hpp"
#include <gtest/gtest.h>

TEST(BasicBlock, id)
{
    BasicBlock b;
    EXPECT_EQ(b.get_id(), 0);
    b = BasicBlock(15);
    EXPECT_EQ(b.get_id(), 15);
    b.set_id(-13);
    EXPECT_EQ(b.get_id(), -13);
}

TEST(BasicBlock, flow)
{
    BasicBlock b0;
    BasicBlock b1(1);
    BasicBlock b2(2);

    EXPECT_EQ(b0.get_next(), nullptr);
    EXPECT_EQ(b0.get_cond(), nullptr);
    EXPECT_EQ(b1.get_next(), nullptr);
    EXPECT_EQ(b1.get_cond(), nullptr);
    EXPECT_EQ(b2.get_next(), nullptr);
    EXPECT_EQ(b2.get_cond(), nullptr);

    b0.set_next(&b1);
    b1.set_next(&b2);
    b1.set_cond(&b0);
    b2.set_cond(&b0);

    EXPECT_EQ(b0.get_next(), &b1);
    EXPECT_EQ(b0.get_cond(), nullptr);
    EXPECT_EQ(b1.get_next(), &b2);
    EXPECT_EQ(b1.get_cond(), &b0);
    EXPECT_EQ(b2.get_next(), nullptr);
    EXPECT_EQ(b2.get_cond(), &b0);
}
