#!/usr/bin/env bash

set -eu
shopt -s failglob

if [ $# -lt 3 ]; then
    echo "$0 RAW_LOGS_DIR OUTPUT_FILE DAY [SURROUNDING_DAY [SURROUNDING_DAY...]]"
    exit 1
fi

RAW_LOGS_DIR="$1"
OUTPUT_FILE="$2"
DAYS=("${@:3}")

TEMP_FILE="$(mktemp)"

get-logs-within "${DAYS[@]}" -- "$RAW_LOGS_DIR"/*.log |
    get-convocations-within "${DAYS[0]}" > "$TEMP_FILE"

# mv isn't atomic on mac os x, as (per its man page) it doesn't necessarily
# use rename() even when on the same FS, which seems nonsensical. So I'm using
# GNU's mv here, which I think should certainly use rename() if I'm moving on
# the same FS. Unfortunately, even rename() not being atomic on all but the
# most recent versions of Mac OS X was a known bug, and I don't know that I
# trust atomocity of rename() on my machine. So... this write & move pattern
# is only increasing the safety of this script, the risk of non-atomocity is
# kinda high. But it's good enough because if I write a corrupt output file
# my scripts will error trying to parse it and I can eaily regenerate it.
# Pros and cons of using the FS as a database...
gmv "$TEMP_FILE" "$OUTPUT_FILE"
