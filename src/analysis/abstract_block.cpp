//
// Created by davide on 7/3/19.
//

#include "abstract_block.hpp"
AbstractBlock::AbstractBlock(int number)
    : id(number), next(nullptr), cond(nullptr), edges_inn(0), edges_out(0),
      type(BASIC)
{
}

const AbstractBlock* AbstractBlock::get_next() const
{
    return next;
}

const AbstractBlock* AbstractBlock::get_cond() const
{
    return cond;
}

int AbstractBlock::get_id() const
{
    return id;
}

void AbstractBlock::set_id(int number)
{
    AbstractBlock::id = number;
}

void AbstractBlock::set_next(AbstractBlock* nxt)
{
    // bit hacks to check if the edges should be increased or not
    //  | A | B |RES|
    //  | - | - | - |
    //  | 0 | 0 | 0 |
    //  | 0 | 1 | 1 |
    //  | 1 | 1 | 0 |
    //  | 1 | 0 |-1 |

    bool a = next != nullptr;
    bool b = nxt != nullptr;
    edges_out += (a ^ b) * (1 - (int(a) << 1));
    if(a) // current target is not null
    {
        // decrease the in edges
        next->edges_inn--;
    }
    if(b) // next target is not null
    {
        nxt->edges_inn++;
    }
    AbstractBlock::next = nxt;
}

void AbstractBlock::set_cond(AbstractBlock* cnd)
{
    // bit hacks to check if the edges should be increased or not
    bool a = cond != nullptr;
    bool b = cnd != nullptr;
    edges_out += (a ^ b) * (1 - (int(a) << 1));
    if(a) // current target is not null
    {
        // decrease the in edges
        cond->edges_inn--;
    }
    if(b) // next target is not null
    {
        cnd->edges_inn++;
    }
    AbstractBlock::cond = cnd;
}

BlockType AbstractBlock::get_type() const
{
    return type;
}

size_t AbstractBlock::size() const
{
    return blocks.size();
}

int AbstractBlock::get_edges_in() const
{
    return edges_inn;
}

int AbstractBlock::get_edges_out() const
{
    return edges_out;
}
const std::vector<const AbstractBlock*>&
AbstractBlock::get_block_components() const
{
    return blocks;
}
