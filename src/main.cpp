#include "analysis/analysis.hpp"
#include "disassembler/radare2/r2_disassembler.hpp"
#include "multithreading/synchronized_cout.hpp"
#include "multithreading/synchronized_queue.hpp"
#include "unistd.h"
#include <iostream>
#include <thread>

static void fatal(const char* message);
static int run(int argc, const char* argv[]);
static void disasm(SynchronizedQueue<Disassembler*>* jobs,
                   SynchronizedQueue<Disassembler*>* done);
int main(int argc, const char* argv[])
{
    return run(argc, argv);
}
int a(){return 0;};
int b(){return 1;};
int c(){return 2;};

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
    delete[] threads;
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
    std::chrono::steady_clock::time_point start;
    std::chrono::steady_clock::time_point end;
    uint32_t skipped = 0;
    uint32_t success = 0;
    uint32_t failed = 0;
    while(!jobs->empty())
    {
        Disassembler* disasm = jobs->front();
        if(disasm != nullptr)
        {
            start = std::chrono::steady_clock::now();
            disasm->analyse();
            end = std::chrono::steady_clock::now();
            auto elapsed =
                std::chrono::duration_cast<std::chrono::milliseconds>(end -
                                                                      start);
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
                start = std::chrono::steady_clock::now();
                Analysis anal(disasm->get_function_body(func.get_name()),
                              disasm->get_arch());
                end = std::chrono::steady_clock::now();
                if(anal.get_cfg()->nodes_no() < 5)
                {
                    skipped++;
                    continue;
                }
                if(anal.get_cfs()->root() != nullptr)
                {
                    elapsed =
                        std::chrono::duration_cast<std::chrono::milliseconds>(
                            end - start);
                    anal.get_cfs()->to_file(output.c_str(), *anal.get_cfg());
                    success++;
                }
                else
                {
                    failed++;
                }
            }
            sout << binary_no_folder << "," << success << "," << failed << ","
                 << skipped << std::endl;
            done->push(disasm);
        }
        else
        {
            // somebody stole the last job between the empty check and the
            // job retrieval
            return;
        }
    }
}

void fatal(const char* message)
{
    fprintf(stderr, "%s\n", message);
    exit(EXIT_FAILURE);
}
