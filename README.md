# BinCC

This is the companion code for the paper 
> BinCC: Scalable Function Similarity Detection in Multiple Cross-Architectural Binaries
>
> D. Pizzolotto, K. Inoue
>

## Pre-requisites
Dependencies required to run the code:
- [Rust](https://www.rust-lang.org/tools/install).
- [radare2](https://github.com/radareorg/radare2). This software must be in the PATH.

## Compiling
Compilation can be done with the following command
```bash
cargo build --release
```
The compiled executable `bincc` will be in the folder `target/release`

Please run `cargo test -q` to ensure the program is working correctly. No test should fail.

## Usage
Running `bincc --help` should list a verbose help with the various configuration settings that can be used.

For a quick usage, `bincc <binary1> <binary2> [<binary3> ...]` should list the binary clones using the default parameters.

## Experiments and Replication

The experimental results provided in the paper can be found in a folder called `experiments` in the experiments branch of this repository. 
Follow the README contained in that folder to replicate the results provided in the paper.
