//
// Created by davide on 7/25/19.
//

#ifndef __COMPARISON_HPP__
#define __COMPARISON_HPP__

#include "analysis.hpp"
#include "cfs.hpp"

class Comparison
{
public:
    Comparison() = default;
    Comparison(const Analysis& disassembled);
    ~Comparison() = default;
    void add_baseline(const ControlFlowStructure& cfs);
    void add_baseline(const Analysis& function);
    bool cloned(uint32_t* clone_a, uint32_t* clone_b) const;
};

#endif //__COMPARISON_HPP__
