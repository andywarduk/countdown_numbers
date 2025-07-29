#!/bin/sh

# target installed with: rustup target add wasm32-wasi

cargo build --quiet --release --bin solve --target wasm32-wasip1
wasmtime --dir=. target/wasm32-wasip1/release/solve.wasm -- $@ 2>&1 | less -R

