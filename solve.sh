#!/bin/sh

CLICOLOR_FORCE=1 cargo run --release --bin solve -- $@ | less -R
