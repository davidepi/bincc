#include "analysis/analysis.hpp"
#include "architectures/architecture_x86.hpp"
#include <gtest/gtest.h>
//
// Created by davide on 6/13/19.
//

TEST(Analysis, get_binary_name)
{
  std::string binary = "ls";
  std::string function = "main";
  std::string func = "sym.if_and\n"
                     "0x610 test edi, edi\n"
                     "0x612 je 0x620\n"
                     "0x614 test esi, esi\n"
                     "0x616 mov eax, 5\n"
                     "0x61b je 0x620\n"
                     "0x61d ret\n"
                     "0x620 mov eax, 6\n"
                     "0x625 ret\n";
  Analysis anal(binary, function, func,
                std::shared_ptr<Architecture>{new ArchitectureUNK()});
  EXPECT_STREQ(anal.get_binary_name().c_str(), binary.c_str());
}

TEST(Analysis, get_function_name)
{
  std::string binary = "ls";
  std::string function = "main";
  std::string func = "sym.if_and\n"
                     "0x610 test edi, edi\n"
                     "0x612 je 0x620\n"
                     "0x614 test esi, esi\n"
                     "0x616 mov eax, 5\n"
                     "0x61b je 0x620\n"
                     "0x61d ret\n"
                     "0x620 mov eax, 6\n"
                     "0x625 ret\n";
  Analysis anal(binary, function, func,
                std::shared_ptr<Architecture>{new ArchitectureUNK()});
  EXPECT_STREQ(anal.get_function_name().c_str(), function.c_str());
}

TEST(Analysis, Analysis_constructor_empty_string)
{
  // empty string
  std::string func;
  Analysis anal("", "", func,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  Statement ins = anal[0];
  EXPECT_EQ(ins.get_offset(), 0x0);
  EXPECT_STREQ(ins.get_command().c_str(), "");
  EXPECT_FALSE(anal.successful());
}

TEST(Analysis, Analysis_constructor_null_vector)
{
  Analysis anal("", "", nullptr,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  Statement ins = anal[0];
  EXPECT_EQ(ins.get_offset(), 0x0);
  EXPECT_STREQ(ins.get_command().c_str(), "");
  EXPECT_FALSE(anal.successful());
}

TEST(Analysis, cfs_getter)
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x612, "jmp 0x620");
  stmts.emplace_back(0x620, "test esi, esi");
  stmts.emplace_back(0x621, "mov eax, 5");
  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  std::shared_ptr<const ControlFlowStructure> cfs = anal.get_cfs();
  EXPECT_TRUE(anal.successful());
  EXPECT_NE(cfs->root(), nullptr);
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
  std::stringstream errors;
  Analysis anal("", "", func,
                std::shared_ptr<Architecture>{new ArchitectureX86()}, errors);
  EXPECT_EQ(errors.str().length(), 0);
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
  stmts.emplace_back(0x612, "je 0x620");
  stmts.emplace_back(0x614, "test esi, esi");
  stmts.emplace_back(0x616, "mov eax, 5");
  stmts.emplace_back(0x61b, "je 0x620");
  stmts.emplace_back(0x61d, "ret");
  stmts.emplace_back(0x620, "mov eax, 6");
  stmts.emplace_back(0x625, "ret");
  std::stringstream errors;
  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()}, errors);
  EXPECT_EQ(errors.str().length(), 0);
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

TEST(Analysis, wrong_architecture_string)
{
  std::string func = "sym.if_and\n";
  std::stringstream errors;
  Analysis anal("", "", func,
                std::shared_ptr<Architecture>{new ArchitectureUNK()}, errors);
  EXPECT_GT(errors.str().length(), 0);
  EXPECT_EQ(anal.get_cfg(), nullptr);
  EXPECT_EQ(anal.get_cfs(), nullptr);
}

TEST(Analysis, wrong_architecture_vector)
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x612, "je 0x620");
  std::stringstream errors;
  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureUNK()}, errors);
  EXPECT_GT(errors.str().length(), 0);
  EXPECT_EQ(anal.get_cfg(), nullptr);
  EXPECT_EQ(anal.get_cfs(), nullptr);
}

