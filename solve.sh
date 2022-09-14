#!/bin/sh

CLICOLOR_FORCE=1 cargo run --release --bin solve -- $@ 2>&1 | less -R
