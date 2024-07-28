#!/bin/bash

pwd=${shell pwd}
build-evn=SLINT_STYLE=material
run-evn=RUST_LOG=error,warn,info,debug,sqlx=off,reqwest=off,html2text=off
version=`git describe --tags --abbrev=0`

all: build-release

build:
	$(build-evn) cargo apk build --lib

build-release:
	$(build-evn) cargo apk build --lib --release
	cp -f target/release/apk/rssbox.apk target/rssbox-${version}.apk

build-release-nixos:
	nix-shell -p steam-run --run "$(build-evn) steam-run cargo apk build --lib --release"
	cp -f target/release/apk/rssbox.apk target/rssbox-${version}.apk

build-release-mold:
	$(build-evn) mold --run cargo apk build --lib --release
	cp -f target/release/apk/rssbox.apk target/rssbox-${version}.apk

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

run-release-mold:
	RUST_BACKTRACE=1 $(run-evn) mold --run cargo apk run --lib --release

install:
	$(build-evn) $(run-evn) cargo apk run --lib --release

debug:
	$(build-evn) $(run-evn) cargo run --features=desktop

debug-mold:
	$(build-evn) $(run-evn) mold --run cargo run --features=desktop

debug-local:
	$(run-evn) ./target/debug/rssbox

release-local:
	$(run-evn) ./target/release/rssbox

build-desktop-release:
	$(build-evn) $(run-evn) cargo build --release --features=desktop

build-desktop-release-qt-backend:
	SLINT_BACKEND=qt $(build-evn) $(run-evn) cargo build --release --features=desktop

build-desktop-release-nixos:
	nix-shell --run "$(build-evn) $(run-evn) cargo build --release --features=desktop"

install-desktop:
	cp -f target/release/rssbox ~/bin/rssbox

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

clean-unused-dependences:
	cargo machete

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

get-font-name:
	fc-scan ./ui/fonts/SourceHanSerifCN.ttf | grep fullname
	fc-scan ./ui/fonts/Plaster-Regular.ttf | grep fullname
