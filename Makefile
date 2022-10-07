
ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

cloc:
	@gocloc --not-match-d="target" .

build-linux:
	@TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

mininet-env:
	@docker run -it --rm --privileged -e DISPLAY \
             -v /tmp/.X11-unix:/tmp/.X11-unix \
             -v /lib/modules:/lib/modules \
			 -v ${ROOT_DIR}/target:/root \
             iwaseyusuke/mininet:ubuntu-20.04 -- mn

lint:
	@cargo clippy

lint-fix:
	@cargo clippy --fix
