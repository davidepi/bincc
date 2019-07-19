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
            bb = static_cast<const BasicBlock*>(bb->get_next());
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
    ControlFlowGraph cfg(3);
    cfg.set_next(2, 0);
    cfg.set_conditional(0, 2);
    const char* expected =
        "digraph {\n0->1\n0->2[arrowhead=\"empty\"];\n2->0\n1->2\n}";
    std::stringstream strstr;
    strstr << cfg;
    EXPECT_STREQ(strstr.str().c_str(), expected);
}

TEST(ControlFlowGraph, dot_file)
{
    ControlFlowGraph cfg(3);
    cfg.set_next(2, 0);
    cfg.set_conditional(0, 2);
    std::string dot = cfg.to_dot();
    const char* expected =
        "digraph {\n0->1\n0->2[arrowhead=\"empty\"];\n2->0\n1->2\n}";
    EXPECT_STREQ(dot.c_str(), expected);
}

TEST(ControlFlowGraph, finalize)
{
    ControlFlowGraph cfg(3);
    cfg.set_next_null(1);
    cfg.set_conditional(0, 2);
    const BasicBlock* bb = cfg.root();
    const BasicBlock* next = static_cast<const BasicBlock*>(bb->get_next());
    const BasicBlock* cond = static_cast<const BasicBlock*>(bb->get_cond());
    ASSERT_NE(next, nullptr);
    ASSERT_NE(cond, nullptr);
    EXPECT_EQ(next->get_id(), 1);
    EXPECT_EQ(cond->get_id(), 2);
    EXPECT_EQ(next->get_next(), nullptr);
    EXPECT_EQ(next->get_cond(), nullptr);
    EXPECT_EQ(cond->get_next(), nullptr);
    EXPECT_EQ(cond->get_cond(), nullptr);

    cfg.finalize();
    bb = cfg.root();
    next = static_cast<const BasicBlock*>(bb->get_next());
    cond = static_cast<const BasicBlock*>(bb->get_cond());
    ASSERT_NE(next, nullptr);
    ASSERT_NE(cond, nullptr);
    EXPECT_EQ(next->get_id(), 1);
    EXPECT_EQ(cond->get_id(), 2);
    ASSERT_NE(next->get_next(), nullptr);
    EXPECT_EQ(next->get_cond(), nullptr);
    ASSERT_NE(cond->get_next(), nullptr);
    EXPECT_EQ(cond->get_cond(), nullptr);
    ASSERT_EQ(next->get_next(), cond->get_next());
    const BasicBlock* exit = static_cast<const BasicBlock*>(next->get_next());
    EXPECT_EQ(exit->get_next(), nullptr);
    EXPECT_EQ(exit->get_cond(), nullptr);
    EXPECT_EQ(cfg.edges_no(), 4);
}

TEST(ControlFlowGraph, dfst)
{
    ControlFlowGraph cfg(8);
    cfg.set_next(0, 5);
    cfg.set_next(5, 6);
    cfg.set_next(6, 5);
    cfg.set_conditional(6, 7);
    cfg.set_conditional(5, 7);
    cfg.set_conditional(0, 1);
    cfg.set_next(1, 3);
    cfg.set_conditional(1, 2);
    cfg.set_next(3, 3);
    cfg.set_conditional(3, 4);
    cfg.set_next(2, 4);
    cfg.set_next(4, 1);
    cfg.set_conditional(4, 7);

    std::queue<const BasicBlock*> postorder = cfg.dfst();
    int expected[8] = {7, 6, 5, 4, 3, 2, 1, 0};
    int index = 0;
    EXPECT_FALSE(postorder.empty());
    while(!postorder.empty())
    {
        EXPECT_EQ(postorder.front()->get_id(), expected[index++]);
        postorder.pop();
    }
}
