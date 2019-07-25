//
// Created by davide on 7/25/19.
//

#include "comparison.hpp"
#include <unordered_set>

Comparison::Comparison(const ControlFlowStructure& a,
                       const ControlFlowStructure& b)
{
    const uint32_t LEN_A = a.nodes_no();
    const uint32_t LEN_B = b.nodes_no();
    hash_a.resize(LEN_A);
    hash_b.resize(LEN_B);
    for(uint32_t i = 0; i < LEN_A; i++)
    {
        hash_a[i] = a.get_node(i)->structural_hash();
    }
    for(uint32_t i = 0; i < LEN_B; i++)
    {
        hash_b[i] = b.get_node(i)->structural_hash();
    }
}

bool Comparison::cloned(uint32_t* clone_a, uint32_t* clone_b) const
{
    *clone_a = UINT32_MAX;
    *clone_b = UINT32_MAX;
    if(hash_a.empty() || hash_b.empty())
    {
        return false;
    }
    std::unordered_set<uint64_t> values;
    for(uint32_t i : hash_a)
    {
        values.insert(i);
    }
    uint32_t idx;
    for(idx = hash_b.size() - 1; idx >= 0; idx--)
    {
        if(values.find(hash_b[idx]) != values.end())
        {
            *clone_b = idx;
            break;
        }
    }
    if(*clone_b == UINT32_MAX)
    {
        return false;
    }
    else
    {
        for(idx = 0; idx < (uint32_t)hash_a.size(); idx++)
        {
            if((uint32_t)hash_a[idx] == hash_b[*clone_b])
            {
                *clone_a = idx;
                return true;
            }
        }
    }
    // unreachable
    return false;
}
