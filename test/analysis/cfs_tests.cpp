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
    // 0 -> 1 -> 2 -> 3 -> 4
    ControlFlowGraph cfg(5);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
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

TEST(ControlFlowStructure, sequence_conditional)
{
    // 0 -> 1 -> 2 ~> 3 -> 4
    ControlFlowGraph cfg(5);
    cfg.set_next_null(2);
    cfg.set_conditional(2, 3);
    cfg.finalize();
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
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
    ControlFlowGraph cfg(3);
    cfg.set_conditional(1, 1);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 3);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* middle = (*structured)[1];
    const AbstractBlock* tail = (*structured)[2];
    EXPECT_EQ(head->get_type(), BlockType::BASIC);
    EXPECT_EQ(head->size(), 0);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->size(), 0);
    EXPECT_EQ(middle->get_type(), BlockType::SELF_LOOP);
    ASSERT_EQ(middle->size(), 1);
    const AbstractBlock* loop = (*middle)[0];
    EXPECT_EQ(loop->size(), 0);
    EXPECT_EQ(loop->get_type(), BlockType::BASIC);
}

TEST(ControlFlowStructure, if_then_next)
{
    //`next` is the `then` block
    // 0 -> 1 -> 2 -> 3 -> 4
    //        ~> 3
    ControlFlowGraph cfg(5);
    cfg.set_conditional(1, 3);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 4);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* middle = (*structured)[1];
    const AbstractBlock* tail = (*structured)[2];
    const AbstractBlock* tail2 = (*structured)[3];
    EXPECT_EQ(head->get_type(), BlockType::BASIC);
    EXPECT_EQ(head->size(), 0);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->size(), 0);
    EXPECT_EQ(tail2->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail2->size(), 0);
    EXPECT_EQ(middle->get_type(), BlockType::IF_THEN);
    const AbstractBlock* ifblock = (*middle)[0];
    const AbstractBlock* thenblock = (*middle)[1];
    EXPECT_EQ(ifblock->get_id(), 1);
    EXPECT_EQ(thenblock->get_id(), 2);
}

TEST(ControlFlowStructure, if_then_cond)
{
    //`cond` is the `then` block
    // 0 -> 1 -> 3      -> 4
    //        ~> 2 -> 3
    ControlFlowGraph cfg(5);
    cfg.set_next(1, 3);
    cfg.set_conditional(1, 2);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 4);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* middle = (*structured)[1];
    const AbstractBlock* tail = (*structured)[2];
    const AbstractBlock* tail2 = (*structured)[3];
    EXPECT_EQ(head->get_type(), BlockType::BASIC);
    EXPECT_EQ(head->size(), 0);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->size(), 0);
    EXPECT_EQ(tail2->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail2->size(), 0);
    EXPECT_EQ(middle->get_type(), BlockType::IF_THEN);
    const AbstractBlock* ifblock = (*middle)[0];
    const AbstractBlock* thenblock = (*middle)[1];
    EXPECT_EQ(ifblock->get_id(), 1);
    EXPECT_EQ(thenblock->get_id(), 2);
}

TEST(ControlFlowStructure, if_chain)
{
    // 0 -> 1 -> 2 -> 3
    // 0 ~> 3, 1 ~> 3
    ControlFlowGraph cfg(4);
    cfg.set_conditional(0, 3);
    cfg.set_conditional(1, 3);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 2);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* tail = (*structured)[1];
    ASSERT_EQ(head->get_type(), BlockType::IF_THEN);
    ASSERT_EQ(head->size(), 3);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->get_id(), 3);
    const AbstractBlock* if0 = (*head)[0];
    const AbstractBlock* if2 = (*head)[1];
    const AbstractBlock* if1 = (*head)[2];
    EXPECT_EQ(if0->get_type(), BASIC);
    EXPECT_EQ(if0->get_id(), 0);
    EXPECT_EQ(if1->get_type(), BASIC);
    EXPECT_EQ(if1->get_id(), 1);
    EXPECT_EQ(if2->get_type(), BASIC);
    EXPECT_EQ(if2->get_id(), 2);
}

