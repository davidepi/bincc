//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"

BasicBlock::BasicBlock(int number) : id(number), next(nullptr), cond(nullptr)
{
}

const BasicBlock* BasicBlock::get_next() const
{
    return next;
}

const BasicBlock* BasicBlock::get_cond() const
{
    return cond;
}

int BasicBlock::get_id() const
{
    return id;
}

void BasicBlock::set_id(int number)
{
    BasicBlock::id = number;
}

void BasicBlock::set_next(BasicBlock* nxt)
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
    BasicBlock::next = nxt;
}

void BasicBlock::set_cond(BasicBlock* cnd)
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
    BasicBlock::cond = cnd;
}

BlockType BasicBlock::get_type() const
{
    return type;
}

size_t BasicBlock::size() const
{
    return blocks.size();
}

int BasicBlock::get_edges_in() const
{
    return edges_inn;
}

int BasicBlock::get_edges_out() const
{
    return edges_out;
}
