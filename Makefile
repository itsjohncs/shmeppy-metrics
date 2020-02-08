.PHONY: ALWAYS_BUILD serve test deps

test: src/get-convocations-within/test.sh
	find . -path '*/node_modules/*' -prune -or -name '*.sh' -print0 | xargs -t -0 shellcheck
	src/get-convocations-within/test.sh


serve: build/site/convocations.json build/site/index.htm
	echo

# The Makefile isn't smart enough to know when convocations needs to be
# regenereated. The `build-convocations-aggregate` program does its own
# dependency analysis to figure that out and is designed to be quite fast.
build/site/convocations.json: ALWAYS_BUILD build/build-convocations-aggregate build/raw-logs | build/site
	env "PATH=$(shell pwd)/build/:$(PATH)" build/build-convocations-aggregate $@ build/convocations-cache build/raw-logs

build/site/index.htm: | build/site
	echo

build/site: | build
	-mkdir $@

build/build-convocations-aggregate: src/build-convocations-aggregate/src/main.py $(shell find src/build-convocations-aggregate/src) build/get-range build/get-logs-within build/get-convocations-within | build build/convocations-cache
	ln -fs $(shell pwd)/$< $@

build/convocations-cache: | build
	-mkdir $@


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
	-true

deps:
	pip3 install tqdm
	brew install findutils
