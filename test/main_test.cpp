#include <gtest/gtest.h>

int main(int argc, char** argv)
{
  ::testing::internal::CaptureStderr();
  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
