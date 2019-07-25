//
// Created by davide on 7/5/19.
//

#include "acyclic_block.hpp"
#include "basic_block.hpp"
#include <cassert>
#include <iostream>
#include <stack>

// The SequenceBlock::delete_list containg elements on which `delete` should be
// called. This because if the components of the sequence are other sequences,
// they are flattened. But they still have the ownership of the contained
// elements and I cannot remove the ownership without violating the const-ness
// (thus modifying the flattened sequence).

SequenceBlock::SequenceBlock(uint32_t id, const AbstractBlock* fst,
                             const AbstractBlock* snd)
    : AbstractBlock(id)
{
    auto merge_blocks = [this](const AbstractBlock* p) -> void {
        // merge all the internals of a sequence, and destroy the sequence
        if(p->get_type() == BlockType::SEQUENCE)
        {
            uint32_t size = p->size();
            for(uint32_t i = 0; i < size; i++)
            {
                components.push_back((*p)[i]);
            }
        }
        // if it was a single node just add the node
        else
        {
            components.push_back(p);
        }
        delete_list.push_back(p);
    };
    merge_blocks(fst);
    merge_blocks(snd);
}

BlockType SequenceBlock::get_type() const
{
    return SEQUENCE;
}

SequenceBlock::~SequenceBlock()
{
    for(const AbstractBlock* block : delete_list)
    {
        delete block;
    }
}

uint32_t SequenceBlock::size() const
{
    return components.size();
}

const AbstractBlock* SequenceBlock::operator[](uint32_t index) const
{
    return components[index];
}

BlockType IfThenBlock::get_type() const
{
    return IF_THEN;
}

IfThenBlock::IfThenBlock(uint32_t id, const BasicBlock* ifb,
                         const AbstractBlock* thenb)
    : AbstractBlock(id), head(ifb), then(thenb)
{
    // resolve chained heads
    std::stack<const BasicBlock*> chain_stack;
    const AbstractBlock* contd = thenb->get_next();
    const AbstractBlock* next =
        head->get_next() != contd ? head->get_next() : head->get_cond();
    int chain_len = 0;
    while(next != thenb)
    {
        chain_len++;
        const BasicBlock* tmp_head = static_cast<const BasicBlock*>(next);
        chain_stack.push(tmp_head);
        next = tmp_head->get_next() != contd ? tmp_head->get_next() :
                                               tmp_head->get_cond();
    }

    if(chain_len != 0)
    {
        // copy the stack into the more space_efficient array
        chain.resize(chain_len);
        for(int i = chain_len - 1; i >= 0; i--)
        {
            chain[i] = chain_stack.top();
            chain_stack.pop();
        }
    }
}

IfThenBlock::~IfThenBlock()
{
    delete then;
    delete head;
    for(const BasicBlock* val : chain)
    {
        delete val;
    }
}

uint32_t IfThenBlock::size() const
{
    return chain.size() + 2;
}

const AbstractBlock* IfThenBlock::operator[](uint32_t index) const
{
    if(index == 0)
    {
        return head;
    }
    else if(index == 1)
    {
        return then;
    }
    else
    {
        return chain[index - 2];
    }
}

IfElseBlock::IfElseBlock(uint32_t id, const BasicBlock* ifb,
                         const AbstractBlock* thenb, const AbstractBlock* elseb)
    : AbstractBlock(id), head(ifb), then(thenb), ellse(elseb), chain(0)
{
    // resolve chained heads
    std::stack<const BasicBlock*> chain_stack;
    const BasicBlock* tmp_head = ifb;
    const AbstractBlock* next = tmp_head->get_next() != elseb ?
                                    tmp_head->get_next() :
                                    tmp_head->get_cond();
    int chain_len = 0;
    while(next != thenb)
    {
        chain_len++;
        tmp_head = static_cast<const BasicBlock*>(next);
        chain_stack.push(tmp_head);
        next = tmp_head->get_next() != elseb ? tmp_head->get_next() :
                                               tmp_head->get_cond();
    }

    if(chain_len != 0)
    {
        // copy the stack into the more space_efficient array
        chain.resize(chain_len);
        for(int i = chain_len - 1; i >= 0; i--)
        {
            chain[i] = chain_stack.top();
            chain_stack.pop();
        }
    }
}

IfElseBlock::~IfElseBlock()
{
    delete ellse;
    delete then;
    delete head;
    for(const BasicBlock* val : chain)
    {
        delete val;
    }
}

BlockType IfElseBlock::get_type() const
{
    return BlockType::IF_ELSE;
}

uint32_t IfElseBlock::size() const
{
    return chain.size() + 3;
}

const AbstractBlock* IfElseBlock::operator[](uint32_t index) const
{
    if(index == 0)
    {
        return head;
    }
    else if(index == 1)
    {
        return then;
    }
    else if(index == 2)
    {
        return ellse;
    }
    else
    {
        return chain[index - 3];
    }
}
