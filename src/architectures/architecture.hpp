//
// Created by davide on 6/14/19.
//

#ifndef __ARCHITECTURE_HPP__
#define __ARCHITECTURE_HPP__

#include <string>

/**
 * \brief Describe if a jump is conditional or not
 */
enum JumpType
{
  /**
   * \brief Not a jump at all
   */
  NONE = 0,

  /**
   * \brief Conditional jump
   */
  JUMP_CONDITIONAL = 1,

  /**
   * \brief Unconditional jump
   */
  JUMP_UNCONDITIONAL = 2,

  /**
   * \brief Unconditional return
   */
  RET_UNCONDITIONAL = 3,

  /**
   * \brief Conditional return (for some architectures like ARM)
   */
  RET_CONDITIONAL = 4
};

/**
 * \brief Class describing architecture-specific features
 */
class Architecture
{
public:
  /**
   * \brief Returns the name of this architecture
   * \return the name of the architecture
   */
  virtual std::string get_name() = 0;

  /**
   * \brief Returns the type of jump of the mnemonic
   *
   * \note A return is considered a jump, and should be addressed by this
   * method
   *
   * \param[in] mnemonic A mnemonic in form of a string
   * \return the type of jump represented by this mnemonic
   */
  virtual JumpType is_jump(const std::string& mnemonic) = 0;
};

/**
 * \brief Base class implementing architecture.
 *
 * This class is used to provide an "unsupported architecture" format.
 * Everything will return false or unknown and the analysis will probably fail
 * or produce unusable results. This is a fallback for unrecognized
 * architectures.
 */
class ArchitectureUNK final : public Architecture
{
public:
  std::string get_name() override
  {
    return "unknown";
  }

  JumpType is_jump(const std::string&) override
  {
    return NONE;
  };
};

#endif //__ARCHITECTURE_HPP__
