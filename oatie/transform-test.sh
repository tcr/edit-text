#!/bin/bash

set -e
set -x

cargo +nightly build

for file in in/*; do
    # do something on $file
    cat "$file" | ./target/debug/oatie-transform
done
