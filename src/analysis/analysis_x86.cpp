//
// Created by davide on 6/13/19.
//

#include "analysis_x86.hpp"
JumpType AnalysisX86::is_jump(const std::string& mnemonic)
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
