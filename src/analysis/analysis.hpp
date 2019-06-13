//
// Created by davide on 6/12/19.
//

#ifndef __ANALYSIS_HPP__
#define __ANALYSIS_HPP__

#include "disassembler/statement.hpp"
#include <string>
#include <unordered_map>
#include <vector>

/**
 * \brief Class used to perform the analysis of the disassembled code
 *
 * TODO: write desc
 */
class Analysis
{
public:
    /**
     * \brief Constructor given a vector of Statement
     *
     * This method is used to run the analysis on the resulting method obtained
     * after calling the Disassembler analyse() function
     *
     * \param[in] stmts A vector of Statement, likely obtained from the
     * Disassembler class
     */
    Analysis(const std::vector<Statement>* stmts);

    /**
     * \brief Constructor given a string representing a function
     *
     * This method is used to perform the analysis on a generic string obtained
     * by other means. A string should represent an entire function. The syntax
     * of the string is expected as follow: <ul> <li> The first line MUST
     * contain everything but a statement (because it is skipped since usually I
     * put the function name)</li> <li> Each line representing a statement with
     * the following syntax: <ol><li>The offset of the statement, either in
     * hexadecimal (prepended by 0x or 0X, who cares about case) or decimal
     * form</li> <li> A single space </li><li>The instruction represented as
     * string</li></ol></li></ul>
     *
     * \param[in] str A string representing a single function, formatted as
     * described in the doc
     */
    Analysis(const std::string& str);

    /**
     * \brief Access the n-th instruction
     *
     * If the instruction does not exists (out of bounds), an Instruction
     * containing an Opcode::INVALID will be returned.
     *
     * \note This funcion expect an index, and not an offset as parameter! So
     * the first instruction can be found with the value 0, not with its offset
     * in the program
     *
     * \param[in] value The index of the instruction. Not the offset!
     * \return The instruction found at the given index
     */
    Statement operator[](int value) const;

protected:
    std::vector<Statement> stmt_list;
    std::unordered_map<uint64_t, const Statement*> stmt_sparse;
};

#endif //__ANALYSIS_HPP__
