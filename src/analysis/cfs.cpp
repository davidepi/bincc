//
// Created by davide on 7/5/19.
//

#include "cfs.hpp"
#include "acyclic_block.hpp"
#include "cyclic_block.hpp"
#include <algorithm>
#include <cassert>
#include <cstring>
#include <fstream>
#include <iostream>
#include <queue>
#include <stack>
#include <unordered_set>
#include <vector>

/**
 * Struct containing information about loops
 */
struct LoopHelpers
{
    // true if the ith node is part of a loop (calculated with recompute_loops)
    std::vector<bool> is_loop;
    // the scc index of the ith node (calculated with recompute_loops)
    std::vector<uint32_t> scc;
    // the dominator ID of the ith node (calculated with dominator)
    std::vector<uint32_t> dom;
};

ControlFlowStructure::~ControlFlowStructure()
{
    if(!bmap.empty())
    {
        delete bmap[bmap.size() - 1];
    }
}

const AbstractBlock* ControlFlowStructure::get_node(uint32_t id) const
{
    return bmap[id];
}
uint32_t ControlFlowStructure::nodes_no() const
{
    return bmap.size();
}

const AbstractBlock* ControlFlowStructure::root() const
{
    if(!bmap.empty() && bmap[bmap.size() - 1]->get_out_edges() == 0)
    {
        return bmap[bmap.size() - 1];
    }
    else
    {
        return nullptr;
    }
}

/**
 * \brief Detect and transform a BasicBlock into a SelfLoopBlock
 * \complexity O(1)
 * \param[in] node The node that will be checked
 * \param[out] created The new SelfLoopBlock that will be heap-allocated
 * \param[in] next_id The ID that will be assigned to the new block
 * \return true if a block has been created, false otherwise
 */
static bool reduce_self_loop(const AbstractBlock* node, AbstractBlock** created,
                             uint32_t next_id)
{
    if(node->get_type() == BlockType::BASIC)
    {
        const BasicBlock* bb = static_cast<const BasicBlock*>(node);
        if(bb->get_cond() == bb || bb->get_next() == bb)
        {
            *created = new SelfLoopBlock(next_id,
                                         static_cast<const BasicBlock*>(node));
            // avoid adding itself as loop. This is based on the fact
            // that next and cond cannot have the same target and next
            // is the preferred one if there is a single target, with
            // cond defaulting to nullptr
            const AbstractBlock* next =
                bb->get_next() != bb ? bb->get_next() : bb->get_cond();
            (*created)->set_next(next);
            return true;
        }
    }
    return false;
}

/**
 * \brief Detect and transform multiple blocks into a SequenceBlock
 * \complexity O(1)
 * \param[in] node The node that will be checked, along with its follower
 * \param[out] created The new SequenceBlock that will be heap-allocated
 * \param[in] next_id The ID that will be assigned to the new block
 * \param[in] preds The predecessors list
 * \return true if a block has been created, false otherwise
 */
static bool
    reduce_sequence(const AbstractBlock* node, AbstractBlock** created,
                    uint32_t next_id,
                    const std::vector<std::unordered_set<uint32_t>>* preds)
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
        if((*preds)[next->get_id()].size() == 1 && next->get_out_edges() < 2)
        {
            // the sequence cannot be conditional: the cfg finalize()
            // step take care of that. In any other case anything else
            // with more than one exit has already been resolved
            *created = new SequenceBlock(next_id, node, next);
            next = next->get_next();
            (*created)->set_next(next);
            return true;
        }
    }
    return false;
}

/**
 * \brief Detect and transform blocks into an IfThenBlock
 * Also If-then chains are resolved (short circuit evaluation)
 * \complexity O(n)
 * \param[in] node The node that will be checked
 * \param[out] created The new IfThenBlock that will be heap-allocated
 * \param[in] next_id The ID that will be assigned to the new block
 * \param[in] bmap The vector containing every node
 * \param[in] preds The predecessors array
 * \return true if a block has been created, false otherwise
 */
