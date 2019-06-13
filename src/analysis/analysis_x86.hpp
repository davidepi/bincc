//
// Created by davide on 6/13/19.
//

#ifndef __ANALYSIS_X86_HPP__
#define __ANALYSIS_X86_HPP__

#include "analysis/analysis.hpp"

class AnalysisX86 : public Analysis
{
public:
    using Analysis::Analysis;
    ~AnalysisX86() override = default;

private:
    JumpType is_jump(const std::string& mnemonic) override;
};

#endif //__ANALYSIS_X86_HPP__
