//
// Created by davide on 7/3/19.
//

#ifndef __ABSTRACT_BLOCK_HPP__
#define __ABSTRACT_BLOCK_HPP__

#include <sstream>

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
    // block is a if-then block
    IF_THEN,
    // block is an if-else block
    IF_ELSE,
    // block is a while block
    WHILE,
    // block is a do-while block
    DO_WHILE,

    // total number of BlockType entry. LEAVE THIS AS LAST ENTRY!!!!
    BLOCK_TOTAL
};

/**
 * \brief Class representing an agglomerate of blocks of code. This class can
 * likely be composed by multiple blocks of any type (basic, loop, etc...) and
 * is used to represent high-level structures like loops or if-else constructs.
 * This kind of blocks always have one and only one successor, since conditional
 * jumps are merged into higher level structures.
 */
class AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor, given the block id
     * \param[in] number The id of this abstract block
     */
    explicit AbstractBlock(uint32_t number);

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
    uint32_t get_id() const;

    /**
     * \brief Setter for the block id
     * \param[in] number the id of the block
     */
    void set_id(uint32_t number);

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
    void set_next(const AbstractBlock* next_blk);

    /**
     * \brief Returns the type of this abstract block
     * \return The type of this abstract block
     */
    virtual BlockType get_type() const = 0;

    /**
     * \brief Returns the name of this abstract block
     * The returned name is based on the BlockType index and the array of names
     * statically declared in this class
     * \return The name of the abstract block
     */
    const char* get_name() const;

    /**
     * \brief Returns the number of nodes contained in this abstract block
     * If not overriden, this method returns 0
     * \return 0, in this implementation
     */
    virtual uint32_t size() const;

    /**
     * \brief Returns the i-th element contained in this abstract block
     * If not overriden, this method returns a pointer to the abstract block
     * itself
     * \param[in] index UNUSED in this implementation
     * \return this, in this implementation
     */
    virtual const AbstractBlock* operator[](uint32_t index) const;

    /**
     * \brief Returns the number of outgoing edges from this class
     * \return 0 if no outgoing edges, 1 otherwise (for this implementation)
     */
    virtual unsigned char get_out_edges() const;

    /**
     * \brief Replace an edge in the block with a new one.
     * This happens only if the class has a matching edge
     * \param[in] match The target that will be looked for matching
     * \param[in] edge The new edge that will be inserted instead of the
     * matching one
     */
    virtual void replace_if_match(const AbstractBlock* match,
                                  const AbstractBlock* edge);

    /**
     * \brief Deleted copy constructor
     * Almost every inherited class will inherit other AbstractBlocks. Also it
     * is very unlikely to have the need to copy this class.
     */
    AbstractBlock(const AbstractBlock&) = delete;

    /**
     * \brief Deleted copy-assignment operator
     * Same reason of the deleted copy-constructor
     * \return nothing
     */
    AbstractBlock& operator=(const AbstractBlock&) = delete;

    /**
     * \brief Print this block in Graphviz dot format using the input stream
     * Then the method will return the updated stream. In this implementation,
     * this method will print just the node number followed by a semicolon
     * \param[in,out] ss The input stream
     * \return The updated stream
     */
    virtual std::ostream& print(std::ostream& ss) const;

    /**
     * \brief Returns the depth of this abstract block
     * The depth refers to the maximum amount of nested nodes contained in this
     * tree
     * \return The maximum depth of the tree rooted in this node
     */
    virtual uint32_t get_depth() const;

    /**
     * \brief Returns an hash representing this abstract block
     * The hash involves only the structure of the block, so it will account for
     * every block contained inside this, but not for the actual value of the
     * basic blocks or the id of the contained block
     * \return an hash representing the structure of the current block
     */
    std::size_t structural_hash() const;

protected:
    // id of the BB
    uint32_t id{0};
    // depth of the tree generating from the abstract block
    uint32_t depth;
    // block following the current one (unconditional jump or unsatisfied
    // conditional one)
    const AbstractBlock* next{nullptr};

private:
    // names that will be returned by the get_name() function
    static const char* block_names[BLOCK_TOTAL];
};

#endif //__ABSTRACT_BLOCK_HPP__
