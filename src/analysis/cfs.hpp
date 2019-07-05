//
// Created by davide on 7/5/19.
//

#ifndef __CFS_HPP__
#define __CFS_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"

class ControlFlowStructure
{
public:
    ControlFlowStructure() = default;
    ~ControlFlowStructure();
    void build(const BasicBlock* root, int nodes);
    const AbstractBlock* root() const;
    ControlFlowStructure(const ControlFlowStructure&) = delete;
    ControlFlowStructure& operator=(const ControlFlowStructure&) = delete;

private:
    AbstractBlock* head{nullptr};
};

#endif //__CFS_HPP__
