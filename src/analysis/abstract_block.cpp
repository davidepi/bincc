//
// Created by davide on 6/26/19.
//

#include "abstract_block.hpp"

AbstractBlock::AbstractBlock(int bb_id)
    : edges_inn(0), edges_out(0), type(BASIC), next(nullptr), cond(nullptr)
{
    blocks.push_back(bb_id);
}

const AbstractBlock* AbstractBlock::get_next() const
{
    return next;
}

void AbstractBlock::set_next(AbstractBlock* nxt)
{
    // bit hacks to check if the edges should be increased or not
    //   IN - OUT - RES
    //    0 -  0  -  0
    //    0 -  1  -  1
    //    1 -  1  -  0
    //    1 -  0  -  -1

    bool a = next != nullptr;
    bool b = nxt != nullptr;
    edges_out += (a ^ b) + (a ^ b) * (-2 * int(a));
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

const AbstractBlock* AbstractBlock::get_cond() const
{
    return cond;
}

void AbstractBlock::set_cond(AbstractBlock* cnd)
{
    // bit hacks to check if the edges should be increased or not
    bool a = cond != nullptr;
    bool b = cnd != nullptr;
    edges_out += (a ^ b) + (a ^ b) * (-2 * int(a));
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

const std::vector<int>& AbstractBlock::get_bbs()
{
    return blocks;
}

int AbstractBlock::get_edges_inn() const
{
    return edges_inn;
}

int AbstractBlock::get_edges_out() const
{
    return edges_out;
}

BlockType AbstractBlock::get_type() const
{
    return type;
}
