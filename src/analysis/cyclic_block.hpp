//
// Created by davide on 7/5/19.
//

#ifndef __CYCLIC_BLOCK_HPP__
#define __CYCLIC_BLOCK_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"

/**
 * \brief Class representing a self-loop block
 *
 * This class reprensets the simplest form of do-while, being a self looping
 * block. The internal edges of the basic block contained by this class are not
 * checked, with the assumption that if an instance of this class exists, only
 * one block is contained and that block is a self loop.
 */
class SelfLoopBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor
     * \note The loop parameter will be inherited by this class
     * \param[in] id The id that will be assigned to this block
     * \param[in] loop The BasicBlock self-looping. This MUST be a basic block
     * since it is the only structure that can have two possible exit edge, one
     * of which is self-looping. Every AbstractBlock (and derived classes except
     * BasicBlock) will always have one edge and thus a self-looping one-edged
     * block is a degenerate situation.
     */
    SelfLoopBlock(int id, const BasicBlock* loop);

    /**
     * \brief Default destructor
     */
    ~SelfLoopBlock() override;

    /**
     * \brief Returns the type of this block
     * \return BlockType::SELF_LOOP
     */
    BlockType get_type() const override;

    /**
     * \brief Returns the number of elements composing the self-loop (always 1)
     * Normally this function returns the number of elements composing the
     * AbstractBlock, but since a self-loop can have a single block, this
     * function will always return one.
     * \return The number 1
     */
    int size() const override;

    /**
     * \brief Returns the i-th element contained in the self-loop
     * However, given that the self-loop is always composed by a single element,
     * the index parameter is ignored and the function will always returns the
     * element itself. Note that despite the return type, the returned element
     * can ALWAYS be statically casted to BasicBlock.
     * \param[in] index IGNORED
     * \return The BasicBlock composing the self-loop.
     */
    const AbstractBlock* operator[](int index) const override;

    /**
     * \brief Print this block in Graphviz dot format using the input stream
     * Then the method will return the updated stream
     * The stream will represent solely this block.
     * \param[in,out] ss The input stream
     * \return The updated stream
     */
    std::ostream& print(std::ostream& ss) const override;

private:
    // the looping block
    const BasicBlock* looping_block;
};

class WhileBlock : public AbstractBlock
{
public:
    WhileBlock(int id, const BasicBlock* head, const AbstractBlock* tail);
    ~WhileBlock() override;
    BlockType get_type() const override;

    /**
     * \brief Returns the number of elements composing the while (always 2)
     * \return The number 2
     */
    int size() const override;

    /**
     * \brief Returns the i-th element contained in the loop
     * However, given that the self-loop composed by a single element,
     * \param[in] index
     * \return The loop head if index is 0, the tail otherwise
     */
    const AbstractBlock* operator[](int index) const override;

    /**
     * \brief Print this block in Graphviz dot format using the input stream
     * Then the method will return the updated stream
     * The stream will represent solely this block.
     * \param[in,out] ss The input stream
     * \return The updated stream
     */
    std::ostream& print(std::ostream& ss) const override;

private:
    const BasicBlock* head;
    const AbstractBlock* tail;
};

/**
 * \brief Returns true if the node is a self-looping node
 * \param[in] node The node that will be checked
 * \return true if node is self looping
 */
bool is_self_loop(const AbstractBlock* node);

/**
 * \brief Returns true if the node is a while, do-while or nat-while node
 * \param[in] node The node that will be checked
 * \param[out] next The next block inside the loop
 * \return true if node is a loop
 */
bool is_loop(const AbstractBlock* node, const AbstractBlock** next);

#endif //__CYCLIC_BLOCK_HPP__
