#ifndef __R2_FUNC_HPP__
#define __R2_FUNC_HPP__

#include "r2_response.hpp"
#include "r2_stmt.hpp"
#include <vector>

/**
 * \brief Type of functions
 *
 * These types are used by the R2Func class, based on the function types
 * returned by radare2
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
enum FunctionT
{
    /// Normal function
    FCN,
    /// System call
    SYM,
    /// dafuq is this
    LOC,
    /// no idea also for this one
    INT
};

/**
 * \brief Class providing information about a single function
 *
 * This class can be used to parse the result of the `aflj` call sent to
 * radare2. The JSON resulting from that call will be an array of object where
 * each object contains information about functions. Each one of those objects
 * can be used to create one instance of this class.
 *
 * \author davidepi &lt;davidepi&#64;ist.osaka-u.ac.jp&gt;
 */
class R2Func : public R2Response
{
public:
    /**
     * \brief Default constructor
     *
     * Initialize the name as empty string, the offset as 0 and the type as
     * FunctionT::FCN
     */
    R2Func();

    /**
     * \brief Default destructor
     */
    ~R2Func() = default;

    /**
     * \brief Parse the string retrieved by the radare2 process
     *
     * This method populates this class by parsing the string retrieved by
     * issuing the `aflj` command to r2. Attempting to parse any other JSON or
     * strings will fail
     *
     * \param[in] json_string The JSON string that will be parsed
     * \return true if the string was valid and this class has been
     * populated, false otherwise
     */
    bool from_JSON(const std::string& json_string) override;

    /**
     * \brief Getter for the offset
     *
     * \return an integer representing the offset of the function from the
     * beginning of the binary file
     */
    int get_offset() const;

    /**
     * \brief Getter for the name
     *
     * \return an automatically generated name for the function. This is quite
     * reliable for syscalls and completely generated (consisting usually of
     * the offset in hex) for normal functions
     */
    const std::string& get_name() const;

    /**
     * \brief Getter for the function type
     *
     * @return a TypeFunc enum representing the function type, as syscall,
     * subroutine or function
     */
    FunctionT get_type() const;

    /**
     * \brief Add an instruction to the body of this function
     *
     * \param[in] stmt The instruction that will be added
     */
    void add_instruction(const R2Stmt& stmt);

    /**
     * \brief Getter for the body of this function
     *
     * \return a vector containing the instructions of this function
     */
    const std::vector<R2Stmt>& get_body() const;

private:
    // offset of the function in the binary
    int offset;

    // name of the function (or generated name)
    std::string name;

    // type of the function
    FunctionT type;

    // body of the function
    std::vector<R2Stmt> body;
};

#endif
