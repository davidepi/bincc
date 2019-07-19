//
// Created by davide on 7/5/19.
//

#ifndef __ACYCLIC_BLOCK_HPP__
#define __ACYCLIC_BLOCK_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"
#include <unordered_set>
#include <vector>

/**
 * \brief Class representing a sequence of BasicBlock
 *
 * This class is a set containing a sequence of basic blocks. The edges
 * (targets) of the nodes contained inside this class are not enforced, in order
 * to keep them constant: thus the node 2 could point to something different
 * than the node 3 (for example in the case where node 3 is itself a set of node
 * which first element is the one pointed by node 2)
 */
class SequenceBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor
     * Construct a sequence of blocks by merging other blocks
     * \note The pointer ownership of first and second will be inherited
     * \param[in] id The id of this AbstractBlock
     * \param[in] first The first element that will compose this sequence. If
     * the element is a sequence itself, it will be flattened. Note that the
     * pointer ownership will be inherited!
     * \param[in] second The second element
     * that will compose this sequence. If the element is a sequence itself, it
     * will be flattened. Note that the pointer ownership will be inherited!
     */
    SequenceBlock(int id, const AbstractBlock* first,
                  const AbstractBlock* second);

    /**
     * \brief Default destructor
     */
    ~SequenceBlock() override;

    /**
     * \brief Returns the type of this block
     * \return BlockType::SEQUENCE
     */
    BlockType get_type() const override;

    /**
     * \brief Returns the number of elements composing the sequence
     * Recall that if multiple sequences are added, they are merged (squeezed)
     * into a single one, so this sequence will effectively contain their
     * content.
     * \return The total number of elements contained in this sequence
     */
    int size() const override;

    /**
     * \brief Returns the i-th element contained in the sequence
     * \warning The bounds of the array are NOT enforced! Use size() method to
     * know the actual size of the abstract block
     * \param[in] index The index of the element that will be returned
     * \return A const pointer to the retrieved element
     */
    const AbstractBlock* operator[](int index) const override;

private:
    // components of the queue
    std::vector<const AbstractBlock*> components;

    // explained in the .cpp file
    std::vector<const AbstractBlock*> delete_list;
};

/**
 * \brief Class representing an If-Then block
 *
 * This class is composed by two blocks: an head that will always be executed
 * and terminates with a conditional statement, and a `then` block that will be
 * executed only if the conditional statement is satisfied. Exactly like the
 * SequenceBlock class, internal linking between these two blocks is not
 * enforced
 */
class IfThenBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor
     * \note The pointer ownership of ifb and thenb will be inherited
     * \param[in] id unique id of the current block
     * \param[in] ifb Pointer to the head block that will be inherited
     * \param[in] thenb Pointer to the then block that will be inherited
     */
    IfThenBlock(int id, const BasicBlock* ifb, const AbstractBlock* thenb);

    /**
     * \brief Default destructor
     */
    ~IfThenBlock() override;

    /**
     * \brief Returns the number of elements composing the if-then
     * \return always 2
     */
    int size() const override;

    /**
     * \brief Returns the i-th element contained in the if-then block
     * If index is 0 the head is returned, for every other number the `then`
     * block is returned instead
     * \param[in] index The index of the element that will be returned
     * \return the head when index equals zero, the `then` block otherwise
     */
    const AbstractBlock* operator[](int index) const override;

    /**
     * \brief Returns the type of this block
     * \return BlockType::IF_THEN
     */
    BlockType get_type() const override;

private:
    // if block
    const BasicBlock* head;
    // then block
    const AbstractBlock* then;
};

/**
 * \brief Class representing an If-Else block
 *
 * This class is composed by three blocks: an head that will always be executed
 * and terminates with a conditional statement, a `then` block that will be
 * executed only if the conditional statement is satisfied, and an `else` block
 * that will be executed if the `then` is not. Exactly like the SequenceBlock
 * class, internal linking between these two blocks is not enforced
 */
class IfElseBlock : public AbstractBlock
{
public:
    /**
     * \brief Parametrized constructor
     * \note The pointer ownership of  ifb, thenb and elseb will be inherited
     * \param[in] id unique id of the current block
     * \param[in] ifb Pointer to the head block that will be inherited
     * \param[in] thenb Pointer to the then block that will be inherited
     * \param[in] elseb Pointer the the else block that will be inherited
     */
    IfElseBlock(int id, const BasicBlock* ifb, const AbstractBlock* thenb,
                const AbstractBlock* elseb);

    /**
     * \brief Default destructor
     */
    ~IfElseBlock() override;

    /**
     * \brief Returns the number of elements composing the if-else
     * \return always 3
     */
    int size() const override;

    /**
     * \brief Returns the i-th element contained in the if-then block
     * If index is 0 the head is returned, if 1 the `then` block is returned and
     * for every other number the `else` block is returned
     * \param[in] index The index of the element that will be returned
     * \return the head when index equals zero, the `then` block when index
     * equals 1, the `else` block otherwise
     */
    const AbstractBlock* operator[](int index) const override;

    /**
     * \brief Returns the type of this block
     * \return BlockType::IF_ELSE
     */
    BlockType get_type() const override;

private:
    // if block
    const BasicBlock* head;
    // then block
    const AbstractBlock* then;
    // else block
    const AbstractBlock* ellse;
    // chained blocks
    const BasicBlock** chain;
    // total size of if-else
    int chain_len{0};
};

#endif //__ACYCLIC_BLOCK_HPP__
