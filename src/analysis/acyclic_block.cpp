//
// Created by davide on 7/5/19.
//

#include "acyclic_block.hpp"

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
            };
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
    for(int i = 1; i < size; i++)
    {
        const AbstractBlock* node0 = components[i - 1];
        const AbstractBlock* node1 = components[i];

        // both are basic blocks, so print em
        if(node0->get_type() == BASIC && node1->get_type() == BASIC)
        {
            ss << node0->get_id() << " -> " << node1->get_id() << ";\n";
        }
        // TODO: add what happens if one of them (or both) are not basic
    }
    ss << "label = \"Sequence\";\n}\n";
    return ss;
}
