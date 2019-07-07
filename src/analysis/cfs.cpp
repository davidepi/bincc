//
// Created by davide on 7/5/19.
//

#include "cfs.hpp"
#include "acyclic_block.hpp"
#include "cyclic_block.hpp"
#include <cassert>
#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <unordered_map>
#include <unordered_set>

ControlFlowStructure::~ControlFlowStructure()
{
    delete head;
}

const AbstractBlock* ControlFlowStructure::root() const
{
    return head;
}

/**
 * \brief Perform a deep copy of the CFG and build the predecessor list and the
 * map {id, block}
 * \param[in] src starting node of the CFG
 * \param[out] bmap Map containing pairs {id, block} with the newly constructed
 * blocks
 * \param[out] pred List of predecessor in form {current id, array of
 * ids} where array of ids is a set containing the id of the predecessor for
 * each key
 * \param[in,out] visited Array containing the already visited nodes
 * \return The newly created block
 */
static AbstractBlock*
deep_copy(const BasicBlock* src, std::unordered_map<int, AbstractBlock*>* bmap,
          std::unordered_map<int, std::unordered_set<int>>* pred,
          std::unordered_set<const AbstractBlock*>* visited)
{
    visited->insert(src);
    // create the node
    int current_id = src->get_id();
    BasicBlock* created = new BasicBlock(current_id);
    bmap->insert({{current_id, created}});
    pred->insert({{current_id, std::move(std::unordered_set<int>())}});
    // resolve the children
    const BasicBlock* next = static_cast<const BasicBlock*>(src->get_next());
    const BasicBlock* cond = static_cast<const BasicBlock*>(src->get_cond());
    if(next != nullptr)
    {
        if(visited->find(next) == visited->end())
        {
            deep_copy(next, bmap, pred, visited);
        }
        int next_id = next->get_id();
        pred->find(next_id)->second.insert(current_id);
        created->set_next(bmap->find(next_id)->second);
    }
    if(cond != nullptr)
    {
        if(visited->find(cond) == visited->end())
        {
            deep_copy(cond, bmap, pred, visited);
        }
        int cond_id = cond->get_id();
        pred->find(cond_id)->second.insert(current_id);
        created->set_cond(bmap->find(cond_id)->second);
    }
    return created;
}

/**
 * \brief Recursive call of the post-order depth-first visit
 * \param[in] node the starting point of the dfs (recursive step)
 * \param[out] list the queue containing the post-order id of the visited nodes
 * \param[in, out] marked the set containing all the already-visited nodes
 * (recall that the cfg is not a tree so we must avoid loops)
 */
static void postorder_visit(const AbstractBlock* node, std::queue<int>* list,
                            std::unordered_set<const AbstractBlock*>* marked)
{
    marked->insert(node);
    // this get_next() force me to put everything const. Note to myself of the
    // future: don't attempt to remove constness just because this function is
    // private
    const AbstractBlock* next = node->get_next();
    if(next != nullptr && marked->find(next) == marked->end())
    {
        postorder_visit(next, list, marked);
    }
    if(node->get_type() == BASIC)
    {
        const BasicBlock* cond = static_cast<const BasicBlock*>(
            static_cast<const BasicBlock*>(node)->get_cond());
        if(cond != nullptr && marked->find(cond) == marked->end())
        {
            postorder_visit(cond, list, marked);
        }
    }
    list->push(node->get_id());
}

void ControlFlowStructure::build(const BasicBlock* root, int nodes)
{
    // first lets start clean and deepcopy
    std::unordered_map<int, AbstractBlock*> bmap;           // pair <id,block>
    std::unordered_map<int, std::unordered_set<int>> preds; // pair <id, preds>
    std::unordered_set<const AbstractBlock*> visited;
    deep_copy(root, &bmap, &preds, &visited);
    visited.clear();
    int next_id = nodes;
    head = bmap.find(0)->second;

    // remove self loops from predecessors, otherwise a new backlink will be
    // added everytime when replacing the parents while resolving a self-loop
    for(int i = 0; i < nodes; i++)
    {
        preds.find(i)->second.erase(i);
    }

    // TODO: reorganize this lambda function
    auto is_sequence = [&preds](const AbstractBlock* cur,
                                const AbstractBlock* next) -> bool {
        if(next != nullptr && cur->get_out_edges() == 1)
        {
            auto entry = preds.find(next->get_id());
            return entry->second.size() == 1;
        }
        return false;
    };

    auto is_self_loop = [](const AbstractBlock* cur) -> bool {
        if(cur->get_type() == BlockType::BASIC)
        {
            const BasicBlock* node = static_cast<const BasicBlock*>(cur);
            return node->get_cond() == node || node->get_next() == node;
        }
        return false;
    };

    // iterate and resolve
    while(head->get_out_edges() != 0)
    {
        std::queue<int> list;
        postorder_visit(head, &list, &visited);
        visited.clear();
        bool modified = false;
        while(!list.empty())
        {
            AbstractBlock* node = bmap.find(list.front())->second;
            list.pop();
            const AbstractBlock* next = node->get_next();
            AbstractBlock* tmp;
            if(is_self_loop(node))
            {
                tmp = new SelfLoopBlock(next_id,
                                        static_cast<const BasicBlock*>(node));
                tmp->set_next(next);
            }
            // resolve sequence:
            // this -> 1 exit
            // next -> 1 entry
            else if(is_sequence(node, next))
            {
                tmp = new SequenceBlock(next_id, node, next);
                next = next->get_next(); // the previous "next" has been merged
                tmp->set_next(next);
            }
            else
            {
                continue;
            }
            modified = true;
            bmap.insert({{next_id, tmp}});
            // change edges of graph (remap predecessor to address new block
            // instead of the old basic block)
            auto entry = preds.find(node->get_id());
            for(int parent_index : entry->second)
            {
                AbstractBlock* parent = bmap.find(parent_index)->second;
                parent->replace_if_match(node, tmp);
            }
            preds.insert({{next_id, entry->second}});
            // now remove the current node from the predecessor of the next.
            // and add the newly created node as predecessor
            if(next != nullptr)
            {
                auto parents = preds.find(next->get_id())->second;
                parents.erase(node->get_id());
                parents.insert(next_id);
            }
            next_id++;
            // account for replacement of root
            if(node == head)
            {
                head = tmp;
            }
            break;
        }
        // irreducible point
        if(!modified)
        {
            break;
        }
    }
}

std::string ControlFlowStructure::to_dot() const
{
    std::stringstream stream;
    stream << *this;
    return stream.str();
}

std::ostream& operator<<(std::ostream& stream, const ControlFlowStructure& cfs)
{
    stream << "digraph {\ncompound=true;\n";
    cfs.head->print(stream);
    stream << "}\n";
    return stream;
}

void ControlFlowStructure::to_file(const char* filename) const
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
