#include <gtest/gtest.h>
#include "disassembler/R2Pipe.hpp"


TEST(R2Pipe, create_and_destroy)
{
    R2Pipe r2;
}

TEST(R2Pipe, analyzed_file)
{
    R2Pipe r2;
    EXPECT_EQ(r2.get_analyzed_file(), nullptr);
    EXPECT_TRUE(r2.set_analyzed_file("/bin/ls"));
    EXPECT_STREQ(r2.get_analyzed_file(), "/bin/ls");
    EXPECT_FALSE(r2.set_analyzed_file("ju,khugljkb"));
    EXPECT_STREQ(r2.get_analyzed_file(), "/bin/ls");
    EXPECT_TRUE(r2.set_analyzed_file("/bin/touch"));
    EXPECT_STREQ(r2.get_analyzed_file(), "/bin/touch");
}

TEST(R2Pipe, executable)
{
    R2Pipe r2;
    EXPECT_STREQ(r2.get_executable(), "r2");
    EXPECT_TRUE(r2.set_executable("/usr/bin/r2"));
    EXPECT_STREQ(r2.get_executable(), "/usr/bin/r2");
    EXPECT_FALSE(r2.set_executable("ouhbk"));
    EXPECT_STREQ(r2.get_executable(), "/usr/bin/r2");
    EXPECT_TRUE(r2.set_executable("/usr/bin/radare2"));
    EXPECT_STREQ(r2.get_executable(), "/usr/bin/radare2");
}