static bool
    reduce_ifthen(const AbstractBlock* node, AbstractBlock** created,
                  uint32_t next_id, std::vector<AbstractBlock*>* bmap,
                  const std::vector<std::unordered_set<uint32_t>>* preds)
{
    if(node->get_out_edges() == 2)
    {
        const BasicBlock* head = static_cast<const BasicBlock*>(node);
        const AbstractBlock* thenb = head->get_next();
        const AbstractBlock* contd = head->get_cond();
        int thenb_preds = (*preds)[thenb->get_id()].size();
        int contd_preds = (*preds)[contd->get_id()].size();
        if(thenb->get_next() == contd && thenb->get_out_edges() == 1 &&
           thenb_preds == 1)
        {
            // variant 0: thenb is the then, cont is the next
        }
        else if(contd->get_next() == thenb && contd->get_out_edges() == 1 &&
                contd_preds == 1)
        {
            // variant 1: contd and thenb are swapped
            const AbstractBlock* tmp = contd;
            contd = thenb;
            thenb = tmp;
        }
        else
        {
            return false;
        }

        // creating a full size array is not convenient in this situation
        std::unordered_set<int> marked;
        uint32_t tmp_id = head->get_id();
        // try to ASCEND the if-then in order to discover short-circuit
        // chains
        while((*preds)[head->get_id()].size() == 1 &&
              marked.find(tmp_id) == marked.end())
        {
            marked.insert(tmp_id); // avoid looping infinitely
            const AbstractBlock* tmp_head;
            tmp_head = (*bmap)[*(*preds)[tmp_id].begin()];
            tmp_id = tmp_head->get_id();
            if(tmp_head->get_out_edges() == 2)
            {
                const BasicBlock* bb = static_cast<const BasicBlock*>(tmp_head);
                // one of the edges must point to the contd block.
                // the other one obviously point to the current head
                if(bb->get_next() == contd || bb->get_cond() == contd)
                {
                    head = bb;
                }
                else
                {
                    break;
                }
            }
            else
            {
                break;
            }
        }

        // create the if-then
        *created = new IfThenBlock(next_id, head, thenb);
        (*created)->set_next(contd);
        return true;
    }
    return false;
}

/**
 * \brief Detect and transform blocks into an IfElseBlock
 * Also If-else chains are resolved (short circuit evaluation)
 * \complexity O(n)
 * \param[in] node The node that will be checked
 * \param[out] created The new IfElseBlock that will be heap-allocated
 * \param[in] next_id The ID that will be assigned to the new block
 * \param[in] preds The predecessors array
 * \return true if a block has been created, false otherwise
 */
static bool reduce_ifelse(const AbstractBlock* node, AbstractBlock** created,
                          uint32_t next_id,
                          std::vector<std::unordered_set<uint32_t>>* preds)
{
    if(node->get_out_edges() == 2)
    {
        std::stack<int> added;

        // init: determine then block and else block based on preds
        const BasicBlock* head = static_cast<const BasicBlock*>(node);
        const AbstractBlock* thenb = head->get_cond();
        const AbstractBlock* elseb = head->get_next();
        int preds_then = (*preds)[thenb->get_id()].size();
        int preds_else = (*preds)[elseb->get_id()].size();
        uint32_t heads = 1;
        if(preds_then > 1)
        {
            if(preds_else > 1)
            {
                // not an if-else
                return false;
            }
            else if(preds_else == 1)
            {
                // could be an if-else but the then and else blocks
                // are swapped
                const AbstractBlock* tmp = thenb;
                thenb = elseb;
                elseb = tmp;
            }
        }

        // iterative step: try to descend as much as possible with the then
        const AbstractBlock* next;
        const AbstractBlock* cond;
        // creating a full size array is not convenient in this situation
        std::unordered_set<int> marked;
        uint32_t tnb_id = thenb->get_id();
        while(thenb->get_out_edges() == 2 &&
              marked.find(tnb_id) == marked.end())
        {
            marked.insert(tnb_id);
            const BasicBlock* new_hd;
            new_hd = static_cast<const BasicBlock*>(thenb);
            next = new_hd->get_next();
            cond = new_hd->get_cond();
            if(next == elseb && (*preds)[cond->get_id()].size() == 1)
            {
                heads++;
                thenb = cond;
                tnb_id = thenb->get_id(); // avoid looping forever
            }
            else if(cond == elseb && (*preds)[next->get_id()].size() == 1)
            {
                heads++;
                thenb = next;
                tnb_id = thenb->get_id(); // avoid looping forever
            }
            else
            {
                break;
            }
        }

        // last check: then and else should merge to the same block and the
        // number of entries to the else must be equal to the number of
        // heads
        if(elseb->get_out_edges() == 1 && thenb->get_out_edges() == 1 &&
           elseb->get_next() == thenb->get_next() &&
           (*preds)[elseb->get_id()].size() == heads)
        {
            *created = new IfElseBlock(next_id, head, thenb, elseb);
            (*created)->set_next(thenb->get_next());
            return true;
        }
    }
    return false;
}

