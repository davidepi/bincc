#ifndef __R2PIPE_HPP__
#define __R2PIPE_HPP__


#include <fcntl.h>
#include <string>

class R2Pipe
{
public:
    R2Pipe();
    R2Pipe(const R2Pipe& old) = delete;
    ~R2Pipe();

    R2Pipe& operator=(const R2Pipe&) = delete;
    bool set_executable(const char* r2exe);
    const char* get_executable()const;
    bool set_analyzed_file(const char* binary);
    const char* get_analyzed_file()const;
    bool open();
    void exec(const char* command, std::string* res)const;
    bool close();
private:
    bool is_open;
    const char* executable;
    const char* analyzed;
    static char* folder;
    //fifo where r2 reads from
    char* r2_read;
    //fifo where r2 writes to
    char* r2_write;
    int in;
    int out;
    pid_t process;
};


#endif
