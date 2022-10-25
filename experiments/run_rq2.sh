#!/bin/bash

# STRUCTURAL ANALYSIS
./bincc -a same --disable-semantic -m 2 $AMD64/* > results_structural_same_2.csv
./bincc -a same --disable-semantic -m 3 $AMD64/* > results_structural_same_3.csv
./bincc -a same --disable-semantic -m 4 $AMD64/* > results_structural_same_4.csv
./bincc -a same --disable-semantic -m 5 $AMD64/* > results_structural_same_5.csv
./bincc -a same --disable-semantic -m 6 $AMD64/* > results_structural_same_6.csv

./bincc -a cross --disable-semantic -m 2 $AMD64/* $AARCH64/* > results_structural_cross_2.csv
./bincc -a cross --disable-semantic -m 3 $AMD64/* $AARCH64/* > results_structural_cross_3.csv
./bincc -a cross --disable-semantic -m 4 $AMD64/* $AARCH64/* > results_structural_cross_4.csv
./bincc -a cross --disable-semantic -m 5 $AMD64/* $AARCH64/* > results_structural_cross_5.csv
./bincc -a cross --disable-semantic -m 6 $AMD64/* $AARCH64/* > results_structural_cross_6.csv

# SEMANTIC ANALYSIS
./bincc -a same --disable-structural --min-similarity 0.9   $AMD64/* > results_structural_same_900.csv
./bincc -a same --disable-structural --min-similarity 0.95  $AMD64/* > results_structural_same_950.csv
./bincc -a same --disable-structural --min-similarity 0.98  $AMD64/* > results_structural_same_980.csv
./bincc -a same --disable-structural --min-similarity 0.99  $AMD64/* > results_structural_same_990.csv
./bincc -a same --disable-structural --min-similarity 0.999 $AMD64/* > results_structural_same_999.csv

./bincc -a cross --disable-structural --min-similarity 0.9   $AMD64/* $AARCH64/* > results_structural_cross_900.csv
./bincc -a cross --disable-structural --min-similarity 0.95  $AMD64/* $AARCH64/* > results_structural_cross_950.csv
./bincc -a cross --disable-structural --min-similarity 0.98  $AMD64/* $AARCH64/* > results_structural_cross_980.csv
./bincc -a cross --disable-structural --min-similarity 0.99  $AMD64/* $AARCH64/* > results_structural_cross_990.csv
./bincc -a cross --disable-structural --min-similarity 0.999 $AMD64/* $AARCH64/* > results_structural_cross_999.csv

# COMBINED ANALYSIS
./bincc -a same -m 2 --min-similarity 0.98  $AMD64/* > results_combined_same_2_980.csv
./bincc -a same -m 2 --min-similarity 0.99  $AMD64/* > results_combined_same_2_990.csv
./bincc -a same -m 2 --min-similarity 0.999 $AMD64/* > results_combined_same_2_999.csv
./bincc -a same -m 3 --min-similarity 0.98  $AMD64/* > results_combined_same_3_980.csv
./bincc -a same -m 3 --min-similarity 0.99  $AMD64/* > results_combined_same_3_990.csv
./bincc -a same -m 3 --min-similarity 0.999 $AMD64/* > results_combined_same_3_999.csv
./bincc -a same -m 4 --min-similarity 0.98  $AMD64/* > results_combined_same_4_980.csv
./bincc -a same -m 4 --min-similarity 0.99  $AMD64/* > results_combined_same_4_990.csv
./bincc -a same -m 4 --min-similarity 0.999 $AMD64/* > results_combined_same_4_999.csv
./bincc -a same -m 5 --min-similarity 0.98  $AMD64/* > results_combined_same_5_980.csv
./bincc -a same -m 5 --min-similarity 0.99  $AMD64/* > results_combined_same_5_990.csv
./bincc -a same -m 5 --min-similarity 0.999 $AMD64/* > results_combined_same_5_999.csv
./bincc -a same -m 6 --min-similarity 0.98  $AMD64/* > results_combined_same_6_980.csv
./bincc -a same -m 6 --min-similarity 0.99  $AMD64/* > results_combined_same_6_990.csv
./bincc -a same -m 6 --min-similarity 0.999 $AMD64/* > results_combined_same_6_999.csv

./bincc -a cross -m 2 --min-similarity 0.98  $AMD64/* $AARCH64/* > results_combined_cross_2_980.csv
./bincc -a cross -m 2 --min-similarity 0.99  $AMD64/* $AARCH64/* > results_combined_cross_2_990.csv
./bincc -a cross -m 2 --min-similarity 0.999 $AMD64/* $AARCH64/* > results_combined_cross_2_999.csv
./bincc -a cross -m 3 --min-similarity 0.98  $AMD64/* $AARCH64/* > results_combined_cross_3_980.csv
./bincc -a cross -m 3 --min-similarity 0.99  $AMD64/* $AARCH64/* > results_combined_cross_3_990.csv
./bincc -a cross -m 3 --min-similarity 0.999 $AMD64/* $AARCH64/* > results_combined_cross_3_999.csv
./bincc -a cross -m 4 --min-similarity 0.98  $AMD64/* $AARCH64/* > results_combined_cross_4_980.csv
./bincc -a cross -m 4 --min-similarity 0.99  $AMD64/* $AARCH64/* > results_combined_cross_4_990.csv
./bincc -a cross -m 4 --min-similarity 0.999 $AMD64/* $AARCH64/* > results_combined_cross_4_999.csv
./bincc -a cross -m 5 --min-similarity 0.98  $AMD64/* $AARCH64/* > results_combined_cross_5_980.csv
./bincc -a cross -m 5 --min-similarity 0.99  $AMD64/* $AARCH64/* > results_combined_cross_5_990.csv
./bincc -a cross -m 5 --min-similarity 0.999 $AMD64/* $AARCH64/* > results_combined_cross_5_999.csv
./bincc -a cross -m 6 --min-similarity 0.98  $AMD64/* $AARCH64/* > results_combined_cross_6_980.csv
./bincc -a cross -m 6 --min-similarity 0.99  $AMD64/* $AARCH64/* > results_combined_cross_6_990.csv
./bincc -a cross -m 6 --min-similarity 0.999 $AMD64/* $AARCH64/* > results_combined_cross_6_999.csv
