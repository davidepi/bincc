//
// Created by davide on 6/12/19.
//

#include "analysis.hpp"
#include <sstream>

Statement Analysis::operator[](int value) const
{
    if(value>=0 && value<stmt_list.size())
    {
        return stmt_list.at(value);
    }
    return Statement();
}

Analysis::Analysis(const std::vector<Statement>* stmts)
{
    if(stmts != nullptr)
    {
        stmt_list = *stmts;
        for(const Statement& stmt : stmt_list)
        {
            stmt_sparse.insert({{stmt.get_offset(), &stmt}});
        }
    }
}

Analysis::Analysis(const std::string& str)
{
    std::istringstream iss(str);
    std::string line;
    std::getline(iss, line); // skip first line
    while(std::getline(iss, line))
    {
        size_t pos = line.find_first_of(' ');
        std::string offset_str = line.substr(0, pos);
        std::string opcode = line.substr(pos + 1, std::string::npos);
        uint64_t offset = std::stoll(offset_str, nullptr, 0);
        stmt_list.emplace_back(offset, opcode);
    }
    for(const Statement& stmt : stmt_list)
    {
        stmt_sparse.insert({{stmt.get_offset(), &stmt}});
    }
}
