#!/bin/bash

cargo clippy
cargo fmt --check --all | less -R
cargo check
