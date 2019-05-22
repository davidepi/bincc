#ifndef __R2_RESPONSE_HPP__
#define __R2_RESPONSE_HPP__


#include <string>

class R2Response
{
public:
    virtual bool fromJSON(const std::string& json_string) = 0;
};


#endif
