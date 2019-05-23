#ifndef __R2_INFO_HPP__
#define __R2_INFO_HPP__


#include "r2_response.hpp"

/**
 * \brief Class storing information about an executable file.
 *
 * The information is expected to be retrieved as the output of an R2Pipe
 * class by issuing the command `ij`
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class R2Info : public R2Response
{
public:
    /**
     * \brief Default constructor, initializes every information to false
     */
    R2Info();

    /**
     * \brief Default destructor
     */
    ~R2Info() = default;

    /**
     * \brief Parse the string retrieved by the radare2 process
     *
     * This method populates this class by parsing the string retrieved by
     * issuing the `ij` command to r2. Attempting to parse any other JSON or
     * strings will fail
     *
     * \param[in] json_string The JSON string that will be parsed
     * \return true if the string was valid and this class has been
     * populated, false otherwise
     */
    bool from_JSON(const std::string& json_string) override;

    /**
     * \brief Getter for the architecture
     *
     * \return true if the architecture is x86 or AMD64, false otherwise
     */
    bool is_x86() const;

    /**
     * \brief Getter for the endianness
     *
     * \return true if the executable is big endian, false otherwise
     */
    bool is_bigendian() const;

    /**
     * \brief Getter for the buffer overflow protections
     *
     * \return true if the executable uses any form of canaries, a.k.a. stack
     * protections, false otherwise
     */
    bool has_canaries() const;

    /**
     * \brief Getter for the stripped files
     *
     * \return true if the executable is stripped (debugging symbols has been
     * removed), false otherwise
     */
    bool is_stripped() const;

    /**
     * \brief Getter for 64 bits executables
     *
     * \return true if the executable is 64 bit, false otherwise
     */
    bool is_64bit() const;

private:
    //true if the architecture string is "x86"
    bool x86_arch;

    //true if big endian
    bool big_endian;

    //true if it has stack protections
    bool canary;

    //true if it is stripped
    bool stripped;

    //true if it is 64 bits
    bool bits_64;
};


#endif
