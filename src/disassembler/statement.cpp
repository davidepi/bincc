#include "statement.hpp"
#include <algorithm>

Statement::Statement() : offset(0x0), args_at(0)
{
}

uint64_t Statement::get_offset() const
{
  return offset;
}
std::string Statement::get_command() const
{
  return instruction;
}

std::string Statement::get_mnemonic() const
{
  return instruction.substr(0, args_at);
}

std::string Statement::get_args() const
{
  if(args_at >= instruction.length())
  {
    return std::string();
  }

  return instruction.substr(args_at + 1, std::string::npos);
}

Statement::Statement(uint64_t offset, std::string opcode)
    : offset(offset), instruction(std::move(opcode))
{
  constexpr const unsigned int NPOS = (unsigned int)std::string::npos;
  // everything lowercase. I'm sorry, little one
  std::transform(instruction.begin(), instruction.end(), instruction.begin(),
                 ::tolower);
  args_at = instruction.find_first_of(' ');
  if(args_at == NPOS)
  {
    args_at = instruction.length();
  }
}