/**
 * \brief Check if head is reachable from next in exactly 2 steps
 * \param[in] head The starting node of the dfs
 * \param[in] next The node reachable from head
 * \return true if a path head->next->head exists. The connection head->next is
 * not checked
 */
static bool dfs_2step(const AbstractBlock* head, const AbstractBlock* next)
{
    bool retval = next->get_next() == head;
    if(next->get_out_edges() == 2)
    {
        const BasicBlock* bb = static_cast<const BasicBlock*>(next);
        retval |= bb->get_cond() == head;
    }
    return retval;
}

/**
 * \brief Detect and transform a block into a WhileBlock or DoWhileBlock
 * The loop are ALWAYS composed of two nodes. In the other cases other
 * reductions must be performed before
 * \complexity O(1)
 * \param[in] node The node that will be checked
 * \param[out] created The new block that will be heap-allocated
 * \param[in] next_id The ID that will be assigned to the new block
 * \param[in] lh A structure composed of helpers for detecting loops. Check the
 * structure documentation for more info
 * \param[in] preds The predecessors array
 * \return true if a block has been created, false otherwise
 */
static bool reduce_loop(const AbstractBlock* node, AbstractBlock** created,
                        uint32_t next_id, const LoopHelpers& lh,
                        const std::vector<std::unordered_set<uint32_t>>* preds)
{
    uint32_t node_id = node->get_id();
    // condition for the loop: being in a strong connected comp, and the head
    // has more than 1 entry point
    if(lh.is_loop[node_id] && (*preds)[node_id].size() > 1)
    {
        const AbstractBlock* head = node;
        const AbstractBlock* next = node->get_next();
        const AbstractBlock* tail = node->get_next();
        if(node->get_out_edges() == 2) // while loop
        {
            const BasicBlock* head_bb = static_cast<const BasicBlock*>(head);
            const AbstractBlock* cond = head_bb->get_cond();
            tail = cond;
            // assert that next and cond are set correctly
            if(dfs_2step(head_bb, next))
            {
                // next is the tail so swap them
                tail = next;
                next = cond;
            }
            // assert that it is really a loop
            else if(!dfs_2step(head_bb, tail))
            {
                return false;
            }
            if(tail->get_out_edges() == 1)
            {
                *created = new WhileBlock(next_id, head_bb, tail);
            }
            else
            {
                // NATURAL LOOP
                // TODO: insert last resort here
                return false;
            }
        }
        else if(tail->get_out_edges() == 2) // do-while loop
        {
            const BasicBlock* tail_bb = static_cast<const BasicBlock*>(tail);
            next = tail->get_next();
            // assert that next and cond are set correctly
            if(next == head)
            {
                next = tail_bb->get_cond();
            }
            // assert that it is really a loop
            if(!dfs_2step(head, head->get_next()))
            {
                return false;
            }
            if(head->get_out_edges() == 1)
            {
                *created = new DoWhileBlock(next_id, head, tail_bb);
            }
            else
            {
                // NATURAL LOOP
                // TODO: insert last resort here
                return false;
            }
        }
        else
        {
            return false;
        }
        (*created)->set_next(next);
        return true;
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
 * \param[in,out] visited Array containing the already visited nodes. The array
 * is indexed by node ID
 * \return The newly created block
 */
static AbstractBlock* deep_copy(const BasicBlock* src,
                                std::vector<AbstractBlock*>* bmap,
                                std::vector<std::unordered_set<uint32_t>>* pred,
                                std::vector<bool>* visited)
{
    // create the node
    uint32_t current_id = src->get_id();
    (*visited)[current_id] = true;
    BasicBlock* created = new BasicBlock(current_id);
    (*bmap)[current_id] = created;
    (*pred)[current_id] = std::unordered_set<uint32_t>();
    // resolve the children
    const BasicBlock* next = static_cast<const BasicBlock*>(src->get_next());
    const BasicBlock* cond = static_cast<const BasicBlock*>(src->get_cond());
    if(next != nullptr)
    {
        uint32_t next_id = next->get_id();
        if(!(*visited)[next_id])
        {
            deep_copy(next, bmap, pred, visited);
        }
        (*pred)[next_id].insert(current_id);
        created->set_next((*bmap)[next_id]);
    }
    if(cond != nullptr)
    {
        uint32_t cond_id = cond->get_id();
        if(!(*visited)[cond_id])
        {
            deep_copy(cond, bmap, pred, visited);
        }
        (*pred)[cond_id].insert(current_id);
        created->set_cond((*bmap)[cond_id]);
    }
    return created;
}

/**
 * \brief Recursive call of the post-order depth-first visit
 * This call also calculate preds
 * \param[in] node the starting point of the dfs (recursive step)
 * \param[out] list the queue containing the post-order id of the visited nodes
 * \param[in, out] marked an array containing all the already-visited nodes. The
 * array index correspond to the node ID. This exploits the fact that IDs are
 * contiguous
 * \param[out] preds The preds array that will be filled by the visit. This
 */
static void postorder_visit_and_preds(
    const AbstractBlock* node, std::queue<uint32_t>* list,
    std::vector<bool>* marked, std::vector<std::unordered_set<uint32_t>>* preds)
{
    (*marked)[node->get_id()] = true;
    // this get_next() force me to put everything const. Note to myself of the
    // future: don't attempt to remove constness just because this function is
    // private
    uint32_t node_id = node->get_id();
    const AbstractBlock* next = node->get_next();
    if(next != nullptr)
    {
        (*preds)[next->get_id()].insert(node_id);
        if(!(*marked)[next->get_id()])
        {
            postorder_visit_and_preds(next, list, marked, preds);
        }
    }
    if(node->get_type() == BASIC)
    {
        const BasicBlock* cond = static_cast<const BasicBlock*>(
            static_cast<const BasicBlock*>(node)->get_cond());
        if(cond != nullptr)
        {
            (*preds)[cond->get_id()].insert(node_id);
            if(!(*marked)[cond->get_id()])
            {
                postorder_visit_and_preds(cond, list, marked, preds);
            }
        }
    }
    list->push(node->get_id());
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
void s_conn(const AbstractBlock* root, uint32_t* index, uint32_t* lowlink,
            bool* onstack, std::stack<uint32_t>* stack, uint32_t* next_index,
            uint32_t* next_scc, std::vector<uint32_t>* scc)
{
    // in the pseudocode of the tarjan paper is written v.index and v.lowlink...
    // I don't want to change the class so arrays are used and this syntax with
    // index[array] is exploited to get a similar wording as the paper
    uint32_t v = root->get_id();
    index[v] = *next_index;
    lowlink[v] = *next_index;
    *next_index = (*next_index) + 1;
    stack->push(v);
    onstack[v] = true;

    const AbstractBlock* successors[2]{nullptr, nullptr};
    successors[0] = root->get_next();
    if(root->get_out_edges() > 1)
    {
        successors[1] = static_cast<const BasicBlock*>(root)->get_cond();
    }
    for(const AbstractBlock* successor : successors)
    {
        if(successor == nullptr)
        {
            continue;
        }

        uint32_t w = successor->get_id();
        if(index[w] == UINT32_MAX)
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
        uint32_t x;
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
 * \brief Return the index of the Strongly Connected Component for every node
 * \complexity linear: O(v+e) where v are vertices and e edges of the CFG
 * \param[in] array The array containing all the graph nodes
 * \param[in] nodes The number of nodes in the graph
 * \return an array where each index correpond to the graph id and the value is
 * a boolean representing whether that node is in a cycle or not
 */
static std::vector<uint32_t> find_sccs(const AbstractBlock* const* array,
                                       uint32_t nodes)
{
    std::stack<uint32_t> stack;
    uint32_t* index = (uint32_t*)malloc(sizeof(uint32_t) * nodes);
    memset(index, 0xFF, sizeof(uint32_t) * nodes); // set everything to -1
    uint32_t* lowlink = (uint32_t*)malloc(sizeof(uint32_t) * nodes);
    bool* onstack = (bool*)malloc(sizeof(bool) * nodes);
    memset(onstack, 0, sizeof(bool) * nodes);
    std::vector<uint32_t> scc(nodes);
    uint32_t next_scc = 0;
    uint32_t next_int = 0;
    for(uint32_t i = 0; i < nodes; i++)
    {
        uint32_t v = array[i]->get_id();
        if(index[v] == UINT32_MAX)
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
static void preorder_visit(const AbstractBlock* node, uint32_t* semi,
                           uint32_t* vertex, uint32_t* parent,
                           std::unordered_set<uint32_t>* pred,
                           uint32_t* next_num)
{
    uint32_t v = node->get_id();
    semi[v] = *next_num;
    vertex[semi[v]] = v;
    (*next_num) = (*next_num) + 1;
    const AbstractBlock* successors[2]{nullptr, nullptr};
    successors[0] = node->get_next();
    if(node->get_out_edges() > 1)
    {
        successors[1] = static_cast<const BasicBlock*>(node)->get_cond();
    }
    for(const AbstractBlock* successor : successors)
    {
        if(successor == nullptr)
        {
            continue;
        }
        uint32_t w = successor->get_id();
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
static void compress(uint32_t v, uint32_t* ancestor, uint32_t* semi,
                     uint32_t* label)
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
static int eval(uint32_t v, uint32_t* ancestor, uint32_t* semi, uint32_t* label)
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
static void link(uint32_t v, uint32_t w, uint32_t* size, uint32_t* label,
                 const uint32_t* semi, uint32_t* child, uint32_t* ancestor)
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
 * variables names reflect the ones in the aforementioned paper.
 * \warning Appearently this algorithm works only if the root node has index 0
 * \param[in] array The CFG for which the dominator tree will be calculated
 * \param[in] nodes The total number of nodes in the CFG
 */
static std::vector<uint32_t> dominator(const AbstractBlock* const* array,
                                       uint32_t nodes)
{
    // super big contiguous array
    uint32_t* cache_friendly_array =
        (uint32_t*)malloc(sizeof(uint32_t) * nodes * 7);
    // other array extracted as part of the big one
    uint32_t* parent = cache_friendly_array + (nodes * 0);
    uint32_t* semi = cache_friendly_array + (nodes * 1);
    uint32_t* vertex = cache_friendly_array + (nodes * 2);
    uint32_t* ancestor = cache_friendly_array + (nodes * 3);
    uint32_t* label = cache_friendly_array + (nodes * 4);
    uint32_t* size = cache_friendly_array + (nodes * 5);
    uint32_t* child = cache_friendly_array + (nodes * 6);
    std::unordered_set<uint32_t>* pred =
        new std::unordered_set<uint32_t>[nodes];
    std::unordered_set<uint32_t>* bucket =
        new std::unordered_set<uint32_t>[nodes];
    std::vector<uint32_t> dom(nodes);

    // step 1
    uint32_t next_num = 0;
    memset(semi, 0, sizeof(uint32_t) * nodes);
    memset(ancestor, 0, sizeof(uint32_t) * nodes);
    memset(child, 0, sizeof(uint32_t) * nodes);
    for(uint32_t i = 0; i < nodes; i++)
    {
        label[i] = i;
        size[i] = 1;
    }
    preorder_visit(array[0], semi, vertex, parent, pred, &next_num);
    size[0] = 0;
    label[0] = 0;
    semi[0] = 0;

    for(uint32_t n = nodes - 1; n > 0; n--)
    {
        uint32_t w = vertex[n];
        // step 2
        for(uint32_t v : pred[w])
        {
            uint32_t u = eval(v, ancestor, semi, label);
            semi[w] = semi[u] < semi[w] ? semi[u] : semi[w];
        }
        bucket[vertex[semi[w]]].insert(w);
        link(parent[w], w, size, label, semi, child, ancestor);

        // step 3
        auto it = bucket[parent[w]].begin();
        while(it != bucket[parent[w]].end())
        {
            uint32_t v = *it;
            it = bucket[parent[w]].erase(it);
            bucket[parent[w]].erase(v);
            uint32_t u = eval(v, ancestor, semi, label);
            dom[v] = semi[u] < semi[v] ? u : parent[w];
        }
    }

    // step 4
    for(uint32_t i = 1; i < nodes; i++)
    {
        uint32_t w = vertex[i];
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
 * \brief Calculate the sccs and the bool array marking if each node is in a
 * cycle or not
 * \param[out] lh The struct containing the dominators and cycles
 * \param[in] nodes Array of nodes
 * \param nodes_len
 */
static void recompute_loops(LoopHelpers* lh, const AbstractBlock* const* nodes,
                            uint32_t nodes_len)
{
    lh->scc = find_sccs(nodes, nodes_len);
    lh->is_loop.clear();
    lh->is_loop.reserve(nodes_len);
    std::vector<int> scc_count(nodes_len, 0);
    for(uint32_t i = 0; i < nodes_len; i++)
    {
        scc_count[lh->scc[i]]++;
    }
    // now writes down the results in the array indexed by node
    for(uint32_t i = 0; i < nodes_len; i++)
    {
        lh->is_loop[i] = scc_count[lh->scc[i]] > 1;
    }
}

/**
 * \brief Remap every block pointing to the content of created, to created
 * The `created` block is an aggregate of blocks, however blocks outside this
 * aggregation will still point to the aggregated ones instead of `created`.
 * This method iterates them and remap them so they will point to `created`
 * instead of its content
 * \complexity O(n^2) in the worst case, O(2n) in the average case, given that
 * most structure are composed by only two nodes, except if-then and if-else
 * chains
 * \param[in] created The newly created block
 * \param[in,out] bmap The vector containing every block
 */
static void remap_nodes(const AbstractBlock* created,
                        std::vector<AbstractBlock*>* bmap)
{
    const uint32_t CREATED_SIZE = created->size();
    const uint32_t BMAP_SIZE = bmap->size();
    for(uint32_t i = 0; i < CREATED_SIZE; i++)
    {
        uint32_t comp = (*created)[i]->get_id();
        // every node pointing to contained nodes now point to
        // container yep, up to O(n^2) but on average will be O(2n)
        for(uint32_t node_idx = 0; node_idx < BMAP_SIZE; node_idx++)
        {
            (*bmap)[node_idx]->replace_if_match((*bmap)[comp], created);
        }
    }
}

/**
 * \brief Return the dominator for the newly created node
 * \complexity O(1)
 * \param[in] created The newly created aggregated node
 * \param[in] preds Predecessors set
 * \param[in] dominators current dominator tree for the graph
 * \return An integer representing the dominator for the newly created node
 */
static int
    compute_dominator(const AbstractBlock* created,
                      const std::vector<std::unordered_set<uint32_t>>& preds,
                      const std::vector<uint32_t>& dominators)
{
    int dom = 0;
    const std::unordered_set<uint32_t>& cur_preds =
        preds[(*created)[0]->get_id()];
    if(!cur_preds.empty())
    {
        dom = dominators[*cur_preds.begin()];
    }
    return dom;
}

bool ControlFlowStructure::build(const ControlFlowGraph& cfg)
{
    // first lets start clean and deepcopy
    const uint32_t NODES = cfg.nodes_no();
    bmap = std::vector<AbstractBlock*>(NODES);              // [id] = block
    std::vector<std::unordered_set<uint32_t>> preds(NODES); // [id] = preds
    std::vector<bool> visited(NODES, false);
    deep_copy(cfg.root(), &bmap, &preds, &visited);
    uint32_t next_id = NODES;
    const AbstractBlock* root_node = bmap[0];
    // nodes that should NOT be deleted in case of failure
    std::vector<bool> inherited(NODES, false);
    LoopHelpers lh;
    lh.dom = dominator(&bmap[0], NODES);

    // iterate and resolve
    while(root_node->get_out_edges() != 0)
    {
        std::queue<uint32_t> list;
        // update visited size if necessary (because it is accesed by index)
        visited.reserve(bmap.size());
        std::fill(visited.begin(), visited.begin() + visited.capacity(), false);
        for(auto& pred : preds)
        {
            pred.clear();
        }
        postorder_visit_and_preds(root_node, &list, &visited, &preds);
        recompute_loops(&lh, &bmap[0], bmap.size());
        bool modified = false;
        while(!list.empty())
        {
            const AbstractBlock* node = bmap[list.front()];
            list.pop();
            AbstractBlock* created = nullptr;

            // exploit short-circuit eval
            modified = reduce_self_loop(node, &created, next_id) ||
                       reduce_ifthen(node, &created, next_id, &bmap, &preds) ||
                       reduce_ifelse(node, &created, next_id, &preds) ||
                       reduce_sequence(node, &created, next_id, &preds) ||
                       reduce_loop(node, &created, next_id, lh, &preds);
            if(modified)
            {
                // this always push at bmap[next_index], without
                // undefined behaviour
                bmap.push_back(created);
                lh.dom.push_back(compute_dominator(created, preds, lh.dom));
                preds.emplace_back(std::unordered_set<uint32_t>());
                remap_nodes(created, &bmap);
                // account for replacement of root
                const uint32_t CREATED_SIZE = created->size();
                for(uint32_t i = 0; i < CREATED_SIZE; i++)
                {
                    if((*created)[i] == root_node)
                    {
                        root_node = created;
                    }
                }
                next_id++;
                if(next_id > 1000)
                {
                    std::cerr << "Killing at 1000 nodes" << std::endl;
                    modified = false;
                }
                break;
            }
        }
        // irreducible point, cleanup memory
        if(!modified)
        {
            std::fill(visited.begin(), visited.begin() + visited.capacity(),
                      false);
            postorder_visit_and_preds(root_node, &list, &visited, &preds);
            while(!list.empty())
            {
                delete bmap[list.front()];
                list.pop();
            }
            bmap.clear();
            return false;
        }
    }
    // calculate hahes for the subtrees
    const int BMAP_SIZE = bmap.size();
    hash.resize(BMAP_SIZE);
    for(int i = 0; i < BMAP_SIZE; i++)
    {
        hash[i] = bmap[i]->structural_hash();
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
    if(!bmap.empty())
    {
        bmap[bmap.size() - 1]->print(ss);
    }
    ss << "}";
    return ss.str();
}
