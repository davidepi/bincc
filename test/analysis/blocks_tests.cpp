//
// Created by davide on 6/13/19.
//
#include "analysis/abstract_block.hpp"
#include "analysis/acyclic_block.hpp"
#include "analysis/basic_block.hpp"
#include "analysis/cyclic_block.hpp"
#include <gtest/gtest.h>

TEST(BasicBlock, id)
{
    BasicBlock b;
    EXPECT_EQ(b.get_id(), 0);
    BasicBlock b2(15);
    EXPECT_EQ(b2.get_id(), 15);
    b2.set_id(-13);
    EXPECT_EQ(b2.get_id(), -13);
}

TEST(BasicBlock, outgoing_edges)
{
    BasicBlock b0(0);
    BasicBlock b1(1);
    BasicBlock b2(2);
    BasicBlock balone(3);
    b0.set_next(&b1);
    b1.set_cond(&b2);
    b2.set_next(&b0);
    b2.set_cond(&b2);

    EXPECT_EQ(b0.get_out_edges(), 1);
    EXPECT_EQ(b1.get_out_edges(), 1);
    EXPECT_EQ(b2.get_out_edges(), 2);
    EXPECT_EQ(balone.get_out_edges(), 0);
}

TEST(BasicBlock, replace_if_match)
{
    BasicBlock b0(0);
    BasicBlock b1(1);
    BasicBlock b2(2);
    BasicBlock b4(4);

    b0.set_next(&b1);
    b1.set_cond(&b2);
    b2.set_next(&b0);
    b2.set_cond(&b1);

    b0.replace_if_match(&b2, &b0);
    EXPECT_EQ(b0.get_next(), &b1);
    EXPECT_EQ(b0.get_cond(), nullptr);
    b1.replace_if_match(&b2, &b0);
    EXPECT_EQ(b1.get_next(), nullptr);
    EXPECT_EQ(b1.get_cond(), &b0);
    b2.replace_if_match(&b0, &b2);
    EXPECT_EQ(b2.get_next(), &b2);
    EXPECT_EQ(b2.get_cond(), &b1);
}

TEST(BasicBlock, size)
{
    BasicBlock b;
    EXPECT_EQ(b.size(), 0);
    EXPECT_EQ(b[0], &b);
    EXPECT_EQ(b[1], &b);
}

TEST(BasicBlok, type)
{
    BasicBlock b;
    EXPECT_EQ(b.get_type(), BlockType::BASIC);
}

TEST(BasicBlok, name)
{
    BasicBlock b;
    EXPECT_STREQ(b.get_name(), "Basic");
}

TEST(BasicBlock, copy)
{
    BasicBlock b0(10);
    BasicBlock b1(11);
    BasicBlock b2(12);
    b0.set_next(&b1);
    b0.set_cond(&b2);
    BasicBlock btmp = b0;
    EXPECT_EQ(btmp.get_id(), b0.get_id());
    EXPECT_EQ(btmp.get_next(), b0.get_next());
    EXPECT_EQ(btmp.get_cond(), b0.get_cond());
}

TEST(BasicBlock, flow)
{
    BasicBlock b0(0);
    BasicBlock b1(1);
    BasicBlock b2(2);
    BasicBlock b3(3);

    EXPECT_EQ(b0.get_next(), nullptr);
    EXPECT_EQ(b0.get_cond(), nullptr);
    EXPECT_EQ(b1.get_next(), nullptr);
    EXPECT_EQ(b1.get_cond(), nullptr);
    EXPECT_EQ(b2.get_next(), nullptr);
    EXPECT_EQ(b2.get_cond(), nullptr);
    EXPECT_EQ(b3.get_next(), nullptr);
    EXPECT_EQ(b3.get_cond(), nullptr);

    b0.set_next(&b1);
    b1.set_next(&b2);
    b1.set_cond(&b0);
    b2.set_next(&b0);
    b2.set_next(&b1);
    b1.set_cond(&b3);
    b3.set_next(nullptr);
    b3.set_next(&b2);
    b1.set_cond(&b0);
    b1.set_cond(&b3);
    b1.set_cond(nullptr);
    b1.set_cond(nullptr);
    b1.set_cond(&b3);

    EXPECT_EQ(b0.get_next(), &b1);
    EXPECT_EQ(b0.get_cond(), nullptr);
    EXPECT_EQ(b1.get_next(), &b2);
    EXPECT_EQ(b1.get_cond(), &b3);
    EXPECT_EQ(b2.get_next(), &b1);
    EXPECT_EQ(b2.get_cond(), nullptr);
}

