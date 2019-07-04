//
// Created by davide on 7/3/19.
//

#ifndef __ABSTRACT_BLOCK_HPP__
#define __ABSTRACT_BLOCK_HPP__

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
 * composed by multiple blocks of any type (basic, loop, etc...)
 * and is used to represent high-level structures like loops or if-else
 * constructs. This kind of blocks always have one and only one successor, since
 * conditional jumps are merged into higher level structures.
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
    virtual ~AbstractBlock() = default;

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
     * \brief Setter for the next block, without conditional jumps
     * \param[in] next_blk The next block that will be executed if no
     * conditional jumps are taken
     */
    void set_next(AbstractBlock* next_blk);

    /**
     * \brief Returns the type of this abstract block
     * \return The type of this abstract block
     */
    virtual BlockType get_type() const = 0;

    // these two fields are modified by subclasses in different instances so
    // must remain public

    /** number of incoming edges */
    int edges_inn{0};

    /** number of outgoing edges */
    int edges_out{0};

protected:
    // id of the BB
    int id{0};
    // block following the current one (unconditional jump or unsatisfied
    // conditional one)
    AbstractBlock* next{nullptr};
};

#endif //__ABSTRACT_BLOCK_HPP__
