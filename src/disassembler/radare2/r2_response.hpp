#ifndef __R2_RESPONSE_HPP__
#define __R2_RESPONSE_HPP__

#include <string>

/**
 * \brief Interface for classes obtained by a radare2 response
 *
 * Every command issued to radare2 terminating with the letter `j` returns a
 * JSON. This class is an interface providing a method useful to reconstruct
 * a particular JSON returned from radare2 in this application.
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class R2Response
{
public:
    /**
     * Initialize the class by using a JSON returned from radare2
     *
     * \param[in] jsonString the JSON string obtained from radare2 program
     */
    virtual bool from_JSON(const std::string& json_string) = 0;
};

#endif