TEST(BasicBlock, hash)
{
    BasicBlock bb0;
    bb0.set_id(0);
    BasicBlock bb1;
    bb1.set_id(1);
    EXPECT_EQ(bb0.structural_hash(), bb1.structural_hash());
}

TEST(SequenceBlock, type)
{
    BasicBlock* b0 = new BasicBlock(1);
    BasicBlock* b1 = new BasicBlock(2);
    b0->set_next(b1);
    SequenceBlock seq(0, b0, b1);
    EXPECT_EQ(seq.get_type(), BlockType::SEQUENCE);
}

TEST(SequenceBlock, name)
{
    BasicBlock* b0 = new BasicBlock(1);
    BasicBlock* b1 = new BasicBlock(2);
    b0->set_next(b1);
    SequenceBlock seq(0, b0, b1);
    EXPECT_STREQ(seq.get_name(), "Sequence");
}

TEST(SequenceBlock, ctor_no_sequences)
{
    // sequence is next
    BasicBlock* b0 = new BasicBlock(1);
    BasicBlock* b1 = new BasicBlock(2);
    b0->set_next(b1);
    SequenceBlock seq(0, b0, b1);
    ASSERT_EQ(seq.size(), 2);
    const AbstractBlock* a0 = seq[0];
    const AbstractBlock* a1 = seq[1];
    EXPECT_EQ(a0->get_id(), 1);
    EXPECT_EQ(a1->get_id(), 2);
}

TEST(SequenceBlock, ctor_sequences)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    AbstractBlock* s0 = new SequenceBlock(4, b0, b1);

    BasicBlock* b2 = new BasicBlock(14);
    BasicBlock* b3 = new BasicBlock(7);
    b2->set_next(b3);
    AbstractBlock* s1 = new SequenceBlock(5, b2, b3);
    s0->set_next(s1);

    SequenceBlock s2(6, s0, s1);
    ASSERT_EQ(s2.size(), 4);
    const AbstractBlock* a0 = s2[0];
    const AbstractBlock* a1 = s2[1];
    const AbstractBlock* a2 = s2[2];
    const AbstractBlock* a3 = s2[3];

    EXPECT_EQ(a0->get_id(), 0);
    EXPECT_EQ(a1->get_id(), 1);
    EXPECT_EQ(a2->get_id(), 14);
    EXPECT_EQ(a3->get_id(), 7);
}

TEST(SequenceBlock, hash)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    BasicBlock* b3 = new BasicBlock(3);
    SequenceBlock sq1(4, b0, b1);
    SequenceBlock sq2(5, b2, b3);
    EXPECT_EQ(sq1.structural_hash(), sq2.structural_hash());

    SequenceBlock* sq3 =
        new SequenceBlock(6, new BasicBlock(7), new BasicBlock(8));
    SequenceBlock sq4(9, new BasicBlock(9), sq3);
    sq4.structural_hash();
    EXPECT_NE(sq4.structural_hash(), sq1.structural_hash());
}

TEST(SequenceBlock, replace_if_match)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    BasicBlock* b3 = new BasicBlock(3);
    BasicBlock* b4 = new BasicBlock(4);
    BasicBlock* b5 = new BasicBlock(5);
    SequenceBlock s0(7, b0, b1);
    SequenceBlock s1(8, b2, b3);
    SequenceBlock s2(9, b4, b5);
    s0.set_next(&s1);

    s0.replace_if_match(&s1, &s2);
    EXPECT_EQ(s0.get_next(), &s2);
    s0.replace_if_match(&s1, &s0);
    EXPECT_EQ(s0.get_next(), &s2);
    s0.replace_if_match(&s2, &s0);
    EXPECT_EQ(s0.get_next(), &s0);
}

TEST(SelfLoopBlock, type)
{
    BasicBlock* b0 = new BasicBlock(1);
    b0->set_cond(b0);
    SelfLoopBlock slb(2, b0);
    EXPECT_EQ(slb.get_type(), BlockType::SELF_LOOP);
}

TEST(SelfLoopBlock, name)
{
    BasicBlock* b0 = new BasicBlock(1);
    b0->set_cond(b0);
    SelfLoopBlock slb(2, b0);
    EXPECT_STREQ(slb.get_name(), "Self-loop");
}

