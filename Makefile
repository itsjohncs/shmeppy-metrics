.PHONY: ALWAYS_BUILD serve test deps

# This'll build everything and is probably the target you want
serve: build/site/data/registrations.json build/site/data/convocations.json build/site/index.htm build/site/index.js
	cd build/site && python3 -m http.server 2783 --bind 127.0.0.1

just-serve:
	cd build/site && python3 -m http.server 2783 --bind 127.0.0.1

# build/raw-logs needs to be set up manually, but once its there this will
# ensure its kept up to date. `refresh.sh` will rsync from a remote server, so
# there's now way for make to tell whether it needs to be run (though honestly,
# they're logs on a production server, they're probably out of date before the
# sync completes...).
build/raw-logs: ALWAYS_BUILD build/raw-logs/refresh.sh | build
	build/raw-logs/refresh.sh


############
# frontend #
############
build/site/index.htm: src/metrics-frontend/index.htm | build/site
	ln -fs $(shell pwd)/$< $@

build/site/index.js: src/metrics-frontend/index.js | build/site
	ln -fs $(shell pwd)/$< $@


#############
# site data #
#############
build/site/data/registrations.json build/site/data/convocations.json: $(shell find build/raw-logs) build/process-logs
	env "PATH=$(shell pwd)/build/:$(PATH)" build/process-logs build/site/data build/raw-logs/

build/process-logs: src/process-logs.sh build/fast-convoker build/count-registrations build/active-users
	ln -fs $(shell pwd)/$< $@

build/site/data: | build/site
	-mkdir $@

build/site: | build
	-mkdir $@


#################
# fast-convoker #
#################
build/fast-convoker: src/fast-convoker/target/release/fast-convoker | build
	ln -fs $(shell pwd)/$< $@

build/active-users: src/fast-convoker/target/release/active-users
	ln -fs $(shell pwd)/$< $@

src/fast-convoker/target/release/fast-convoker src/fast-convoker/target/release/active-users: $(shell find src/fast-convoker/src) src/fast-convoker/Cargo.toml src/fast-convoker/Cargo.lock
	cd src/fast-convoker; cargo build --release
	touch -c $@


###############################################
# fast-log-utils (get-logs-within, get-range) #
###############################################
build/count-registrations: src/fast-log-utils/target/release/count-registrations | build
	ln -fs $(shell pwd)/$< $@

src/fast-log-utils/target/release/get-logs-within src/fast-log-utils/target/release/get-range src/fast-log-utils/target/release/count-registrations src/fast-log-utils/target/release/filter-bad-versions: $(shell find src/fast-log-utils/src) src/fast-log-utils/Cargo.toml src/fast-log-utils/Cargo.lock
	cd src/fast-log-utils; cargo build --release
	touch -c $@


###########
test:
	find . -path '*/node_modules/*' -prune -or -name '*.sh' -print0 | xargs -t -0 shellcheck

build:
	-mkdir $@

ALWAYS_BUILD:
	@true

deps:
	# This gives me the fancy progress bar when rebuilding the convocation
	# cache.
	pip3 install tqdm

	# This gives me the GNU utils like gxargs and gmv
	brew install findutils
