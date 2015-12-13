#!/bin/sh
set -e -u
set -x
cargo new --vcs none --bin "$(printf "day%02d" "$1")${2:-}"
