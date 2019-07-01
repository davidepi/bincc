//
// Created by davide on 6/13/19.
//
#include "analysis/abstract_block.hpp"
#include <gtest/gtest.h>

TEST(AbstractBlock, type)
{
    AbstractBlock a(0);
    EXPECT_EQ(a.get_type(), BlockType::BASIC);
}

TEST(AbstractBlock, flow)
{
    AbstractBlock a0(0);
    AbstractBlock a1(1);
    AbstractBlock a2(2);
    AbstractBlock a3(3);

    EXPECT_EQ(a0.get_next(), nullptr);
    EXPECT_EQ(a0.get_cond(), nullptr);
    EXPECT_EQ(a1.get_next(), nullptr);
    EXPECT_EQ(a1.get_cond(), nullptr);
    EXPECT_EQ(a2.get_next(), nullptr);
    EXPECT_EQ(a2.get_cond(), nullptr);
    EXPECT_EQ(a3.get_next(), nullptr);
    EXPECT_EQ(a3.get_cond(), nullptr);

    a0.set_next(&a1);
    a1.set_next(&a2);
    a1.set_cond(&a0);
    a2.set_next(&a0);
    a2.set_next(&a1);
    a1.set_cond(&a3);
    a3.set_next(nullptr);
    a3.set_next(&a2);
    a1.set_cond(&a0);
    a1.set_cond(&a3);
    a1.set_cond(nullptr);
    a1.set_cond(nullptr);
    a1.set_cond(&a3);

    EXPECT_EQ(a0.get_next(), &a1);
    EXPECT_EQ(a0.get_cond(), nullptr);
    EXPECT_EQ(a1.get_next(), &a2);
    EXPECT_EQ(a1.get_cond(), &a3);
    EXPECT_EQ(a2.get_next(), &a1);
    EXPECT_EQ(a2.get_cond(), nullptr);
    EXPECT_EQ(a0.get_edges_inn(), 0);
    EXPECT_EQ(a0.get_edges_out(), 1);
    EXPECT_EQ(a1.get_edges_inn(), 2);
    EXPECT_EQ(a1.get_edges_out(), 2);
    EXPECT_EQ(a2.get_edges_inn(), 2);
    EXPECT_EQ(a2.get_edges_out(), 1);
    EXPECT_EQ(a3.get_edges_inn(), 1);
    EXPECT_EQ(a3.get_edges_out(), 1);
}
