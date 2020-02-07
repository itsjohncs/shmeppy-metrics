.PHONY: ALWAYS_BUILD serve test

test: src/get-convocations-within/test.sh
	find . -path '*/node_modules/*' -prune -or -name '*.sh' -print0 | xargs -t -0 shellcheck
	src/get-convocations-within/test.sh


serve: build/site/convocations.json build/site/index.htm
	echo

# The Makefile isn't smart enough to know when convocations needs to be
# regnereated. The `build-convocations-aggregate` program does its own
# dependency analysis to figure that out and is designed to be quite fast.
build/site/convocations.json: ALWAYS_BUILD build/build-convocations-aggregate | build/site
	build/build-convocations-aggregate ...

build/site/index.htm: | build/site
	echo

build/site: | build
	-mkdir $@

build/build-convocations-aggregate: $(shell find src/build-convocations-aggregate -name '*.py') build/get-logs-within build/get-convocations-within | build
	echo


###############################################
# fast-log-utils (get-logs-within, get-range) #
###############################################
build/get-logs-within: src/fast-log-utils/target/release/get-logs-within | build
	ln -fs $(shell pwd)/$< $@

build/get-range: src/fast-log-utils/target/release/get-range | build
	ln -fs $(shell pwd)/$< $@

src/fast-log-utils/target/release/get-logs-within src/fast-log-utils/target/release/get-range: $(shell find src/fast-log-utils/src) src/fast-log-utils/Cargo.toml src/fast-log-utils/Cargo.lock
	cd src/fast-log-utils; cargo build --release


###########################
# get-convocations-within #
###########################
build/get-convocations-within: src/get-convocations-within/get-convocations-within.sh | build
	ln -fs $(shell pwd)/$< $@

src/get-convocations-within/get-convocations-within.sh: src/get-convocations-within/node_modules
	touch -c $@

src/get-convocations-within/test.sh: src/get-convocations-within/node_modules
	touch -c $@

src/get-convocations-within/node_modules: src/get-convocations-within/package.json
	cd src/get-convocations-within; npm install . && npm update . && npm prune
	touch -c $@


build:
	-mkdir $@

ALWAYS_BUILD:
	true
