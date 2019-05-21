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

    const char* get_executable() const;

    bool set_analyzed_file(const char* binary);

    const char* get_analyzed_file() const;

    bool open();

    void exec(const char* command, std::string* res) const;

    bool close();

private:
    //true if the r2 process is active
    bool is_open;
    //r2 executable
    const char* executable;
    //analyzed file
    const char* analyzed;
    //fifo where r2 reads from
    char* r2_read;
    //fifo where r2 writes to
    char* r2_write;
    //file descriptor for the channel r2=>this
    int in;
    //file descriptor for the channel this=>r2
    int out;
    //pid of the r2 process
    pid_t process;

    //folder where the r2 pipes will be opened
    static char* folder;
    //instances of this class, when 0 the aforementioned folder will be deleted
    static int instances;
    //lock to preserve atomicity of the instances
    pthread_spinlock_t lock;
};


#endif
