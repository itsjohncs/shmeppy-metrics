.PHONY: ALWAYS_BUILD serve

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
	mkdir $@

build/build-convocations-aggregate: $(shell find src/build-convocations-aggregate -name '*.py') build/get-logs-within build/get-convocations-within | build
	echo

build/get-logs-within: src/get-logs-within/target/release/get-logs-within | build
	echo

src/get-logs-within/target/release/get-logs-within: $(shell find src/get-logs-within/src) src/get-logs-within/Cargo.toml
	cd src/get-logs-within; carge build --release

build/get-convocations-within: src/get-convocations-within/package.json $(shell find src/get-convocations-within/ -name '*.js') | build
	echo

build:
	mkdir $@

ALWAYS_BUILD:
	true
