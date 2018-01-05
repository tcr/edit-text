#!/bin/bash

# set -e

cargo build

for file in in/*; do
    # do something on $file
    echo $file
    cat "$file" | RUST_BACKTRACE=1 ../target/debug/oatie-transform > /dev/null
done