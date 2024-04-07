#!/bin/bash

root_url=$1

cd ..
RUST_LOG=error,warn,info ./target/debug/tool-gen-rss -r $root_url

