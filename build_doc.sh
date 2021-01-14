#!/bin/bash
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps --all-features $1