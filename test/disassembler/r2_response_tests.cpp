#include <gtest/gtest.h>
#include "disassembler/r2_func.hpp"
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

    EXPECT_FALSE(info.from_JSON("totally random"));

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

    ASSERT_FALSE(info.from_JSON(""));
    ASSERT_TRUE(info.from_JSON(json));
    EXPECT_TRUE(info.has_canaries());
    EXPECT_TRUE(info.is_64bit());
    EXPECT_TRUE(info.is_x86());
    EXPECT_TRUE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    //should remain the same
    ASSERT_FALSE(info.from_JSON(""));
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
    ASSERT_TRUE(info.from_JSON(jsn2));
    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_FALSE(info.is_x86());
    EXPECT_FALSE(info.is_stripped());
    EXPECT_TRUE(info.is_bigendian());
}

TEST(R2Res, Func)
{
    //default
    R2Func func;
    std::string json;
    EXPECT_STREQ("", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x0);
    EXPECT_EQ(func.get_type(), FunctionT::FCN);
    EXPECT_FALSE(func.from_JSON("totally random"));
    EXPECT_STREQ("", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x0);
    EXPECT_EQ(func.get_type(), FunctionT::FCN);

    //SYM
    json = "{\"offset\":90988,\"name\":\"sym._fini\",\"size\":9,\"realsz\":9,\""
           "cc\":1,\"cost\":5,\"nbbs\":1,\"edges\":0,\"ebbs\":1,\"calltype\":\""
           "amd64\",\"type\":\"sym\",\"diff\":\"NEW\",\"difftype\":\"new\",\"in"
           "degree\":0,\"outdegree\":0,\"nargs\":0,\"nlocals\":0}";
    ASSERT_TRUE(func.from_JSON(json));
    EXPECT_STREQ("sym._fini", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x1636C);
    EXPECT_EQ(func.get_type(), FunctionT::SYM);

    //FCN
    json = "{\"offset\":48,\"name\":\"fcn.00000030\",\"size\":16,\"realsz\":16,"
           "\"cc\":1,\"cost\":7,\"nbbs\":1,\"edges\":0,\"ebbs\":1,\"calltype\":"
           "\"amd64\",\"type\":\"fcn\",\"diff\":\"NEW\",\"difftype\":\"new\",\""
           "indegree\":0,\"outdegree\":0,\"nargs\":0,\"nlocals\":0}";
    ASSERT_TRUE(func.from_JSON(json));
    EXPECT_STREQ("fcn.00000030", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x30);
    EXPECT_EQ(func.get_type(), FunctionT::FCN);

    //none of the above
    json = "{\"offset\":48,\"name\":\"fcn.00000030\",\"size\":16,\"realsz\":16,"
           "\"cc\":1,\"cost\":7,\"nbbs\":1,\"edges\":0,\"ebbs\":1,\"calltype\":"
           "\"amd64\",\"type\":\"sub\",\"diff\":\"NEW\",\"difftype\":\"new\",\""
           "indegree\":0,\"outdegree\":0,\"nargs\":0,\"nlocals\":0}";
    ASSERT_FALSE(func.from_JSON(json));
    EXPECT_STREQ("fcn.00000030", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x30);
    EXPECT_EQ(func.get_type(), FunctionT::FCN);
}
