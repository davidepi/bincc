#include "disassembler/disassembler.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include <gtest/gtest.h>
#include <sstream>

TEST(Disassembler, R2_arch)
{
    DisassemblerR2 disasm_x86(TESTS_DIR "resources/add_x86");
    EXPECT_STREQ(disasm_x86.get_arch()->get_name().c_str(), "unknown");
    disasm_x86.analyse();
    EXPECT_STREQ(disasm_x86.get_arch()->get_name().c_str(), "x86");

    DisassemblerR2 disasm_arm(TESTS_DIR "resources/add_arm");
    EXPECT_STREQ(disasm_arm.get_arch()->get_name().c_str(), "unknown");
    disasm_arm.analyse();
    EXPECT_STREQ(disasm_arm.get_arch()->get_name().c_str(), "arm");
}

TEST(Disassembler, R2_functions)
{
    std::set<Function> functions;
    DisassemblerR2 disasm(TESTS_DIR "resources/add_x86");
    functions = disasm.get_function_names();
    EXPECT_EQ(functions.size(), 0);
    disasm.analyse();
    functions = disasm.get_function_names();
    EXPECT_GT(functions.size(), 0);
}

TEST(Disassembler, R2_function_bodies)
{
    std::string body;
    DisassemblerR2 disasm(TESTS_DIR "resources/add_x86");
    body = disasm.get_function_as_string("sym.add_multiple");
    EXPECT_EQ(body.length(), 0);
    disasm.analyse();
    body = disasm.get_function_as_string("sym.add_multiple");
    EXPECT_GT(body.length(), 0);
}

TEST(Disassembler, change_binary)
{
    std::string new_name = TESTS_DIR "resources/add_arm";
    DisassemblerR2 disasm(TESTS_DIR "resources/add_x86");
    disasm.set_binary(new_name.c_str());
    EXPECT_STREQ(disasm.get_binary_name().c_str(), new_name.c_str());
}

