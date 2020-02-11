#!/usr/bin/env bash

set -eu
shopt -s failglob

if [ $# -lt 2 ]; then
    echo "$0 WRITE_TO LOGS_DIR"
    exit 1
fi

WRITE_TO="$1"
LOGS_DIR="$2"

TEMP_FILE="$(mktemp)"

find "$LOGS_DIR" -name '*.log' -print0 | xargs -0 count-registrations > "$TEMP_FILE"

gmv "$TEMP_FILE" "$WRITE_TO"
