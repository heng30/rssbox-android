#!/bin/bash

build-evn=SLINT_STYLE=material
run-evn=RUST_LOG=error,warn,info,debug,sqlx=off,reqwest=off,html2text=off

all: build-release

build:
	$(build-evn) cargo apk build --lib

build-release:
	$(build-evn) cargo apk build --lib --release

# not work well
xbuild:
	$(build-evn) x build --debug --platform android --format apk --arch arm64

# not work well
xbuild-release:
	$(build-evn) x build --release --platform android --format apk --arch arm64

run:
	RUST_BACKTRACE=1 $(run-evn) cargo apk run --lib

run-release:
	RUST_BACKTRACE=1 $(run-evn) cargo apk run --lib --release

install:
	$(build-evn) $(run-evn) cargo apk run --lib --release

debug:
	$(build-evn) $(run-evn) cargo run --bin rssbox-desktop --features=desktop

debug-local:
	$(run-evn) ./target/debug/rssbox-desktop

tool-gen-rss-build:
	cargo build --bin tool-gen-rss --features=tool-gen-rss

tool-gen-rss-run-generate:
	RUST_BACKTRACE=1 RUST_LOG=error,warn,info cargo run --bin tool-gen-rss --features=tool-gen-rss -- -g

tool-gen-rss-run-local-generate:
	RUST_LOG=error,warn,info ./target/debug/tool-gen-rss -g

tool-gen-rss-run-send-cn:
	RUST_BACKTRACE=1 RUST_LOG=error,warn,info cargo run --bin tool-gen-rss --features=tool-gen-rss -- -r http://0.0.0.0:8004

tool-gen-rss-run-local-send-cn:
	RUST_BACKTRACE=1 RUST_LOG=error,warn,info ./target/debug/tool-gen-rss -r http://0.0.0.0:8004

tool-gen-rss-run-local-send-en:
	RUST_BACKTRACE=1 RUST_LOG=error,warn,info ./target/debug/tool-gen-rss -r --is_cn http://0.0.0.0:8004

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

slint-view-light:
	slint-viewer --style material-light --auto-reload -I ui ./ui/appwindow.slint

slint-view-dark:
	slint-viewer --style material-dark --auto-reload -I ui ./ui/appwindow.slint
