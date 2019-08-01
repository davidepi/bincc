//
// Created by davide on 7/25/19.
//
#include "analysis/cfs.hpp"
#include "analysis/comparison.hpp"
#include "architectures/architecture_x86.hpp"
#include <gtest/gtest.h>

std::vector<Statement> create_function()
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x00, "test eax, eax");
  stmts.emplace_back(0x04, "jg 0x38");
  stmts.emplace_back(0x08, "add ebx, 5");
  stmts.emplace_back(0x0C, "jmp 0x10");
  stmts.emplace_back(0x10, "cmp eax, ebx");
  stmts.emplace_back(0x14, "jne 0x20");
  stmts.emplace_back(0x18, "cmp ebx, 5");
  stmts.emplace_back(0x1C, "jne 0x18");
  stmts.emplace_back(0x20, "mov ecx, [ebp+8]");
  stmts.emplace_back(0x24, "jmp 0x28");
  stmts.emplace_back(0x28, "cmp ecx, eax");
  stmts.emplace_back(0x2C, "mov eax, -1");
  stmts.emplace_back(0x30, "jne 0x08");
  stmts.emplace_back(0x34, "ret");
  stmts.emplace_back(0x38, "incl eax");
  stmts.emplace_back(0x3C, "mov ebx, [ebp+20]");
  stmts.emplace_back(0x40, "cmp eax, ebx");
  stmts.emplace_back(0x44, "je 0x58");
  stmts.emplace_back(0x48, "mov ecx, [ebp+20]");
  stmts.emplace_back(0x4C, "decl ecx");
  stmts.emplace_back(0x50, "mov [ebp+20], ecx");
  stmts.emplace_back(0x54, "jmp 0x38");
  stmts.emplace_back(0x58, "test eax, eax");
  stmts.emplace_back(0x5C, "mov eax, 0");
  stmts.emplace_back(0x60, "je 0x68");
  stmts.emplace_back(0x64, "mov eax, 1");
  stmts.emplace_back(0x68, "ret");
  return stmts;
}

TEST(Comparison, failed_analysis)
{
  std::stringstream out;
  Analysis anal("", std::shared_ptr<Architecture>{new ArchitectureUNK()}, out);
  ASSERT_GT(out.str().length(), 0);
  Comparison cmp;
  EXPECT_FALSE(anal.successful());
  cmp.add_baseline("test", "not_successful", anal);
  std::vector<CloneReport> clones;
  bool res = cmp.cloned(anal, &clones);
  EXPECT_FALSE(res);
  EXPECT_EQ(clones.size(), 0);
}

TEST(Comparison, cloned_full)
{
  std::vector<Statement> stmts = create_function();
  Analysis orig(&stmts, std::shared_ptr<Architecture>{new ArchitectureX86()});
  ASSERT_TRUE(orig.successful());
  Comparison cmp;
  cmp.add_baseline("test", "test_function", orig);
  std::vector<CloneReport> clones;
  bool res = cmp.cloned(orig, &clones);
  ASSERT_TRUE(res);
  ASSERT_GT(clones.size(), 0);
  CloneReport c0;
  c0.subtree_size = 0;
  // just grab the report with the highest depth
  for(CloneReport& report : clones)
  {
    if(report.subtree_size > c0.subtree_size)
    {
      c0 = report;
    }
  }
  EXPECT_STREQ(c0.binary.c_str(), "test");
  EXPECT_STREQ(c0.function.c_str(), "test_function");
  EXPECT_EQ(c0.block_id, c0.cloned_id);
  // the cloned source block should be the root (entire tree is identical)
  EXPECT_EQ(orig.get_cfs()->get_node(c0.block_id), orig.get_cfs()->root());
  // the cloned target block should be the root (entire tree is identical)
  EXPECT_EQ(orig.get_cfs()->get_node(c0.cloned_id), orig.get_cfs()->root());
}

TEST(Comparison, cloned_partial)
{
  std::shared_ptr<Architecture> arch;
  arch = std::shared_ptr<Architecture>{new ArchitectureX86()};
  std::vector<Statement> stmts = create_function();
  Analysis orig(&stmts, arch);
  ASSERT_TRUE(orig.successful());
  stmts[2] = Statement(0x08, "nop");
  stmts[3] = Statement(0x0C, "nop");
  stmts[10] = Statement(0x28, "nop");
  stmts[11] = Statement(0x2C, "nop");
  stmts[12] = Statement(0x30, "nop");
  Analysis check(&stmts, arch);
  ASSERT_TRUE(check.successful());
  Comparison cmp;
  cmp.add_baseline("original", "crafted", orig);
  std::vector<CloneReport> clones;
  bool res = cmp.cloned(check, &clones);
  ASSERT_TRUE(res);
  ASSERT_GT(clones.size(), 0);

  // check first clone
  for(const CloneReport& rep : clones)
  {
    const AbstractBlock* original = orig.get_cfs()->get_node(rep.block_id);
    const AbstractBlock* cloned = check.get_cfs()->get_node(rep.cloned_id);
    EXPECT_EQ(original->get_type(), cloned->get_type());
    EXPECT_EQ(original->size(), cloned->size());
    ASSERT_EQ(original->structural_hash(), cloned->structural_hash());
    if(original->size() == 3) // the sequence
    {
      EXPECT_EQ((*original)[0]->get_type(), WHILE);
      EXPECT_EQ((*original)[0]->get_type(), (*cloned)[0]->get_type());
      EXPECT_EQ((*original)[1]->get_type(), IF_THEN);
      EXPECT_EQ((*original)[1]->get_type(), (*cloned)[1]->get_type());
      EXPECT_EQ((*original)[2]->get_type(), BASIC);
      EXPECT_EQ((*original)[2]->get_type(), (*cloned)[2]->get_type());
    }
    else // the if-then
    {
      EXPECT_EQ((*original)[0]->get_type(), BASIC);
      EXPECT_EQ((*original)[0]->get_type(), (*cloned)[0]->get_type());
      EXPECT_EQ((*original)[1]->get_type(), SELF_LOOP);
      EXPECT_EQ((*original)[1]->get_type(), (*cloned)[1]->get_type());
    }
  }
}

