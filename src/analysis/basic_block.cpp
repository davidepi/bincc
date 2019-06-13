//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"
BasicBlock::BasicBlock(int number)
    : id(number), next(nullptr), conditional(nullptr)
{
}

const BasicBlock* BasicBlock::get_next() const
{
    return next;
}

const BasicBlock* BasicBlock::get_conditional() const
{
    return conditional;
}

void BasicBlock::set_next(const BasicBlock* next_blk)
{
    BasicBlock::next = next_blk;
}

void BasicBlock::set_next(const BasicBlock* next_blk,
                          const BasicBlock* conditional_blk)
{
    BasicBlock::next = next_blk;
    BasicBlock::conditional = conditional_blk;
}

int BasicBlock::get_id() const
{
    return id;
}

void BasicBlock::set_id(int number)
{
    BasicBlock::id = number;
}
