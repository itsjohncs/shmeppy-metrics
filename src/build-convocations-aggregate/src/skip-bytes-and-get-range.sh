#!/usr/bin/env bash

# This script takes in two args: N and FILE. First it uses tail to skip N bytes
# of FILE and then pipes the remaining bytes (which will start at the (N+1)th
# byte) into get-range (who will then print to stdout.

set -eu
shopt -s failglob

if ! command -v get-range > /dev/null; then
	echo "Cannot find get-range. Ensure build directory is in PATH." >&2
	exit 1
fi

if [ $# -ne 2 ]; then
    echo "$0 N FILE" >&2
    exit 1
fi

N="$1"
FILE="$2"

# This is GNU's tail because my system's tail is INCREDIBLY slower. I timed it
# vs cat on shmeppy-0's big log file and I never had the patience to actually
# let tail finish because it was taking over a minute (whereas cat completes
# in a quarter of a second). Dissapointing. gtail is still slower than cat by
# about 10 times, but it's still acceptably fast.
gtail -c "+$N" "$FILE" | get-range
