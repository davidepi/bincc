//
// Created by davide on 6/13/19.
//

#include "basic_block.hpp"
#include <cstdio>
#include <stack>
#include <set>

BasicBlock::BasicBlock(int number)
    : id(number), next(nullptr), conditional(nullptr)
{
}

const BasicBlock* BasicBlock::get_next() const
{
    return next;
}

const BasicBlock* BasicBlock::get_conditional() const
{
    return conditional;
}

void BasicBlock::set_next(const BasicBlock* next_blk)
{
    BasicBlock::next = next_blk;
}

void BasicBlock::set_next(const BasicBlock* next_blk,
                          const BasicBlock* conditional_blk)
{
    BasicBlock::next = next_blk;
    BasicBlock::conditional = conditional_blk;
}

int BasicBlock::get_id() const
{
    return id;
}

void BasicBlock::set_id(int number)
{
    BasicBlock::id = number;
}

void BasicBlock::set_conditional(const BasicBlock* conditional_blk)
{
    BasicBlock::conditional = conditional_blk;
}

void print_cfg(const BasicBlock* bb, const char* filename)
{
    // first open the file
    FILE* fout = fopen(filename, "w");
    if(fout == nullptr)
    {
        return;
    }
    fprintf(fout, "%s\n", "digraph {");

    // print iteratively
    std::set<int> visited;
    std::stack<const BasicBlock*> nodes;
    nodes.push(bb);
    visited.insert(bb->get_id());
    do
    {
        const BasicBlock* current;
        const BasicBlock* next;
        const BasicBlock* cond;
        current = nodes.top();
        nodes.pop();
        next = current->get_next();
        cond = current->get_conditional();
        if(next != nullptr)
        {
            fprintf(fout, "%d -> %d\n", current->get_id(), next->get_id());
            if(visited.find(next->get_id())==visited.end())
            {
                nodes.push(next);
                visited.insert(next->get_id());
            }
        }
        if(cond != nullptr)
        {
            fprintf(fout, "%d -> %d\n", current->get_id(), cond->get_id());
            if(visited.find(cond->get_id())==visited.end())
            {
                nodes.push(cond);
                visited.insert(cond->get_id());
            }
        }
    } while(!nodes.empty());

    fprintf(fout, "%s\n", "}");
    fclose(fout);
}