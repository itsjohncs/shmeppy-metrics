#!/usr/bin/env bash

SCRIPT_DIR="$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )"

find "$SCRIPT_DIR/src" -name '*.unitTest.js' |
	xargs "$SCRIPT_DIR/node_modules/mocha/bin/mocha" \
	--experimental-modules
