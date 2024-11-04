#!/bin/bash

set -ex

export ASAN_OPTIONS="detect_odr_violation=0 detect_leaks=0"

# Run address sanitizer
RUSTFLAGS="--cfg all_tests -Z sanitizer=address" \
cargo test -p waterark --lib --features sync --no-default-features --target x86_64-unknown-linux-gnu

# Run leak sanitizer
RUSTFLAGS="--cfg all_tests -Z sanitizer=leak" \
cargo test -p waterark --lib --features sync --no-default-features --target x86_64-unknown-linux-gnu

# Run memory sanitizer
RUSTFLAGS="--cfg all_tests -Zsanitizer=memory -Zsanitizer-memory-track-origins" \
RUSTDOCFLAGS="-Zsanitizer=memory -Zsanitizer-memory-track-origins" \
cargo test -Zbuild-std --release --tests --target x86_64-unknown-linux-gnu -p waterark --features sync --no-default-features

# Run thread sanitizer
RUSTFLAGS="--cfg all_tests -Z sanitizer=thread" \
cargo -Zbuild-std test --lib --target x86_64-unknown-linux-gnu -p waterark --features sync --no-default-features

