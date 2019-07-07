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

int SelfLoopBlock::print(std::ostream& ss) const
{
    int id = looping_block->get_id();
    ss << "subgraph cluster_" << this->get_id() << "{\n"
       << id << " -> " << id << ";\nlabel = \"Self-loop\";\n}\n";
    return id;
}
