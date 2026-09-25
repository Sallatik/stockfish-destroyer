#!/usr/bin/env bash
# Build the engine at a git ref into bin/destroyer-<ref> (for A/B tests against older versions).
# Usage: scripts/build-engine.sh engine-v0      (no ref = current working tree -> bin/destroyer)
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ $# -eq 0 ]]; then
  cargo build --release --manifest-path engine/Cargo.toml
  cp engine/target/release/destroyer bin/destroyer
  echo bin/destroyer
  exit
fi

ref=$1
tmp=$(mktemp -d)
git archive "$ref" engine | tar -x -C "$tmp"
CARGO_TARGET_DIR=engine/target/ref cargo build --release --manifest-path "$tmp/engine/Cargo.toml"
cp engine/target/ref/release/destroyer "bin/destroyer-$ref"
rm -rf "$tmp"
echo "bin/destroyer-$ref"