TEST(SelfLoopBlock, ctor)
{
    BasicBlock* b0 = new BasicBlock(1);
    b0->set_cond(b0);
    SelfLoopBlock slb(2, b0);
    EXPECT_EQ(slb.size(), 1);
    const AbstractBlock* a0 = slb[0];
    EXPECT_EQ(a0->get_id(), 1);
}

TEST(IfThenBlock, type)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b2);
    b0->set_cond(b1);
    b1->set_next(b2);
    IfThenBlock ift(3, b0, b1);
    EXPECT_EQ(ift.get_type(), BlockType::IF_THEN);
    delete b2;
}

TEST(IfThenBlock, name)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b2);
    b0->set_cond(b1);
    b1->set_next(b2);
    IfThenBlock ift(3, b0, b1);
    EXPECT_STREQ(ift.get_name(), "If-then");
    delete b2;
}

TEST(IfThenBlock, size)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b2);
    b0->set_cond(b1);
    b1->set_next(b2);
    IfThenBlock ift(3, b0, b1);
    EXPECT_EQ(ift.size(), 2);
    delete b2;
}

TEST(IfThenBlock, access)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b2);
    b0->set_cond(b1);
    b1->set_next(b2);
    IfThenBlock ift(3, b0, b1);
    ASSERT_EQ(ift.size(), 2);
    EXPECT_EQ(ift[0], b0);
    EXPECT_EQ(ift[1], b1);
    delete b2;
}

TEST(IfElseBlock, type)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b1);
    b0->set_cond(b2);
    IfElseBlock ift(3, b0, b1, b2);
    EXPECT_EQ(ift.get_type(), BlockType::IF_ELSE);
}

TEST(IfElseBlock, name)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b1);
    b0->set_cond(b2);
    IfElseBlock ift(3, b0, b1, b2);
    EXPECT_STREQ(ift.get_name(), "If-else");
}

TEST(IfElseBlock, size)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b1);
    b0->set_cond(b2);
    IfElseBlock ift(3, b0, b1, b2);
    EXPECT_EQ(ift.size(), 3);
}

TEST(IfElseBlock, access)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    BasicBlock* b2 = new BasicBlock(2);
    b0->set_next(b1);
    b0->set_cond(b2);
    IfElseBlock ift(3, b0, b1, b2);
    ASSERT_EQ(ift.size(), 3);
    EXPECT_EQ(ift[0], b0);
    EXPECT_EQ(ift[1], b1);
    EXPECT_EQ(ift[2], b2);
}

TEST(WhileBlock, type)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    WhileBlock wb(2, b0, b1);
    EXPECT_EQ(wb.get_type(), BlockType::WHILE);
}

TEST(WhileBlock, name)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    WhileBlock wb(2, b0, b1);
    EXPECT_STREQ(wb.get_name(), "While");
}

TEST(WhileBlock, size)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    WhileBlock wb(2, b0, b1);
    EXPECT_EQ(wb.size(), 2);
}

TEST(WhileBlock, access)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    WhileBlock wb(2, b0, b1);
    const AbstractBlock* head = wb[0];
    const AbstractBlock* tail = wb[1];
    EXPECT_EQ(head->get_id(), 0);
    EXPECT_EQ(tail->get_id(), 1);
}

TEST(DoWhileBlock, type)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    DoWhileBlock wb(2, b0, b1);
    EXPECT_EQ(wb.get_type(), BlockType::DO_WHILE);
}

TEST(DoWhileBlock, name)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    DoWhileBlock wb(2, b0, b1);
    EXPECT_STREQ(wb.get_name(), "Do-While");
}

TEST(DoWhileBlock, size)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    DoWhileBlock wb(2, b0, b1);
    EXPECT_EQ(wb.size(), 2);
}

TEST(DoWhileBlock, access)
{
    BasicBlock* b0 = new BasicBlock(0);
    BasicBlock* b1 = new BasicBlock(1);
    b0->set_next(b1);
    b1->set_next(b0);
    DoWhileBlock wb(2, b0, b1);
    const AbstractBlock* head = wb[0];
    const AbstractBlock* tail = wb[1];
    EXPECT_EQ(head->get_id(), 0);
    EXPECT_EQ(tail->get_id(), 1);
}
