//
// Created by davide on 6/20/19.
//

#ifndef __CFG_HPP__
#define __CFG_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"
#include <queue>
#include <string>

/**
 * \brief Control Flow Graph of a function
 *
 * This class wraps and contains the CFG of a function. The number of nodes is
 * fixed and must be specified at creation time. By default the CFG will be a
 * sequence from node 0 up to node n. Every node correspond to a basic block of
 * the CFG, represented by the BasicBlock class. Links between the blocks can be
 * set up with the following methods: <ul><li>ControlFlowGraph::set_next to add
 * an unconditional jump</li> <li>ControlFlowGraph::set_next_null to remove an
 * unconditional jump</li> <li>ControlFlowGraph::set_conditional to add a
 * conditional jump</li> <li>ControlFlowGraph::set_conditional_null to remove a
 * conditional jump</li></ul>
 */
class ControlFlowGraph
{
public:
    /**
     * \brief Default constructor
     *
     * Initialize the CFG and assigns a 0-based ID to every block. Then links
     * every block with its successor
     *
     * \param[in] size number of blocks of the CFG
     */
    ControlFlowGraph(unsigned int size);

    /**
     * \brief Default destructor
     */
    ~ControlFlowGraph();

    /**
     * \brief Finalize the computation of the CFG
     *
     * This method is used to perform things such as compute a single exit for
     * the CFG
     */
    void finalize();

    /**
     * \brief Sets an unconditional jump for this block
     *
     * If the source or target id are higher than the number of blocks, nothing
     * is performed
     *
     * \param[in] id_src block ID of the jump source
     * \param[in] id_target block ID of the jump target
     */
    void set_next(unsigned int id_src, unsigned int id_target);

    /**
     * \brief Remove an unconditional jump for this block
     *
     * If the source id is higher than the number of blocks, nothing
     * is performed
     *
     * \param[in] id_src block ID of the jump that will be removed
     */
    void set_next_null(unsigned int id_src);

    /**
     * \brief Sets a conditional jump for this block
     *
     * If the source or target id are higher than the number of blocks, nothing
     * is performed
     *
     * \param[in] id_src block ID of the jump source
     * \param[in] id_target block ID of the jump target
     */
    void set_conditional(unsigned int id_src, unsigned int id_target);

    /**
     * \brief Remove a conditional jump for this block
     *
     * If the source id is higher than the number of blocks, nothing
     * is performed
     *
     * \param[in] id_src block ID of the jump that will be removed
     */
    void set_conditional_null(unsigned int id_src);

    /**
     * \brief Retrieves the root of the CFG
     * \return The root block of the CFG
     */
    const BasicBlock* root() const;

    /**
     * \brief Returns the number of blocks in the CFG
     * \return the number of blocks of the CFG
     */
    unsigned int nodes_no() const;

    /**
     * \brief Returns the number of edges of the CFG
     * \return the number of edges of the CFG
     */
    unsigned int edges_no() const;

    /**
     * \brief Return a Graphviz dot representation of this CFG
     * \return a string containing the dot representation of the CFG
     */
    std::string to_dot() const;

    /**
     * \brief Saves this CFG to file as a Graphviz dot file
     * \param[in] filename name of the output file. The extension is NOT
     * enforced
     */
    void to_file(const char* filename) const;

    /**
     * \brief Write a CFG as Graphviz dot onto a stream
     * \param[in,out] stream the input stream that will be used
     * \param[in] cfg the CFG that will be written
     * \return the input stream after performing the write
     */
    friend std::ostream& operator<<(std::ostream& stream,
                                    const ControlFlowGraph& cfg);

    /**
     * \brief Performs a depth-first post order visit
     * \return A queue containing the blocks in postorder, depth first
     */
    std::queue<const BasicBlock*> dfst() const;

    /**
     * \brief Get a node given its id
     * \param[in] id The id of the node
     * \return the node with the given id
     */
    const BasicBlock* get_node(unsigned int id) const;

    ControlFlowGraph(ControlFlowGraph&) = delete;
    ControlFlowGraph& operator=(ControlFlowGraph&) = delete;

private:
    // number of nodes of the CFG
    unsigned int nodes;
    // number of edges of the CFG
    unsigned int edges;
    // root of the nodes
    BasicBlock* blocks;
};

#endif //__CFG_HPP__
