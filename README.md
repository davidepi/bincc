# BCC

This is the companion code for the paper 
> **Function Clones Detection in Cross-Architectural Binary Code using Structural Analysis**
>
> D. Pizzolotto, K. Inoue
>

## Pre-requisites
In order to run the code:
- rust must be installed in the system. This can be easily done following [these instructions](https://www.rust-lang.org/tools/install).
- `radare2` must be installed in the system and on PATH. This disassembler can be found in most linux package managers and even on homebrew on macOS. Alternatively it can be downloaded and compiled from source [here](https://github.com/radareorg/radare2).

## Dataset
The manually generated dataset can be found at the 
[following link](https://zenodo.org/record/3865122#.X0XzttP7T_Q).

Not every archive on the previous link is necessary though: in the experiments on the paper we used only the following folder: `amd64-gcc-o0`, `amd64-gcc-o2`, `amd64-gcc-os`, `aarch64-gcc-o0`, `aarch64-gcc-o2`, `aarch64-gcc-os`.

## Compilation
Compilation can be done with the following command
```bash
cargo build --release
```
The compiled executables will be in the folder `target/release`

## Usage
After compilation, two executables will be present in the `target/release` folder: `rq1` and `rq2`.
`rq1` was used to perform Research Questions #1 and #4, while `rq2` was used for Research Questions #2 and #3.

`rq1` is used to measure the high level reconstruction accuracy/time of this sofware.
`rq2` can be used to check multiple binaries for function-level clones (reported as function names).

The input order can be found by invoking each program with `--help`

