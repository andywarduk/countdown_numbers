#!/bin/sh

# relies on cargo-wasi being installed: cargo install cargo-wasi [--force]
# target installed with: rustup target add wasm32-wasi

cargo wasi run --quiet --release --bin solve -- $@ 2>&1 | less -R
