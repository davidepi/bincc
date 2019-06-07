//
// Created by davide on 6/7/19.
//

#ifndef __BAP_DISASSEMBLER_HPP__
#define __BAP_DISASSEMBLER_HPP__

#include "disassembler/disassembler.hpp"

class DisassemblerBAP : public Disassembler
{
public:
    /**
     * \brief Default constructor
     * \param[in] binary Path to the binary file that will be decompiled
     */
    explicit DisassemblerBAP(const char* binary);

    /**
     * \brief Default destructor
     */
    ~DisassemblerBAP() override = default;

    void analyse() override;
};

#endif //__BAP_DISASSEMBLER_HPP__
