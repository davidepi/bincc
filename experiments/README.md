# Replication

This folder contains all the scripts we used to produce the experimental results shown in the paper.

In order to run them, please first install the dependencies listed in `requirements.txt` and build the software contained in the root of this repository using, in this branch, using `cargo build -r`.
This will produce two additional executables: `rq1` and `rq4`. The source for these executables can be found in [rq1.rs](src/rq1.rs) and [rq4.rs](src/rq4.rs).

## RQ1

Research Question 1 requires the dataset available at [the following address](https://zenodo.org/record/3865122#.X0XzttP7T_Q).
In particular, the following archives are required: `amd64-gcc-o0`, `amd64-gcc-o2`, `amd64-gcc-os`, `aarch64-gcc-o0`, `aarch64-gcc-o2`, `aarch64-gcc-os`.
Uncompress the archives anywhere, but keep the folder with the same name of the archive (excluding the extension).

Experiments were run with the following commands:
```bash
./rq1 -t 3600 --prefix aarch64-gcc- aarch64-gcc-o0/* aarch64-gcc-o2/* aarch64-gcc-os/* <output_dir>
./rq1 -t 3600 --prefix amd64-gcc- amd64-gcc-o0/* amd64-gcc-o2/* amd64-gcc-os/* <output_dir>
```
This experiment will take a lot of time (~15h). After completition, the files `amd64-gcc-times.csv`, `amd64-gcc-funcs.csv`, `aarch64-gcc-times.csv` and `aarch64-gcc-funcs.csv` can be found inside `<output_dir>`. Put everything in the folder containing this README.

Our results for these experiments can be found at [this address](https://sel.ist.osaka-u.ac.jp/people/davidepi/bincc/bincc_rq1_results.tar.xz).


Running `python rq1.py` will produce Figure 11, Figure 12 and Figure 13 of the paper.

## RQ2

Research Question 2 requires the [coreutils](https://www.gnu.org/software/coreutils/) package, tag `v.9.1` compiled with GCC in both `x86_64` and `aarch64`. The binaries we used for our experiments can be found at [this address](https://sel.ist.osaka-u.ac.jp/people/davidepi/bincc/coreutils.tar.xz).

Several commands are required for this RQ, and they are provided in the script [run_rq2.sh](https://github.com/davidepi/bincc/blob/experiments/experiments/run_rq2.sh). Set the `AMD64` and `AARCH64` environment variables to the folder containing the coretuils amd64 and aarch64 binaries respectively.
```bash
AMD64="coreutils/amd64" AARCH64="coreutils/aarch64" ./run_rq2.sh
```

The results of our experiments can be found at [the following address](https://sel.ist.osaka-u.ac.jp/people/davidepi/bincc/bincc_rq2_results.tar.xz).

Then, in order to get our tables results, the following commands is needed to generate the ground truth for both `x86_64` and `aarch64`:
```bash
python generate_ground_truth.py
```
Then, the `calculate_metrics.py` file can be used to extract True Positives, False Positives and False Negatives from the experimental results.
Its usage is the following:
```bash
python calculate_metrics.py ground_truth_same.csv results_structural_same_2.csv 2
python calculate_metrics.py ground_truth_cross.csv results_structural_cross_2.csv 2
```
The first parameter after the script name should be either `ground_truth_same.csv` or `ground_truth_cross.csv`, obtained by running the `generate_ground_truth.py`. The second, after the script name, is the file containing the results. The last parameter is the number corresponding to the structural threshold of the experiment, or `0` in case of semantic-only analysis.

Finally, the folder [comparison](https://github.com/davidepi/bincc/tree/experiments/experiments/comparison) contains the results of ours comparison with other state-of-the-art tools. The script [`compare.py`](https://github.com/davidepi/bincc/blob/experiments/experiments/comparison/compare.py) can be run to calculate the results shows in the paper on Table 7, using the `AMD64` environment variable as before.


## RQ3

Research Question 3 was a manual analysis and every result has been gathered by manually checking and disassembling the results reported by the `bincc` tool.
For this reason, nothing is reported here.

## RQ4

Research Question 4 requires a subset of the RQ1 dataset. Only `amd64-gcc-o2` is required. The archive should be uncompressed in a folder keeping the original name.

Experiments were run with the following commands:
```bash
./rq1 -t 3600 --prefix amd64-gcc- amd64-gcc-o2/* <output_dir>
./rq4 -t 3600 amd64-gcc-o2/* > <output_dir>/amd64-gcc-detailed-times.csv
```
After completion the files `amd64-gcc-times.csv`, `amd64-gcc-funcs.csv` and `amd64-gcc-detailed-times.csv` should be available. Put everything in the folder containing this README.

Our results for these experiments can be found at [this address](https://sel.ist.osaka-u.ac.jp/people/davidepi/bincc/bincc_rq4_results.tar.xz).

Running `python rq4.py`  will produce Figure 14, Figure 15 and Figure 16 of the paper.
