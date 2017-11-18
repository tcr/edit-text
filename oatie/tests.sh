#!/bin/bash

set -e
set -x

for file in in/*; do
    # do something on $file
    cat "$file" | ./target/debug/oatie-transform
done
