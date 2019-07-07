//
// Created by davide on 7/5/19.
//

#ifndef __CFS_HPP__
#define __CFS_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"

class ControlFlowStructure
{
public:
    ControlFlowStructure() = default;
    ~ControlFlowStructure();
    void build(const BasicBlock* root, int nodes);
    const AbstractBlock* root() const;
    ControlFlowStructure(const ControlFlowStructure&) = delete;
    ControlFlowStructure& operator=(const ControlFlowStructure&) = delete;

    /**
     * \brief Return a Graphviz dot representation of this CFS
     * \return a string containing the dot representation of the CFS
     */
    std::string to_dot() const;

    /**
     * \brief Saves this CFS to file as a Graphviz dot file
     * \param[in] filename name of the output file. The extension is NOT
     * enforced
     */
    void to_file(const char* filename) const;

    /**
     * \brief Write a CFS as Graphviz dot onto a stream
     * \param[in,out] stream the input stream that will be used
     * \param[in] cfs the CFS that will be written
     * \return the input stream after performing the write
     */
    friend std::ostream& operator<<(std::ostream& stream,
                                    const ControlFlowStructure& cfs);

private:
    AbstractBlock* head{nullptr};
};

#endif //__CFS_HPP__
