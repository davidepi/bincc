//
// Created by davide on 7/2/19.
//

#include "analysis/abstract_block.hpp"
#include "analysis/cfs.hpp"
#include <analysis/cfg.hpp>
#include <fstream>
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

TEST(ControlFlowStructure, do_whileb_T1)
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

TEST(ControlFlowStructure, do_whileb_T2)
{
  ControlFlowGraph cfg(5);
  cfg.set_conditional(3, 1);
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 3);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 4);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, do_whileb_T3)
{
  ControlFlowGraph cfg(5);
  cfg.set_conditional(2, 4);
  cfg.set_next(3, 1);
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 3);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 4);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, do_whileb_T4)
{
  ControlFlowGraph cfg(6);
  cfg.set_next(3, 5);
  cfg.set_conditional(3, 4);
  cfg.set_next(4, 1);
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 3);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 5);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), DO_WHILE);
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
  ControlFlowGraph cfg(5);
  cfg.set_conditional(2, 1);
  cfg.set_conditional(3, 2);
  cfg.set_conditional(1, 2);
  cfg.set_next_null(1);
  cfg.finalize();
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  EXPECT_EQ(node->get_type(), SEQUENCE);
  ASSERT_EQ(node->size(), 3);
  const AbstractBlock* middle = (*node)[1];
  EXPECT_EQ(middle->get_type(), DO_WHILE);
  const AbstractBlock* nested = (*middle)[0];
  EXPECT_EQ(nested->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, nested_loop)
{
  ControlFlowGraph cfg(6);
  cfg.set_conditional(3, 2);
  cfg.set_conditional(4, 1);
  cfg.finalize();
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* root = cfs.root();
  EXPECT_EQ(root->get_type(), SEQUENCE);
  ASSERT_EQ(root->size(), 3);
  const AbstractBlock* outer_loop = (*root)[1];
  EXPECT_EQ(outer_loop->get_type(), DO_WHILE);
  ASSERT_EQ((*outer_loop)[0]->size(), 2);
  const AbstractBlock* inner_loop = (*(*outer_loop)[0])[1];
  EXPECT_EQ(inner_loop->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, nat_loop_break_while)
{
  ControlFlowGraph cfg(5);
  cfg.set_conditional(1, 4); // break
  cfg.set_conditional(2, 4); // end of loop
  cfg.set_next(3, 1);
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 3);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 4);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), WHILE);
}

TEST(ControlFlowStructure, nat_loop_break_do_while)
{
  ControlFlowGraph cfg(5);
  cfg.set_conditional(2, 4); // break
  cfg.set_conditional(3, 1); // end of loop
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 3);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 4);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, nat_loop_return_while)
{
  ControlFlowGraph cfg(9);
  cfg.set_conditional(1, 6);
  cfg.set_conditional(2, 6);
  cfg.set_conditional(3, 4);
  cfg.set_next(3, 6);
  cfg.set_conditional(4, 8);
  cfg.set_next(5, 8);
  cfg.set_conditional(5, 1);
  cfg.to_file("/tmp/test.dot");
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 5);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 6);
  EXPECT_EQ((*node)[3]->get_id(), 7);
  EXPECT_EQ((*node)[4]->get_id(), 8);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), WHILE);
}

TEST(ControlFlowStructure, nat_loop_return_do_while)
{
  ControlFlowGraph cfg(9);
  cfg.set_conditional(2, 6);
  cfg.set_conditional(3, 4);
  cfg.set_next(3, 6);
  cfg.set_conditional(4, 8);
  cfg.set_next(5, 8);
  cfg.set_conditional(5, 1);
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.root();
  ASSERT_EQ(node->size(), 5);
  EXPECT_EQ((*node)[0]->get_id(), 0);
  EXPECT_EQ((*node)[2]->get_id(), 6);
  EXPECT_EQ((*node)[3]->get_id(), 7);
  EXPECT_EQ((*node)[4]->get_id(), 8);
  const AbstractBlock* loop = (*node)[1];
  EXPECT_EQ(loop->get_type(), DO_WHILE);
}

TEST(ControlFlowStructure, get_node)
{
  ControlFlowStructure cfs;
  EXPECT_EQ(cfs.nodes_no(), 0);
  ControlFlowGraph cfg(5);
  cfg.set_next(1, 3);
  cfg.set_conditional(1, 2);
  ASSERT_TRUE(cfs.build(cfg));
  const AbstractBlock* node = cfs.get_node(1);
  EXPECT_EQ(node->get_id(), 1);
  node = cfs.get_node(6);
  EXPECT_EQ(node->get_id(), 6);
  EXPECT_EQ(cfs.nodes_no(), 7);
}

