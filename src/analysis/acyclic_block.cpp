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

int SequenceBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << " {\n";
    int size = components.size();
    int last_node = 0;
    if(!components.empty() && components[0]->get_type() != BASIC)
    {
        // recursively print first node.
        // later on, only the second node of the pair will be printed to avoid
        // repetitions. The id of the innermost node will be saved in saved_id
        last_node = components[0]->print(ss);
    }
    // nodes are taken two by two. Only the second one is called recursively.
    // the first one has already been called in the previous iteration
    for(int i = 1; i < size; i++)
    {
        const AbstractBlock* node0 = components[i - 1];
        const AbstractBlock* node1 = components[i];

        // both are basic blocks, so print em
        if(node0->get_type() == BASIC && node1->get_type() == BASIC)
        {
            ss << last_node << " -> " << node1->get_id() << ";\n";
            last_node = node1->get_id();
        }
        // the first one is not basic, but has already been processed.
        else if(node0->get_type() != BASIC && node1->get_type() == BASIC)
        {
            // print recursively and obtains the innermost id of the block
            ss << last_node << " -> " << node1->get_id() << "[ltail=cluster_"
               << node0->get_id() << "];\n";
            last_node = node1->get_id();
        }
        else if(node0->get_type() == BASIC && node1->get_type() != BASIC)
        {
            int id1 = node1->print(ss);
            ss << node0->get_id() << " -> " << id1 << "[head=cluster_"
               << node1->get_id() << "];\n";
            last_node = id1;
        }
        else
        {
            int id1 = node1->print(ss);
            ss << last_node << " -> " << id1 << "[ltail=cluster_"
               << node0->get_id() << ",lhead=cluster_" << node1->get_id()
               << "];\n";
            last_node = id1;
        }
    }
    ss << "label = \"Sequence\";\n}\n";
    return last_node;
}
