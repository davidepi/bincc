//
// Created by davide on 6/18/19.
//

#ifndef __ARCHITECTURE_ARM_HPP__
#define __ARCHITECTURE_ARM_HPP__

#include "architecture.hpp"

/**
 * \brief Class implementing the ARM architecture
 *
 * Implements the Architecture class by using the ARM specifications
 */
class ArchitectureARM final : public Architecture
{
public:
  std::string get_name() override;
  JumpType is_jump(const std::string& mnemonic) override;
};

#endif //__ARCHITECTURE_ARM_HPP__
