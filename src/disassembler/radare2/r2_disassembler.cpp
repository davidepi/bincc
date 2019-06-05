//
// Created by davide on 6/5/19.
//

#include "r2_disassembler.hpp"

DisassemblerR2::DisassemblerR2(const char* binary) : Disassembler(binary)
{
    bool res = r2.set_analyzed_file(binary);
    r2.set_executable(RADARE2_PATH);
    if(!res)
    {
        fprintf(stderr, "Could not perform disassembly.\n");
        exit(EXIT_FAILURE);
    }
}

void DisassemblerR2::analyze()
{
    bool res = r2.open();
    if(res)
    {
        std::string json;
        R2Info info;
        info.from_JSON(r2.exec("ij"));
        exec_arch = info.get_arch();
        r2.close();
    }
}

// DisassemblerR2::DisassemblerR2(const char* executable, const char* binary)
//    : analysis_done(false)
//{
//    bool res = r2.set_executable(executable);
//    res &= r2.set_analyzed_file(binary);
//    res &= r2.open();
//    if(!res)
//    {
//        fprintf(stderr, "Could not perform disassembly.\n");
//        exit(EXIT_FAILURE);
//    }
//}
//
// R2Info DisassemblerR2::executable_info() const
//{
//    R2Info retval;
//    retval.from_JSON(r2.exec("ij"));
//    return retval;
//}
