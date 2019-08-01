//
// Created by davide on 7/25/19.
//

#ifndef __COMPARISON_HPP__
#define __COMPARISON_HPP__

#include "analysis.hpp"
#include "cfs.hpp"

struct CloneReport
{
  std::string binary;
  std::string function;
  uint32_t block_id;
  uint32_t cloned_id;
  uint32_t subtree_size;
};

class Comparison
{
public:
  Comparison() = default;
  explicit Comparison(unsigned int minimum_depth);
  ~Comparison() = default;
  void add_baseline(const std::string& binary_name,
                    const std::string& method_name, const Analysis& binary);
  bool cloned(const Analysis& binary, std::vector<CloneReport>* cloned) const;

private:
  unsigned int min_depth{2};
  std::unordered_map<uint64_t, std::vector<CloneReport>> hash_table;
};

#endif //__COMPARISON_HPP__
