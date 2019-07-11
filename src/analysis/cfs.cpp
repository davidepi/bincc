//
// Created by davide on 7/5/19.
//

#include "cfs.hpp"
#include "acyclic_block.hpp"
#include "cyclic_block.hpp"
#include <cassert>
#include <cstring>
#include <fstream>
#include <iostream>
#include <queue>
#include <sstream>
#include <stack>
#include <unordered_set>
#include <vector>

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
 * \param[out] bmap Vector containing pairs {id, block} with the newly
 * constructed blocks. Given that nodes are indexed subsequentially, the index
 * represents the key. This should be resized to the correct size!
 * \param[out] pred List of predecessor in form {current id, array of ids} where
 * array of ids is a set containing the id of the predecessor for each key. As
 * for the bmap parameter the index of the vector is the id of the node. Also
 * this, should be resized!
 * \param[in,out] visited Array containing the already visited nodes
 * \return The newly created block
 */
static AbstractBlock*
deep_copy(const BasicBlock* src, std::vector<AbstractBlock*>* bmap,
          std::vector<std::unordered_set<int>>* pred,
          std::unordered_set<const AbstractBlock*>* visited)
{
    visited->insert(src);
    // create the node
    int current_id = src->get_id();
    BasicBlock* created = new BasicBlock(current_id);
    (*bmap)[current_id] = created;
    (*pred)[current_id] = std::move(std::unordered_set<int>());
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
        (*pred)[next_id].insert(current_id);
        created->set_next((*bmap)[next_id]);
    }
    if(cond != nullptr)
    {
        if(visited->find(cond) == visited->end())
        {
            deep_copy(cond, bmap, pred, visited);
        }
        int cond_id = cond->get_id();
        (*pred)[cond_id].insert(current_id);
        created->set_cond((*bmap)[cond_id]);
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

static void DEBUG_PREDS(std::vector<std::unordered_set<int>>* preds)
{
    // TODO: REMOVEME LATER
#ifndef DNEBUG
    std::cout << "Predecessor list: \n";
    int size = preds->size();
    for(int i = 0; i < size; i++)
    {
        std::cout << "\t" << i << " -> [";
        for(int parent : (*preds)[i])
        {
            std::cout << parent << ",";
        }
        std::cout << "]\n";
    }
    std::cout << std::flush;
#endif
}

/**
 * \brief Strong connected components, recursive step
 * For every array the array index is the node id
 * \param[in] root root node of the graph
 * \param[in,out] index array representing tarjan's index for every node
 * \param[in,out] lowlink array representing backlinks indexes
 * \param[in,out] onstack array tracking if the node is on the stack
 * \param[in,out] stack Stack containing all the nodes of a scc
 * \param[in,out] next_index Next index that will be assigned to a node
 * \param[in,out] next_scc The next index that will be assigned to an scc
 * \param[out] scc array reporting the scc index for every node
 */
void s_conn(const BasicBlock* root, int* index, int* lowlink, bool* onstack,
            std::stack<int>* stack, int* next_index, int* next_scc, int* scc)
{
    // in the pseudocode of the tarjan paper is written v.index and v.lowlink...
    // I don't want to change the class so arrays are used and this syntax with
    // index[array] is exploited to get a similar wording as the paper
    int v = root->get_id();
    v[index] = *next_index;
    v[lowlink] = *next_index;
    *next_index = (*next_index) + 1;
    stack->push(v);
    v[onstack] = true;

    const BasicBlock* successors[2];
    successors[0] = static_cast<const BasicBlock*>(root->get_next());
    successors[1] = static_cast<const BasicBlock*>(root->get_cond());
    for(auto successor : successors)
    {
        if(successor == nullptr)
        {
            continue;
        }

        int w = successor->get_id();
        if(w[index] == -1)
        {
            s_conn(successor, index, lowlink, onstack, stack, next_index,
                   next_scc, scc);
            v[lowlink] = std::min(v[lowlink], w[lowlink]);
        }
        else if(w[onstack])
        {
            v[lowlink] = std::min(v[lowlink], w[index]);
        }
    }

    if(v[lowlink] == v[index])
    {
        int x;
        do
        {
            x = stack->top();
            stack->pop();
            x[onstack] = false;
            scc[x] = *next_scc;
        } while(x != v);
        (*next_scc) = (*next_scc) + 1;
    }
}

/**
 * \brief Return a bool array where every index represent if a node is a cycle
 * \param[in] array The array containing all the graph nodes
 * \param[in] nodes The number of nodes in the graph
 * \return an array where each index correpond to the graph id and the value is
 * a boolean representing whether that node is in a cycle or not
 */
static std::vector<bool> find_cycles(const BasicBlock** array, int nodes)
{
    std::stack<int> stack;
    int* index = (int*)malloc(sizeof(int) * nodes);
    memset(index, -1, sizeof(int) * nodes);
    int* lowlink = (int*)malloc(sizeof(int) * nodes);
    bool* onstack = (bool*)malloc(sizeof(bool) * nodes);
    memset(onstack, 0, sizeof(bool) * nodes);
    int* scc = (int*)malloc(sizeof(int) * nodes);
    int next_scc = 0;
    int next_int = 0;
    for(int i = 0; i < nodes; i++)
    {
        int v = array[i]->get_id();
        if(v[index] == -1)
        {
            s_conn(array[i], index, lowlink, onstack, &stack, &next_int,
                   &next_scc, scc);
        }
    }

    // now use counting sort to record IF a scc appears more than once
    // recycle the index array for this

    free(lowlink);
    free(onstack);
    memset(index, 0, sizeof(int) * nodes);
    for(int i = 0; i < nodes; i++)
    {
        // array containing how many times an scc appears
        index[scc[i]]++;
    }
    std::vector<bool> retval(nodes);
    // now writes down the results in the array indexed by node
    for(int i = 0; i < nodes; i++)
    {
        retval[i] = index[scc[i]] > 1;
    }
    free(scc);
    free(index);
    return std::move(retval);
}

/**
 * \brief Update the predecessor list
 * Replace value of old block composing an aggregator with the new aggregator
 * id, remove the predecessors of the aggregated nodes
 * \param[in] added The newly created aggregator
 * \param[in,out] preds Predecessors map (but it is an array)
 */
static void update_pred(const AbstractBlock* added,
                        std::vector<std::unordered_set<int>>* preds)
{
    // insert the entry point list as predecessor for the newly created
    // node. This is here and not in update_preds so the intent is clear
    const AbstractBlock* oep = (*added)[0];
    preds->push_back(std::move((*preds)[oep->get_id()]));

    // get predecessors list for the newly created abstract block
    std::unordered_set<int>* next_preds = nullptr;
    if(added->get_next() != nullptr)
    {
        next_preds = &((*preds)[added->get_next()->get_id()]);
    }

    // for every member of the newly created abstract block
    for(int i = 0; i < added->size(); i++)
    {
        // destroy its predecessor list (to avoid inconsistent states)
        int member_id = (*added)[i]->get_id();
        (*preds)[member_id].clear();
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
    std::vector<AbstractBlock*> bmap(cfg.nodes_no());           //[id] = block>
    std::vector<std::unordered_set<int>> preds(cfg.nodes_no()); // [id] = preds
    std::unordered_set<const AbstractBlock*> visited;
    deep_copy(cfg.root(), &bmap, &preds, &visited);
    visited.clear();
    const int NODES = cfg.nodes_no();
    int next_id = NODES;
    head = bmap[0];

    // remove self loops from predecessors, otherwise a new backlink will be
    // added everytime when replacing the parents while resolving a self-loop
    for(int i = 0; i < NODES; i++)
    {
        preds[i].erase(i);
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
            const AbstractBlock* node = bmap[list.front()];
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
            else if(is_ifthen(node, &then, &preds))
            {
                tmp = new IfThenBlock(next_id, node, then);
                tmp->set_next(then->get_next());
            }
            else if(is_ifelse(node, &preds))
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
            else if(is_sequence(node, &preds))
            {
                // the sequence cannot be conditional: the cfg finalize() step
                // take care of that. In any other case anything else with more
                // than one exit has already been resolved
                tmp = new SequenceBlock(next_id, node, next);
                next = next->get_next();
                tmp->set_next(next);
            }
            else
            {
                continue;
            }

            modified = true;
            // this always push at bmap[next_index], without undefined behaviour
            bmap.push_back(tmp);
            std::cout << "Adding " << tmp->get_id() << "\n";
            DEBUG_PREDS(&preds);

            // change edges of graph (remap predecessor to address new block
            // instead of the old basic block)
            for(int parent_index : preds[node->get_id()])
            {
                AbstractBlock* parent = bmap[parent_index];
                parent->replace_if_match(node, tmp);
            }

            update_pred(tmp,
                        &preds); // create new node and udpate predecessor list
            std::cout << "Then:\n" << std::endl;
            DEBUG_PREDS(&preds);
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
