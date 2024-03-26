#!/bin/bash

# build-evn=SLINT_STYLE=fluent
build-evn=SLINT_STYLE=material
run-evn=RUST_LOG=error,warn,info,debug,sqlx=off,reqwest=off,html2text=off

all: build-release

build:
	$(build-evn) cargo apk build --lib

build-release:
	$(build-evn) cargo apk build --lib --release

run:
	$(build-evn) $(run-evn) cargo apk run --lib

install:
	$(build-evn) $(run-evn) cargo apk run --lib --release

debug:
	$(build-evn) $(run-evn) cargo run --bin rssbox --features=desktop

test:
	$(build-evn) $(run-evn) cargo test -- --nocapture

clippy:
	cargo clippy

clean-incremental:
	rm -rf ./target/debug/incremental/*

clean:
	cargo clean

slint-view:
	slint-viewer --style material --auto-reload -I ui ./ui/appwindow.slint
