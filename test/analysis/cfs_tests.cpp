//
// Created by davide on 7/2/19.
//

#include "analysis/abstract_block.hpp"
#include "analysis/cfs.hpp"
#include <analysis/cfg.hpp>
#include <gtest/gtest.h>

TEST(ControlFlowStructure, build_uncalled)
{
    // variant 0: conditional loop
    ControlFlowStructure cfs;
    const AbstractBlock* structured = cfs.root();
    EXPECT_EQ(structured, nullptr);
}

TEST(ControlFlowStructure, sequence)
{
    //0 -> 1 -> 2 -> 3 -> 4 with no conditional loops whatsoever
    ControlFlowGraph cfg(5);
    ControlFlowStructure cfs;
    cfs.build(cfg.root(), cfg.nodes_no());
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 5);
    const AbstractBlock* a0 = (*structured)[0];
    const AbstractBlock* a1 = (*structured)[1];
    const AbstractBlock* a2 = (*structured)[2];
    const AbstractBlock* a3 = (*structured)[3];
    const AbstractBlock* a4 = (*structured)[4];
    EXPECT_EQ(a0->get_id(), 0);
    EXPECT_EQ(a1->get_id(), 1);
    EXPECT_EQ(a2->get_id(), 2);
    EXPECT_EQ(a3->get_id(), 3);
    EXPECT_EQ(a4->get_id(), 4);
}

TEST(ControlFlowStructure, self_loop)
{
    // 0 -> 1 -> 2 with 1 -> 1 conditional loop and 1 -> 2 unconditional
//    ControlFlowGraph cfg(3);
//    cfg.set_conditional(1, 1);
//    ControlFlowStructure cfs;
//    cfs.build(cfg.root(), cfg.nodes_no());
//    const AbstractBlock* structured = cfs.root();
//    ASSERT_NE(structured, nullptr);
//    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
//    ASSERT_EQ(structured->size(), 3);
//    const AbstractBlock* head = (*structured)[0];
//    const AbstractBlock* middle = (*structured)[1];
//    const AbstractBlock* tail = (*structured)[2];
//    EXPECT_EQ(head->get_type(), BlockType::BASIC);
//    EXPECT_EQ(head->size(), 0);
//    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
//    EXPECT_EQ(tail->size(), 0);
//    EXPECT_EQ(middle->get_type(), BlockType::SELF_LOOP);
//    ASSERT_EQ(middle->size(), 1);
//    const AbstractBlock* loop = (*middle)[0];
//    EXPECT_EQ(loop->size(), 0);
//    EXPECT_EQ(loop->get_type(), BlockType::BASIC);
}
