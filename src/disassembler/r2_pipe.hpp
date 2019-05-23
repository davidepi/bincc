#ifndef __R2_PIPE_HPP__
#define __R2_PIPE_HPP__

#include <fcntl.h>
#include <string>

/**
 * \brief Class used to interface with the `radare2` program CLI.
 *
 * As written in the official documentation, "using the native API is more
 * complex and slower than just using raw command strings and parsing the
 * output", hence the reason of this class.
 *
 * This class assumes that radare2 is installed on the system.
 * The nominal usage of this class consist of:
 * <ol>
 * <li>Construction</li>
 * <li>Eventual setup of the executable with the set_executable() method</li>
 * <li>Eventual setup of the analyzed file with set_analyzed_file() method
 * (if not done in the construction)</li>
 * <li>Process spawning with the open() method</li>
 * <li>Eventual command issuing with the exec() method</li>
 * <li>Process killing with the close() method</li>
 * </ol>
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class R2Pipe
{
public:

    /**
     * \brief Default constructor
     */
    R2Pipe();

    /**
     * \brief Delete copy costructor
     */
    R2Pipe(const R2Pipe& old) = delete;

    /**
     * \brief Default destructor
     */
    ~R2Pipe();

    /**
     * \brief Delete copy-assignment operator
     */
    R2Pipe& operator=(const R2Pipe&) = delete;

    /**
     * \brief Changes the underlying `radare2` executable used by this class.
     *
     * If the input executable is not `radare2` the actual behaviour is
     * undefined. If there is a file currently open, this method does nothing
     * until the next analysed file.
     *
     * \param[in] executable A string pointing to the radare2 executable
     *
     * \warning This method does not search inside the PATH, so provide an
     * absolute path to the executable
     *
     * \return false if the given string is not an executable, true otherwise
     * file
     */
    bool set_executable(const char* r2exe);

    /**
     * \brief Returns the name of the `radare2` executable
     * \return The name of the executable
     */
    const char* get_executable() const;

    /**
     * \brief Sets the file that will be analyzed
     *
     * In case another file is being analyzed, this function just returns false.
     *
     * \param[in] binary A String pointing to the binary file that will be
     * analyzed
     * \return false in case the file does not exists, it can not be read or
     * another file is still open, true otherwise
     */
    bool set_analyzed_file(const char* binary);

    /**
     * \brief Returns the name of the analyzed binary file
     * \return The name of the analyzed binary
     */
    const char* get_analyzed_file() const;

    /**
     * \brief Starts analysing the binary
     *
     * Starts analysing the file by spawning the radare2 process with the given
     * binary file.
     * In case another instance is already open (for this particular object) the
     * call will fail and return false. This happens also in case the analyzed
     * file has not been set.
     *
     * \return true if the process was spawned successfully, false otherwise
     */
    bool open();

    /**
     * \brief Run a radare2 command and returns the response as string
     *
     * This is interpreted as a raw command: no JSON or strange parsing involed.
     * To the command is appended a \n and it is passed as is to the r2 process.
     * In the same way the answer is returned as is in form of string.
     *
     * \param[in] command a string representing the command that will be issued
     * to the radare2 process (null-terminated, no \n at the end)
     * \return The answer of the radare2 process
     */
    std::string exec(const char* command) const;

    /**
     * \brief Terminates the radare process
     *
     * Terminates the underlying process analysing the binary and frees up the
     * memory
     * \return
     */
    void close();

private:

    //true if the r2 process is active
    bool is_open;

    //r2 executable
    const char* executable;

    //analyzed file
    const char* analyzed;

    //pipe file descriptor, this=>r2
    int pipe_out[2];

    //pipe file descriptor, r2=>this
    int pipe_in[2];

    //pid of the r2 process
    pid_t process;
};


#endif
