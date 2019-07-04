//
// Created by davide on 6/20/19.
//

#include "cfg.hpp"
#include <climits>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stack>
#include <unordered_set>

ControlFlowGraph::ControlFlowGraph(unsigned int size) : nodes(size), edges(0)
{
    // an extra block is allocated, in case a single exit point is needed:
    // otherwise it will be a mess to update every pointer.
    blocks = (BasicBlock*)malloc(sizeof(BasicBlock) * (nodes + 1));
    for(unsigned int i = 0; i < size - 1; i++)
    {
        blocks[i] = BasicBlock(); // call constructor
        blocks[i].set_id(i);
        blocks[i].set_next(&(blocks[i + 1]));
        edges++;
        blocks[i].set_cond(nullptr);
    }
    blocks[size - 1].set_id(size - 1);
    blocks[size - 1].set_next(nullptr);
    blocks[size - 1].set_cond(nullptr);
}

ControlFlowGraph::~ControlFlowGraph()
{
    free(blocks);
}

std::string ControlFlowGraph::to_dot() const
{
    std::stringstream stream;
    stream << *this;
    return stream.str();
}

void ControlFlowGraph::set_next(unsigned int id_src, unsigned int id_target)
{
    if(id_src < nodes && id_target < nodes)
    {
        edges += (unsigned int)(blocks[id_src].get_next() == nullptr);
        blocks[id_src].set_next(&(blocks[id_target]));
    }
}

void ControlFlowGraph::set_next_null(unsigned int id_src)
{
    if(id_src < nodes)
    {
        edges -= (unsigned int)(blocks[id_src].get_next() != nullptr);
        blocks[id_src].set_next(nullptr);
    }
}

void ControlFlowGraph::set_conditional(unsigned int id_src,
                                       unsigned int id_target)
{
    if(id_src < nodes && id_target < nodes)
    {
        edges += (unsigned int)(blocks[id_src].get_cond() == nullptr);
        blocks[id_src].set_cond(&(blocks[id_target]));
    }
}

void ControlFlowGraph::set_conditional_null(unsigned int id_src)
{
    if(id_src < nodes)
    {
        edges -= (unsigned int)(blocks[id_src].get_cond() != nullptr);
        blocks[id_src].set_cond(nullptr);
    }
}

const BasicBlock* ControlFlowGraph::root() const
{
    return &(blocks[0]);
}

unsigned int ControlFlowGraph::nodes_no() const
{
    return nodes;
}

unsigned int ControlFlowGraph::edges_no() const
{
    return edges;
}

std::ostream& operator<<(std::ostream& stream, const ControlFlowGraph& cfg)
{
    stream << "digraph {\n";
    std::unordered_set<int> visited;
    std::stack<const BasicBlock*> unvisited;
    unvisited.push(cfg.root());
    visited.insert(cfg.root()->get_id());
    do
    {
        const BasicBlock* current;
        const BasicBlock* next;
        const BasicBlock* cond;
        current = unvisited.top();
        unvisited.pop();
        // these are created by this class so will ALWAYS be of type BasicBlock
        next = static_cast<const BasicBlock*>(current->get_next());
        cond = static_cast<const BasicBlock*>(current->get_cond());
        if(next != nullptr)
        {
            stream << current->get_id() << "->" << next->get_id() << "\n";
            if(visited.find(next->get_id()) == visited.end())
            {
                unvisited.push(next);
                visited.insert(next->get_id());
            }
        }
        if(cond != nullptr)
        {
            stream << current->get_id() << "->" << cond->get_id() << "\n";
            if(visited.find(cond->get_id()) == visited.end())
            {
                unvisited.push(cond);
                visited.insert(cond->get_id());
            }
        }
    } while(!unvisited.empty());

    stream << "}";
    return stream;
}

void ControlFlowGraph::to_file(const char* filename) const
{
    std::ofstream fout;
    fout.open(filename, std::ios::out);
    if(fout.is_open())
    {
        fout << *this;
        fout.close();
    }
    else
    {
        std::cerr << "Could not write file" << filename << std::endl;
    }
}

void postorder_visit(const BasicBlock* node,
                     std::queue<const BasicBlock*>* list,
                     std::unordered_set<int>* marked)
{
    marked->insert(node->get_id());
    // these are created by this class so will ALWAYS be of type BasicBlock
    const BasicBlock* next = static_cast<const BasicBlock*>(node->get_next());
    const BasicBlock* cond = static_cast<const BasicBlock*>(node->get_cond());
    if(next != nullptr && marked->find(next->get_id()) == marked->end())
    {
        postorder_visit(next, list, marked);
    }
    if(cond != nullptr && marked->find(cond->get_id()) == marked->end())
    {
        postorder_visit(cond, list, marked);
    }
    list->push(node);
}

std::queue<const BasicBlock*> ControlFlowGraph::dfst() const
{
    std::queue<const BasicBlock*> retval;
    std::unordered_set<int> visited;
    postorder_visit(root(), &retval, &visited);
    return retval;
}

void ControlFlowGraph::finalize()
{
    // check for single exit
    std::unordered_set<int> exit_nodes;
    for(unsigned int i = 0; i < nodes; i++)
    {
        if(blocks[i].get_next() == nullptr && blocks[i].get_cond() == nullptr)
        {
            exit_nodes.insert(i);
        }
    }

    if(exit_nodes.size() > 1)
    {
        // this extra node is already allocated in the constructor just in case
        nodes++;
        blocks[nodes - 1].set_id(nodes - 1);
        blocks[nodes - 1].set_next(nullptr);
        blocks[nodes - 1].set_cond(nullptr);
        for(int id : exit_nodes)
        {
            set_next(id, nodes - 1);
        }
    }
}
