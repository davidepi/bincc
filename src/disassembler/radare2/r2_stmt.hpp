#ifndef __R2_STMT_HPP__
#define __R2_STMT_HPP__

#include "r2_response.hpp"

/**
 * \brief Class providing information about a function statement
 *
 * This class can be used to parse the result of the `pdfj` call sent to
 * radare2. The JSON resulting from that call will be an array of object where
 * each object contains information about a statement of the function.
 * Each one of those objects can be used to create one instance of this class.
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class R2Stmt : public R2Response
{
public:
    /**
     * \brief Default constructor
     *
     * Initializes the offset and the target as 0x0 and the opcodes as ""
     */
    R2Stmt();

    /**
     * \brief Default destructor
     */
    ~R2Stmt() = default;

    bool from_JSON(const std::string& json_string) override;

    /**
     * \brief Getter for the offset
     *
     * \return the offset at which the instruction is located
     */
    int get_offset() const;

    /**
     * \brief Getter for the target
     *
     * \return the target of a call, or 0x0 if the opcode is not a call
     */
    int get_target() const;

    /**
     * \brief Getter for the esil representation
     *
     * \return a string representing the opcode as esil
     */
    const std::string& get_esil() const;

    /**
     * \brief Getter for the opcode
     *
     * \return the opcode represented as string (i.e. xor eax, eax)
     */
    const std::string& get_opcode() const;

private:
    // offset of the instruction in the code
    int offset;

    // target of the jump (0x0 if not a jump)
    int target;

    // representation of the instruction in esil
    std::string esil;

    // representation of the instruction with architecture specific code
    std::string opcode;
};

#endif
