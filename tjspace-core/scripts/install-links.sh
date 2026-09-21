#!/usr/bin/env sh
set -eu
BIN_DIR=${BIN_DIR:-/usr/local/bin}
mkdir -p "$BIN_DIR"
ln -sf "$PWD/target/release/tjs-box" "$BIN_DIR/tjsd"
ln -sf "$PWD/target/release/tjs-box" "$BIN_DIR/tjs-cli"
ln -sf "$PWD/target/release/tjs-box" "$BIN_DIR/tjs-box"
