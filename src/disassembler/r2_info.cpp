#include "r2_info.hpp"
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

R2Info::R2Info():x86_arch(false), bits_64(false), stripped(false),
                 canary(false), big_endian(false)
{

}

bool R2Info::fromJSON(const std::string& json_string)
{
    bool retval;
    if(!json_string.empty())
    {
        try
        {
            Json parsed = Json::parse(json_string)["bin"];
            x86_arch = parsed["arch"].get<std::string>() == "x86";
            big_endian = parsed["endian"].get<std::string>() == "big";
            canary = parsed["canary"].get<bool>() == true;
            stripped = parsed["stripped"].get<bool>() == true;
            bits_64 = parsed["bits"].get<int>() == 64;
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

bool R2Info::is_x86() const
{
    return x86_arch;
}

bool R2Info::is_bigendian() const
{
    return big_endian;
}

bool R2Info::has_canaries() const
{
    return canary;
}

bool R2Info::is_stripped() const
{
    return stripped;
}

bool R2Info::is_64bit() const
{
    return bits_64;
}

