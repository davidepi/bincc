//
// Created by davide on 7/3/19.
//

#include "abstract_block.hpp"
AbstractBlock::AbstractBlock(int number) : id(number), next(nullptr)
{
}

const AbstractBlock* AbstractBlock::get_next() const
{
    return next;
}

int AbstractBlock::get_id() const
{
    return id;
}

void AbstractBlock::set_id(int number)
{
    AbstractBlock::id = number;
}

void AbstractBlock::set_next(const AbstractBlock* nxt)
{
    AbstractBlock::next = nxt;
}

int AbstractBlock::size() const
{
    return 0;
}

const AbstractBlock* AbstractBlock::operator[](int) const
{
    return this;
}

unsigned char AbstractBlock::get_out_edges() const
{
    return (unsigned char)(next != nullptr);
}

void AbstractBlock::replace_if_match(const AbstractBlock* match,
                                     const AbstractBlock* edge)
{
    if(next == match)
    {
        next = edge;
    }
}

int AbstractBlock::print(std::ostream& ss) const
{
    return id;
}
