#include "disassembler/radare2/r2_pipe.hpp"
#include <gtest/gtest.h>

/**
 * \brief Tests for the R2Pipe class
 */
TEST(R2Pipe, create_and_destroy)
{
    R2Pipe r2;
}

TEST(R2Pipe, analyzed_file)
{
    R2Pipe r2;
    EXPECT_EQ(r2.get_analyzed_file(), nullptr);
    EXPECT_TRUE(r2.set_analyzed_file(TESTS_DIR "resources/ls_unstripped_x86"));
    EXPECT_STREQ(r2.get_analyzed_file(), TESTS_DIR "resources/ls_unstripped_x86");
    EXPECT_FALSE(r2.set_analyzed_file("ju,khugljkb"));
    EXPECT_STREQ(r2.get_analyzed_file(), TESTS_DIR "resources/ls_unstripped_x86");
    EXPECT_TRUE(r2.set_analyzed_file(TESTS_DIR "resources/touch_unstripped_x86"));
    EXPECT_STREQ(r2.get_analyzed_file(),
                 TESTS_DIR "resources/touch_unstripped_x86");
}

TEST(R2Pipe, executable)
{
    R2Pipe r2;
    EXPECT_STREQ(r2.get_executable(), "/usr/bin/r2");
    EXPECT_TRUE(r2.set_executable(RADARE2_PATH));
    EXPECT_STREQ(r2.get_executable(), RADARE2_PATH);
    EXPECT_FALSE(r2.set_executable("ouhbk"));
    EXPECT_STREQ(r2.get_executable(), RADARE2_PATH);
}

TEST(R2Pipe, analyze)
{
    std::string res;

    R2Pipe r2;
    ASSERT_TRUE(r2.set_executable(RADARE2_PATH));
    ASSERT_TRUE(r2.set_analyzed_file(TESTS_DIR "resources/ls_unstripped_x86"));
    ASSERT_TRUE(r2.open());
    ASSERT_FALSE(r2.set_analyzed_file(TESTS_DIR "resources/touch_unstripped_x86"));
    ASSERT_FALSE(r2.open());
    res = r2.exec("ij");
    r2.close();
    EXPECT_STRNE(res.c_str(), "");

    res = "";
    res = r2.exec("ij");
    EXPECT_STREQ(res.c_str(), "");
}
