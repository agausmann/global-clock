#!/bin/sh
set -e

mkdir -p webapp
cp static/* webapp/

cargo build --release --target wasm32-unknown-unknown --features web
wasm-bindgen --target web --no-typescript --out-dir webapp/ target/wasm32-unknown-unknown/release/global-clock.wasm
