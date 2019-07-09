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

static void DEBUG_PREDS(std::unordered_map<int, std::unordered_set<int>> preds)
{
    //TODO: REMOVEME
#ifndef DNEBUG
    std::cout <<"Predecessor list: \n";
    for(auto& parents : preds)
    {
        std::cout << "\t" << parents.first << " -> [";
        for(int parent : parents.second)
        {
            std::cout<<parent<<",";
        }
        std::cout<<"]\n";
    }
    std::cout<<std::flush;
#endif
}

/**
 * \brief Update the predecessor list
 * Replace value of old block composing an aggregator with the new aggregator
 * id, remove the predecessors of the aggregated nodes
 * \param[in] added The newly created aggregator
 * \param[in,out] preds Predecessors map
 */
static void update_pred(const AbstractBlock* added,
                        std::unordered_map<int, std::unordered_set<int>>* preds)
{
    // insert the entry point list as predecessor for the newly created node
    const AbstractBlock* oep = (*added)[0];
    std::unordered_set<int> oep_preds = preds->find(oep->get_id())->second;
    preds->insert({{added->get_id(), std::move(oep_preds)}});

    std::unordered_set<int>* next_preds = nullptr;
    if(added->get_next() != nullptr)
    {
        next_preds = &(preds->find(added->get_next()->get_id())->second);
    }

    // for every member of the newly created abstract block
    for(int i = 0; i < added->size(); i++)
    {
        // destroy its predecessor list (to avoid inconsistent states)
        int member_id = (*added)[i]->get_id();
        preds->find(member_id)->second.clear();
        // if in the predecessors of the next block there is the current member,
        // it is replaced it with the new block id.
        // i.e. if the situation was 1 -> 2 -> 3 and we replace 2 with 4, on the
        // predecessors of the follower(3), the member (2) it is now replaced
        // with 4
        if(next_preds != nullptr &&
           next_preds->find(member_id) != next_preds->end())
        {
            next_preds->erase(member_id);
            next_preds->insert(added->get_id());
        }
    }
}

void ControlFlowStructure::build(const ControlFlowGraph& cfg)
{
    // first lets start clean and deepcopy
    std::unordered_map<int, AbstractBlock*> bmap;           // pair <id,block>
    std::unordered_map<int, std::unordered_set<int>> preds; // pair <id, preds>
    std::unordered_set<const AbstractBlock*> visited;
    deep_copy(cfg.root(), &bmap, &preds, &visited);
    visited.clear();
    const int NODES = cfg.nodes_no();
    int next_id = NODES;
    head = bmap.find(0)->second;

    // remove self loops from predecessors, otherwise a new backlink will be
    // added everytime when replacing the parents while resolving a self-loop
    for(int i = 0; i < NODES; i++)
    {
        preds.find(i)->second.erase(i);
    }

    // iterate and resolve
    while(head->get_out_edges() != 0)
    {
        std::queue<int> list;
        postorder_visit(head, &list, &visited);
        visited.clear();
        bool modified = false;
        while(!list.empty())
        {
            const AbstractBlock* node = bmap.find(list.front())->second;
            list.pop();
            const AbstractBlock* next = node->get_next();
            AbstractBlock* tmp;
            // block used to track the `then` part in an if-then
            const AbstractBlock* then;
            if(is_self_loop(node))
            {
                tmp = new SelfLoopBlock(next_id,
                                        static_cast<const BasicBlock*>(node));
                tmp->set_next(next);
            }
            else if(is_ifthen(node, &then, preds))
            {
                tmp = new IfThenBlock(next_id, node, then);
                tmp->set_next(then->get_next());
            }
            else if(is_ifelse(node, preds))
            {
                const BasicBlock* bb = static_cast<const BasicBlock*>(node);
                then = bb->get_next();
                tmp = new IfElseBlock(next_id, node, then, bb->get_cond());
                tmp->set_next(then->get_next());
            }
            else if(is_loop(node, &then))
            {
                const AbstractBlock* in;
                const AbstractBlock* tail;
                if(node->get_id() < then->get_id())
                {
                    in = node;
                    tail = then;
                }
                else
                {
                    in = then;
                    tail = node;
                    node = then;
                }
                if(in->get_out_edges() == 2) // while
                {
                    const BasicBlock* bb = static_cast<const BasicBlock*>(in);
                    tmp = new WhileBlock(next_id, bb, tail);
                    if(bb->get_next() == tail)
                    {
                        next = bb->get_cond();
                        tmp->set_next(next);
                    }
                    else
                    {
                        next = bb->get_next();
                        tmp->set_next(next);
                    }
                }
            }
            else if(is_sequence(node, preds))
            {
                // nominal case
                if(next != nullptr)
                {
                    tmp = new SequenceBlock(next_id, node, next);
                    next = next->get_next();
                }
                // conditional sequence
                else
                {
                    const AbstractBlock* cond =
                        static_cast<const BasicBlock*>(node)->get_cond();
                    tmp = new SequenceBlock(next_id, node, cond);
                    next = cond->get_next();
                }
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
            std::cout<<"Adding "<<tmp->get_id()<<"\n";
            DEBUG_PREDS(preds);
            update_pred(tmp, &preds); // udpate predecessor list
            std::cout<<"Then:\n"<<std::endl;
            DEBUG_PREDS(preds);
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

void ControlFlowStructure::to_file(const char* filename,
                                   const ControlFlowGraph& cfg) const
{
    std::ofstream fout;
    fout.open(filename, std::ios::out);
    if(fout.is_open())
    {
        fout << to_dot(cfg);
        fout.close();
    }
    else
    {
        std::cerr << "Could not write file" << filename << std::endl;
    }
}

std::string ControlFlowStructure::to_dot(const ControlFlowGraph& cfg) const
{
    std::stringstream ss;
    std::string cfg_dot = cfg.to_dot();
    ss << cfg_dot.substr(0, cfg_dot.find_last_of('}'));
    head->print(ss);
    ss << "}";
    return ss.str();
}
