#include "r2_pipe.hpp"
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <signal.h>
#include <sstream>
#include <sys/stat.h>
#include <unistd.h>

#define READ_END 0
#define WRITE_END 1

R2Pipe::R2Pipe() : is_open(false), analyzed(nullptr), process(0)
{
    executable = strdup("/usr/bin/r2");
}

R2Pipe::~R2Pipe()
{
    free((void*)executable);
    if(analyzed != nullptr)
    {
        free((void*)analyzed);
    }
    // if there is a child process still alive kill it with fire
    if(process != 0 && kill(process, 0) != -1)
    {
        kill(process, SIGTERM);
    }
}

const char* R2Pipe::get_executable() const
{
    return executable;
}

bool R2Pipe::set_executable(const char* r2exe)
{
    bool retval;
    // assert existence of executable
    if(access(r2exe, X_OK) == -1)
    {
        fprintf(stderr,
                "The radare2 executable %s does not exist or has "
                "wrong permissions\n",
                r2exe);
        retval = false;
    }
    else
    {
        free((void*)executable);
        executable = strdup(r2exe);
        retval = true;
    }
    return retval;
}

const char* R2Pipe::get_analyzed_file() const
{
    return analyzed;
}

bool R2Pipe::set_analyzed_file(const char* binary)
{
    bool retval;
    if(!is_open)
    {
        // assert existence of binary
        if(access(binary, R_OK) == -1)
        {
            fprintf(stderr,
                    "The binary to be analyzed %s does not exist or "
                    "has wrong permissions",
                    binary);
            retval = false;
        }
        else
        {
            if(analyzed != nullptr)
            {
                free((void*)analyzed);
            }
            analyzed = strdup(binary);
            retval = true;
        }
        return retval;
    }

    return false;
}

bool R2Pipe::open()
{
    if(analyzed == nullptr || is_open)
    {
        return false;
    }

    // fork
    if(pipe(pipe_out) == -1 || pipe(pipe_in) == -1)
    {
        perror("Communication channel error: ");
    }
    else
    {
        // before forking check that child does not already exists (failed to
        // exit)
        if(process != 0 && kill(process, 0) != -1)
        {
            kill(process, SIGTERM);
        }
        process = fork();
        if(process == -1)
        {
            perror("Error while creating disassembler: ");
        }
        else if(process == 0)
        {
            ::close(pipe_out[WRITE_END]);
            ::close(pipe_in[READ_END]);

            // child process is radare
            dup2(pipe_out[READ_END], STDIN_FILENO);
            dup2(pipe_in[WRITE_END], STDOUT_FILENO);
            dup2(pipe_in[WRITE_END], STDERR_FILENO);

            ::close(pipe_in[WRITE_END]);
            ::close(pipe_out[READ_END]);

            execl(executable, executable, "-q0", analyzed, NULL);
        }
        else
        {
            ::close(pipe_out[READ_END]);
            ::close(pipe_in[WRITE_END]);

            // read the /0 produced by r2 upon opening
            char buf;
            while(read(pipe_in[READ_END], &buf, 1) > 0)
            {
                if(buf == 0x0)
                {
                    break;
                }
            }
            is_open = true;
        }
    }
    return is_open;
}

std::string R2Pipe::exec(const char* command) const
{
    if(is_open)
    {
        std::stringstream stream;
        int len = strlen(command);
        write(pipe_out[WRITE_END], command, len);
        write(pipe_out[WRITE_END], "\n", 1);
        // read answer
        char buf;
        while(read(pipe_in[READ_END], &buf, 1) > 0)
        {
            if(buf == 0x0)
            {
                break;
            }

            stream << buf;
        }
        return stream.str();
    }
    return std::string("");
}

void R2Pipe::close()
{
    if(is_open)
    {
        exec("q");
        ::close(pipe_out[WRITE_END]);
        ::close(pipe_in[READ_END]);
    }
    analyzed = nullptr;
}
