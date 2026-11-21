#!/usr/bin/env bash
# Kitokey's speed on its release build (D105), in the session scripts/headless.sh made:
#
#     KITOKEY_SPEED=<runs> scripts/headless.sh
#
# The release build is the `release` verb's (scripts/release.sh: cargo build --locked --release). The
# result goes to the evidence folder, speed.json, its medians said last (scripts/speed.py).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=${KITOKEY_E2E_OUT:-$here/../.evidence}
mkdir -p "$out"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
(cd "$here/.." && cargo build --locked --release 2>&1 | tail -1)
bin=${CARGO_TARGET_DIR:-$here/../target}/release/kitokey
command -v tauri-driver >/dev/null || cargo install tauri-driver --locked --version 2.1.0 2>&1 | tail -1
uv run --quiet --with selenium==4.51.0 --with jsonschema==4.23.0 --with rfc3339-validator==0.1.4 \
  python -u "$here/speed.py" "$bin" "$work" "${KITOKEY_SPEED:?the number of runs}" "$out/speed.json" | tee "$out/speed.txt"
