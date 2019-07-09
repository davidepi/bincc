//
// Created by davide on 7/5/19.
//

#include "cyclic_block.hpp"

SelfLoopBlock::SelfLoopBlock(int id, const BasicBlock* loop) : AbstractBlock(id)
{
    looping_block = loop;
}

SelfLoopBlock::~SelfLoopBlock()
{
    delete looping_block;
}

BlockType SelfLoopBlock::get_type() const
{
    return SELF_LOOP;
}

int SelfLoopBlock::size() const
{
    return 1;
}

const AbstractBlock* SelfLoopBlock::operator[](int) const
{
    return looping_block;
}

std::ostream& SelfLoopBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << "{\n";
    looping_block->print(ss);
    ss << "label=\"Self-loop\"\n}\n";
    return ss;
}

bool is_self_loop(const AbstractBlock* node)
{
    if(node->get_type() == BlockType::BASIC)
    {
        const BasicBlock* bb = static_cast<const BasicBlock*>(node);
        return bb->get_cond() == bb || bb->get_next() == bb;
    }
    return false;
}

WhileBlock::WhileBlock(int id, const BasicBlock* head,
                       const AbstractBlock* tail)
    : AbstractBlock(id), head(head), tail(tail)
{
}

WhileBlock::~WhileBlock()
{
    delete tail;
    delete head;
}

BlockType WhileBlock::get_type() const
{
    return WHILE;
}

int WhileBlock::size() const
{
    return 2;
}

const AbstractBlock* WhileBlock::operator[](int index) const
{
    return index == 0 ? head : tail;
}

std::ostream& WhileBlock::print(std::ostream& ss) const
{
    ss << "subgraph cluster_" << this->get_id() << "{\n";
    head->print(ss);
    tail->print(ss);
    ss << "label=\"While\"\n}\n";
    return ss;
}

bool is_loop(const AbstractBlock* node, const AbstractBlock** next)
{
    // TODO: SUPER IZI VERSION, REPLACE WITH TARJAN LATER
    //      this is just a dummy implementation,  to get an idea of the required
    //      changes to the code structure. It MUST be replaced by an actual loop
    //      detection such as Tarjan algorithm

    // FIXME: this works only with while or do-while and produces segfault for
    //       nat-while

    if(node->get_out_edges() == 1) // do-while
    {
        const BasicBlock* tail =
            static_cast<const BasicBlock*>(node->get_next());
        *next = tail;
        return tail->get_next() == node || tail->get_cond() == node;
    }
    else if(node->get_out_edges() == 2) // while
    {
        const BasicBlock* head = static_cast<const BasicBlock*>(node);
        if(head->get_next()->get_next() == head)
        {
            *next = head->get_next();
            return true;
        }
        else if(head->get_cond()->get_next() == head)
        {
            *next = head->get_cond();
            return true;
        }
        else
        {
            return false;
        }
    }
    else
    {
        return false;
    }
}
