//
// Created by davide on 7/5/19.
//

#ifndef __ACYCLIC_BLOCK_HPP__
#define __ACYCLIC_BLOCK_HPP__

#include "abstract_block.hpp"
#include <unordered_map>
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

    /**
     * \brief Print this block in Graphviz dot format using the input stream
     * Then the method will return the last block of the cluster
     * The stream will represent solely this block. In this case it
     * will do nothing and return the block id
     * \param[in,out] ss The input stream
     * \return The id of the last node of the block
     */
    int print(std::ostream& ss) const override;

private:
    // components of the queue
    std::vector<const AbstractBlock*> components;

    // explained in the .cpp file
    std::vector<const AbstractBlock*> delete_list;
};

class IfThenBlock : public AbstractBlock
{
public:
    IfThenBlock(int id, const AbstractBlock* ifb, const AbstractBlock* thenb);
    ~IfThenBlock() override;
    int size() const override;
    const AbstractBlock* operator[](int index) const override;
    int print(std::ostream& ss) const override;
    BlockType get_type() const override;

private:
    // if block
    const AbstractBlock* head;
    // then block
    const AbstractBlock* then;
};

/**
 * \brief Returns true if node and next are a sequence
 * \param[in] node The node that will be checked if belongs to a sequence
 * \param[in] preds Map of {key, list(key)} where the list contain the
 * predecessors id for the current node
 * \return true if node and its successor forms a sequence
 */
bool is_sequence(const AbstractBlock* node,
                 const std::unordered_map<int, std::unordered_set<int>>& preds);

/**
 * \brief Returns true if node represents the root of an if-then block
 * \param[in] node The node that will be checked as the root of if-then block
 * \param[out] then_node The node representing the `then` block
 * \param[in] preds Map of {key, list(key)} where the list contain the
 * predecessors id for the current node
 * \return true if node is the root of an if-then block
 */
bool is_ifthen(const AbstractBlock* node, const AbstractBlock** then_node,
               const std::unordered_map<int, std::unordered_set<int>>& preds);

#endif //__ACYCLIC_BLOCK_HPP__
