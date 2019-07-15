//
// Created by davide on 7/5/19.
//

#include "acyclic_block.hpp"
#include "basic_block.hpp"
#include <cassert>
#include <iostream>

// The SequenceBlock::delete_list containg elements on which `delete` should be
// called. This because if the components of the sequence are other sequences,
// they are flattened. But they still have the ownership of the contained
// elements and I cannot remove the ownership without violating the const-ness
// (thus modifying the flattened sequence).

SequenceBlock::SequenceBlock(int id, const AbstractBlock* fst,
                             const AbstractBlock* snd)
    : AbstractBlock(id)
{
    auto merge_blocks = [this](const AbstractBlock* p) -> void {
        // merge all the internals of a sequence, and destroy the sequence
        if(p->get_type() == BlockType::SEQUENCE)
        {
            int size = p->size();
            for(int i = 0; i < size; i++)
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

int SequenceBlock::size() const
{
    return components.size();
}

const AbstractBlock* SequenceBlock::operator[](int index) const
{
    return components[index];
}

std::ostream& SequenceBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << " {\n";
    int size = components.size();
    for(int i = 0; i < size; i++)
    {
        components[i]->print(ss);
    }
    ss << "label = \"Sequence\";\n}\n";
    return ss;
}

BlockType IfThenBlock::get_type() const
{
    return IF_THEN;
}

IfThenBlock::IfThenBlock(int id, const AbstractBlock* ifb,
                         const AbstractBlock* thenb)
    : AbstractBlock(id), head(ifb), then(thenb)
{
}

IfThenBlock::~IfThenBlock()
{
    delete head;
    delete then;
}

int IfThenBlock::size() const
{
    return 2;
}

const AbstractBlock* IfThenBlock::operator[](int index) const
{
    return index == 0 ? head : then;
}

std::ostream& IfThenBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << " {\n";
    head->print(ss);
    then->print(ss);
    ss << "label=\"If-Then\";\n}\n";
    return ss;
}

IfElseBlock::IfElseBlock(int id, const AbstractBlock* ifb,
                         const AbstractBlock* thenb, const AbstractBlock* elseb)
    : AbstractBlock(id), head(ifb), then(thenb), ellse(elseb)
{
}

IfElseBlock::~IfElseBlock()
{
    delete ellse;
    delete then;
    delete head;
}

BlockType IfElseBlock::get_type() const
{
    return BlockType::IF_ELSE;
}

int IfElseBlock::size() const
{
    return 3;
}

const AbstractBlock* IfElseBlock::operator[](int index) const
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
        return ellse;
    }
}

std::ostream& IfElseBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << " {\n";
    head->print(ss);
    then->print(ss);
    ellse->print(ss);
    ss << "label=\"If-Else\";\n}\n";
    return ss;
}