TEST(ControlFlowStructure, if_else)
{
    //`next` is the `then` block
    // 0 -> 1 -> 2 -> 4 -> 5
    //        ~> 3 -> 4
    ControlFlowGraph cfg(6);
    cfg.set_next(2, 4);
    cfg.set_conditional(1, 3);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 4);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* middle = (*structured)[1];
    const AbstractBlock* tail = (*structured)[2];
    const AbstractBlock* tail2 = (*structured)[3];
    EXPECT_EQ(head->get_type(), BlockType::BASIC);
    EXPECT_EQ(head->size(), 0);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->size(), 0);
    EXPECT_EQ(tail2->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail2->size(), 0);
    EXPECT_EQ(middle->get_type(), BlockType::IF_ELSE);
    const AbstractBlock* ifblock = (*middle)[0];
    const AbstractBlock* elseblock = (*middle)[1];
    const AbstractBlock* thenblock = (*middle)[2];
    EXPECT_EQ(ifblock->get_id(), 1);
    EXPECT_EQ(thenblock->get_id(), 2);
    EXPECT_EQ(elseblock->get_id(), 3);
}

TEST(ControlFlowStructure, whileb)
{
    // 0 -> 1 -> 2 -> 1
    //      1 ~> 3
    ControlFlowGraph cfg(4);
    cfg.set_next(2, 1);
    cfg.set_conditional(1, 3);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 3);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* middle = (*structured)[1];
    const AbstractBlock* tail = (*structured)[2];
    EXPECT_EQ(head->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(middle->get_type(), BlockType::WHILE);
    head = (*middle)[0];
    tail = (*middle)[1];
    EXPECT_EQ(head->get_id(), 1);
    EXPECT_EQ(tail->get_id(), 2);
}

TEST(ControlFlowStructure, do_whileb)
{
    // 0 -> 1 -> 2 -> 1
    //      2 ~> 3
    ControlFlowGraph cfg(4);
    cfg.set_next(2, 1);
    cfg.set_conditional(2, 3);
    cfg.finalize();
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* structured = cfs.root();
    ASSERT_NE(structured, nullptr);
    ASSERT_EQ(structured->get_type(), BlockType::SEQUENCE);
    ASSERT_EQ(structured->size(), 3);
    const AbstractBlock* head = (*structured)[0];
    const AbstractBlock* middle = (*structured)[1];
    const AbstractBlock* tail = (*structured)[2];
    EXPECT_EQ(head->get_type(), BlockType::BASIC);
    EXPECT_EQ(tail->get_type(), BlockType::BASIC);
    EXPECT_EQ(middle->get_type(), BlockType::DO_WHILE);
    head = (*middle)[0];
    tail = (*middle)[1];
    EXPECT_EQ(head->get_id(), 1);
    EXPECT_EQ(tail->get_id(), 2);
}

TEST(ControlFlowStructure, impossible_CFG)
{
    ControlFlowGraph cfg(3);
    cfg.set_next(0, 1);
    cfg.set_conditional(0, 2);
    cfg.set_next(1, 2);
    cfg.set_next(2, 1);
    cfg.finalize();
    ControlFlowStructure cfs;
    EXPECT_FALSE(cfs.build(cfg));
}

TEST(ControlFlowStructure, short_circuit_if_else)
{
    ControlFlowGraph cfg(6);
    cfg.set_conditional(0, 3);
    cfg.set_conditional(1, 3);
    cfg.set_conditional(2, 4);
    cfg.set_next(3, 5);
    cfg.finalize();
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* seq = cfs.root();
    ASSERT_EQ(seq->size(), 2);
    const AbstractBlock* ifelse = (*seq)[0];
    EXPECT_EQ(ifelse->get_type(), IF_ELSE);
    ASSERT_EQ(ifelse->size(), 5);
    const AbstractBlock* node = (*ifelse)[0];
    EXPECT_EQ(node->get_id(), 0);
    node = (*ifelse)[1];
    EXPECT_EQ(node->get_id(), 4);
    node = (*ifelse)[2];
    EXPECT_EQ(node->get_id(), 3);
    node = (*ifelse)[3];
    EXPECT_EQ(node->get_id(), 1);
    node = (*ifelse)[4];
    EXPECT_EQ(node->get_id(), 2);
}