TEST(Analysis, cfg_conditional)
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x612, "je 0x620");
  stmts.emplace_back(0x614, "test esi, esi");
  stmts.emplace_back(0x616, "mov eax, 5");
  stmts.emplace_back(0x61b, "je 0x620");
  stmts.emplace_back(0x61d, "ret");
  stmts.emplace_back(0x620, "mov eax, 6");
  stmts.emplace_back(0x625, "ret");

  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  const BasicBlock* cfg = anal.get_cfg()->root();
  const BasicBlock* next;
  const BasicBlock* cond;

  // check if cfg is correct
  EXPECT_EQ(cfg->get_id(), 0); // 0
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  ASSERT_NE(cond, nullptr);
  EXPECT_EQ(next->get_id(), 1);
  EXPECT_EQ(cond->get_id(), 3);

  cfg = next; // 1
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  ASSERT_NE(cond, nullptr);
  EXPECT_EQ(next->get_id(), 2);
  EXPECT_EQ(cond->get_id(), 3);

  const BasicBlock* end = next; // 2
  next = static_cast<const BasicBlock*>(end->get_next());
  cond = static_cast<const BasicBlock*>(end->get_cond());
  EXPECT_NE(next, nullptr);
  EXPECT_EQ(cond, nullptr);

  cfg = static_cast<const BasicBlock*>(cfg->get_cond()); // 3
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  EXPECT_NE(next, nullptr);
  EXPECT_EQ(cond, nullptr);
}

TEST(Analysis, cfg_unconditional)
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x61E, "push rbp");
  stmts.emplace_back(0x61F, "mov rbp, rsp");
  stmts.emplace_back(0x622, "mov dword [var_4h], edi");
  stmts.emplace_back(0x625, "mov dword [var_8h], esi");
  stmts.emplace_back(0x628, "cmp dword [var_4h], 5");
  stmts.emplace_back(0x62C, "jne 0x633");
  stmts.emplace_back(0x62E, "mov eax, dword [var_8h]");
  stmts.emplace_back(0x631, "jmp 0x638");
  stmts.emplace_back(0x633, "mov eax, 6");
  stmts.emplace_back(0x638, "pop rbp");
  stmts.emplace_back(0x639, "ret");

  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  const BasicBlock* cfg = anal.get_cfg()->root();
  const BasicBlock* next;
  const BasicBlock* cond;

  // check if cfg is correct
  EXPECT_EQ(cfg->get_id(), 0); // 0
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  ASSERT_NE(cond, nullptr);
  EXPECT_EQ(next->get_id(), 1);
  EXPECT_EQ(cond->get_id(), 2);

  cfg = next; // 1
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  EXPECT_EQ(next->get_id(), 3);
  EXPECT_EQ(cond, nullptr);

  cfg = next; // 3
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  EXPECT_EQ(next, nullptr);
  EXPECT_EQ(cond, nullptr);

  cfg = static_cast<const BasicBlock*>(anal.get_cfg()->root()->get_cond()); // 2
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  EXPECT_EQ(next->get_id(), 3);
  EXPECT_EQ(cond, nullptr);
}

TEST(Analysis, cfg_indirect)
{
  // this is crafted so offsets are completely random
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x611, "jne qword [var_4h]");
  stmts.emplace_back(0x612, "jmp dword [var_8h]");
  stmts.emplace_back(0x613, "ret");

  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  const BasicBlock* cfg = anal.get_cfg()->root();
  const BasicBlock* next;
  const BasicBlock* cond;

  // check if cfg is correct
  EXPECT_EQ(cfg->get_id(), 0); // 0
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  EXPECT_EQ(next->get_id(), 1);
  EXPECT_EQ(cond, nullptr);

  cfg = next; // 1
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  EXPECT_NE(next, nullptr); // finalize() created a new node
  EXPECT_EQ(cond, nullptr);
}

TEST(Analysis, cfg_long_conditional_jmp)
{
  // this is crafted so offsets are completely random
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x611, "jo 0xFFFFFFFFFFFFFFFC");
  stmts.emplace_back(0x612, "jno 0x600");
  stmts.emplace_back(0x615, "ret");

  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  const BasicBlock* cfg = anal.get_cfg()->root();
  const BasicBlock* next;
  const BasicBlock* cond;

  // check if cfg is correct
  EXPECT_EQ(cfg->get_id(), 0); // 0
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  EXPECT_EQ(next->get_id(), 1);
  EXPECT_EQ(cond, nullptr);

  cfg = next; // 1
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  EXPECT_EQ(next->get_id(), 2);
  EXPECT_EQ(cond, nullptr);

  cfg = next; // 2
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  EXPECT_EQ(next->get_id(), 3);
  EXPECT_EQ(cond, nullptr);

  cfg = next; // 3
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  EXPECT_NE(next, nullptr); // again, finalize() created an exit node
  EXPECT_EQ(cond, nullptr);
}

