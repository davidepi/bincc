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
    delete root_node;
}

const AbstractBlock* ControlFlowStructure::root() const
{
    return root_node;
}

static bool is_self_loop(const AbstractBlock* node)
{
    if(node->get_type() == BlockType::BASIC)
    {
        const BasicBlock* bb = static_cast<const BasicBlock*>(node);
        return bb->get_cond() == bb || bb->get_next() == bb;
    }
    return false;
}

static bool is_sequence(const AbstractBlock* node,
                        const std::vector<std::unordered_set<int>>* preds)
{
    // conditions for a sequence:
    // - current node has only one successor node
    // - sucessor has only one predecessor (the current node)
    // - successor has one or none successors
    //   ^--- this is necessary to avoid a double exit sequence
    if(node->get_out_edges() == 1)
    {
        // nominal case next is the correct node
        const AbstractBlock* next = node->get_next();

        // if there is only ONE out node it MUST be the next
        assert(next != nullptr);

        // return the number of parents for the next node
        return (*preds)[next->get_id()].size() == 1 &&
               next->get_out_edges() < 2;
    }
    return false;
}

static bool is_ifthen(const AbstractBlock* node,
                      const AbstractBlock** then_node,
                      const std::vector<std::unordered_set<int>>* preds)
{
    if(node->get_out_edges() == 2)
    {
        const BasicBlock* bb = static_cast<const BasicBlock*>(node);
        const AbstractBlock* next = bb->get_next();
        const AbstractBlock* cond = bb->get_cond();
        if(next->get_next() == cond)
        {
            // variant 0: next is the "then"
            *then_node = next;
            return (next->get_out_edges() == 1) &&
                   ((*preds)[next->get_id()].size() == 1);
        }
        else if(cond->get_next() == next)
        {
            // variant 1: cond is the "then"
            *then_node = cond;
            return (cond->get_out_edges() == 1) &&
                   ((*preds)[cond->get_id()].size() == 1);
        }
    }
    return false;
}

static bool is_ifelse(const AbstractBlock* node,
                      const std::vector<std::unordered_set<int>>* preds)
{
    if(node->get_out_edges() == 2)
    {
        const BasicBlock* bb = static_cast<const BasicBlock*>(node);
        const AbstractBlock* next = bb->get_next();
        const AbstractBlock* cond = bb->get_cond();
        if(next->get_out_edges() == 1 && cond->get_out_edges() == 1)
        {
            return ((*preds)[next->get_id()].size() == 1) &&
                   ((*preds)[cond->get_id()].size() == 1) &&
                   next->get_next() == cond->get_next();
        }
    }
    return false;
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
            std::stack<int>* stack, int* next_index, int* next_scc,
            std::vector<int>* scc)
{
    // in the pseudocode of the tarjan paper is written v.index and v.lowlink...
    // I don't want to change the class so arrays are used and this syntax with
    // index[array] is exploited to get a similar wording as the paper
    int v = root->get_id();
    index[v] = *next_index;
    lowlink[v] = *next_index;
    *next_index = (*next_index) + 1;
    stack->push(v);
    onstack[v] = true;

    const BasicBlock* successors[2];
    successors[0] = static_cast<const BasicBlock*>(root->get_next());
    successors[1] = static_cast<const BasicBlock*>(root->get_cond());
    for(const BasicBlock* successor : successors)
    {
        if(successor == nullptr)
        {
            continue;
        }

        int w = successor->get_id();
        if(index[w] == -1)
        {
            s_conn(successor, index, lowlink, onstack, stack, next_index,
                   next_scc, scc);
            lowlink[v] = std::min(lowlink[v], lowlink[w]);
        }
        else if(onstack[w])
        {
            lowlink[v] = std::min(lowlink[v], index[w]);
        }
    }

    if(index[v] == lowlink[v])
    {
        int x;
        do
        {
            x = stack->top();
            stack->pop();
            onstack[x] = false;
            (*scc)[x] = *next_scc;
        } while(x != v);
        (*next_scc) = (*next_scc) + 1;
    }
}

