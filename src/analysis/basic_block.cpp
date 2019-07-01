//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"
#include <cstdio>
#include <stack>
#include <set>

BasicBlock::BasicBlock(int number)
    : id(number), next(nullptr), cond(nullptr)
{
}

const BasicBlock* BasicBlock::get_next() const
{
    return next;
}

const BasicBlock* BasicBlock::get_cond() const
{
    return cond;
}

void BasicBlock::set_next(const BasicBlock* next_blk)
{
    BasicBlock::next = next_blk;
}

int BasicBlock::get_id() const
{
    return id;
}

void BasicBlock::set_id(int number)
{
    BasicBlock::id = number;
}

void BasicBlock::set_cond(const BasicBlock* conditional_blk)
{
    BasicBlock::cond = conditional_blk;
}
