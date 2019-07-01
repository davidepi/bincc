//
// Created by davide on 6/26/19.
//

#ifndef __ABSTRACT_BLOCK_HPP__
#define __ABSTRACT_BLOCK_HPP__

#include <vector>

enum BlockType
{
    BASIC = 0,
    SELF_WHILE,
};

class AbstractBlock
{
public:
    AbstractBlock(int bb_id);
    ~AbstractBlock() = default;

    const std::vector<int>& get_bbs();
    const AbstractBlock* get_next() const;
    void set_next(AbstractBlock* nxt);
    const AbstractBlock* get_cond() const;
    void set_cond(AbstractBlock* cnd);
    int get_edges_inn() const;
    int get_edges_out() const;
    BlockType get_type() const;

private:
    std::vector<int> blocks;
    int edges_inn;
    int edges_out;
    BlockType type;
    AbstractBlock* next;
    AbstractBlock* cond;
};

#endif //__ABSTRACT_BLOCK_HPP__
