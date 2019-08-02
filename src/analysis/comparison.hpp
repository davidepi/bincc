//
// Created by davide on 7/25/19.
//

#ifndef __COMPARISON_HPP__
#define __COMPARISON_HPP__

#include "analysis.hpp"
#include "cfs.hpp"

/**
 * \brief Struct reporting a part of code that has been detected as clone
 * This struct does not provide info about the cloned function because that info
 * should be available when checking for clones (when invoking the
 * Comparison::cloned function)
 */
struct CloneReport
{
  // original binary name
  std::string binary;
  // original function name
  std::string function;
  // original block id (in the CFS)
  uint32_t block_id;
  // cloned block id (in the cloned CFS)
  uint32_t cloned_id;
  // height of the cloned tree
  uint32_t subtree_size;
};

/**
 * \brief Class performing the actual clone detection between two functions
 * This class takes as input one or more Analysis object(s) to be set as
 * baseline and then performs the comparison against other Analysis objects
 * using tree hashing in to achieve fast performance. Only CFS with a
 * ControlFlowStructure::depth() higher than Comparison::min_depth will be
 * reported
 */
class Comparison
{
public:
  /**
   * Default constructor
   */
  Comparison() = default;

  /**
   * \brief Constructor, setting the minimum depth required
   * If this parameter is not set, every basic block will report an hash equal
   * to every other basic block and this is clearly unwanted. For this reason,
   * this is the minimum depth that will be used to take into consideration two
   * hashes that are identical.
   * \param[in] minimum_depth The minimum depth of the tree that will be
   * considered
   */
  explicit Comparison(uint32_t minimum_depth);

  /**
   * Default destructor
   */
  ~Comparison() = default;

  /**
   * \brief Add a function to be considered as baseline.
   * This method can be invoked multiple times
   * \param[in] binary The analysis performed on the function
   */
  void add_baseline(const Analysis& binary);

  /**
   * \brief Check if the input function is a clone of the baseline
   * This method takes as input a function in form of Analysis and checks if it
   * is a clone of the ones set as baseline. If this is true, the function fills
   * the cloned vector with the reported clones
   * \note The reported clones may contain duplicates, especially subtrees,
   * since every subtree is considered a root. It is then required to
   * post-process these results.
   * \param[in] binary The function that will be checked, in form of Analysis
   * class
   * \param[out] cloned The vector that will be filled in case a clone is
   * found \return true if a clone was found, false otherwise
   */
  bool cloned(const Analysis& binary, std::vector<CloneReport>* cloned) const;

  /**
   * \brief Print two dot files, highlighting the clones parts in red
   * These files represent the CFS in dot format.
   * \note If no match between baseline and clone is found in the report,
   * calling this function is equivalent to calling the to_file of the CFS for
   * both the baseline and clone
   * \param[in] baseline_file The path to the baseline dot file
   * \param[in] clone_file The path to the clone dot file
   * \param[in] baseline The baseline analysis
   * \param[in] clone The clone analysis
   * \param[in] report The reported clones
   */
  void to_file(const char* baseline_file, const char* clone_file,
               const Analysis& baseline, const Analysis& clone,
               const std::vector<CloneReport>& report) const;

private:
  // minimum depth that a tree must be in order to be considered clone
  uint32_t min_depth{2};
  // the hashes of every subtree in the baseline
  std::unordered_map<uint64_t, std::vector<CloneReport>> hash_table;
};

#endif //__COMPARISON_HPP__
