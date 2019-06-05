//
// Created by davide on 6/5/19.
//

#ifndef __R2_DISASSEMBLER_HPP__
#define __R2_DISASSEMBLER_HPP__

#include "disassembler/disassembler.hpp"
#include "r2_info.hpp"
#include "r2_pipe.hpp"

/**
 * \brief Disassembler using the `radare2` disassembler
 *
 * This class implements a Disassembler service by means of the `radare2`
 * disassembler.
 */
class DisassemblerR2 : public Disassembler
{
public:
    /**
     * \brief Default constructor
     * \param[in] binary Path to the binary file that will be decompiled
     */
    explicit DisassemblerR2(const char* binary);

    /**
     * \brief Default destructor
     */
    ~DisassemblerR2() override = default;

    /**
     * \brief Performs the analysis
     *
     * Performs the actual analysis using the radare2 disassembler and populates
     * the necessary fields of the superclass
     */
    void analyze() override;

private:
    /**
     * \brief Utility class used to interface with radare2
     */
    R2Pipe r2;
};

#endif
