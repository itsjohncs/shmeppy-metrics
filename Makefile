.PHONY: ALWAYS_BUILD serve test

test: src/get-convocations-within/test.sh
	find . -path '*/node_modules/*' -prune -or -name '*.sh' -print | xargs shellcheck
	src/get-convocations-within/test.sh

src/get-convocations-within/test.sh: src/get-convocations-within/node_modules
	touch -c $@

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


build/get-logs-within: src/get-logs-within/target/release/get-logs-within | build
	ln -fs ../$< $@

src/get-logs-within/target/release/get-logs-within: $(shell find src/get-logs-within/src) src/get-logs-within/Cargo.toml
	cd src/get-logs-within; cargo build --release


build/get-convocations-within: src/get-convocations-within/get-convocations-within.sh | build
	ln -fs ../$< $@

src/get-convocations-within/get-convocations-within.sh: src/get-convocations-within/node_modules
	touch -c $@

src/get-convocations-within/node_modules: src/get-convocations-within/package.json
	cd src/get-convocations-within; npm install . && npm update . && npm prune
	touch $@


build:
	-mkdir $@

ALWAYS_BUILD:
	true
