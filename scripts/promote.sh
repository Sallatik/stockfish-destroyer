#!/usr/bin/env bash
# Make the current bin/destroyer the engine the campaign runner uses. Atomic rename, so games in
# progress keep their old binary.
set -euo pipefail
cd "$(dirname "$0")/.."
cp bin/destroyer bin/destroyer-official.tmp
mv bin/destroyer-official.tmp bin/destroyer-official
echo "promoted $(printf 'uci\nquit\n' | bin/destroyer-official | grep 'id name')"
