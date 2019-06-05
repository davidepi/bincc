#include "r2_stmt.hpp"
#include "r2_func.hpp"
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

R2Stmt::R2Stmt() : offset(0x0), target(0x0) {}

bool R2Stmt::from_JSON(const std::string& json_string)
{
    bool retval;
    if(!json_string.empty())
    {
        try
        {
            Json parsed = Json::parse(json_string);
            if(parsed.empty())
            {
                retval = false;
            }
            else if(strcmp(parsed["type"].get<std::string>().c_str(),
                           "invalid") == 0)
            {
                opcode = "invalid";
                esil = opcode;
            }
            else
            {
                offset = parsed["offset"].get<int>();
                opcode = parsed["disasm"].get<std::string>();
                esil = parsed["esil"].get<std::string>();
            }
            if(strcmp(parsed["type"].get<std::string>().c_str(), "call") == 0)
            {
                target = parsed["jump"].get<int>();
            }
            retval = true;
        }
        catch(Json::exception& e)
        {
            fprintf(stderr, "%s\n", e.what());
            retval = false;
        }
    }
    else
    {
        retval = false;
    }
    return retval;
}

int R2Stmt::get_offset() const
{
    return offset;
}

int R2Stmt::get_target() const
{
    return target;
}

const std::string& R2Stmt::get_esil() const
{
    return esil;
}

const std::string& R2Stmt::get_opcode() const
{
    return opcode;
}
