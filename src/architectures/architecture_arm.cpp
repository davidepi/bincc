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
            retval = JumpType::CONDITIONAL;
        }
        else
        {
            retval = JumpType::UNCONDITIONAL;
        }
    }
    else if(mne == "bx" && mne != mnemonic)
    {
        retval = JumpType::CONDITIONAL;
    }
    else
    {
        retval = JumpType::NONE;
    }
    return retval;
}

bool ArchitectureARM::is_return(const std::string& mnemonic)
{
    // TODO: bxle is not added because it exploits the structure of the
    //       build_cfg function. However this is obviously unmaintainable and
    //       will become a nightmare in the future, so fix it
    return mnemonic == "bx";
}
