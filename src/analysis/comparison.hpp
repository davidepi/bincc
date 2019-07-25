//
// Created by davide on 7/25/19.
//

#ifndef __COMPARISON_HPP__
#define __COMPARISON_HPP__

#include "cfs.hpp"
class Comparison
{
public:
    Comparison(const ControlFlowStructure& a, const ControlFlowStructure& b);
    ~Comparison() = default;
    bool cloned(uint32_t* clone_a, uint32_t* clone_b) const;

private:
    std::vector<uint64_t> hash_a{0};
    std::vector<uint64_t> hash_b{0};
};

#endif //__COMPARISON_HPP__
