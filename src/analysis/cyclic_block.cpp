//
// Created by davide on 7/5/19.
//

#include "cyclic_block.hpp"

SelfLoopBlock::SelfLoopBlock(int id, const BasicBlock* loop) : AbstractBlock(id)
{
    looping_block = loop;
}

SelfLoopBlock::~SelfLoopBlock()
{
    delete looping_block;
}

BlockType SelfLoopBlock::get_type() const
{
    return SELF_LOOP;
}

int SelfLoopBlock::size() const
{
    return 1;
}

const AbstractBlock* SelfLoopBlock::operator[](int) const
{
    return looping_block;
}

std::ostream& SelfLoopBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << "{\n";
    looping_block->print(ss);
    ss << "label=\"Self-loop\"\n}\n";
    return ss;
}

WhileBlock::WhileBlock(int id, const BasicBlock* head,
                       const AbstractBlock* tail)
    : AbstractBlock(id), head(head), tail(tail)
{
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

int WhileBlock::size() const
{
    return 2;
}

const AbstractBlock* WhileBlock::operator[](int index) const
{
    return index == 0 ? head : tail;
}

std::ostream& WhileBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << "{\n";
    head->print(ss);
    tail->print(ss);
    ss << "label=\"While\"\n}\n";
    return ss;
}

DoWhileBlock::DoWhileBlock(int id, const AbstractBlock* head,
                           const BasicBlock* tail)
    : AbstractBlock(id), head(head), tail(tail)
{
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

int DoWhileBlock::size() const
{
    return 2;
}

const AbstractBlock* DoWhileBlock::operator[](int index) const
{
    return index == 0 ? head : tail;
}

std::ostream& DoWhileBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << "{\n";
    head->print(ss);
    tail->print(ss);
    ss << "label=\"Do-While\"\n}\n";
    return ss;
}