/**
 * \brief Return a bool array where every index represent if a node is a cycle
 * The Tarjan's SCC algorithm is used underneath
 * \complexity linear: O(v+e) where v are vertices and e edges of the CFG
 * \param[in] array The array containing all the graph nodes
 * \param[in] nodes The number of nodes in the graph
 * \return an array where each index correpond to the graph id and the value is
 * a boolean representing whether that node is in a cycle or not
 */
static std::vector<int> find_cycles(const BasicBlock** array, int nodes)
{
    std::stack<int> stack;
    int* index = (int*)malloc(sizeof(int) * nodes);
    memset(index, 0xFF, sizeof(int) * nodes); // set everything to -1
    int* lowlink = (int*)malloc(sizeof(int) * nodes);
    bool* onstack = (bool*)malloc(sizeof(bool) * nodes);
    memset(onstack, 0, sizeof(bool) * nodes);
    std::vector<int> scc(nodes);
    int next_scc = 0;
    int next_int = 0;
    for(int i = 0; i < nodes; i++)
    {
        int v = array[i]->get_id();
        if(index[v] == -1)
        {
            s_conn(array[i], index, lowlink, onstack, &stack, &next_int,
                   &next_scc, &scc);
        }
    }

    // now use counting sort to record IF a scc appears more than once
    // recycle the index array for this

    free(lowlink);
    free(onstack);
    free(index);
    return scc;
}

/**
 * \brief Depth first visit used in the Tarjan's dominator tree algorithm
 * \param[in] node The current node being visited
 * \param[in, out] semi Semidominator number. Check paper for more info
 * \param[in, out] vertex array of node id's corresponding to numbering
 * \param[in, out] parent array of parents in the generated spanning tree
 * \param[in, out] pred array of predecessors in the original graph
 * \param[in, out] next_num next number that will be assigned to a node
 */
static void preorder_visit(const BasicBlock* node, int* semi, int* vertex,
                           int* parent, std::unordered_set<int>* pred,
                           int* next_num)
{
    int v = node->get_id();
    semi[v] = *next_num;
    vertex[semi[v]] = v;
    (*next_num) = (*next_num) + 1;
    const BasicBlock* successors[2];
    successors[0] = static_cast<const BasicBlock*>(node->get_next());
    successors[1] = static_cast<const BasicBlock*>(node->get_cond());
    for(const BasicBlock* successor : successors)
    {
        if(successor == nullptr)
        {
            continue;
        }
        int w = successor->get_id();
        if(semi[w] == 0)
        {
            parent[w] = v;
            preorder_visit(successor, semi, vertex, parent, pred, next_num);
        }
        pred[w].insert(v);
    }
}

/**
 * \brief COMPRESS function as presented in the Tarjan's dominator algorithm
 */
static void compress(int v, int* ancestor, int* semi, int* label)
{
    if(ancestor[ancestor[v]] != 0)
    {
        compress(ancestor[v], ancestor, semi, label);
        if(semi[label[ancestor[v]]] < semi[label[v]])
        {
            label[v] = label[ancestor[v]];
        }
        ancestor[v] = ancestor[ancestor[v]];
    }
}

/**
 * \brief EVAL function as presented in the Tarjan's dominator algorithm
 */
static int eval(int v, int* ancestor, int* semi, int* label)
{
    if(ancestor[v] == 0)
    {
        return label[v];
    }
    else
    {
        compress(v, ancestor, semi, label);
        if(semi[label[ancestor[v]]] >= semi[label[v]])
        {
            return label[v];
        }
        else
        {
            return label[ancestor[v]];
        }
    }
}

/**
 * \brief LINK function as presented in the Tarjan's dominator algorithm
 */
