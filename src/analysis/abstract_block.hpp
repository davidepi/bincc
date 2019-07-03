//
// Created by davide on 7/3/19.
//

#ifndef __ABSTRACT_BLOCK_HPP__
#define __ABSTRACT_BLOCK_HPP__

#include <vector>

/**
 * \brief Identifies the type of block represented by the AbstractBlock
 */
enum BlockType
{
    // block is just a basic block
    BASIC = 0,
    // block is a self-loop
    SELF_LOOP,
    // block is a sequence
    SEQUENCE,
};

/**
 * \brief Class representing a portion of code. This class can be likely
 * composed by multiple basic blocks and is used to represent high-level
 * structures like loops or if-else constructs
 */
class AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor, given the block id
     * \param[in] number The id of this abstract block
     */
    AbstractBlock(int number);

    /**
     * \brief Default constructor
     */
    AbstractBlock() = default;

    /**
     * \brief Default constructor
     */
    ~AbstractBlock() = default;

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
     * \return The next abstract block that will be executed in the code,
     * nullptr if the function returns
     */
    const AbstractBlock* get_next() const;

    /**
     * \brief Getter the conditional jump
     *
     * If the abstract block ends with a conditional jump, this is the block
     * where the execution continues if the condition is satisfied. Usually the
     * conditional jumps are part of high-level structures (such as if-else) so
     * only low level abstract blocks retains this information
     *
     * \return  The next abstract block that will be executed in the code if the
     * condition is satisfied. nullptr if no conditional jump exists
     */
    const AbstractBlock* get_cond() const;

    /**
     * \brief Setter for the next block, without conditional jumps
     * \param[in] next_blk The next block that will be executed if no
     * conditional jumps are taken
     */
    void set_next(AbstractBlock* next_blk);

    /**
     * \brief Setter for the conditional block only
     * \param[in] conditional_blk The next block that will be executed if a
     * conditional jump is taken
     */
    void set_cond(AbstractBlock* conditional_blk);

    /**
     * \brief Returns the type of this abstract block
     * \return The type of this abstract block
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

    /**
     * \brief Getter for the abstract blocks composing a particular structure
     * \return A reference to a vector containing a pointer to every abstract
     * block composing the structure represented by the current block
     */
    const std::vector<const AbstractBlock*>& get_block_components() const;

private:
    // id of the BB
    int id{0};
    // block following the current one (unconditional jump or unsatisfied
    // conditional one)
    AbstractBlock* next{nullptr};
    // target of the conditional jump if the condition is satisfied
    AbstractBlock* cond{nullptr};
    // number of incoming edges
    int edges_inn{0};
    // number of outgoing edges
    int edges_out{0};
    // the type of block. Despite the name, a basic block could be an
    // agglomerate of other basic blocks representing a while for example
    BlockType type{BASIC};
    // the other blocks contained in this one (useful for structural analysis)
    std::vector<const AbstractBlock*> blocks;
};

#endif //__ABSTRACT_BLOCK_HPP__
