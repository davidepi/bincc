#ifndef __INFO_HPP__
#define __INFO_HPP__

#include "architectures/architecture.hpp"
#include <memory>

/**
 * \brief Class storing information about an executable file.
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class Info
{
public:
  /**
   * \brief Default constructor, initializes every information to false
   *
   * The Architecture will be initialized to UNKNOWN
   */
  Info();

  /**
   * \brief Parametrized constructor
   * \param[in] be true if big endian
   * \param[in] has_canary true if canaries are present
   * \param[in] stripped true if the executable is stripped
   * \param[in] b64 true if the executable is 64-bit
   */
  Info(bool be, bool has_canary, bool stripped, bool b64);

  /**
   * \brief Default destructor
   */
  ~Info() = default;

  /**
   * \brief Getter for the endianness
   *
   * \return true if the executable is big endian, false otherwise
   */
  bool is_bigendian() const;

  /**
   * \brief Getter for the buffer overflow protections
   *
   * \return true if the executable uses any form of canaries, a.k.a. stack
   * protections, false otherwise
   */
  bool has_canaries() const;

  /**
   * \brief Getter for the stripped files
   *
   * \return true if the executable is stripped (debugging symbols has been
   * removed), false otherwise
   */
  bool is_stripped() const;

  /**
   * \brief Getter for 64 bits executables
   *
   * \return true if the executable is 64 bit, false otherwise
   */
  bool is_64bit() const;

private:
  // true if big endian
  bool big_endian;

  // true if it has stack protections
  bool canary;

  // true if it is stripped
  bool stripped;

  // true if it is 64 bits
  bool bits_64;
};

#endif