static void link(int v, int w, int* size, int* label, const int* semi,
                 int* child, int* ancestor)
{
    int s = w;
    while(semi[label[w]] < semi[label[child[s]]])
    {
        if(size[s] + size[child[child[s]]] >= 2 * size[child[s]])
        {
            ancestor[child[s]] = s;
            child[s] = child[child[s]];
        }
        else
        {
            size[child[s]] = size[s];
            ancestor[s] = child[s];
            s = ancestor[s];
        }
    }
    label[s] = label[w];
    size[v] = size[v] + size[w];
    if(size[v] < 2 * size[w])
    {
        // swap
        s ^= child[v];
        s ^= child[v];
        s ^= child[v];
    }

    while(s != 0)
    {
        ancestor[s] = v;
        s = child[s];
    }
}

/**
 * \brief Find dominators using tarjan algorithm
 * A full version of the pseudocode implemented here can be found in the paper
 * by T.Lengauer and R.E.Tarjan named "A Fast Algorithm for Finding Dominators
 * in a Flowgraph". The array used in this implementation as well as the
 * variables names reflect the ones in the aforementioned paper
 * \param[in] array The CFG for which the dominator tree will be calculated
 * \param[in] nodes The total number of nodes in the CFG
 */
static std::vector<int> dominator(const BasicBlock** array, int nodes)
{
    // super big contiguous array
    int* cache_friendly_array = (int*)malloc(sizeof(int) * nodes * 7);
    // other array extracted as part of the big one
    int* parent = cache_friendly_array + (nodes * 0);
    int* semi = cache_friendly_array + (nodes * 1);
    int* vertex = cache_friendly_array + (nodes * 2);
    int* ancestor = cache_friendly_array + (nodes * 3);
    int* label = cache_friendly_array + (nodes * 4);
    int* size = cache_friendly_array + (nodes * 5);
    int* child = cache_friendly_array + (nodes * 6);
    std::unordered_set<int>* pred = new std::unordered_set<int>[nodes];
    std::unordered_set<int>* bucket = new std::unordered_set<int>[nodes];
    std::vector<int> dom(nodes);

    // step 1
    int next_num = 0;
    memset(semi, 0, sizeof(int) * nodes);
    memset(ancestor, 0, sizeof(int) * nodes);
    memset(child, 0, sizeof(int) * nodes);
    for(int i = 0; i < nodes; i++)
    {
        label[i] = i;
        size[i] = 1;
    }
    preorder_visit(array[0], semi, vertex, parent, pred, &next_num);
    size[0] = 0;
    label[0] = 0;
    semi[0] = 0;

    for(int n = nodes - 1; n > 0; n--)
    {
        int w = vertex[n];
        // step 2
        for(int v : pred[w])
        {
            int u = eval(v, ancestor, semi, label);
            semi[w] = semi[u] < semi[w] ? semi[u] : semi[w];
        }
        bucket[vertex[semi[w]]].insert(w);
        link(parent[w], w, size, label, semi, child, ancestor);

        // step 3
        auto it = bucket[parent[w]].begin();
        while(it != bucket[parent[w]].end())
        {
            int v = *it;
            it = bucket[parent[w]].erase(it);
            bucket[parent[w]].erase(v);
            int u = eval(v, ancestor, semi, label);
            dom[v] = semi[u] < semi[v] ? u : parent[w];
        }
    }

    // step 4
    for(int i = 1; i < nodes; i++)
    {
        int w = vertex[i];
        if(dom[w] != vertex[semi[w]])
        {
            dom[w] = dom[dom[w]];
        }
    }
    dom[0] = 0;

    free(cache_friendly_array);
    delete[] pred;
    delete[] bucket;
    return dom;
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
    // insert the entry point list as predecessor for the newly created node
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

bool ControlFlowStructure::build(const ControlFlowGraph& cfg)
{
    // first lets start clean and deepcopy
    std::vector<AbstractBlock*> bmap(cfg.nodes_no());           //[id] = block>
    std::vector<std::unordered_set<int>> preds(cfg.nodes_no()); // [id] = preds
    std::unordered_set<const AbstractBlock*> visited;
    deep_copy(cfg.root(), &bmap, &preds, &visited);
    visited.clear();
    const int NODES = cfg.nodes_no();
    int next_id = NODES;
    root_node = bmap[0];

    // prepare data for the loop resolution
    std::vector<int> scc = find_cycles((const BasicBlock**)&bmap[0], NODES);
    std::vector<int> dom = dominator((const BasicBlock**)&bmap[0], NODES);
    std::vector<bool> is_loop(NODES);
    // array containing how many times an scc appears
    int* scc_count = (int*)malloc(sizeof(int) * NODES);
    memset(scc_count, 0, sizeof(int) * NODES);
    for(int i = 0; i < NODES; i++)
    {
        scc_count[scc[i]]++;
    }
    // now writes down the results in the array indexed by node
    for(int i = 0; i < NODES; i++)
    {
        is_loop[i] = scc_count[scc[i]] > 1;
    }
    free(scc_count);

    // remove self loops from predecessors, otherwise a new backlink will be
    // added everytime when replacing the parents while resolving a self-loop
    for(int i = 0; i < NODES; i++)
    {
        preds[i].erase(i);
    }

    // iterate and resolve
    while(root_node->get_out_edges() != 0)
    {
        std::queue<int> list;
        postorder_visit(root_node, &list, &visited);
        visited.clear();
        bool modified = false;
        while(!list.empty())
        {
            const AbstractBlock* node = bmap[list.front()];
            int node_id = node->get_id();
            list.pop();
            const AbstractBlock* next = node->get_next();
            AbstractBlock* tmp;
            // block used to track the `then` part in an if-then
            const AbstractBlock* then;
            bool was_loop = false;
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
            else if(is_sequence(node, &preds))
            {
                // the sequence cannot be conditional: the cfg finalize() step
                // take care of that. In any other case anything else with more
                // than one exit has already been resolved
                tmp = new SequenceBlock(next_id, node, next);
                next = next->get_next();
                tmp->set_next(next);
            }
            // condition for the loop: being in a strong connected comp. (scc)
            // and the dominator either is in a different scc or i'm the root
            // (implies this node is the head of the cycle)
            else if(is_loop[node_id] &&
                    (scc[dom[node_id]] != scc[node_id] || node_id == 0))
            {
                const AbstractBlock* head = node;
                const AbstractBlock* tail = node->get_next();
                if(node->get_out_edges() == 2) // while loop
                {
                    const BasicBlock* head_bb = (const BasicBlock*)(head);
                    const AbstractBlock* cond = head_bb->get_cond();
                    tail = cond;
                    if(scc[next->get_id()] == scc[node_id])
                    {
                        // next is the tail so swap them
                        tail = next;
                        next = cond;
                    }
                    tmp = new WhileBlock(next_id, head_bb, tail);
                }
                else // do-while loop
                {
                    const BasicBlock* tail_bb = (const BasicBlock*)(tail);
                    next = tail->get_next();
                    if(scc[next->get_id()] == scc[node_id])
                    {
                        next = tail_bb->get_cond();
                    }
                    tmp = new DoWhileBlock(next_id, head, tail_bb);
                }

                // first check that the loop is not impossible
                if(dom[node_id] == dom[tail->get_id()])
                {
                    // this case is impossible to reduce
                    delete tmp;
                    return false;
                }
                was_loop = true;
                // TODO: insert code for nested whiles
                tmp->set_next(next);

                // remove tail from predecessors or they will propagate wrongly
                preds[node_id].erase(tail->get_id());
            }
            else
            {
                continue;
            }

            modified = true;
            // this always push at bmap[next_index], without undefined behaviour
            bmap.push_back(tmp);
            is_loop.push_back(was_loop ? false : is_loop[node_id]);
            scc.push_back(scc[node_id]);
            dom.push_back(dom[node_id]);
            std::cout << "Adding " << tmp->get_id() << " as " << tmp->get_name()
                      << " \n";
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
            if(node == root_node)
            {
                root_node = tmp;
            }
            break;
        }
        // irreducible point
        if(!modified)
        {
            return false;
        }
    }
    return true;
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
    root_node->print(ss);
    ss << "}";
    return ss.str();
}
