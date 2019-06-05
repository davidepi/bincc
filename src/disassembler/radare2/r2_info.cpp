#include "r2_info.hpp"
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

R2Info::R2Info()
    : arch(UNKNOWN), bits_64(false), stripped(false), canary(false),
      big_endian(false)
{
}

bool R2Info::from_JSON(const std::string& json_string)
{
    bool retval;
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
            if(strarch == "x86")
            {
                arch = Architecture::X86;
            }
            else
            {
                arch = Architecture::UNKNOWN;
            }
            big_endian = endian;
            canary = can;
            stripped = strip;
            bits_64 = bits;
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

Architecture R2Info::get_arch() const
{
    return arch;
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
