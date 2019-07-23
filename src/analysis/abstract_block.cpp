//
// Created by davide on 7/3/19.
//

#include "abstract_block.hpp"
#include <cassert>

const char* AbstractBlock::block_names[BLOCK_TOTAL] = {
    "Basic",   "Self-loop", "Sequence", "If-then",
    "If-else", "While",     "Do-While"};

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

const char* AbstractBlock::get_name() const
{
    return AbstractBlock::block_names[this->get_type()];
}

std::ostream& AbstractBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << " {\n";
    int size = this->size();
    for(int i = 0; i < size; i++)
    {
        (*this)[i]->print(ss);
    }
    ss << "label = \"" << this->get_name() << "\";\n}\n";
    return ss;
}

inline uint64_t rotl64(uint64_t x, int8_t r)
{
    return (x << r) | (x >> (64 - r));
}

uint64_t AbstractBlock::structural_hash() const
{
    assert(BLOCK_TOTAL < 64);
    uint64_t hash = 1 << this->get_type();
    const int SIZE = this->size();
    for(int i = 0; i < SIZE; i++)
    {
        // combine hash
        hash ^= rotl64((*this)[i]->structural_hash(), i);
    }
    return hash;
}
