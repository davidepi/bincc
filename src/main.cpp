#include "analysis/analysis.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include "multithreading/synchronized_queue.hpp"
#include "unistd.h"
#include <thread>

static void fatal(const char* message)
{
    fprintf(stderr, "%s\n", message);
    exit(EXIT_FAILURE);
}

int main(int argc, const char* argv[])
{
    if(argc < 2)
    {
        fatal("Usage: ./analyze binary0 [binary1 ...]");
    }

    unsigned int core_no = std::thread::hardware_concurrency();
    fprintf(stdout, "Cores: %d\n", core_no);

    SynchronizedQueue<Disassembler*> disasm_jobs;

    for(int i = 1; i < argc; i++)
    {
        if(access(argv[i], R_OK) == -1)
        {
            fatal("Input file does not exists or is not readable");
        }
        disasm_jobs.push(new DisassemblerR2(argv[i]));
    }

    // TODO: use real multithreading
    while(!disasm_jobs.empty())
    {
        Disassembler* disasm = disasm_jobs.front();
        disasm->analyse();
        std::set<Function> names = disasm->get_function_names();
        for(const Function& func : names)
        {
            std::string output = func.get_name() + ".dot";
            Analysis anal(disasm->get_function_body(func.get_name()),
                          disasm->get_arch());
            const BasicBlock* cfg = anal.get_cfg();
            print_cfg(cfg, output.c_str());
        }
    }
}
