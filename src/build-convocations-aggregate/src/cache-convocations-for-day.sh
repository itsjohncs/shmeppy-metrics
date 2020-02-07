#!/usr/bin/env bash

set -eu
shopt -s failglob

if [ $# -ne 6 ]; then
    echo "$0 BUILD_DIR RAW_LOGS_DIR DAY_BEFORE DAY DAY_AFTER OUTPUT_FILE"
    exit 1
fi

BUILD_DIR="$1"
RAW_LOGS_DIR="$2"
DAY_BEFORE="$3"
DAY="$4"
DAY_AFTER="$5"
OUTPUT_FILE="$6"

TEMP_FILE="$(mktemp)"

"$BUILD_DIR/get-logs-within" \
    "$DAY_BEFORE" "$DAY" "$DAY_AFTER" -- "$RAW_LOGS_DIR"/*.log |
    "$BUILD_DIR/get-convocations-within" "$DAY" > "$TEMP_FILE"

# mv isn't atomic on mac os x, as (per its man page) it doesn't necessarily
# use rename() even when on the same FS, which seems nonsensical. So I'm using
# GNU's mv here, which I think should certainly use rename() if I'm moving on
# the same FS. Unfortunately, even rename() not being atomic on all but the
# most recent versions of Mac OS X was a known bug, and I don't know that I
# trust atomocity of rename() on my machine. So... this write & move pattern
# is only increasing the safety of this script, the risk of non-atomocity is
# kinda high. But it's good enough because if I write a corrupt output file
# my scripts will error trying to parse it and I can just regenerate it.
gmv "$TEMP_FILE" "$OUTPUT_FILE"
