#!/usr/bin/env bash
# Run on a Mac by scripts/release.sh: Kitokey.app from the artefacts each module's release
# left — the four programs side by side in Contents/MacOS, where the shell looks for them, and
# transcription's kitokey-speech beside its own (D42) —, its
# Info.plist and icon, signed ad hoc (an Apple chip runs nothing unsigned), zipped as macOS keeps a
# bundle whole. Usage: build.sh <artefacts> <version> <out>
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
artefacts=$1 version=$2 out=$3
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
app=$work/Kitokey.app
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
for program in shell/kitokey capture/kitokey-capture transcription/kitokey-transcription transcription/kitokey-speech \
  summary/kitokey-summary; do
  install -m755 "$artefacts/$program" "$app/Contents/MacOS/${program#*/}"
done
python3 "$here/info.py" "$app" "$version"
icons=$work/Kitokey.iconset
mkdir "$icons"
for size in 16 32 128 256 512; do
  sips -z $size $size "$artefacts/shell/com.napkinstack.Kitokey.png" --out "$icons/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  [ $double -le 512 ] && sips -z $double $double "$artefacts/shell/com.napkinstack.Kitokey.png" --out "$icons/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$icons" -o "$app/Contents/Resources/Kitokey.icns"
# Each program, then the bundle: its seal covers the Info.plist and the resources added above.
for program in "$app"/Contents/MacOS/kitokey-*; do codesign --force --sign - "$program"; done
codesign --force --sign - "$app"
ditto -c -k --keepParent "$app" "$out/Kitokey-$version-macos-$(uname -m).zip"
