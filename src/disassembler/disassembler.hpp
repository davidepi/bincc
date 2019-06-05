#ifndef __DISASSEMBLER_HPP__
#define __DISASSEMBLER_HPP__

#include "architecture.hpp"
#include <set>
#include <string>

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
     * populate the various protected fields (except for the \p binary one)
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

    std::set<std::string> get_function_names() const;

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
    std::set<std::string> function_names;
};

#endif
