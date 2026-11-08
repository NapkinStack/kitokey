#!/usr/bin/env bash
# Every icon the shell ships, drawn from the one mark, icons/kitokey.svg (D89): the application's
# PNG (512 px, which the packages take: release.sh, modules/install) and Windows' .ico.
# `icons.sh` writes them; `icons.sh --check` renders them again and fails when a committed file
# differs from its source by more than antialiasing. Needs ImageMagick 7 with librsvg.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
render() { magick -background none -density 384 "$here/icons/$1" -resize "$2x$2" "PNG32:$3"; }

render kitokey.svg 512 "$work/icon.png"
for size in 256 64 48 32 16; do render kitokey.svg "$size" "$work/icon-$size.png"; done
magick "$work"/icon-{256,64,48,32,16}.png "$work/icon.ico"

if [ "${1:-}" != --check ]; then
  cp "$work/icon.png" "$work/icon.ico" "$here/icons/"
  exit 0
fi
# The .ico holds the same renders; its sizes are what is compared.
differs=0
same() { # committed, rendered: at most 0.5 % of the pixels apart by more than 8 %
  local total off
  total=$(magick identify -format '%[fx:w*h]' "$2")
  off=$(magick compare -metric AE -fuzz 8% "$1" "$2" null: 2>&1 | cut -d' ' -f1) || true
  awk -v o="$off" -v t="$total" 'BEGIN { exit !(o + 0 <= t * 0.005) }'
}
same "$here/icons/icon.png" "$work/icon.png" || { echo "icons/icon.png is not icons/kitokey.svg drawn: run scripts/icons.sh"; differs=1; }
for n in 0 1 2 3 4; do
  magick "$here/icons/icon.ico[$n]" "PNG32:$work/committed-$n.png"
  size=$(magick identify -format '%w' "$work/committed-$n.png")
  same "$work/committed-$n.png" "$work/icon-$size.png" || { echo "icons/icon.ico at $size px is not icons/kitokey.svg drawn: run scripts/icons.sh"; differs=1; }
done
exit $differs
