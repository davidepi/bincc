#ifndef __STATEMENT_HPP__
#define __STATEMENT_HPP__

#include <cstdint>
#include <string>

/**
 * \brief Class providing information about a function statement
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class Statement
{
public:
    /**
     * \brief Default constructor
     *
     * Initializes the offset and the target as 0x0 and the opcodes as ""
     */
    Statement();

    /**
     * \brief Parametrized constructor
     * \param[in] offset The offset where the instruction can be found. Case
     * insensitive
     * \param[in] opcode A string representing the opcode (Intel
     * syntax)
     */
    Statement(uint64_t offset, std::string opcode);

    /**
     * \brief Default destructor
     */
    ~Statement() = default;

    /**
     * \brief Getter for the offset
     *
     * \return the offset at which the instruction is located
     */
    int get_offset() const;

    /**
     * \brief Getter for the entire command composed by opcode and args.
     *
     * The command will ALWAYS be lowercase
     *
     * \return the entire command composing the instruction (i.e. xor eax, eax)
     */
    std::string get_command() const;

    /**
     * \brief Getter for the mnemonic
     *
     * The mnemonic will ALWAYS be lowercase
     *
     * \return the mnemonic represented as string (i.e. xor)
     */
    std::string get_mnemonic() const;

    /**
     * \brief Getter for the arguments
     *
     * The registers will ALWAYS be represented lowercase
     *
     * \return the arguments of the instruction (i.e. eax, eax)
     */
    std::string get_args() const;

private:
    // offset of the instruction in the code
    uint64_t offset;

    // representation of the instruction with architecture specific code
    std::string instruction;

    // index where the arguments of the instruction start (in the instruction
    // string)
    int args_at;
};

#endif
