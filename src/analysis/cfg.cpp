//
// Created by davide on 6/20/19.
//

#include "cfg.hpp"
#include <climits>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <set>
#include <sstream>
#include <stack>

ControlFlowGraph::ControlFlowGraph(int size) : nodes(size), edges(0)
{
    blocks = new BasicBlock[size];
    for(unsigned int i = 0; i < size - 1; i++)
    {
        blocks[i].set_id(i);
        blocks[i].set_next(&(blocks[i + 1]));
        edges++;
        blocks[i].set_conditional(nullptr);
    }
    blocks[size - 1].set_id(size - 1);
    blocks[size - 1].set_next(nullptr);
    blocks[size - 1].set_conditional(nullptr);
}

ControlFlowGraph::~ControlFlowGraph()
{
    delete[] blocks;
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
        if(blocks[id_src].get_next() == nullptr)
        {
            edges++;
        }
        blocks[id_src].set_next(&(blocks[id_target]));
    }
}

void ControlFlowGraph::set_next_null(unsigned int id_src)
{
    if(id_src < nodes)
    {
        if(blocks[id_src].get_next() != nullptr)
        {
            edges--;
        }
        blocks[id_src].set_next(nullptr);
    }
}

void ControlFlowGraph::set_conditional(unsigned int id_src,
                                       unsigned int id_target)
{
    if(id_src < nodes && id_target < nodes)
    {
        if(blocks[id_src].get_conditional() == nullptr)
        {
            edges++;
        }
        blocks[id_src].set_conditional(&(blocks[id_target]));
    }
}

void ControlFlowGraph::set_conditional_null(unsigned int id_src)
{
    if(id_src < nodes)
    {
        if(blocks[id_src].get_conditional() != nullptr)
        {
            edges--;
        }
        blocks[id_src].set_conditional(nullptr);
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
    std::set<int> visited;
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
        next = current->get_next();
        cond = current->get_conditional();
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
