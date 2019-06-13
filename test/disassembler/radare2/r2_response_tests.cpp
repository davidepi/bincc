#include "disassembler/function.hpp"
#include "disassembler/info.hpp"
#include "disassembler/radare2/r2_json_parser.hpp"
#include "disassembler/statement.hpp"
#include <gtest/gtest.h>

/**
 * \brief Tests for the classes implementing R2Response
 */
TEST(R2Parser, Info)
{
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

    // opposite values of before
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

    Info info;
    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_EQ(info.get_arch(), Architecture::UNKNOWN);
    EXPECT_FALSE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    info = R2JsonParser::parse_info("totally random");
    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_EQ(info.get_arch(), Architecture::UNKNOWN);
    EXPECT_FALSE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_EQ(info.get_arch(), Architecture::UNKNOWN);
    EXPECT_FALSE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    info = R2JsonParser::parse_info(json);
    EXPECT_TRUE(info.has_canaries());
    EXPECT_TRUE(info.is_64bit());
    EXPECT_EQ(info.get_arch(), Architecture::X86);
    EXPECT_TRUE(info.is_stripped());
    EXPECT_FALSE(info.is_bigendian());

    info = R2JsonParser::parse_info(jsn2);
    EXPECT_FALSE(info.has_canaries());
    EXPECT_FALSE(info.is_64bit());
    EXPECT_EQ(info.get_arch(), Architecture::ARM);
    EXPECT_FALSE(info.is_stripped());
    EXPECT_TRUE(info.is_bigendian());
}

TEST(R2Parser, Func)
{
    // default
    Function func;

    std::string json;
    json = "{\"offset\":90988,\"name\":\"sym._fini\",\"size\":9,\"realsz\":9,\""
           "cc\":1,\"cost\":5,\"nbbs\":1,\"edges\":0,\"ebbs\":1,\"calltype\":\""
           "amd64\",\"type\":\"sym\",\"diff\":\"NEW\",\"difftype\":\"new\",\"in"
           "degree\":0,\"outdegree\":0,\"nargs\":0,\"nlocals\":0}";

    func = R2JsonParser::parse_function("totally random");
    EXPECT_STREQ("", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x0);

    func = R2JsonParser::parse_function("");
    EXPECT_STREQ("", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x0);

    func = R2JsonParser::parse_function(json);
    EXPECT_STREQ("sym._fini", func.get_name().c_str());
    EXPECT_EQ(func.get_offset(), 0x1636C);
}

TEST(R2Parser, stmt)
{
    Statement stmt;
    std::string json;
    json = "{\"offset\":83072,\"esil\":\"rbx,8,rsp,-=,rsp,=[8]\",\"refptr\":fal"
           "se,\"fcn_addr\":83072,\"fcn_last\":83153,\"size\":1,\"opcode\":\"pu"
           "sh rbx\",\"disasm\":\"push rbx\",\"bytes\":\"53\",\"family\":\"cpu"
           "\",\"type\":\"upush\",\"type_num\":12,\"type2_num\":0}";

    EXPECT_EQ(stmt.get_offset(), 0x0);
    EXPECT_STREQ(stmt.get_command().c_str(), "");

    stmt = R2JsonParser::parse_statement("totally random");
    EXPECT_EQ(stmt.get_offset(), 0x0);
    EXPECT_STREQ(stmt.get_command().c_str(), "");

    stmt = R2JsonParser::parse_statement("");
    EXPECT_EQ(stmt.get_offset(), 0x0);
    EXPECT_STREQ(stmt.get_command().c_str(), "");

    stmt = R2JsonParser::parse_statement("{}");
    EXPECT_EQ(stmt.get_offset(), 0x0);
    EXPECT_STREQ(stmt.get_command().c_str(), "");

    stmt = R2JsonParser::parse_statement(json);
    EXPECT_EQ(stmt.get_offset(), 0x14480);
    EXPECT_STREQ(stmt.get_command().c_str(), "push rbx");
}
