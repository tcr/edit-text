#!/bin/bash

# Adds a header to indicate when we are building the script for the first time
# TODO this would be nice to merge upstream

GREY='\033[1;30m'
NC='\033[0m' # No Color
HEADER="${GREY}Building cargo script... (this will be cached afterward)${NC} \r\n"

temp_file=$(mktemp)

cargo script --debug --build-only $1 2>&1 |
    perl -pe "s/^(.)/$HEADER\1/g if $. == 1; print STDERR \"$.\n\"" 2>$temp_file
BUILD_STATUS=${PIPESTATUS[0]}
HEADER_STATUS=$(cat $temp_file | wc -l)

if [[ $BUILD_STATUS != 0 ]]; then
    exit $BUILD_STATUS
else
    if [[ $HEADER_STATUS -gt 0 ]]; then
        echo ''
        echo -e "${GREY}Running cargo script...${NC}"
    fi
    exec cargo script --debug $1 -- ${@:2}
fi
