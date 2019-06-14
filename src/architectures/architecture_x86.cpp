//
// Created by davide on 6/14/19.
//

#include "architecture_x86.hpp"
JumpType ArchitectureX86::is_jump(const std::string& mnemonic)
{
    if(mnemonic[0] != 'j')
    {
        return NONE;
    }
    if(mnemonic == "jmp")
    {
        return UNCONDITIONAL;
    }
    return CONDITIONAL;
}

bool ArchitectureX86::is_return(const std::string& mnemonic)
{
    return mnemonic == "ret";
}

std::string ArchitectureX86::get_name()
{
    return "x86";
}
