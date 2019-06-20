//
// Created by davide on 6/20/19.
//

#include "analysis/cfg.hpp"
#include <gtest/gtest.h>

TEST(ControlFlowGraph, constructor)
{
    int SIZE = 1000;
    ControlFlowGraph cfg(SIZE);
    const BasicBlock* bb = cfg.root();

    int id = 0;
    do
    {
        EXPECT_EQ(bb->get_id(), id);
        if(id == SIZE - 1)
        {
            ASSERT_EQ(bb->get_next(), nullptr);
        }
        else
        {
            ASSERT_NE(bb->get_next(), nullptr);
            bb = bb->get_next();
        }
        id++;
    } while(id < SIZE);

    EXPECT_EQ(cfg.nodes_no(), SIZE);
    EXPECT_EQ(cfg.edges_no(), SIZE - 1);
}

TEST(ControlFlowGraph, edges_math)
{
    int SIZE = 100;
    ControlFlowGraph cfg(SIZE);
    int expected_edges = cfg.edges_no();
    cfg.set_next(8, 14);         // replace
    cfg.set_conditional(34, 46); // add
    expected_edges++;
    cfg.set_conditional(45, 43); // add
    expected_edges++;
    cfg.set_conditional(45, 42); // replace
    cfg.set_conditional(43, 89); // add
    expected_edges++;
    cfg.set_conditional_null(43); // remove
    expected_edges--;
    cfg.set_next_null(43); // remove
    expected_edges--;
    EXPECT_EQ(cfg.nodes_no(), SIZE);
    EXPECT_EQ(cfg.edges_no(), expected_edges);
}

TEST(ControlFlowGraph, stream)
{
}

TEST(ControlFlowGraph, dot_file)
{
    ControlFlowGraph cfg(3);
    cfg.set_next(2, 0);
    cfg.set_conditional(0, 2);
    std::string dot = cfg.to_dot();
    const char* expected = "digraph {\n0->1\n0->2\n2->0\n1->2\n}";
    EXPECT_STREQ(dot.c_str(), expected);
}
