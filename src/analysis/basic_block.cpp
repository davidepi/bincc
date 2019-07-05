//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"

BasicBlock::BasicBlock(int number) : AbstractBlock(number), cond(nullptr)
{
}

const AbstractBlock* BasicBlock::get_cond() const
{
    return cond;
}

void BasicBlock::set_cond(const AbstractBlock* cnd)
{
    BasicBlock::cond = cnd;
}

BlockType BasicBlock::get_type() const
{
    return BASIC;
}
