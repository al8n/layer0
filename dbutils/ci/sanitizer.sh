#!/bin/bash

set -ex

export ASAN_OPTIONS="detect_odr_violation=0 detect_leaks=0"

# Run address sanitizer
cargo clean
RUSTFLAGS="-Z sanitizer=address" \
cargo test -p dbutils

# Run leak sanitizer
cargo clean
RUSTFLAGS="-Z sanitizer=leak" \
cargo test -p dbutils

cargo clean
RUSTFLAGS="--cfg all_tests -Zsanitizer=memory -Zsanitizer-memory-track-origins" \
RUSTDOCFLAGS="-Zsanitizer=memory -Zsanitizer-memory-track-origins" \
cargo test -Zbuild-std --release --tests --target x86_64-unknown-linux-gnu -p dbutils
