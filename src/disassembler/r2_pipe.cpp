#include "r2_pipe.hpp"
#include <unistd.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sys/stat.h>
#include <sstream>
#include "utility/settings.hpp"

#define FIFO_NAME "r2XXXXXX"

R2Pipe::R2Pipe():analyzed(nullptr), process(), r2_read(nullptr),
                 r2_write(nullptr), is_open(false)
{
    ;
    executable = strdup("/usr/bin/r2");
}

R2Pipe::~R2Pipe()
{
    free((void*)executable);
    if(analyzed != nullptr)
        free((void*)analyzed);
}

const char* R2Pipe::get_executable() const
{
    return executable;
}

bool R2Pipe::set_executable(const char* r2exe)
{
    bool retval;
    //assert existence of executable
    if(access(r2exe, X_OK) == -1)
    {
        fprintf(stderr, "The radare2 executable %s does not exist or has "
                        "wrong permissions", r2exe);
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
    //assert existence of binary
    if(access(binary, R_OK) == -1)
    {
        fprintf(stderr, "The binary to be analyzed %s does not exist or "
                        "has wrong permissions", binary);
        retval = false;
    }
    else
    {
        if(analyzed != nullptr)
            free((void*)analyzed);
        analyzed = strdup(binary);
        retval = true;
    }
    return retval;
}

bool R2Pipe::open()
{
    if(analyzed == nullptr || is_open)
        return false;
    size_t buf_len = Settings::instance().working_folder_len();
    buf_len += strlen(FIFO_NAME)+1;
    r2_write = (char*)malloc(sizeof(char)*buf_len);
    r2_read = (char*)malloc(sizeof(char)*buf_len);
    strcpy(r2_read, Settings::instance().working_folder());
    strcat(r2_read, FIFO_NAME);
    mktemp(r2_read);
    strcpy(r2_write, r2_read);
    strcat(r2_read, "r");
    strcat(r2_write, "w");
    if(mkfifo(r2_read, S_IRUSR | S_IWUSR) == -1)
    {
        //could not create the first FIFO, dealloc
        free(r2_write);
        free(r2_read);
        perror("Could not create the fifo: ");
    }
    else if(mkfifo(r2_write, S_IRUSR | S_IWUSR) == -1)
    {
        //could not create the second FIFO, dealloc and erase the first one
        unlink(r2_read);
        free(r2_write);
        free(r2_read);
        perror("Could not create the fifo: ");
    }
    else
    {
        //fork
        process = fork();
        if(process == -1)
        {
            //fork failed, dealloc and erase FIFOs
            unlink(r2_write);
            unlink(r2_read);
            free(r2_write);
            free(r2_read);
            perror("Error while creating disassembler: ");
        }
        else if(process == 0)
        {
            //child process is radare
            execl(executable, executable,
                  "-q0", analyzed, "<", r2_read, ">", r2_write, NULL);
        }
        else
        {
            //read the /0 produced by r2 upon opening
            in = ::open(r2_write, O_RDONLY);
            out = ::open(r2_read, O_WRONLY);
            char buf;
            while(read(in, &buf, 1) == 1)
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

void R2Pipe::exec(const char* command, std::string* res) const
{
    if(is_open)
    {
        std::stringstream stream;
        int len = strlen(command);
        write(out, command, len);
        write(out, "\n", 1);
        //read answer
        char buf;
        while((buf = read(in, &buf, 1))>0)
        {
            if(buf == 0x0)
                break;
            else
                stream << buf;
        }
        if(res != nullptr)
        {
            *res = stream.str();
        }
    }
}

bool R2Pipe::close()
{
    if(is_open)
    {
        exec("q", nullptr);
        ::close(in);
        ::close(out);
    }
    analyzed = nullptr;
    unlink(r2_write);
    unlink(r2_read);
    free(r2_write);
    free(r2_read);
    return true;
}
