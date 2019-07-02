//
// Created by davide on 6/13/19.
//

#ifndef __BASICBLOCK_HPP__
#define __BASICBLOCK_HPP__

#include <vector>

enum BlockType
{
    BASIC = 0,
    SELF_WHILE,
};

/**
 * \brief Basic Block representing a portion of code
 *
 * This class represents a basic block, the minimum portion of code with only a
 * single entry point and one or two exit point, located as the last instruction
 * of the block. These blocks are used to represent the flow in a portion of
 * code, thus they will contain a pointer to the next block (and a pointer to a
 * conditional block in case a conditional jump is satisfied).
 * This class includes additional logic (such as nested basic blocks and a
 * BlockType enum) in order to reuse the class (and thus an entire CFG) also for
 * structure recovery
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
     * condition is satisfied. nullptr if no conditional jump exists
     */
    const BasicBlock* get_cond() const;

    /**
     * \brief Setter for the next block, without conditional jumps
     * \param[in] next_blk The next block that will be executed if no
     * conditional jumps are taken
     */
    void set_next(BasicBlock* next_blk);

    /**
     * \brief Setter for the conditional block only
     * \param[in] conditional_blk The next block that will be executed if a
     * conditional jump is taken
     */
    void set_cond(BasicBlock* conditional_blk);

    /**
     * \brief Returns the type of this basic block
     * In case of an agglomerate of blocks, returns the structure represented by
     * this basic block
     * \return The type of this basic block
     */
    BlockType get_type() const;

    /**
     * \brief Returns the number of blocks agglomerated
     * If this block type is Basic, this number will be 0, meaning that it is
     * just the block itself
     * \return The number of blocks contained inside this one
     */
    size_t size() const;

    /**
     * \brief Returns the number of incoming edges of this block
     * \return the number of incoming edges
     */
    int get_edges_in() const;

    /**
     * \brief Returns the number of outgoing edges from this block
     * \return the number of outgoing edges
     */
    int get_edges_out() const;

private:
    // id of the BB
    int id{0};
    // block following the current one (unconditional jump or unsatisfied
    // conditional one)
    BasicBlock* next{nullptr};
    // target of the conditional jump if the condition is satisfied
    BasicBlock* cond{nullptr};
    // number of incoming edges
    int edges_inn{0};
    // number of outgoing edges
    int edges_out{0};
    // the type of block. Despite the name, a basic block could be an
    // agglomerate of other basic blocks representing a while for example
    BlockType type{BASIC};
    // the other blocks contained in this one (useful for structural analysis)
    std::vector<BasicBlock*> blocks;
};

#endif