TEST(Disassembler, stream_operator)
{
    std::string name = TESTS_DIR "resources/add_arm";
    DisassemblerR2 disasm(name.c_str());
    std::stringstream expected;
    expected << "--- " << TESTS_DIR << "resources/add_arm ---\n"
             << "-----------------------------------------------------";
    std::stringstream res;
    res << disasm;
    EXPECT_STREQ(res.str().c_str(), expected.str().c_str());

    disasm.analyse();
    expected.clear();
    res.clear();
    res << disasm;
    expected << "--- " << TESTS_DIR << "resources/add_arm ---\n";
    expected << "sym._init\n"
                "|0x1029C\tpush\n"
                "|0x102A0\tbl\n"
                "|0x102A4\tpop\n"
                ";\n"
                "\n"
                "sym.imp.__libc_start_main\n"
                "|0x102BC\tadd\n"
                "|0x102C0\tadd\n"
                "|0x102C4\tldr\n"
                ";\n"
                "\n"
                "sym.imp.abort\n"
                "|0x102D4\tadd\n"
                "|0x102D8\tadd\n"
                "|0x102DC\tldr\n"
                ";\n"
                "\n"
                "entry0\n"
                "|0x102E0\tmov\n"
                "|0x102E4\tmov\n"
                "|0x102E8\tpop\n"
                "|0x102EC\tmov\n"
                "|0x102F0\tstr\n"
                "|0x102F4\tstr\n"
                "|0x102F8\tldr\n"
                "|0x102FC\tstr\n"
                "|0x10300\tldr\n"
                "|0x10304\tldr\n"
                "|0x10308\tbl\n"
                ";\n"
                "\n"
                "sym.call_weak_fn\n"
                "|0x1031C\tldr\n"
                "|0x10320\tldr\n"
                "|0x10324\tadd\n"
                "|0x10328\tldr\n"
                "|0x1032C\tcmp\n"
                "|0x10330\tbxeq\n"
                "|0x10334\tb\n"
                ";\n"
                "\n"
                "sym.deregister_tm_clones\n"
                "|0x10340\tldr\n"
                "|0x10344\tldr\n"
                "|0x10348\tsub\n"
                "|0x1034C\tcmp\n"
                "|0x10350\tbxls\n"
                "|0x10354\tldr\n"
                "|0x10358\tcmp\n"
                "|0x1035C\tbxeq\n"
                "|0x10360\tbx\n"
                ";\n"
                "\n"
                "sym.register_tm_clones\n"
                "|0x10370\tldr\n"
                "|0x10374\tldr\n"
                "|0x10378\tsub\n"
                "|0x1037C\tasr\n"
                "|0x10380\tadd\n"
                "|0x10384\tasrs\n"
                "|0x10388\tbxeq\n"
                "|0x1038C\tldr\n"
                "|0x10390\tcmp\n"
                "|0x10394\tbxeq\n"
                "|0x10398\tbx\n"
                ";\n"
                "\n"
                "entry.fini0\n"
                "|0x103A8\tpush\n"
                "|0x103AC\tldr\n"
                "|0x103B0\tldrb\n"
                "|0x103B4\tcmp\n"
                "|0x103B8\tpopne\n"
                "|0x103BC\tbl\n"
                "|0x103C0\tmov\n"
                "|0x103C4\tstrb\n"
                "|0x103C8\tpop\n"
                ";\n"
                "\n"
                "entry.init0\n"
                "|0x103D0\tldr\n"
                "|0x103D4\tldr\n"
                "|0x103D8\tcmp\n"
                "|0x103DC\tbne\n"
                "|0x103E0\tb\n"
                "|0x103E4\tldr\n"
                "|0x103E8\tcmp\n"
                "|0x103EC\tbeq\n"
                "|0x103F0\tpush\n"
                "|0x103F4\tblx\n"
                "|0x103F8\tpop\n"
                "|0x103FC\tb\n"
                ";\n"
                "\n"
                "sym.add_constants\n"
                "|0x10408\tstr\n"
                "|0x1040C\tadd\n"
                "|0x10410\tsub\n"
                "|0x10414\tstr\n"
                "|0x10418\tstr\n"
                "|0x1041C\tldr\n"
                "|0x10420\tldr\n"
                "|0x10424\tadd\n"
                "|0x10428\tmov\n"
                "|0x1042C\tadd\n"
                "|0x10430\tpop\n"
                "|0x10434\tbx\n"
                ";\n"
                "\n"
                "sym.add_multiple\n"
                "|0x10438\tstr\n"
                "|0x1043C\tadd\n"
                "|0x10440\tsub\n"
                "|0x10444\tstr\n"
                "|0x10448\tstr\n"
                "|0x1044C\tstr\n"
                "|0x10450\tstr\n"
                "|0x10454\tldr\n"
                "|0x10458\tldr\n"
                "|0x1045C\tadd\n"
                "|0x10460\tldr\n"
                "|0x10464\tadd\n"
                "|0x10468\tldr\n"
                "|0x1046C\tadd\n"
                "|0x10470\tmov\n"
                "|0x10474\tadd\n"
                "|0x10478\tpop\n"
                "|0x1047C\tbx\n"
                ";\n"
                "\n"
                "main\n"
                "|0x10480\tpush\n"
                "|0x10484\tadd\n"
                "|0x10488\tsub\n"
                "|0x1048C\tstr\n"
                "|0x10490\tstr\n"
                "|0x10494\tmov\n"
                "|0x10498\tmov\n"
                "|0x1049C\tbl\n"
                "|0x104A0\tstr\n"
                "|0x104A4\tmov\n"
                "|0x104A8\tmov\n"
                "|0x104AC\tsub\n"
                "|0x104B0\tpop\n"
                ";\n"
                "\n"
                "sym.__libc_csu_init\n"
                "|0x104B4\tpush\n"
                "|0x104B8\tmov\n"
                "|0x104BC\tldr\n"
                "|0x104C0\tldr\n"
                "|0x104C4\tadd\n"
                "|0x104C8\tadd\n"
                "|0x104CC\tsub\n"
                "|0x104D0\tmov\n"
                "|0x104D4\tmov\n"
                "|0x104D8\tbl\n"
                "|0x104DC\tasrs\n"
                "|0x104E0\tpopeq\n"
                "|0x104E4\tmov\n"
                "|0x104E8\tadd\n"
                "|0x104EC\tldr\n"
                "|0x104F0\tmov\n"
                "|0x104F4\tmov\n"
                "|0x104F8\tmov\n"
                "|0x104FC\tblx\n"
                "|0x10500\tcmp\n"
                "|0x10504\tbne\n"
                "|0x10508\tpop\n"
                ";\n"
                "\n"
                "sym.__libc_csu_fini\n"
                "|0x10514\tbx\n"
                ";\n"
                "\n"
                "sym._fini\n"
                "|0x10518\tpush\n"
                "|0x1051C\tpop\n"
                ";\n"
                "\n"
                "-----------------------------------------------------";
    EXPECT_STREQ(res.str().c_str(), expected.str().c_str());
}
