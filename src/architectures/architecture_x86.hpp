//
// Created by davide on 6/14/19.
//

#ifndef __ARCHITECTUREX86_HPP__
#define __ARCHITECTUREX86_HPP__

#include "architecture.hpp"
#include <string>

class ArchitectureX86 : public Architecture
{
public:
    std::string get_name() override;
    JumpType is_jump(const std::string& mnemonic) override;
    bool is_return(const std::string& mnemonic) override;
};

#endif //__ARCHITECTUREX86_HPP__
