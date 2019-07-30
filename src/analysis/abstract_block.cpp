//
// Created by davide on 7/3/19.
//

#include "abstract_block.hpp"
#include <cassert>

const char* AbstractBlock::block_names[BLOCK_TOTAL] = {
    "Basic",   "Self-loop", "Sequence", "If-then",
    "If-else", "While",     "Do-While"};

AbstractBlock::AbstractBlock(uint32_t number) : id(number), next(nullptr)
{
}

const AbstractBlock* AbstractBlock::get_next() const
{
    return next;
}

uint32_t AbstractBlock::get_id() const
{
    return id;
}

void AbstractBlock::set_id(uint32_t number)
{
    AbstractBlock::id = number;
}

void AbstractBlock::set_next(const AbstractBlock* nxt)
{
    AbstractBlock::next = nxt;
}

uint32_t AbstractBlock::size() const
{
    return 0;
}

const AbstractBlock* AbstractBlock::operator[](uint32_t) const
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
    uint32_t size = this->size();
    for(uint32_t i = 0; i < size; i++)
    {
        (*this)[i]->print(ss);
    }
    ss << "label = \"" << this->get_name() << "\";\n}\n";
    return ss;
}

uint32_t AbstractBlock::get_depth() const
{
    return depth;
}

/**
 * \brief Rotate left
 * \param[in] x Value to be shifted
 * \param[in] r Shift magnitude
 * \return the shifted and possibly rotated value
 */
static inline uint64_t rotl64(uint64_t x, int8_t r)
{
    return (x << r) | (x >> (64 - r));
}

uint64_t AbstractBlock::structural_hash() const
{
    static_assert(BLOCK_TOTAL < 64, "Hash func. supports max 64 block types");
    uint64_t hash = 1 << this->get_type();
    const uint32_t SIZE = this->size();
    for(uint32_t i = 0; i < SIZE; i++)
    {
        // combine hash
        hash ^= rotl64((*this)[i]->structural_hash(), i);
    }
    return hash;
}
