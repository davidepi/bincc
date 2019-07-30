//
// Created by davide on 7/25/19.
//
#include "analysis/cfs.hpp"
#include "analysis/comparison.hpp"
#include <gtest/gtest.h>

// TEST(Comparison, failed_cfs)
//{
//    uint32_t a;
//    uint32_t b;
//    Comparison cmp((ControlFlowStructure()), ControlFlowStructure());
//    a = 0;
//    b = 0;
//    EXPECT_FALSE(cmp.cloned(&a, &b));
//    EXPECT_EQ(a, UINT32_MAX);
//    EXPECT_EQ(b, UINT32_MAX);
//    ControlFlowGraph cfg(2);
//    ControlFlowStructure cfs;
//    ASSERT_TRUE(cfs.build(cfg));
//    a = 0;
//    b = 0;
//    cmp = Comparison((ControlFlowStructure()), cfs);
//    EXPECT_FALSE(cmp.cloned(&a, &b));
//    EXPECT_EQ(a, UINT32_MAX);
//    EXPECT_EQ(b, UINT32_MAX);
//    a = 0;
//    b = 0;
//    cmp = Comparison(cfs, ControlFlowStructure());
//    EXPECT_FALSE(cmp.cloned(&a, &b));
//    EXPECT_EQ(a, UINT32_MAX);
//    EXPECT_EQ(b, UINT32_MAX);
//}
//
// TEST(Comparison, same_cfs)
//{
//    uint32_t a;
//    uint32_t b;
//    ControlFlowGraph cfg(11);
//    cfg.set_next(3, 3);
//    cfg.set_conditional(2, 4);
//    cfg.set_conditional(3, 5);
//    cfg.set_next(5, 10);
//    cfg.set_conditional(5, 1);
//    cfg.set_conditional(0, 6);
//    cfg.set_next(7, 6);
//    cfg.set_conditional(6, 8);
//    cfg.set_conditional(8, 10);
//    cfg.finalize();
//    ControlFlowStructure cfs;
//    ASSERT_TRUE(cfs.build(cfg));
//    Comparison cmp(cfs, cfs);
//    EXPECT_TRUE(cmp.cloned(&a, &b));
//}
