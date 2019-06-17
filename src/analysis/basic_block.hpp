//
// Created by davide on 6/13/19.
//

#ifndef __BASICBLOCK_HPP__
#define __BASICBLOCK_HPP__

/**
 * \brief Basic Block representing a portion of code
 *
 * This class represents a basic block, the minimum portion of code with only a
 * single entry point and one or two exit point, located as the last instruction
 * of the block. These blocks are used to represent the flow in a portion of
 * code, thus they will contain a pointer to the next block (and a pointer to a
 * conditional block in case a conditional jump is satisfied)
 */
class BasicBlock
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
    ~BasicBlock() = default;

    /**
     * \brief Getter for the block id
     * \return the id of the block
     */
    int get_id() const;

    /**
     * \brief Setter for the block id
     * \param[in] number the id of the block
     */
    void set_id(int number);

    /**
     * \brief Getter for the next block
     *
     * Every basic block except the one representing the return of the function
     * contains a pointer to the next one: this is the next block that will be
     * executed or the block that will be executed if a conditional jump is
     * unsatisfied
     *
     * \return The next basic block that will be executed in the code, nullptr
     * if the function returns
     */
    const BasicBlock* get_next() const;

    /**
     * \brief Getter the conditional jump
     *
     * If the basic block ends with a conditional jump, this is the block where
     * the execution continues if the condition is satisfied
     *
     * \return  The next basic block that will be executed in the code if the
     * condition is satified. nullptr if no conditional jump exists
     */
    const BasicBlock* get_conditional() const;

    /**
     * \brief Setter for the next block, without conditional jumps
     * \param[in] next_blk The next block that will be executed if no
     * conditional jumps are taken
     */
    void set_next(const BasicBlock* next_blk);

    /**
     * \brief Setter for the next block, with conditional jumps
     * \param[in] next_blk The next block that will be executed if no
     * conditional jumps are taken
     * \param[in] conditional_blk The next block
     * that will be executed if a conditional jump is taken
     */
    void set_next(const BasicBlock* next_blk,
                  const BasicBlock* conditional_blk);

    /**
     * \brief Setter for the conditional block only
     * \param[in] conditional_blk The next block that will be executed if a
     * conditional jump is taken
     */
    void set_conditional(const BasicBlock* conditional_blk);

private:
    int id{0};
    const BasicBlock* next{nullptr};
    const BasicBlock* conditional{nullptr};
};

/**
 * \brief Print the control flow graph in form of .dot file
 * \param[in] bb The root node of the control flow graph
 * \param[in] filename The file where the file will be written.
 */
void print_cfg(const BasicBlock* bb, const char* filename);

#endif