TEST(ControlFlowStructure, short_circuit_if_then)
{
    ControlFlowGraph cfg(7);
    cfg.set_conditional(0, 6);
    cfg.set_conditional(1, 6);
    cfg.set_next(2, 6);
    cfg.set_conditional(2, 3);
    cfg.set_conditional(3, 6);
    cfg.set_conditional(4, 6);
    cfg.set_conditional(5, 6);
    cfg.finalize();
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* head = cfs.root();
    ASSERT_EQ(head->get_type(), SEQUENCE);
    ASSERT_EQ(head->size(), 2);
    head = (*head)[0];
    EXPECT_EQ(head->get_type(), IF_THEN);
    EXPECT_EQ(head->size(), 6);
}

// test implemented in order to replicate and fix a bug

TEST(ControlFlowStructure, if_else_abstract)
{
    ControlFlowGraph cfg(4);
    cfg.set_conditional(0, 2);
    cfg.set_conditional(1, 1);
    cfg.set_conditional(2, 2);
    cfg.set_next(1, 3);
    cfg.set_next(5, 7);
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    EXPECT_EQ(cfs.root()->get_type(), BlockType::SEQUENCE);
}

TEST(ControlFlowStructure, structures_inside_loop)
{
    ControlFlowGraph cfg(7);
    cfg.set_conditional(2, 4);
    cfg.set_next(3, 5);
    cfg.set_conditional(3, 3);
    cfg.set_next(5, 1);
    cfg.set_conditional(5, 6);
    cfg.finalize();
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* head = cfs.root();
    EXPECT_EQ(head->get_type(), SEQUENCE);
    EXPECT_EQ((*head)[0]->get_type(), BASIC);
    EXPECT_EQ((*head)[2]->get_type(), BASIC);
    const AbstractBlock* middle = (*head)[1];
    ASSERT_EQ(middle->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, nested_while)
{
    ControlFlowGraph cfg(5);
    cfg.set_conditional(2, 1);
    cfg.set_next(3, 2);
    cfg.set_conditional(1, 2);
    cfg.set_next(1, 4);
    cfg.finalize();
    ControlFlowStructure cfs;
    ASSERT_TRUE(cfs.build(cfg));
    const AbstractBlock* node = cfs.root();
    EXPECT_EQ(node->get_type(), SEQUENCE);
    ASSERT_EQ(node->size(), 3);
    const AbstractBlock* middle = (*node)[1];
    EXPECT_EQ(middle->get_type(), WHILE);
    const AbstractBlock* nested = (*middle)[1];
    EXPECT_EQ(nested->get_type(), WHILE);
}

TEST(ControlFlowStructure, nested_do_while)
{
    //    ControlFlowGraph cfg(5);
    //    cfg.set_conditional(2, 1);
    //    cfg.set_conditional(3, 2);
    //    cfg.set_conditional(1, 2);
    //    cfg.set_next_null(1);
    //    cfg.finalize();
    //    ControlFlowStructure cfs;
    //    ASSERT_TRUE(cfs.build(cfg));
    //    const AbstractBlock* node = cfs.root();
    //    EXPECT_EQ(node->get_type(), SEQUENCE);
    //    ASSERT_EQ(node->size(), 3);
    //    const AbstractBlock* middle = (*node)[1];
    //    EXPECT_EQ(middle->get_type(), DO_WHILE);
    //    const AbstractBlock* nested = (*middle)[1];
    //    EXPECT_EQ(nested->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, nested_loop)
{
    //        ControlFlowGraph cfg(6);
    //        cfg.set_conditional(3, 2);
    //        cfg.set_conditional(4, 1);
    //        cfg.finalize();
    //        ControlFlowStructure cfs;
    //        ASSERT_TRUE(cfs.build(cfg));
    //        cfg.to_file("/home/davide/Desktop/test.dot");
}

TEST(ControlFLowStructure, get_node)
{
    //    ControlFlowStructure cfs;
    //    EXPECT_EQ(cfs.nodes_no(), 0);
    //    ControlFlowGraph cfg(5);
    //    cfg.set_next(1, 3);
    //    cfg.set_conditional(1, 2);
    //    ASSERT_TRUE(cfs.build(cfg));
    //    const AbstractBlock* node = cfs.get_node(1);
    //    EXPECT_EQ(node->get_id(), 1);
    //    node = cfs.get_node(6);
    //    EXPECT_EQ(node->get_id(), 6);
    //    EXPECT_EQ(cfs.nodes_no(), 8);
}
