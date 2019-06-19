//
// Created by davide on 6/14/19.
//

#include "architecture_x86.hpp"
JumpType ArchitectureX86::is_jump(const std::string& mnemonic)
{
    if(mnemonic == "ret")
    {
        return RET_UNCONDITIONAL;
    }
    if(mnemonic[0] != 'j')
    {
        return NONE;
    }
    if(mnemonic == "jmp")
    {
        return JUMP_UNCONDITIONAL;
    }
    return JUMP_CONDITIONAL;
}

std::string ArchitectureX86::get_name()
{
    return "x86";
}
