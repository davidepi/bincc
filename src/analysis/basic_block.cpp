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

void BasicBlock::set_cond(AbstractBlock* cnd)
{
    bool a = cond != nullptr;
    bool b = cnd != nullptr;
    edges_out += (a ^ b) * (1 - (int(a) << 1));
    if(a) // current target is not null
    {
        // decrease the in edges
        cond->edges_inn--;
    }
    if(b) // next target is not null
    {
        cnd->edges_inn++;
    }
    BasicBlock::cond = cnd;
}

BlockType BasicBlock::get_type() const
{
    return BASIC;
}
