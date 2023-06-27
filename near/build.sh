#!/usr/bin/env bash

echo ">> Building contracts"

rm -rf artifacts
mkdir artifacts

RUSTFLAGS='-C target-feature=-crt-static -C link-arg=-nostartfiles -C link-arg=-Wl,--no-entry'
rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release

cp -r target/wasm32-unknown-unknown/release/*.wasm artifacts/
