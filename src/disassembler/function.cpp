#include <utility>

#include "function.hpp"

Function::Function() : offset(0), name("")
{
}

int Function::get_offset() const
{
    return offset;
}

const std::string& Function::get_name() const
{
    return name;
}
Function::Function(uint64_t offset, std::string name)
    : offset(offset), name(std::move(name))
{
}

bool Function::operator<(const Function& second)const
{
    return offset < second.offset;
}
