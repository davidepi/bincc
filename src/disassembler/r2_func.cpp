#include "r2_func.hpp"
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

bool R2Func::from_JSON(const std::string& json_string)
{
    bool retval;
    if(!json_string.empty())
    {
        try
        {
            Json parsed = Json::parse(json_string);
            //first save to tmp vars
            int tmp_off = parsed["offset"].get<int>();
            std::string tmp_name = parsed["name"].get<std::string>();
            std::string type_str = parsed["type"].get<std::string>();
            FunctionT tmp_type;
            if(strcmp(type_str.c_str(), "sym") == 0)
            {
                tmp_type = FunctionT::SYM;
            }
            else if(strcmp(type_str.c_str(), "fcn") == 0)
            {
                tmp_type = FunctionT::FCN;
            }
            else if(strcmp(type_str.c_str(), "loc") == 0)
            {
                tmp_type = FunctionT::LOC;
            }
            else if(strcmp(type_str.c_str(), "int") == 0)
            {
                tmp_type = FunctionT::INT;
            }
            else
            {
                fprintf(stderr, "Unknown function type %s", type_str.c_str());
                return false;
            }

            //at this point if no exceptions, copy to the actual values
            offset = tmp_off;
            name = tmp_name;
            type = tmp_type;
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

R2Func::R2Func():name(""), type(FCN), offset(0)
{

}

int R2Func::get_offset() const
{
    return offset;
}

const std::string& R2Func::get_name() const
{
    return name;
}

FunctionT R2Func::get_type() const
{
    return type;
}

void R2Func::add_instruction(const R2Stmt& stmt)
{
    body.push_back(stmt);
}

const std::vector<R2Stmt>& R2Func::get_body() const
{
    return body;
}
