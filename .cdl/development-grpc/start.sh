#!/usr/bin/env bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

mkdir -p "$SCRIPT_DIR/.tmp"

export CDL_CONFIG="$SCRIPT_DIR"
cargo build --workspace || exit 1
./target/debug/api > "$SCRIPT_DIR/.tmp/api.log" 2>&1 &
./target/debug/data-router > "$SCRIPT_DIR/.tmp/data-router.log" 2>&1 &
./target/debug/query-router > "$SCRIPT_DIR/.tmp/query-router.log" 2>&1 &
./target/debug/schema-registry > "$SCRIPT_DIR/.tmp/schema-registry.log" 2>&1 &
./target/debug/command-service > "$SCRIPT_DIR/.tmp/command-service.log" 2>&1 &
./target/debug/query-service > "$SCRIPT_DIR/.tmp/query-service.log" 2>&1 &
