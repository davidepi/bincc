#include <gtest/gtest.h>
#include "disassembler/r2_info.hpp"


/**
 * \brief Tests for the classes implementing R2Response
 */
TEST(R2Res, Info)
{
    R2Info info;
    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_FALSE(info.is_x86());
    EXPECT_FALSE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    EXPECT_FALSE(info.fromJSON("totally random"));

    std::string json = "{\"core\":{\"type\":\"DYN (Shared object file)\","
                       "\"file\":\"/bin/ls\",\"fd\":3,\"size\":133792,\""
                       "humansz\":\"130.7K\",\"iorw\":false,\"mode\":\"-r-x\","
                       "\"obsz\":0,\"block\":256,\"format\":\"elf64\"},\"bin\":"
                       "{\"arch\":\"x86\",\"binsz\":131997,\"bintype\":\"elf\","
                       "\"bits\":64,\"canary\":true,\"class\":\"ELF64\",\"compi"
                       "led\":\"\",\"crypto\":false,\"dbg_file\":\"\",\"endia"
                       "n\":\"little\",\"havecode\":true,\"guid\":\"\",\"intrp"
                       "\":\"/lib64/ld-linux-x86-64.so.2\",\"lang\":\"c\",\""
                       "linenum\":false,\"lsyms\":false,\"machine\":\"AMD x86-"
                       "64 architecture\",\"maxopsz\":16,\"minopsz\":1,\"nx\":t"
                       "rue,\"os\":\"linux\",\"pcalign\":0,\"pic\":true,\"relo"
                       "cs\":false,\"relro\":\"full\",\"rpath\":\"NONE\",\"stat"
                       "ic\":false,\"stripped\":true,\"subsys\":\"linux\",\"va"
                       "\":true,\"checksums\":{}}}";

    ASSERT_FALSE(info.fromJSON(""));
    ASSERT_TRUE(info.fromJSON(json));
    EXPECT_TRUE(info.has_canaries());
    EXPECT_TRUE(info.is_64bit());
    EXPECT_TRUE(info.is_x86());
    EXPECT_TRUE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    //should remain the same
    ASSERT_FALSE(info.fromJSON(""));
    EXPECT_TRUE(info.has_canaries());
    EXPECT_TRUE(info.is_64bit());
    EXPECT_TRUE(info.is_x86());
    EXPECT_TRUE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    //opposite values of before
    std::string jsn2 = "{\"core\":{\"type\":\"DYN (Shared object file)\","
                       "\"file\":\"/bin/ls\",\"fd\":3,\"size\":133792,\""
                       "humansz\":\"130.7K\",\"iorw\":false,\"mode\":\"-r-x\","
                       "\"obsz\":0,\"block\":256,\"format\":\"elf64\"},\"bin\":"
                       "{\"arch\":\"arm\",\"binsz\":131997,\"bintype\":\"elf\","
                       "\"bits\":32,\"canary\":false,\"class\":\"ELF\",\"compi"
                       "led\":\"\",\"crypto\":false,\"dbg_file\":\"\",\"endia"
                       "n\":\"big\",\"havecode\":true,\"guid\":\"\",\"intrp"
                       "\":\"/lib64/ld-linux-x86-64.so.2\",\"lang\":\"c\",\""
                       "linenum\":false,\"lsyms\":false,\"machine\":\"AMD x86-"
                       "64 architecture\",\"maxopsz\":16,\"minopsz\":1,\"nx\":t"
                       "rue,\"os\":\"linux\",\"pcalign\":0,\"pic\":true,\"relo"
                       "cs\":false,\"relro\":\"full\",\"rpath\":\"NONE\",\"stat"
                       "ic\":false,\"stripped\":false,\"subsys\":\"linux\",\"va"
                       "\":true,\"checksums\":{}}}";
    ASSERT_TRUE(info.fromJSON(jsn2));
    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_FALSE(info.is_x86());
    EXPECT_FALSE(info.is_stripped());
    EXPECT_TRUE(info.is_bigendian());
}
