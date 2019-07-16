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

private:
    // the looping block
    const BasicBlock* looping_block;
};

/**
 * \brief Class representing a while loop
 * This class represents a loop where the entry and the exit are represented by
 * the same block, called head from here on. This while can be composed solely
 * by two blocks, meaning that everything in between should be reduced
 * beforehand. Breaks and continues should be handles manually before performing
 * the reduction
 */
class WhileBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor
     * \note The head and tail parameters will be inherited by this class
     * \param[in] id The id that will be assigned to this block
     * \param[in] head BasicBlock representing the head of the loop, containing
     * both the entry point and the exit point
     * \param[in] tail BasicBlock representing the tail of the loop, a block
     * reachable only from the head and pointing only towards the head
     */
    WhileBlock(int id, const BasicBlock* head, const AbstractBlock* tail);

    /**
     * \brief Destructor
     */
    ~WhileBlock() override;

    /**
     * \brief Returns the type of this block
     * \return BlockType::WHILE
     */
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

private:
    // entry and exit point of the loop
    const BasicBlock* head;
    // bottom point of the loop
    const AbstractBlock* tail;
};

/**
 * \brief Class representing a do-while loop
 * This class represents a loop where the entry and the exit are represented by
 * the different blocks. From here on the entry block will be called head, while
 * the exit will be called tail. This while can be composed solely by two
 * blocks, meaning that everything in between should be reduced beforehand.
 * Breaks and continues should be handles manually before performing the
 * reduction
 */
class DoWhileBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor
     * \note The head and tail parameters will be inherited by this class
     * \param[in] id The id that will be assigned to this block
     * \param[in] head BasicBlock representing the head of the loop, containing
     * both the entry point and the exit point
     * \param[in] tail BasicBlock representing the tail of the loop, a block
     * reachable only from the head and pointing only towards the head
     */
    DoWhileBlock(int id, const AbstractBlock* head, const BasicBlock* tail);

    /**
     * \brief Destructor
     */
    ~DoWhileBlock() override;

    /**
     * \brief Returns the type of this block
     * \return BlockType::WHILE
     */
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

private:
    // entry and exit point of the loop
    const AbstractBlock* head;
    // bottom point of the loop
    const BasicBlock* tail;
};

#endif //__CYCLIC_BLOCK_HPP__
