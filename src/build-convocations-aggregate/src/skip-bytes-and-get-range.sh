#!/usr/bin/env bash

# This script takes in two args (besides the build directory): N and FILE.
# First it uses tail to skip N bytes of FILE and then pipes the remaining bytes
# (which will start at the (N+1)th byte) into get-range (who will then print to
# stdout. It will also check that the Nth byte is a newline and that the last
# byte is also a newline.

set -eu
shopt -s failglob

if [ $# -ne 3 ]; then
    echo "$0 BUILD_DIR N FILE"
    exit 1
fi

BUILD_DIR="$1"
N="$2"
FILE="$3"

# These two checks work together to ensure we never build the cache from a raw
# log file that was only partially downloaded. All log entries in the raw logs
# must be complete.
if [ "$N" -ne "0" ] && [ "$(tail -c "+$N" "$FILE" | head -c 1)" != $'\n' ]; then
	echo "${N}th byte of $FILE is not a newline." >&2
	exit 1
fi

if [ "$(tail -c 1 "$FILE")" != $'\n' ]; then
	echo "$FILE does not end with a newline." >&2
	exit 1
fi

tail -c "+$N" "$FILE" | "$BUILD_DIR/get-range"
