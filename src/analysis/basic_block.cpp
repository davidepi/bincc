//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"

BasicBlock::BasicBlock(uint32_t id_in, uint64_t off_s, uint64_t off_e)
    : AbstractBlock(id_in),
      cond(nullptr),
      offset_start(off_s),
      offset_end(off_e)
{
    if(offset_start > offset_end)
    {
        // swap
        offset_start ^= offset_end;
        offset_end ^= offset_start;
        offset_start ^= offset_end;
    }
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
    BasicBlock::offset_start = orig.offset_start;
    BasicBlock::offset_end = orig.offset_end;
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

uint32_t BasicBlock::get_depth() const
{
    return 0;
}

std::ostream& BasicBlock::print(std::ostream& ss) const
{
    ss << id << ";\n";
    return ss;
}

void BasicBlock::get_offset(uint64_t* start, uint64_t* end) const
{
    *start = offset_start;
    *end = offset_end;
}

void BasicBlock::set_offset(uint64_t start, uint64_t end)
{
    BasicBlock::offset_start = start;
    BasicBlock::offset_end = end;
    if(offset_start > offset_end)
    {
        // swap
        offset_start ^= offset_end;
        offset_end ^= offset_start;
        offset_start ^= offset_end;
    }
}