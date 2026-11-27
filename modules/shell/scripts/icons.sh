#!/usr/bin/env bash
# Every icon the shell ships, drawn from the one mark, icons/kitokey.svg (D89): the application's
# PNG (512 px, which the packages take: release.sh, modules/install), Windows' .ico, and the tray's
# two states (64 px; icons/tray.svg and icons/tray-recording.svg, the same mark on its tile).
# `icons.sh` writes them; `icons.sh --check` renders them again and fails when a committed file
# differs from its source by more than antialiasing. Needs ImageMagick 7 with librsvg.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# No date chunk: the same source writes the same bytes, and a run leaves nothing to commit.
render() { magick -background none -density 384 "$here/icons/$1" -resize "$2x$2" -define png:exclude-chunks=date,time "PNG32:$3"; }

render kitokey.svg 512 "$work/icon.png"
for size in 256 64 48 32 16; do render kitokey.svg "$size" "$work/icon-$size.png"; done
magick "$work"/icon-{256,64,48,32,16}.png "$work/icon.ico"
render tray.svg 64 "$work/tray.png"
render tray-recording.svg 64 "$work/tray-recording.png"

if [ "${1:-}" != --check ]; then
  cp "$work/icon.png" "$work/icon.ico" "$work/tray.png" "$work/tray-recording.png" "$here/icons/"
  exit 0
fi
# The .ico holds the same renders; its sizes are what is compared.
differs=0
same() { # committed, rendered: at most 0.1 % of the pixels apart by more than 8 % — a dot moved by
  # one unit of the mark's grid is 0.2 % at 512 px
  local total off
  total=$(magick identify -format '%[fx:w*h]' "$2")
  off=$(magick compare -metric AE -fuzz 8% "$1" "$2" null: 2>&1 | cut -d' ' -f1) || true
  awk -v o="$off" -v t="$total" 'BEGIN { exit !(o + 0 <= t * 0.001) }'
}
same "$here/icons/icon.png" "$work/icon.png" || { echo "icons/icon.png is not icons/kitokey.svg drawn: run scripts/icons.sh"; differs=1; }
for tray in tray tray-recording; do
  same "$here/icons/$tray.png" "$work/$tray.png" || { echo "icons/$tray.png is not icons/$tray.svg drawn: run scripts/icons.sh"; differs=1; }
done
sizes=$(magick identify -format '%w ' "$here/icons/icon.ico")
if [ "$sizes" != "256 64 48 32 16 " ]; then
  echo "icons/icon.ico holds the sizes $sizes, not 256 64 48 32 16: run scripts/icons.sh"; differs=1
else
  for n in 0 1 2 3 4; do
    magick "$here/icons/icon.ico[$n]" "PNG32:$work/committed-$n.png"
    size=$(magick identify -format '%w' "$work/committed-$n.png")
    same "$work/committed-$n.png" "$work/icon-$size.png" || { echo "icons/icon.ico at $size px is not icons/kitokey.svg drawn: run scripts/icons.sh"; differs=1; }
  done
fi
exit $differs
