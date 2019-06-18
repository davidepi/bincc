//
// Created by davide on 6/18/19.
//

#ifndef __ARCHITECTURE_ARM_HPP__
#define __ARCHITECTURE_ARM_HPP__

#include "architecture.hpp"

class ArchitectureARM : public Architecture {
public:
    std::string get_name() override;
    JumpType is_jump(const std::string& mnemonic) override;
    bool is_return(const std::string& mnemonic) override;
};



#endif //__ARCHITECTURE_ARM_HPP__
