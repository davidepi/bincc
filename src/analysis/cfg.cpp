//
// Created by davide on 6/20/19.
//

#include "cfg.hpp"
#include <fstream>
#include <iostream>
#include <stack>
#include <unordered_set>

ControlFlowGraph::ControlFlowGraph(uint32_t size)
    : nodes(size), edges(0), blocks(size + 1)
{
    for(uint32_t i = 0; i < size - 1; i++)
    {
        blocks[i].set_id(i);
        blocks[i].set_next(&(blocks[i + 1]));
        edges++;
        blocks[i].set_cond(nullptr);
    }
    blocks[size - 1].set_id(size - 1);
    blocks[size - 1].set_next(nullptr);
    blocks[size - 1].set_cond(nullptr);
}

std::string ControlFlowGraph::to_dot() const
{
    std::stringstream stream;
    stream << *this;
    return stream.str();
}

void ControlFlowGraph::set_next(uint32_t id_src, uint32_t id_target)
{
    if(id_src < nodes && id_target < nodes)
    {
        edges += (uint32_t)(blocks[id_src].get_next() == nullptr);
        blocks[id_src].set_next(&(blocks[id_target]));
    }
}

void ControlFlowGraph::set_next_null(uint32_t id_src)
{
    if(id_src < nodes)
    {
        edges -= (uint32_t)(blocks[id_src].get_next() != nullptr);
        blocks[id_src].set_next(nullptr);
    }
}

void ControlFlowGraph::set_conditional(uint32_t id_src, uint32_t id_target)
{
    if(id_src < nodes && id_target < nodes)
    {
        edges += (uint32_t)(blocks[id_src].get_cond() == nullptr);
        blocks[id_src].set_cond(&(blocks[id_target]));
    }
}

void ControlFlowGraph::set_conditional_null(uint32_t id_src)
{
    if(id_src < nodes)
    {
        edges -= (uint32_t)(blocks[id_src].get_cond() != nullptr);
        blocks[id_src].set_cond(nullptr);
    }
}

const BasicBlock* ControlFlowGraph::root() const
{
    return &(blocks[0]);
}

uint32_t ControlFlowGraph::nodes_no() const
{
    return nodes;
}

uint32_t ControlFlowGraph::edges_no() const
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
            stream << current->get_id() << "->" << cond->get_id()
                   << "[arrowhead=\"empty\"];\n";
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

static void postorder_visit(const BasicBlock* node,
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

/**
 * \brief Performs a DFS without additional actions. Just to mark reachability
 * \param[in] root The root node
 * \param[in] visited The already visited nodes
 */
static void dfs(const BasicBlock* root, std::vector<bool>* visited)
{
    (*visited)[root->get_id()] = true;
    const BasicBlock* left = static_cast<const BasicBlock*>(root->get_next());
    const BasicBlock* right = static_cast<const BasicBlock*>(root->get_cond());
    if(left != nullptr && !(*visited)[left->get_id()])
    {
        dfs(left, visited);
    }
    if(right != nullptr && !(*visited)[right->get_id()])
    {
        dfs(right, visited);
    }
}

void ControlFlowGraph::finalize()
{
    // check for single exit
    std::unordered_set<int> exit_nodes;
    for(uint32_t i = 0; i < nodes; i++)
    {
        if(blocks[i].get_next() == nullptr)
        {
            if(blocks[i].get_cond() == nullptr)
            {
                // return node
                exit_nodes.insert(i);
            }
            else
            {
                // node has conditional branch but no next branch... no sense
                // swap 'em
                blocks[i].set_next(blocks[i].get_cond());
                blocks[i].set_cond(nullptr);
            }
        }
        else if(blocks[i].get_next() == blocks[i].get_cond())
        {
            // both children are the same
            blocks[i].set_cond(nullptr);
        }
    }

    if(exit_nodes.size() > 1)
    {
        nodes++;
        blocks[nodes - 1].set_id(nodes - 1);
        blocks[nodes - 1].set_next(nullptr);
        blocks[nodes - 1].set_cond(nullptr);
        for(int id : exit_nodes)
        {
            set_next(id, nodes - 1);
        }
    }

    struct BBCopy
    {
        uint32_t id;
        uint32_t left_id;
        uint32_t right_id;
    };

    // at this point perform a deep copy and keep only reachable nodes
    std::vector<bool> marked(nodes, false);
    // how many skipped nodes prior to the indexed one
    std::vector<uint32_t> skipped(nodes, 0);
    // old cfg representation using only ids, in case a realloc is needed
    std::vector<BBCopy> bbmap(nodes);

    dfs(&(blocks[0]), &marked);
    int skip_counter = 0;
    for(uint32_t i = 0; i < nodes; i++)
    {
        bbmap[i].id = blocks[i].get_id();
        bbmap[i].left_id =
            blocks[i].get_next() != nullptr ?
                (const BasicBlock*)(blocks[i].get_next()) - &(blocks[0]) :
                UINT32_MAX;
        bbmap[i].right_id =
            blocks[i].get_cond() != nullptr ?
                (const BasicBlock*)(blocks[i].get_cond()) - &(blocks[0]) :
                UINT32_MAX;
        if(!marked[i])
        {
            skip_counter++;
        }
        skipped[i] = skip_counter;
    }

    // realloc everything only if there are skipped nodes
    if(skip_counter != 0)
    {
        std::vector<BasicBlock> old_blocks = std::move(blocks);
        nodes = nodes - skip_counter;
        blocks = std::vector<BasicBlock>(nodes);
        edges = 0;
        const int SIZE = bbmap.size();
        for(int old_id = 0; old_id < SIZE; old_id++)
        {
            if(marked[old_id])
            {
                const uint32_t NEW_ID = old_id - skipped[old_id];
                // this lines copies additional data of the basic block
                // that will not change
                blocks[NEW_ID] = old_blocks[old_id];
                // then the new id is assigned
                blocks[NEW_ID].set_id(NEW_ID);
                if(bbmap[old_id].left_id != UINT32_MAX)
                {
                    edges++;
                    const uint32_t LEFT_ID = bbmap[old_id].left_id;
                    blocks[NEW_ID].set_next(
                        &blocks[LEFT_ID - skipped[LEFT_ID]]);
                }
                if(bbmap[old_id].right_id != UINT32_MAX)
                {
                    edges++;
                    const uint32_t RIGHT_ID = bbmap[old_id].right_id;
                    blocks[NEW_ID].set_cond(
                        &blocks[RIGHT_ID - skipped[RIGHT_ID]]);
                }
            }
        }
    }
}

const BasicBlock* ControlFlowGraph::get_node(uint32_t id) const
{
    if(id < nodes)
    {
        return &(blocks[id]);
    }
    return nullptr;
}

void ControlFlowGraph::set_offsets(uint32_t id, uint64_t start, uint64_t end)
{
    blocks[id].set_offset(start, end);
}
