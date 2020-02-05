#!/usr/bin/env bash

# There's probably a better way to cd into this script's actual directory,
# even when executed through a symlink, but I don't know it.
SCRIPT_DIR="$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )"
if [ -L "${BASH_SOURCE[0]}" ]; then
	REAL_SOURCE="$(readlink "${BASH_SOURCE[0]}")"
	cd "$(dirname "$SCRIPT_DIR/$REAL_SOURCE")"
else
	cd "$SCRIPT_DIR"
fi

# Using --experimental-modules causes node to emit an annoying warning that's
# AFAIK impossible to disable. But like... I obviously know that it's
# experimental, I had to write out the word in the flag. Anyways, filter that
# shit out without resorting to the ludicrous `--no-warnings` flag as suggested
# by users onlihne.
node --experimental-modules src/index.js "$@" \
	2> >(sed '/(node:[^)]*) ExperimentalWarning:/ d' 1>&2)
