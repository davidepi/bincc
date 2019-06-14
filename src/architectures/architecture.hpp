//
// Created by davide on 6/14/19.
//

#ifndef __ARCHITECTURE_HPP__
#define __ARCHITECTURE_HPP__

#include <string>

/**
 * \brief Describe if a jump is conditional or not
 */
enum JumpType
{
    /**
     * \brief Not a jump at all
     */
    NONE = 0,

    /**
     * \brief Conditional jump
     */
    CONDITIONAL = 1,

    /**
     * \brief Unconditional jump
     */
    UNCONDITIONAL = 2
};

class Architecture
{
public:
    /**
     * \brief Returns the name of this architecture
     * \return the name of the architecture
     */
    virtual std::string get_name() = 0;

    /**
     * \brief Returns the type of jump of the mnemonic
     *
     * \param[in] mnemonic A mnemonic in form of a string
     * \return the type of jump represented by this mnemonic
     */
    virtual JumpType is_jump(const std::string& mnemonic) = 0;

    /**
     * \brief Returns true if the mnemonic is used to return from a function
     * \param[in] mnemonic
     * \return
     */
    virtual bool is_return(const std::string& mnemonic) = 0;
};

class ArchitectureUNK : public Architecture
{
public:
    std::string get_name() override
    {
        return "unknown";
    }
    JumpType is_jump(const std::string&) override
    {
        return NONE;
    };
    bool is_return(const std::string&) override
    {
        return false;
    };
};

#endif //__ARCHITECTURE_HPP__
