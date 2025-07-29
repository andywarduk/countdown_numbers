#!/bin/sh

# target installed with: rustup target add wasm32-wasip1

cargo build --quiet --release --bin solve --target wasm32-wasip2
wasmtime --dir=. target/wasm32-wasip2/release/solve.wasm -- $@ 2>&1 | less -R

