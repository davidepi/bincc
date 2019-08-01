//
// Created by davide on 7/25/19.
//

#include "comparison.hpp"
#include <stack>

Comparison::Comparison(unsigned int minimum_depth) : min_depth(minimum_depth)
{
}

void Comparison::add_baseline(const Analysis& binary)
{
  if(binary.successful())
  {
    const AbstractBlock* node = binary.get_cfs()->root();
    std::stack<const AbstractBlock*> to_visit;
    to_visit.push(node);
    std::vector<bool> visited(binary.get_cfs()->nodes_no(), false);
    while(!to_visit.empty())
    {
      node = to_visit.top();
      to_visit.pop();
      visited[node->get_id()] = true;
      const uint32_t CHILDREN_NO = node->size();
      for(uint32_t i = 0; i < CHILDREN_NO; i++)
      {
        const AbstractBlock* children = (*node)[i];
        if(!visited[children->get_id()])
        {
          to_visit.push(children);
        }
      }

      // actual logic
      if(node->get_depth() >= min_depth)
      {
        uint64_t hash = node->structural_hash();
        CloneReport report;
        report.binary = binary.get_binary_name();
        report.function = binary.get_function_name();
        report.block_id = node->get_id();
        auto it = hash_table.find(hash);
        if(it == hash_table.end())
        {
          hash_table.insert({{hash, std::vector<CloneReport>(1, report)}});
        }
        else
        {
          it->second.push_back(report);
        }
      }
    }
  }
}

bool Comparison::cloned(const Analysis& binary,
                        std::vector<CloneReport>* cloned) const
{
  bool retval = false;
  if(binary.successful())
  {
    const AbstractBlock* node = binary.get_cfs()->root();
    std::stack<const AbstractBlock*> to_visit;
    to_visit.push(node);
    std::vector<bool> visited(binary.get_cfs()->nodes_no(), false);
    while(!to_visit.empty())
    {
      node = to_visit.top();
      to_visit.pop();
      visited[node->get_id()] = true;
      const uint32_t CHILDREN_NO = node->size();
      for(uint32_t i = 0; i < CHILDREN_NO; i++)
      {
        const AbstractBlock* children = (*node)[i];
        if(!visited[children->get_id()])
        {
          to_visit.push(children);
        }
      }

      // actual logic: for every subtree of the current function check if an
      // hash of the original function exists. If min depth is respected
      if(node->get_depth() >= min_depth)
      {
        uint64_t hash = node->structural_hash();
        // iterator for the original function node
        auto orig_hash = hash_table.find(hash);
        // hash exists so there is a clone
        if(orig_hash != hash_table.end())
        {
          for(CloneReport report : orig_hash->second)
          {
            report.cloned_id = node->get_id();
            report.subtree_size = node->get_depth();
            cloned->push_back(report);
            retval = true;
          }
        }
      }
    }
  }
  return retval;
}
