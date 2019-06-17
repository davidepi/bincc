#include <utility>

//
// Created by davide on 6/12/19.
//

#include "analysis.hpp"
#include <set>
#include <sstream>

Statement Analysis::operator[](int value) const
{
    if(value >= 0 && value < stmt_list.size())
    {
        return stmt_list.at(value);
    }
    return Statement();
}

Analysis::Analysis(const std::vector<Statement>* stmts,
                   std::shared_ptr<Architecture> arch)
    : architecture(std::move(arch))
{
    if(stmts != nullptr)
    {
        stmt_list = *stmts;
        for(const Statement& stmt : stmt_list)
        {
            stmt_sparse.insert({{stmt.get_offset(), &stmt}});
        }
    }
    build_cfg();
}

Analysis::Analysis(const std::string& str, std::shared_ptr<Architecture> arch)
    : architecture(std::move(arch))
{
    std::istringstream iss(str);
    std::string line;
    std::getline(iss, line); // skip first line
    while(std::getline(iss, line))
    {
        size_t pos = line.find_first_of(' ');
        std::string offset_str = line.substr(0, pos);
        std::string opcode = line.substr(pos + 1, std::string::npos);
        uint64_t offset = std::stoll(offset_str, nullptr, 16);
        stmt_list.emplace_back(offset, opcode);
    }
    for(const Statement& stmt : stmt_list)
    {
        stmt_sparse.insert({{stmt.get_offset(), &stmt}});
    }
    build_cfg();
}

const BasicBlock* Analysis::get_cfg() const
{
    return &(cfg[0]);
}

void Analysis::build_cfg()
{
    // contains all the targets of the jumps
    std::set<uint64_t> targets;
    // contains a pair <src,dest> for conditional jumps
    std::unordered_map<uint64_t, uint64_t> conditional_src;
    // contains a pair <src,dest> for unconditional jumps
    std::unordered_map<uint64_t, uint64_t> unconditional_src;
    // last block of the cfg for the function
    std::set<uint64_t> dead_end;

    // find all the jumps and the blocks pointing nowhere

    // If the previous instruction was a conditional jump, the next one is the
    // target if the condition is not true. However, for variable-lenght opcode
    // architectures such as X86 it is harder to look forward than to lookback,
    // hence the reason of this boolean.
    // The initial value if true to target the starting block.
    bool previous_was_jump = true;
    for(const Statement& stmt : stmt_list)
    {
        if(previous_was_jump)
        {
            targets.insert(stmt.get_offset());
            previous_was_jump = false;
        }
        const std::string mnemonic = stmt.get_mnemonic();
        JumpType jmp = architecture->is_jump(mnemonic);
        if(jmp == JumpType::CONDITIONAL)
        {
            uint64_t target = std::stoll(stmt.get_args(), nullptr, 16);
            targets.insert(target);
            conditional_src.insert({{stmt.get_offset(), target}});
            previous_was_jump = true;
        }
        else if(jmp == JumpType::UNCONDITIONAL)
        {
            uint64_t target = std::stoll(stmt.get_args(), nullptr, 16);
            targets.insert(target);
            unconditional_src.insert({{stmt.get_offset(), target}});
        }
        else
        {
            if(architecture->is_return(mnemonic))
            {
                dead_end.insert(stmt.get_offset());
            }
        }
    }

    // create the cfg and concatenate every block
    uint64_t bb_no = targets.size();
    cfg = new BasicBlock[bb_no];
    for(int i = 0; i < bb_no - 1; i++)
    {
        cfg[i].set_id(i);
        cfg[i].set_next(&(cfg[i + 1]));
    }
    cfg[bb_no - 1].set_id(bb_no - 1);

    // maps every target to the block number. Otherwise I need to perform this
    // operation multiple times inside a loop and the complexity grows
    std::unordered_map<uint64_t, int> blocks_id;
    int index = 0;
    for(uint64_t block_beginning : targets)
    {
        blocks_id.insert({{block_beginning, index++}});
    }

    // TODO: The three loops here just perform the same thing and call a
    //       different function at the end. Maybe use functional programming?

    // set the conditional jumps target
    for(std::pair<uint64_t, uint64_t> jmp_src : conditional_src)
    {
        // resolve the current block by finding the next id in the set higher
        // than the current offset, and decreasing the id by 1
        uint64_t next_beginning;
        std::unordered_map<uint64_t, int>::const_iterator next_block;

        // resolve current block
        next_beginning = *targets.upper_bound(jmp_src.first);
        next_block = blocks_id.find(next_beginning);
        int current_id = next_block != blocks_id.end() ?
                             (blocks_id.find(next_beginning)->second) - 1 :
                             targets.size() - 1;
        // resolve target block
        next_beginning = *targets.upper_bound(jmp_src.second);
        next_block = blocks_id.find(next_beginning);
        int target_id = next_block != blocks_id.end() ?
                            (blocks_id.find(next_beginning)->second) - 1 :
                            targets.size() - 1;
        // set the pointer
        cfg[current_id].set_conditional(&(cfg[target_id]));
    }

    // set the conditional jumps target
    for(std::pair<uint64_t, uint64_t> jmp_src : unconditional_src)
    {
        // same procedure as the previous loop
        uint64_t next_beginning;
        std::unordered_map<uint64_t, int>::const_iterator next_block;

        // resolve current block
        next_beginning = *targets.upper_bound(jmp_src.first);
        next_block = blocks_id.find(next_beginning);
        int current_id = next_block != blocks_id.end() ?
                             (blocks_id.find(next_beginning)->second) - 1 :
                             targets.size() - 1;

        // resolve target block
        next_beginning = *targets.upper_bound(jmp_src.second);
        next_block = blocks_id.find(next_beginning);
        int target_id = next_block != blocks_id.end() ?
                            (blocks_id.find(next_beginning)->second) - 1 :
                            targets.size() - 1;
        // set the pointer
        cfg[current_id].set_next(&(cfg[target_id]));
    }

    // set the blocks pointing nowhere. Otherwise they point to the next block
    for(uint64_t ret : dead_end)
    {
        // resolve current block
        uint64_t next_beginning;
        std::unordered_map<uint64_t, int>::const_iterator next_block;

        // resolve current block
        next_beginning = *targets.upper_bound(ret);
        next_block = blocks_id.find(next_beginning);
        int current_id = next_block != blocks_id.end() ?
                             (blocks_id.find(next_beginning)->second) - 1 :
                             targets.size() - 1;

        // mark the block as endblock
        cfg[current_id].set_next(nullptr);
    }
}
