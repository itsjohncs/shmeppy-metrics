#!/usr/bin/env bash

set -eu
shopt -s failglob

SCRIPT_DIR="$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )"

find "$SCRIPT_DIR/src" -name '*.unitTest.js' -print0 |
	xargs -0 "$SCRIPT_DIR/node_modules/mocha/bin/mocha" \
		--color --experimental-modules 2>&1 |
	sed '1 { /(node:[^)]*) ExperimentalWarning:/ d; }'
