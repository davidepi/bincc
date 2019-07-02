//
// Created by davide on 7/1/19.
//

#ifndef __CFS_HPP__
#define __CFS_HPP__

#include "basic_block.hpp"
class ControlFlowStructure
{
public:
    ControlFlowStructure() = default;
    ~ControlFlowStructure() = default;

    void build(const BasicBlock* root, unsigned int nodes);

private:
    BasicBlock* root;
};

#endif //__CFS_HPP__
