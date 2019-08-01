#ifndef __FUNCTION_HPP__
#define __FUNCTION_HPP__

#include <cstdint>
#include <string>

/**
 * \brief Class providing information about a single function
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class Function
{
public:
  /**
   * \brief Default constructor
   *
   * Initialize the name as empty string, the offset as 0
   */
  Function();

  /**
   * \brief Parametrized constructor
   *
   * Creates a Function instance with the given offset and the given name
   *
   * \param[in] offset The offset where the function starts
   * \param[in] name The name of the function
   */
  Function(uint64_t offset, std::string name);

  /**
   * \brief Default destructor
   */
  ~Function() = default;

  /**
   * \brief Getter for the offset
   *
   * \return an integer representing the offset of the function from the
   * beginning of the binary file
   */
  int get_offset() const;

  /**
   * \brief Getter for the name
   *
   * \return The name of the function
   */
  const std::string& get_name() const;

  /**
   * \brief Comparison of two functions based on their offset
   *
   *  This class needs to be put inside a set, hence the reason of this
   * comparator. The offset on the binary file is used as a comparison (and as
   * uniqueness of a function)
   *
   * \param[in] second The other class that will be used as a comparison
   * \return true if the current class is less (i.e. its offset is lower) than
   * the other class
   */
  bool operator<(const Function& second) const;

private:
  // offset of the function in the binary
  uint64_t offset;

  // name of the function (or generated name)
  std::string name;
};

#endif
