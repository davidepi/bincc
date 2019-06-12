#ifndef __DISASSEMBLER_HPP__
#define __DISASSEMBLER_HPP__

#include "architecture.hpp"
#include "function.hpp"
#include "statement.hpp"
#include <set>
#include <string>
#include <unordered_map>
#include <vector>

/**
 * \brief Interface providing disassembler utilities
 *
 * This class provides generic disassembly services. In order to add a new
 * disassembler to the project, the user should override this class an implement
 * the Disassembler::analyze() method. The analyzed method is responsible of
 * populating the various fields with information about the actual disassembled
 * result.
 *
 * Every instance of this class is specific to one single binary file that must
 * be analysed. It is possible to change the analysed binary with the set_binary
 * method, but this requires calling analyze() again.
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class Disassembler
{
public:
    /**
     * \brief Default constructor initialising the class
     * \param[in] binary The binary file that will be disassembled
     */
    explicit Disassembler(const char* binary);

    /**
     * \brief Sets the file that will be initialized
     *
     * \note This method will destroy previously performed analyses and requires
     * another call to analyse()
     *
     * \param[in] binary The binary file that will be disassembled
     */
    void set_binary(const char* binary);

    /**
     * \brief Default destructor
     */
    virtual ~Disassembler() = default;

    /**
     * \brief Starts the analysis and populates the fields.
     *
     * This method must be implemented by subclasses that will call the binary
     * analysis and disassembly specifically for each disassembler, and will
     * populate the following fields:
     * <ul><li> Disassembler::exec_arch the architecture of the executable </li>
     * <li> Disassembler::function_names a Function class representing the name
     * and the offset of every function starting point </li> <li>
     * Disassembler::function_bodies for each function, an array of  Statement
     * class representing every opcode of the function. These should be in Intel
     * syntax, but this is not enforced.</li></ul>
     */
    virtual void analyse() = 0;

    /**
     * \brief Returns the architecture of the analysed file.
     *
     * Note that this enum does not account for bit size, i.e. X86 and X86_64
     * will be merged into a single X86 architecture.
     *
     * \warning This method requires the analysis to be performed: if this is
     * not the case, Architecture::UNKNOWN will be returned
     *
     * \return An enum representing the architecture family of the analysed file
     */
    Architecture get_arch() const;

    /**
     * \brief Returns the function names of the analyzed executable
     *
     * Most of the names will be generated, and if possible, will not contain
     * syscalls. However, this is not guaranteed, as it depends from the various
     * implementations of the analyse() function.
     *
     * \return A set containig the function names
     */
    std::set<Function> get_function_names() const;

    /**
     * \brief Returns a function in form of a string
     *
     * This function returns a single function in form of a string. The string
     * is formatted as following:
     * <ul><li>First line is the name of the function</li>
     * <li>Every other line is composed by
     * <ol><li> Offset in hexadecimal </li>
     * <li> Space </li>
     * <li> String representing the opcode and the various deps </li>
     * <li> Line Feed </li></ol></li>
     *
     * The assembly syntax is not enforced and depends on the implementation of
     * the analyse() function.
     *
     * \param[in] name The name of the function that will be converted as string
     * \return The function represented as string. The first line will be the
     * function name. If the function has not been disassembled, empty line is
     * returned
     */
    std::string get_function_as_string(const std::string& name) const;

protected:
    /**
     * \brief The binary that is being analysed
     */
    std::string binary;

    /**
     * \brief Architecture of the analysed binary
     *
     * Architecture::UNKNOWN if the analysis has not been performed
     */
    Architecture exec_arch;

    /**
     * \brief List of functions of the analysed binary
     *
     * Empty set if the analysis has not been performed
     */
    std::set<Function> function_names;

    /**
     * \brief Hash map containing the function bodies
     *
     * This map will contain pairs <function name, array of stmts> used to
     * define a function body
     */
    std::unordered_map<std::string, std::vector<Statement>> function_bodies;

    /**
     * \brief Stream operator override
     *
     * Prints the functions disassembled from the binary, with every statement
     *
     * \param[in] stream The current stream
     * \param[in] disasm An instance of this class
     * \return The current stream
     */
    friend std::ostream& operator<<(std::ostream& stream,
                                    const Disassembler& disasm);
};

#endif
