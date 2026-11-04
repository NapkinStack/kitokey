#!/usr/bin/env bash
# The source package Copr builds Kitokey from (D78): the four programs' sources at a git ref, the
# crates their lock files name vendored beside them — Copr builds without the network —, and
# kitokey.spec, its version written in. Made in a Fedora container, so that nothing is installed
# here. Usage: copr/srpm.sh <ref> <out-folder>; the version is the shell's at that ref.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
ref=${1:?usage: copr/srpm.sh <ref> <out-folder>}
out=${2:?usage: copr/srpm.sh <ref> <out-folder>}
mkdir -p "$out" && out=$(cd "$out" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)
version=$(git -C "$root" show "$ref:modules/shell/tauri.conf.json" | python3 -c 'import json, sys; print(json.load(sys.stdin)["version"])')
work=$(mktemp -d)
trap 'podman unshare rm -rf "$work"' EXIT
# The spec of that ref, its version written in: Copr reads it again from the source package alone.
{ echo "%global kitokey_version $version"; git -C "$root" show "$ref:modules/install/kitokey.spec"; } > "$work/kitokey.spec"
git -C "$root" archive --format=tar --prefix="kitokey-$version/" "$ref" \
  LICENSE modules/shell modules/capture modules/transcription modules/summary | tar -x -C "$work"
# No relabelling of what is mounted: labels left alone.
podman run --rm --security-opt label=disable -v "$work:/work" -v "$out:/out" -w /work \
  registry.fedoraproject.org/fedora:44 bash -euo pipefail -c "
dnf install -y -q cargo rpm-build xz >/dev/null 2>&1
cd kitokey-$version && mkdir .cargo
cargo vendor --locked --versioned-dirs --manifest-path modules/shell/Cargo.toml \
  -s modules/capture/Cargo.toml -s modules/transcription/Cargo.toml -s modules/summary/Cargo.toml \
  vendor 2>/dev/null > .cargo/config.toml
grep -q vendored-sources .cargo/config.toml
cd .. && XZ_OPT=-T0 tar -cJf kitokey-$version.tar.xz kitokey-$version
rpmbuild -bs kitokey.spec --define '_sourcedir /work' --define '_srcrpmdir /out' --quiet
"
ls "$out"/kitokey-"$version"-*.src.rpm