TEST(Analysis, cfg_long_unconditional_jump)
{
  // this is crafted so offsets are completely random
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x611, "je 0x613");
  stmts.emplace_back(0x612, "jmp 0xFFFFFFFFFFFFFFFC");
  stmts.emplace_back(0x613, "jmp 0x600");
  stmts.emplace_back(0x614, "jmp 0x615");
  stmts.emplace_back(0x615, "ret");

  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  anal.get_cfg()->to_file("/home/davide/Desktop/test.dot");
  const BasicBlock* cfg = anal.get_cfg()->root();
  const BasicBlock* next;
  const BasicBlock* cond;

  // check if cfg is correct
  EXPECT_EQ(cfg->get_id(), 0); // 0
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  ASSERT_NE(next, nullptr);
  ASSERT_NE(cond, nullptr);
  EXPECT_EQ(next->get_id(), 1);
  EXPECT_EQ(cond->get_id(), 2);

  cfg = next; // 1
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  EXPECT_NE(next, nullptr); // again, finalize() created an exit node
  EXPECT_EQ(cond, nullptr);

  cfg = static_cast<const BasicBlock*>(anal.get_cfg()->root()->get_cond()); // 2
  next = static_cast<const BasicBlock*>(cfg->get_next());
  cond = static_cast<const BasicBlock*>(cfg->get_cond());
  EXPECT_NE(next, nullptr); // again, finalize() created an exit node
  EXPECT_EQ(cond, nullptr);
}

TEST(Analysis, offset_retained)
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x610, "test edi, edi");
  stmts.emplace_back(0x614, "je 0x628");
  stmts.emplace_back(0x618, "test esi, esi");
  stmts.emplace_back(0x61C, "mov eax, 5");
  stmts.emplace_back(0x620, "je 0x628");
  stmts.emplace_back(0x624, "ret");
  stmts.emplace_back(0x628, "mov eax, 6");
  stmts.emplace_back(0x62C, "ret");
  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  std::shared_ptr<const ControlFlowGraph> cfg = anal.get_cfg();
  std::shared_ptr<const ControlFlowStructure> cfs = anal.get_cfs();
  ASSERT_TRUE(anal.successful());
  const AbstractBlock* root = cfs->root();
  ASSERT_NE(root, nullptr);
  EXPECT_EQ(root->get_type(), SEQUENCE);
  ASSERT_EQ(root->size(), 2);
  root = (*root)[0];
  EXPECT_EQ(root->get_type(), IF_ELSE);
  ASSERT_EQ(root->size(), 4);
  const AbstractBlock* node;
  const BasicBlock* leaf;
  uint64_t start;
  uint64_t end;
  node = (*root)[0]; // head
  ASSERT_EQ(node->get_type(), BASIC);
  leaf = static_cast<const BasicBlock*>(node);
  leaf->get_offset(&start, &end);
  EXPECT_EQ(start, 0x610);
  EXPECT_EQ(end, 0x618);
  node = (*root)[2]; // else
  ASSERT_EQ(node->get_type(), BASIC);
  leaf = static_cast<const BasicBlock*>(node);
  leaf->get_offset(&start, &end);
  EXPECT_EQ(start, 0x628);
  EXPECT_EQ(end, 0x62C);
}

TEST(Analysis, offset_64bit)
{
  std::vector<Statement> stmts;
  stmts.emplace_back(0x3FD1A7EF534, "jmp 0x3FD1A7EF538");
  stmts.emplace_back(0x3FD1A7EF538, "incl eax");
  stmts.emplace_back(0x3FD1A7EF53C, "mov ebx, [ebp+20]");
  stmts.emplace_back(0x3FD1A7EF540, "cmp eax, ebx");
  stmts.emplace_back(0x3FD1A7EF544, "je 0x3FD1A7EF558");
  stmts.emplace_back(0x3FD1A7EF548, "mov ecx, [ebp+20]");
  stmts.emplace_back(0x3FD1A7EF54C, "decl ecx");
  stmts.emplace_back(0x3FD1A7EF550, "mov [ebp+20], ecx");
  stmts.emplace_back(0x3FD1A7EF554, "jmp 0x3FD1A7EF538");
  stmts.emplace_back(0x3FD1A7EF558, "test eax, eax");
  stmts.emplace_back(0x3FD1A7EF55C, "mov eax, 0");
  stmts.emplace_back(0x3FD1A7EF560, "je 0x3FD1A7EF568");
  stmts.emplace_back(0x3FD1A7EF564, "mov eax, 1");
  stmts.emplace_back(0x3FD1A7EF568, "ret");
  Analysis anal("", "", &stmts,
                std::shared_ptr<Architecture>{new ArchitectureX86()});
  ASSERT_TRUE(anal.successful());
  ASSERT_EQ(anal.get_cfg()->nodes_no(), 6);
  const BasicBlock* root = anal.get_cfg()->root();
  const BasicBlock* current;
  EXPECT_EQ(root->get_id(), 0);
  ASSERT_EQ(root->get_out_edges(), 1);
  current = static_cast<const BasicBlock*>(root->get_next());
  EXPECT_EQ(current->get_id(), 1);
  ASSERT_EQ(current->get_out_edges(), 2);
}
