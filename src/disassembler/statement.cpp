#include <utility>

#include "function.hpp"
#include "statement.hpp"
#include <nlohmann/json.hpp>

using Json = nlohmann::json;

Statement::Statement() : offset(0x0)
{
}

int Statement::get_offset() const
{
    return offset;
}

const std::string& Statement::get_opcode() const
{
    return opcode;
}
Statement::Statement(uint64_t offset, std::string opcode)
    : offset(offset), opcode(std::move(opcode))
{
}
