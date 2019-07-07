//
// Created by davide on 7/5/19.
//

#ifndef __ACYCLIC_BLOCK_HPP__
#define __ACYCLIC_BLOCK_HPP__

#include "abstract_block.hpp"
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
     * \brief Returns a stream representing this block in Graphviz dot format
     * The returned stream will represent solely this block
     * \param[in,out] ss The input stream
     * \return The output stream
     */
    std::ostream& print(std::ostream& ss) const override;

private:

    // components of the queue
    std::vector<const AbstractBlock*> components;

    // explained in the .cpp file
    std::vector<const AbstractBlock*> delete_list;
};

#endif //__ACYCLIC_BLOCK_HPP__
