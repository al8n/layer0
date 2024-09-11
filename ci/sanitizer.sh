#!/bin/bash

set -ex

export ASAN_OPTIONS="detect_odr_violation=0 detect_leaks=0"

# Run address sanitizer with cargo-hack
RUSTFLAGS="-Z sanitizer=address" \
cargo test -p dbutils --all-features

# Run leak sanitizer with cargo-hack
RUSTFLAGS="-Z sanitizer=leak" \
cargo test -p dbutils --all-features

# Run thread sanitizer with cargo-hack
RUSTFLAGS="-Z sanitizer=thread" \
cargo -Zbuild-std test -p dbutils --all-features
