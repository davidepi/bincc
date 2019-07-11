//
// Created by davide on 7/5/19.
//

#ifndef __CFS_HPP__
#define __CFS_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"
#include "cfg.hpp"

class ControlFlowStructure
{
public:
    ControlFlowStructure() = default;
    ~ControlFlowStructure();
    bool build(const ControlFlowGraph& cfg);
    const AbstractBlock* root() const;
    ControlFlowStructure(const ControlFlowStructure&) = delete;
    ControlFlowStructure& operator=(const ControlFlowStructure&) = delete;

    /**
     * \brief Return a Graphviz dot representation of this CFS
     * \param[in] cfg the same ControlFlowGraph given at cosntruction time. The
     * CFG is needed given that the edges of the CFS are different and the CFG
     * given at construction time is not kept in memory. Passing a different CFG
     * may lead to undefined behaviour
     * \return a string containing the dot representation of the CFS
     */
    std::string to_dot(const ControlFlowGraph& cfg) const;

    /**
     * \brief Saves this CFS to file as a Graphviz dot file
     * \param[in] filename name of the output file. The extension is NOT
     * \param[in] cfg the same ControlFlowGraph given at cosntruction time. The
     * CFG is needed given that the edges of the CFS are different and the CFG
     * given at construction time is not kept in memory. Passing a different CFG
     * may lead to undefined behaviour
     */
    void to_file(const char* filename, const ControlFlowGraph& cfg) const;

private:
    AbstractBlock* head{nullptr};
};

std::vector<int> dominator(const BasicBlock** array, int nodes);

#endif //__CFS_HPP__
