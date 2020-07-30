#!/usr/bin/env bash

set -eu
shopt -s failglob

if [ $# -lt 2 ]; then
    echo "$0 WRITE_INTO LOGS_DIR"
    exit 1
fi

WRITE_INTO="$1"
LOGS_DIR="$2"

TEMP_REGISTRATIONS="$(mktemp)"
TEMP_CONVOCATIONS="$(mktemp)"
TEMP_ACTIVE_USERS="$(mktemp)"

find "$LOGS_DIR" -name '*.log' -print0 |
	xargs -0 pv |
	tee >(count-registrations > "$TEMP_REGISTRATIONS") \
		>(fast-convoker > "$TEMP_CONVOCATIONS") \
		>(active-users > "$TEMP_ACTIVE_USERS") \
		> /dev/null

gmv "$TEMP_REGISTRATIONS" "$WRITE_INTO/registrations.json"
gmv "$TEMP_CONVOCATIONS" "$WRITE_INTO/convocations.json"
gmv "$TEMP_ACTIVE_USERS" "$WRITE_INTO/active-users.json"