TEST(Comparison, cloned_multiple_functions)
{
  std::shared_ptr<Architecture> arch;
  arch = std::shared_ptr<Architecture>{new ArchitectureX86()};
  std::vector<Statement> stmts = create_function();
  Analysis check(&stmts, arch);
  ASSERT_TRUE(check.successful());
  stmts.clear();
  stmts.emplace_back(0x1A7EF534, "jmp 0x1A7EF538");
  stmts.emplace_back(0x1A7EF538, "incl eax");
  stmts.emplace_back(0x1A7EF53C, "mov ebx, [ebp+20]");
  stmts.emplace_back(0x1A7EF540, "cmp eax, ebx");
  stmts.emplace_back(0x1A7EF544, "je 0x1A7EF558");
  stmts.emplace_back(0x1A7EF548, "mov ecx, [ebp+20]");
  stmts.emplace_back(0x1A7EF54C, "decl ecx");
  stmts.emplace_back(0x1A7EF550, "mov [ebp+20], ecx");
  stmts.emplace_back(0x1A7EF554, "jmp 0x1A7EF538");
  stmts.emplace_back(0x1A7EF558, "test eax, eax");
  stmts.emplace_back(0x1A7EF55C, "mov eax, 0");
  stmts.emplace_back(0x1A7EF560, "je 0x1A7EF568");
  stmts.emplace_back(0x1A7EF564, "mov eax, 1");
  stmts.emplace_back(0x1A7EF568, "ret");
  Analysis orig0(&stmts, arch);
  ASSERT_TRUE(orig0.successful());
  stmts.clear();
  stmts.emplace_back(0x12F4000, "jmp 0x12F4008");
  stmts.emplace_back(0x12F4008, "add ebx, 5");
  stmts.emplace_back(0x12F400C, "jmp 0x12F4010");
  stmts.emplace_back(0x12F4010, "cmp eax, ebx");
  stmts.emplace_back(0x12F4014, "jne 0x12F4020");
  stmts.emplace_back(0x12F4018, "cmp ebx, 5");
  stmts.emplace_back(0x12F401C, "jne 0x12F4018");
  stmts.emplace_back(0x12F4020, "mov ecx, [ebp+8]");
  stmts.emplace_back(0x12F4024, "jmp 0x12F4028");
  stmts.emplace_back(0x12F4028, "cmp ecx, eax");
  stmts.emplace_back(0x12F402C, "mov eax, -1");
  stmts.emplace_back(0x12F4030, "jne 0x12F4008");
  stmts.emplace_back(0x12F4034, "ret");
  Analysis orig1(&stmts, arch);
  ASSERT_TRUE(orig1.successful());
  Comparison cmp(1);
  cmp.add_baseline("binary", "orig0", orig0);
  cmp.add_baseline("binary", "orig1", orig1);
  std::vector<CloneReport> clones;
  bool res = cmp.cloned(check, &clones);
  ASSERT_TRUE(res);
  ASSERT_GT(clones.size(), 0);

  for(const CloneReport& rep : clones)
  {
    const AbstractBlock* original;
    if(rep.function == "orig0")
    {
      original = orig0.get_cfs()->get_node(rep.block_id);
    }
    else if(rep.function == "orig1")
    {
      original = orig1.get_cfs()->get_node(rep.block_id);
    }
    else
    {
      FAIL();
    }
    const AbstractBlock* cloned = check.get_cfs()->get_node(rep.cloned_id);
    ASSERT_EQ(original->structural_hash(), cloned->structural_hash());
    EXPECT_EQ(original->get_type(), cloned->get_type());
    EXPECT_EQ(original->size(), cloned->size());

    // long clone, this is the do-while
    if(rep.subtree_size == 4)
    {
      EXPECT_STREQ(rep.binary.c_str(), "binary");
      EXPECT_STREQ(rep.function.c_str(), "orig1");
      EXPECT_EQ(original->get_type(), DO_WHILE);
      EXPECT_EQ(original->get_type(), cloned->get_type());
    }
    else if(rep.subtree_size == 1)
    {
      // this is the clone of the other function
      if(original->get_type() == WHILE)
      {
        EXPECT_STREQ(rep.binary.c_str(), "binary");
        EXPECT_STREQ(rep.function.c_str(), "orig0");
      }
    }
  }
}
