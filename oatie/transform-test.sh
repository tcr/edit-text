#!/bin/bash

set -e
cargo build --release

set +e

for file in in/*; do
    test -f "${file}" || continue
    echo
    echo $file
    cat "$file" | RUST_BACKTRACE=1 ../target/release/oatie-transform > /dev/null
done
