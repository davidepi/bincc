//
// Created by davide on 7/5/19.
//

#include "cfs.hpp"
#include "acyclic_block.hpp"
#include <cassert>
#include <queue>
#include <stack>
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
 * \param[in] src Source list of BasicBlocks to copy
 * \param[out] dst Already allocated array of AbstractBlock* that will be filled
 * \param[out] bmap Map containing pairs {id, block}
 * \param[out] pred List of predecessor in form {current id, array of ids} where
 * array of ids is a set containing the id of the predecessor for each key
 */
static void deep_copy(const BasicBlock* src, AbstractBlock** dst,
                      std::unordered_map<int, AbstractBlock*>* bmap,
                      std::unordered_map<int, std::unordered_set<int>>* pred)
{
    std::unordered_set<int> visited;
    std::stack<const BasicBlock*> unvisited;
    unvisited.push(src);
    visited.insert(src->get_id());
    do
    {
        const BasicBlock* current;
        const BasicBlock* next;
        const BasicBlock* cond;
        current = unvisited.top();
        unvisited.pop();
        // operate on current node: create and populate the precedence list if
        // not already existing
        int current_id = current->get_id();
        dst[current_id] = new BasicBlock(current_id);
        bmap->insert({{current_id, dst[current_id]}});
        if(pred->find(current_id) == pred->end())
        {
            pred->insert({{current_id, std::move(std::unordered_set<int>())}});
        }
        next = static_cast<const BasicBlock*>(current->get_next());
        cond = static_cast<const BasicBlock*>(current->get_cond());
        if(next != nullptr)
        {
            int next_id = next - src;
            // TODO: remove this when restructuring is fully implemented
            assert(next_id == next->get_id());
            dst[current_id]->set_next(dst[next_id]);
            auto got = pred->find(next_id);
            if(got == pred->end())
            {
                // entry does not exits in the predecessor map. create the set
                std::unordered_set<int> seq;
                seq.insert(current_id);
                pred->insert({{next_id, std::move(seq)}});
            }
            else
            {
                // entry exists, just push the predecessor
                got->second.insert(current_id);
            }
            if(visited.find(next->get_id()) == visited.end())
            {
                unvisited.push(next);
                visited.insert(next->get_id());
            }
        }
        if(cond != nullptr)
        {
            int cond_id = cond - src;
            // TODO: remove this when restructuring is fully implemented
            assert(cond_id == cond->get_id());
            static_cast<BasicBlock*>(dst[current_id])->set_cond(dst[cond_id]);
            auto got = pred->find(cond_id);
            if(got == pred->end())
            {
                // entry does not exits in the predecessor map. create the set
                std::unordered_set<int> seq;
                seq.insert(current_id);
                pred->insert({{cond_id, std::move(seq)}});
            }
            else
            {
                // entry exists, just push the predecessor
                got->second.insert(current_id);
            }
            if(visited.find(cond->get_id()) == visited.end())
            {
                unvisited.push(cond);
                visited.insert(cond->get_id());
            }
        }
    } while(!unvisited.empty());
}
/**
 * \brief Recursive call of the post-order depth-first visit
 * \param[in] node the starting point of the dfs (recursive step)
 * \param[out] list the queue containing the post-order id of the visited nodes
 * \param[in, out] marked the set containing all the already-visited nodes (this
 * is not a tree)
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

/**
 * \brief Visit a CFG and generates the dfst
 * \param[in] root Starting root of the CFG
 * \return The queue containing the CFG ids in postorder
 */
static std::queue<int> dfst(const AbstractBlock* root)
{
    std::queue<int> retval;
    std::unordered_set<const AbstractBlock*> visited;
    postorder_visit(root, &retval, &visited);
    return retval;
}

// void resolve_self_loop(AbstractBlock* original, int* next_id,
//                       std::unordered_map<int, AbstractBlock*>* bmap,
//                       std::unordered_map<int, std::unordered_set<int>>*
//                       preds)
//{
//    AbstractBlock* next = original->next;
//    AbstractBlock* cond = original->cond;
//    // keep only the self-loop edge in the original node, transfer the other
//    // edge to the wrapping node
//    if(original->get_next() == original)
//    {
//        original->set_cond(nullptr);
//    }
//    else if(original->get_cond() == original)
//    {
//        original->set_next(nullptr);
//    }
//    else
//    {
//        return; // no self loop
//    }
//    AbstractBlock* wrapper = new AbstractBlock(*next_id);
//    bmap->insert({{*next_id, wrapper}});
//    (*next_id)++;
//    wrapper->blocks.push_back(original);
//    wrapper->set_cond(cond);
//    wrapper->set_next(next);
//}

void ControlFlowStructure::build(const BasicBlock* root, int nodes)
{
    // first lets start clean and deepcopy
    AbstractBlock** absb =
        (AbstractBlock**)malloc(sizeof(AbstractBlock*) * nodes);
    std::unordered_map<int, AbstractBlock*> bmap;           // pair <id,block>
    std::unordered_map<int, std::unordered_set<int>> preds; // pair <id, preds>
    deep_copy(root, absb, &bmap, &preds);
    int next_id = nodes;
    AbstractBlock* my_root = absb[0];

    // TODO: reorganize this lambda functions
    auto is_sequence = [preds](const AbstractBlock* cur,
                               const AbstractBlock* next) -> bool {
        if(next != nullptr && cur->get_out_edges() == 1)
        {
            auto entry = preds.find(next->get_id());
            return entry->second.size() == 1;
        }
        return false;
    };

    // iterate and resolve
    while(my_root->get_out_edges() != 0)
    {
        std::queue<int> list = dfst(my_root);
        bool modified = false;
        while(!list.empty())
        {
            auto iterator = bmap.find(list.front());
            AbstractBlock* node = iterator->second;
            const AbstractBlock* next = node->get_next();
            // resolve sequence:
            // this -> 1 exit
            // next -> 1 entry
            if(is_sequence(node, next))
            {
                modified = true;
                AbstractBlock* tmp = new SequenceBlock(next_id, node, next);
                bmap.insert({{next_id, tmp}});
                // change edges of graph
                auto entry = preds.find(next->get_id());
                for(int parent_index : entry->second)
                {
                    auto parent = bmap.find(parent_index);
                    parent->second->replace_if_match(node, tmp);
                }
                next_id++;
                break;
            }
        }
        // irreducible point
        if(!modified)
        {
            break;
        }
    }

    free(absb);
}
