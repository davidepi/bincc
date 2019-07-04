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
