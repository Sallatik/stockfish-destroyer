#!/usr/bin/env bash
# One-shot setup for a fresh laptop: toolchains, pinned Stockfish, Python + web deps, engine build.
set -euo pipefail
cd "$(dirname "$0")/.."

SF_VERSION="sf_17.1"

export PATH="$HOME/.local/bin:$PATH"
command -v mise >/dev/null || curl -fsSL https://mise.run | sh
mise trust --yes >/dev/null
mise install
eval "$(mise env -s bash)"

# Pinned Stockfish binary -> bin/stockfish
if [[ ! -x bin/stockfish ]] || ! grep -q "$SF_VERSION" bin/.sf_version 2>/dev/null; then
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64)  asset=stockfish-macos-m1-apple-silicon.tar ;;
    Darwin-x86_64) asset=stockfish-macos-x86-64-avx2.tar ;;
    Linux-x86_64)  asset=stockfish-ubuntu-x86-64-avx2.tar ;;
    *) echo "Unsupported platform $(uname -s)-$(uname -m)"; exit 1 ;;
  esac
  mkdir -p bin
  tmp=$(mktemp -d)
  curl -fsSL "https://github.com/official-stockfish/Stockfish/releases/download/$SF_VERSION/$asset" | tar -x -C "$tmp"
  find "$tmp" -type f -name 'stockfish-*' -perm -u+x -exec cp {} bin/stockfish \;
  chmod +x bin/stockfish
  echo "$SF_VERSION" > bin/.sf_version
  rm -rf "$tmp"
fi

uv sync
cargo build --release --manifest-path engine/Cargo.toml
cp engine/target/release/destroyer bin/destroyer
(cd web && npm install --no-audit --no-fund)

echo "Setup done. Try: uv run arena --elo 1320"
