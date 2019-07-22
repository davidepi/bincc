//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"

BasicBlock::BasicBlock(int number) : AbstractBlock(number), cond(nullptr)
{
}

BasicBlock::BasicBlock(const BasicBlock& orig)
{
    *this = orig;
}

BasicBlock& BasicBlock::operator=(const BasicBlock& orig)
{
    BasicBlock::id = orig.id;
    BasicBlock::next = orig.next;
    BasicBlock::cond = orig.cond;
    return *this;
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

unsigned char BasicBlock::get_out_edges() const
{
    return (unsigned char)(next != nullptr) + (unsigned char)(cond != nullptr);
}

void BasicBlock::replace_if_match(const AbstractBlock* match,
                                  const AbstractBlock* edge)
{
    if(next == match)
    {
        next = edge;
    }
    else if(cond == match)
    {
        cond = edge;
    }
}

std::ostream& BasicBlock::print(std::ostream& ss) const
{
    ss << id << ";\n";
    return ss;
}
