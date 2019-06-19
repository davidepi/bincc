//
// Created by davide on 6/18/19.
//

#include "architecture_arm.hpp"

static std::string remove_condition(const std::string& mnemonic)
{
    if(mnemonic.length() < 3)
    {
        return mnemonic;
    }
    std::string cond =
        mnemonic.substr(mnemonic.length() - 2, std::string::npos);
    if(cond == "eq" || cond == "ne" || cond == "cs" || cond == "hs" ||
       cond == "cc" || cond == "lo" || cond == "mi" || cond == "pl" ||
       cond == "vs" || cond == "vc" || cond == "hi" || cond == "ls" ||
       cond == "ge" || cond == "gt" || cond == "lt" || cond == "le")
    {
        return mnemonic.substr(0, mnemonic.length() - 2);
    }
    return mnemonic;
}

std::string ArchitectureARM::get_name()
{
    return "arm";
}

JumpType ArchitectureARM::is_jump(const std::string& mnemonic)
{
    std::string mne = remove_condition(mnemonic);
    JumpType retval;
    if(mne == "b")
    {
        if(mne != mnemonic)
        {
            retval = JumpType::JUMP_CONDITIONAL;
        }
        else
        {
            retval = JumpType::JUMP_UNCONDITIONAL;
        }
    }
    else if(mne == "bx")
    {
        if(mne != mnemonic)
        {
            retval = JumpType::RET_CONDITIONAL;
        }
        else
        {
            retval = JumpType::RET_UNCONDITIONAL;
        }
    }
    else
    {
        retval = JumpType::NONE;
    }
    return retval;
}
