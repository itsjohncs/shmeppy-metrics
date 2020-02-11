.PHONY: ALWAYS_BUILD serve test deps

serve: build/site/data/registrations.json build/site/data/convocations.json build/site/index.htm build/site/index.js
	cd build/site && python3 -m http.server 2783 --bind 127.0.0.1

build/site/index.htm: src/metrics-frontend/index.htm | build/site
	ln -fs $(shell pwd)/$< $@

build/site/index.js: src/metrics-frontend/index.js | build/site
	ln -fs $(shell pwd)/$< $@

build/raw-logs: $(shell find build/raw-logs/ -name '*.log' || true) | build
	touch -c $@

######################
# registrations.json #
######################
build/site/data/registrations.json: build/raw-logs build/build-registrations | build/site/data
	env "PATH=$(shell pwd)/build/:$(PATH)" build/build-registrations $@ build/raw-logs/

build/build-registrations: src/fast-log-utils/build-registrations.sh build/count-registrations | build
	ln -fs $(shell pwd)/$< $@

#####################
# convocations.json #
#####################
# The Makefile isn't smart enough to know when convocations needs to be
# regenereated. The `build-convocations-aggregate` program does its own
# dependency analysis to figure that out and is designed to be quite fast.
build/site/data/convocations.json: ALWAYS_BUILD build/build-convocations-aggregate build/raw-logs | build/site/data
	env "PATH=$(shell pwd)/build/:$(PATH)" build/build-convocations-aggregate $@ build/convocations-cache build/raw-logs

build/site/data: | build/site
	-mkdir $@

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

build/count-registrations: src/fast-log-utils/target/release/count-registrations | build
	ln -fs $(shell pwd)/$< $@

src/fast-log-utils/target/release/get-logs-within src/fast-log-utils/target/release/get-range src/fast-log-utils/target/release/count-registrations: $(shell find src/fast-log-utils/src) src/fast-log-utils/Cargo.toml src/fast-log-utils/Cargo.lock
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
	cd src/get-convocations-within && npm install . && npm update . && npm prune
	touch -c $@

test: src/get-convocations-within/test.sh
	find . -path '*/node_modules/*' -prune -or -name '*.sh' -print0 | xargs -t -0 shellcheck
	src/get-convocations-within/test.sh

build:
	-mkdir $@

ALWAYS_BUILD:
	@true

deps:
	pip3 install tqdm
	brew install findutils
