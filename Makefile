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
	RUST_BACKTRACE=1 $(run-evn) cargo apk run --lib

install:
	$(build-evn) $(run-evn) cargo apk run --lib --release

debug:
	$(build-evn) $(run-evn) cargo run --bin rssbox-desktop --features=desktop

tool-gen-rss-build:
	cargo build --bin tool-gen-rss --features=tool-gen-rss

tool-gen-rss-run:
	RUST_BACKTRACE=1 RUST_LOG=error,warn,info cargo run --bin tool-gen-rss --features=tool-gen-rss

tool-gen-rss-build:
	RUST_LOG=error,warn,info cargo build --bin tool-gen-rss --features=tool-gen-rss

test:
	$(build-evn) $(run-evn) cargo test -- --nocapture

clippy:
	cargo clippy

clean-incremental:
	rm -rf ./target/debug/incremental/*
	rm -rf ./target/aarch64-linux-android/debug/incremental

clean:
	cargo clean

sweep:
	cargo sweep --maxsize 10GB

slint-view:
	slint-viewer --style material --auto-reload -I ui ./ui/appwindow.slint
