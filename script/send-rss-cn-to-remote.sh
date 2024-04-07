#!/bin/bash

root_url=$1

cd ..
RUST_LOG=error,warn,info ./target/debug/tool-gen-rss -i -r $root_url

