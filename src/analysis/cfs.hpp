//
// Created by davide on 7/5/19.
//

#ifndef __CFS_HPP__
#define __CFS_HPP__

#include "abstract_block.hpp"
#include "basic_block.hpp"
#include "cfg.hpp"

/**
 * \brief Class used to recognize the high level structure of a CFG
 *
 * This class takes a ControlFlowGraph as input an generates the high level
 * structure. This structure is composed of members of the BLOCK_TYPE enum, and
 * is represented as a tree where the root node wraps all its children
 * recursively. The CFS build must be called manually with the
 * ControlFlowStructure::build() function and is NOT guaranteed to succeed.
 */
class ControlFlowStructure
{
public:
  /**
   * \brief Default constructor: actually does nothing
   */
  ControlFlowStructure() = default;

  /**
   * \brief Destructor
   */
  ~ControlFlowStructure();

  /**
   * \brief Build the CFS starting from the CFG
   * Starting from a ControlFlowGraph, this function reconstruct the
   * high-level structures of the code, ultimately generating a single
   * AbstractBlock node wrapping everything. Note that this is not guaranteed
   * to succed, hence the reason of the bool return type.
   * \param[in] cfg The input CFG that will be used to build the CFS
   * \return true if the CFS was built successfully, false otherwise
   */
  bool build(const ControlFlowGraph& cfg);

  /**
   * \brief Return the root node of the generated CFS, nullptr if not built
   * \return The root node of the CFS, or null if the generation has not been
   * performed or failed
   */
  const AbstractBlock* root() const;

  /**
   * \brief Returns a specific node of the CFS
   * \note No checks are performed in order to ensure that the node actually
   * exists: ControlFlowStructure::nodes_no() should be used!
   * \param[in] id The id of the node that will be returned
   * \return The node with the requested id
   */
  const AbstractBlock* get_node(uint32_t id) const;

  /**
   * \brief Returns the total number of nodes of this CFS
   * This includes structural nodes (otherwise the count would be equal to the
   * ControlFlowGraph::nodes_no())
   * \return The total number of nodes of the CFS
   */
  uint32_t nodes_no() const;

  /**
   * \brief Return a Graphviz dot representation of this CFS
   * The created representation contains a CFG with nodes clustered by high
   * level structures.
   * \param[in] cfg the same ControlFlowGraph given at cosntruction time. The
   * CFG is needed given that the edges of the CFS are different and the CFG
   * given at construction time is not kept in memory. Passing a different CFG
   * may lead to undefined behaviour
   * \return a string containing the dot representation of the CFS, as a
   * clustered CFG
   */
  std::string to_dot(const ControlFlowGraph& cfg) const;

  /**
   * \brief Return a Graphviz dot representation of this CFS
   * The created representation contains the CFS represented as a tree of high
   * level structures \return a string containing the dot representation of
   * the CFS, as tree
   */
  std::string to_dot() const;

  /**
   * \brief Saves this CFS to file as a Graphviz dot file, CFG variant
   * \param[in] filename name of the output file. The extension is NOT added
   * \param[in] cfg the same ControlFlowGraph given at cosntruction time. The
   * CFG is needed given that the edges of the CFS are different and the CFG
   * given at construction time is not kept in memory. Passing a different CFG
   * may lead to undefined behaviour
   */
  void to_file(const char* filename, const ControlFlowGraph& cfg) const;

  /**
   * \brief Saves this CFS to file as a Graphviz dot file, tree variant
   * \param[in] filename name of the output file. The extension is NOT added
   */
  void to_file(const char* filename) const;

  /**
   * \brief Delete copy-constructor
   */
  ControlFlowStructure(const ControlFlowStructure&) = delete;

  /**
   * \brief Delete copy-assignment operator
   * \return NA
   */
  ControlFlowStructure& operator=(const ControlFlowStructure&) = delete;

private:
  // array of every block (basic and reconstructed) used by this class
  std::vector<AbstractBlock*> bmap;
  // array containing the hash of every block (consider that non-basic blocks
  // can be seen as subtrees)
  std::vector<uint64_t> hash;
};

#endif //__CFS_HPP__
