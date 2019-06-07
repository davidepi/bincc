
#include "r2_json_parser.hpp"
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

Function R2JsonParser::parse_function(const std::string& json_string)
{
    if(!json_string.empty())
    {
        try
        {
            Json parsed = Json::parse(json_string);
            // first save to tmp vars
            uint64_t tmp_off = parsed["offset"].get<uint64_t>();
            std::string tmp_name = parsed["name"].get<std::string>();

            // at this point if no exceptions, copy to the actual values
            return Function(tmp_off, std::move(tmp_name));
        }
        catch(Json::exception& e)
        {
            fprintf(stderr, "%s\n", e.what());
            return Function();
        }
    }
    else
    {
        return Function();
    }
}

Info R2JsonParser::parse_info(const std::string& json_string)
{
    if(!json_string.empty())
    {
        try
        {
            Json parsed = Json::parse(json_string)["bin"];
            // first save to tmp vars
            std::string strarch = parsed["arch"].get<std::string>();
            bool endian = parsed["endian"].get<std::string>() == "big";
            bool can = parsed["canary"].get<bool>();
            bool strip = parsed["stripped"].get<bool>();
            bool bits = parsed["bits"].get<int>() == 64;

            // at this point if no exceptions, copy to the actual values
            Architecture arch;
            if(strarch == "x86")
            {
                arch = Architecture::X86;
            }
            else if(strarch == "arm")
            {
                arch = Architecture::ARM;
            }
            else
            {
                arch = Architecture::UNKNOWN;
            }
            return Info(arch, endian, can, strip, bits);
        }
        catch(Json::exception& e)
        {
            fprintf(stderr, "%s\n", e.what());
            return Info();
        }
    }
    else
    {
        return Info();
    }
}

Statement R2JsonParser::parse_statement(const std::string& json_string)
{
    if(!json_string.empty())
    {
        try
        {
            uint64_t offset;
            std::string opcode;
            Json parsed = Json::parse(json_string);
            if(parsed.empty())
            {
                return Statement();
            }

            offset = parsed["offset"].get<uint64_t>();
            opcode = parsed["type"].get<std::string>();
            if(opcode != "invalid")
            {
                opcode = parsed["disasm"].get<std::string>();
            }
            else
            {
                opcode = "nop";
            }
            return Statement(offset, std::move(opcode));
        }
        catch(Json::exception& e)
        {
            return Statement();
        }
    }
    else
    {
        return Statement();
    }
}
