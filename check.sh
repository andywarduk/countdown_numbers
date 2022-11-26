#!/bin/bash

cargo clippy --color always -q 2>&1 | less -R
cargo check --color always -q 2>&1 | less -R
cargo test --color always 2>&1 | less -R
