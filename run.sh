#!/bin/bash
set -e
cargo build --release --features layer-shell
./target/release/quickfind "$@"
