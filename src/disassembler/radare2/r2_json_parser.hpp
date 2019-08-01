#ifndef __R2_JSON_PARSER_HPP__
#define __R2_JSON_PARSER_HPP__

#include "disassembler/function.hpp"
#include "disassembler/info.hpp"
#include "disassembler/statement.hpp"
#include <memory>
#include <string>

/**
 * \brief Parsing utilities for radare2
 *
 * This namespace contains functions to parse the result of the commands call
 * sent to radare2. These functions are useful in order to put the specific
 * radare2 return values into the structures required by this project
 */
namespace R2JsonParser
{
  /**
   * \brief Parse the string retrieved by the `aflj` radare2 command
   *
   * This method populates the Function class by parsing the string retrieved
   * by issuing the `aflj` command to r2. Attempting to parse any other JSON
   * or strings will fail. The JSON resulting from the call will be an array
   * of object where each object contains information about functions. Each
   * one of those objects can be used to create one instance of the Function
   * class. For this reason, this function expects just a single element of
   * the array.
   *
   * \param[in] json_string The JSON string that will be parsed
   * \return the populated Function class, a default one if any error occured
   */
  Function parse_function(const std::string& json_string);

  /**
   * \brief Parse the string retrieved by the `ij` radare2 command
   *
   * This method populates the Info class by parsing the string retrieved by
   * issuing the `ij` command to r2. Attempting to parse any other JSON or
   * strings will fail
   *
   * \param[in] json_string The JSON string that will be parsed
   * \return the populated Info class, a default one if any error occured
   */
  Info parse_info(const std::string& json_string);

  /**
   * \brief Parse the string retrieved by the `pdfj` radare2 command
   *
   * This method populates the Statement class by parsing the string retrieved
   * by issuing the `pdfj` command to r2. Attempting to parse any other JSON
   * or strings will fail. The JSON resulting from that call will be an array
   * of object where each object contains information about a statement of the
   * function. Each one of those objects can be used to create one instance of
   * this class. For this reason, this function expects just a single element
   * of the array.
   *
   * \param[in] json_string The JSON string that will be parsed
   * \return the populated Statement class, a default one if any error occured
   */
  Statement parse_statement(const std::string& json_string);

  /**
   * \brief Parse the string retrieved by the `ij` radare2 command
   *
   * This method gathers the architecture by parsing the string retrieved by
   * issuing the `ij` command to r2. Attempting to parse any other JSON or
   * strings will fail
   *
   * \param[in] json_string The JSON string that will be parsed
   * \return the architecture of the executable, ArchitectureUNK if any error
   * occured
   */
  std::shared_ptr<Architecture>
      parse_architecture(const std::string& json_string);

} // namespace R2JsonParser

#endif