TEST(ControlFlowStructure, print_cfg)
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
  cfs.to_file("/tmp/test.dot", cfg);
  std::ifstream read("/tmp/test.dot");
  read.seekg(0, std::ios::end);
  std::string str;
  str.reserve(read.tellg());
  read.seekg(0, std::ios::beg);
  str.assign((std::istreambuf_iterator<char>(read)),
             std::istreambuf_iterator<char>());
  std::string expected;
  expected = "digraph "
             "{\n0->1;\n1->2;\n2->3;\n2->4[arrowhead=\"empty\"];\n4->5;\n5->1;"
             "\n5->6[arrowhead=\"empty\",style=\"dotted\"];\n6[style="
             "\"dotted\"];\n3->5;\n3->3[arrowhead=\"empty\"];\nsubgraph "
             "cluster_11 {\n0;\nsubgraph cluster_10 {\nsubgraph "
             "cluster_9 {\n1;\nsubgraph cluster_8 {\n2;\n4;\nsubgraph "
             "cluster_7 {\n3;\nlabel = \"Self-loop\";\n}\nlabel = "
             "\"If-else\";\n}\nlabel = \"Sequence\";\n}\n5;\nlabel = "
             "\"Do-While\";\n}\n6;\nlabel = \"Sequence\";\n}\n}";
  EXPECT_STREQ(str.c_str(), expected.c_str());
  read.close();
  unlink("/tmp/test.dot");
}

TEST(ControlFlowStructure, print_tree)
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
  cfs.to_file("/tmp/test.dot");
  std::ifstream read("/tmp/test.dot");
  read.seekg(0, std::ios::end);
  std::string str;
  str.reserve(read.tellg());
  read.seekg(0, std::ios::beg);
  str.assign((std::istreambuf_iterator<char>(read)),
             std::istreambuf_iterator<char>());
  std::string expected;
  expected =
      "digraph {\n11[label=\"Sequence\"];\n11 -> 0\n11 -> 10\n11 -> "
      "6\n6[label=\"Basic\" shape=\"box\"];\n10[label=\"Do-While\"];\n10 -> "
      "9\n10 -> 5\n5[label=\"Basic\" "
      "shape=\"box\"];\n9[label=\"Sequence\"];\n9 -> 1\n9 -> "
      "8\n8[label=\"If-else\"];\n8 -> 2\n8 -> 4\n8 -> "
      "7\n7[label=\"Self-loop\"];\n7 -> 3\n3[label=\"Basic\" "
      "shape=\"box\"];\n4[label=\"Basic\" shape=\"box\"];\n2[label=\"Basic\" "
      "shape=\"box\"];\n1[label=\"Basic\" shape=\"box\"];\n0[label=\"Basic\" "
      "shape=\"box\"];\n}\n";
  EXPECT_STREQ(str.c_str(), expected.c_str());
  read.close();
  unlink("/tmp/test.dot");
}

TEST(ControlFlowStructure, offset_retained)
{
  ControlFlowGraph cfg(7);
  cfg.set_offsets(0, 0x630, 0x634);
  cfg.set_offsets(1, 0x634, 0x638);
  cfg.set_offsets(2, 0x638, 0x63C);
  cfg.set_offsets(3, 0x63C, 0x640);
  cfg.set_offsets(4, 0x640, 0x644);
  cfg.set_offsets(5, 0x644, 0x648);
  cfg.set_offsets(6, 0x648, 0x64C);
  cfg.set_next(1, 4);
  cfg.set_conditional(1, 6);
  cfg.set_conditional(2, 5);
  cfg.set_next(5, 1);
  cfg.set_conditional(5, 4);
  cfg.finalize();
  ControlFlowStructure cfs;
  ASSERT_TRUE(cfs.build(cfg));
  uint64_t start;
  uint64_t end;
  const AbstractBlock* root = cfs.root();
  EXPECT_EQ(root->get_type(), SEQUENCE);
  EXPECT_EQ(root->size(), 3);
  const AbstractBlock* node = (*root)[0];
  const BasicBlock* leaf;
  ASSERT_EQ(node->get_type(), BASIC);
  leaf = static_cast<const BasicBlock*>(node);
  leaf->get_offset(&start, &end);
  EXPECT_EQ(start, 0x630);
  EXPECT_EQ(end, 0x634);
  node = (*root)[2];
  ASSERT_EQ(node->get_type(), BASIC);
  leaf = static_cast<const BasicBlock*>(node);
  leaf->get_offset(&start, &end);
  EXPECT_EQ(start, 0x648);
  EXPECT_EQ(end, 0x64C);
}
