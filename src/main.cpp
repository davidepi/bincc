#include "analysis/analysis.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include "multithreading/synchronized_queue.hpp"
#include "unistd.h"
#include <analysis/cfs.hpp>
#include <iostream>
#include <thread>

static void fatal(const char* message);
static int run(int argc, const char* argv[]);
static void disasm(SynchronizedQueue<Disassembler*>* jobs,
                   SynchronizedQueue<Disassembler*>* done);
int main(int argc, const char* argv[])
{
    printf("%lu\n", sizeof(std::vector<void*>));
    return run(argc, argv);
}

int run(int argc, const char* argv[])
{
    if(argc < 2)
    {
        fatal("Usage: ./analyze binary0 [binary1 ...]");
    }
#ifndef NDEBUG
    unsigned int core_no = 1;
#else
    unsigned int core_no = std::thread::hardware_concurrency();
#endif

    SynchronizedQueue<Disassembler*> disasm_jobs;
    SynchronizedQueue<Disassembler*> disasmed;

    for(int i = 1; i < argc; i++)
    {
        if(access(argv[i], R_OK) == -1)
        {
            fatal("Input file does not exists or is not readable");
        }
        disasm_jobs.push(new DisassemblerR2(argv[i]));
    }

    std::thread* threads = new std::thread[core_no];
    for(unsigned int i = 0; i < core_no; i++)
    {
        threads[i] = std::thread(disasm, &disasm_jobs, &disasmed);
    }
    for(unsigned int i = 0; i < core_no; i++)
    {
        threads[i].join();
    }
    while(!disasmed.empty())
    {
        Disassembler* disasm = disasmed.front();
        delete disasm;
    }
    return 0;
}

static void disasm(SynchronizedQueue<Disassembler*>* jobs,
                   SynchronizedQueue<Disassembler*>* done)
{
    while(!jobs->empty())
    {
        Disassembler* disasm = jobs->front();
        if(disasm != nullptr)
        {
            disasm->analyse();
            std::set<Function> names = disasm->get_function_names();
            std::string binary_no_folder = disasm->get_binary_name();
            // TODO: the job is disasm-bounded for now. When the analysis will
            //       take more time, consider making a new thread that wait on a
            //       condition variable and performs the various analyses
            for(const Function& func : names)
            {
                binary_no_folder = binary_no_folder.substr(
                    binary_no_folder.find_last_of('/') + 1);
                std::string output =
                    binary_no_folder + "." + func.get_name() + ".dot";
                std::cout << disasm->get_binary_name() << " : "
                          << func.get_name() << std::endl;
                Analysis anal(disasm->get_function_body(func.get_name()),
                              disasm->get_arch());
                anal.get_cfg()->to_file(output.c_str());
            }
            done->push(disasm);
        }
        else
        {
            // somebody stole the last job between the empty check and the job
            // retrieval
            return;
        }
    }
}

void fatal(const char* message)
{
    fprintf(stderr, "%s\n", message);
    exit(EXIT_FAILURE);
}
