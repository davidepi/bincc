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
    void set_cond(AbstractBlock* cnd);

    /**
     * \brief Returns the type of this block
     * \return BlockType::BASIC
     */
    BlockType get_type() const override;

private:
    // target of the conditional jump if the condition is satisfied
    AbstractBlock* cond{nullptr};
};

#endif
