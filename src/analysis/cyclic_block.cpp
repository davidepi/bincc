//
// Created by davide on 7/5/19.
//

#include "cyclic_block.hpp"

SelfLoopBlock::SelfLoopBlock(uint32_t id, const BasicBlock* loop)
    : AbstractBlock(id)
{
    looping_block = loop;
    depth = 1;
}

SelfLoopBlock::~SelfLoopBlock()
{
    delete looping_block;
}

BlockType SelfLoopBlock::get_type() const
{
    return SELF_LOOP;
}

uint32_t SelfLoopBlock::size() const
{
    return 1;
}

const AbstractBlock* SelfLoopBlock::operator[](uint32_t) const
{
    return looping_block;
}

WhileBlock::WhileBlock(uint32_t id, const BasicBlock* head,
                       const AbstractBlock* tail)
    : AbstractBlock(id), head(head), tail(tail)
{
    depth = std::max(head->get_depth(), tail->get_depth())+1;
}

WhileBlock::~WhileBlock()
{
    delete tail;
    delete head;
}

BlockType WhileBlock::get_type() const
{
    return WHILE;
}

uint32_t WhileBlock::size() const
{
    return 2;
}

const AbstractBlock* WhileBlock::operator[](uint32_t index) const
{
    return index == 0 ? head : tail;
}

DoWhileBlock::DoWhileBlock(uint32_t id, const AbstractBlock* head,
                           const BasicBlock* tail)
    : AbstractBlock(id), head(head), tail(tail)
{
    depth = std::max(head->get_depth(), tail->get_depth())+1;
}

DoWhileBlock::~DoWhileBlock()
{
    delete tail;
    delete head;
}

BlockType DoWhileBlock::get_type() const
{
    return DO_WHILE;
}

uint32_t DoWhileBlock::size() const
{
    return 2;
}

const AbstractBlock* DoWhileBlock::operator[](uint32_t index) const
{
    return index == 0 ? head : tail;
}
