//
// Created by davide on 7/25/19.
//

#include "comparison.hpp"

Comparison::Comparison(unsigned int minimum_depth)
{
}

void Comparison::add_baseline(const std::string& binary_name,
                              const std::string& method_name,
                              const Analysis& binary)
{
}
bool Comparison::cloned(const Analysis& binary,
                        std::vector<CloneReport>* cloned) const
{
    return false;
}
