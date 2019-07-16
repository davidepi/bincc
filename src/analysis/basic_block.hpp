//
// Created by davide on 6/13/19.
//

#ifndef __BASICBLOCK_HPP__
#define __BASICBLOCK_HPP__

#include "abstract_block.hpp"

/**
 * \brief Basic Block representing a portion of code
 *
 * This class represents a basic block, the minimum portion of code with only a
 * single entry point and one or two exit point, located as the last instruction
 * of the block. These blocks are used to represent the flow in a portion of
 * code, thus they will contain a pointer to the next block (and a pointer to a
 * conditional block in case a conditional jump is satisfied)
 */
class BasicBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor, given the block id
     * \param[in] number The id of this basic block
     */
    BasicBlock(int number);

    /**
     * \brief Default constructor
     */
    BasicBlock() = default;

    /**
     * \brief Default constructor
     */
    ~BasicBlock() override = default;

    /**
     * \brief Getter the conditional jump
     *
     * If the basic block ends with a conditional jump, this is the block where
     * the execution continues if the condition is satisfied
     *
     * \return  The next basic block that will be executed in the code if the
     * condition is satisfied. nullptr if no conditional jump exists
     */
    const AbstractBlock* get_cond() const;

    /**
     * \brief Setter for the conditional block only
     * \param[in] cnd The next block that will be executed if a
     * conditional jump is taken
     */
    void set_cond(const AbstractBlock* cnd);

    /**
     * \brief Returns the type of this block
     * \return BlockType::BASIC
     */
    BlockType get_type() const override;

    /**
     * \brief Returns the number of outgoing edges from this class
     * \return 0, 1 or 2, depending on the number of outgoing edges
     */
    unsigned char get_out_edges() const override;

    /**
     * \brief Replace an edge in the block with a new one.
     * This happens only if the class has a matching edge
     * \param[in] match The target that will be looked for matching
     * \param[in] edge The new edge that will be inserted instead of the
     * matching one
     */
    void replace_if_match(const AbstractBlock* match,
                          const AbstractBlock* edge) override;

    /**
     * \brief Print this block in Graphviz dot format using the input stream
     * Then the method will return the updated stream
     * The stream will represent solely this block.
     * \param[in,out] ss The input stream
     * \return The updated stream
     */
    std::ostream& print(std::ostream& ss) const override;

private:
    // target of the conditional jump if the condition is satisfied
    const AbstractBlock* cond{nullptr};
};

#endif
