#!/usr/bin/env bash
# The source package Copr builds Kitokey from (D78), at a git ref: the sources of the modules this
# one assembles (MANIFEST.yaml's `assembles`), every crate their lock files name vendored beside
# them — Copr builds without the network —, assemble.sh, which runs each module's own `release`
# command as `nstack release` does, and kitokey.spec, its version written in. Made in a Fedora
# container, so that nothing is installed here. Usage: copr/srpm.sh <ref> <version> <out-folder>.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
usage='usage: copr/srpm.sh <ref> <version> <out-folder>'
ref=${1:?$usage}
version=${2:?$usage}
out=${3:?$usage}
mkdir -p "$out" && out=$(cd "$out" && pwd)
root=$(git -C "$here" rev-parse --show-toplevel)
at() { git -C "$root" show "$ref:$1"; }
assembled=$(at modules/install/MANIFEST.yaml | sed -n 's/^assembles: *\[\(.*\)\]/\1/p' | tr -d ' ' | tr ',' ' ')
[ -n "$assembled" ] || { echo "srpm: install's manifest names no module it assembles" >&2; exit 2; }
work=$(mktemp -d)
trap 'podman unshare rm -rf "$work"' EXIT
top=$work/kitokey-$version
mkdir -p "$top"
git -C "$root" archive --format=tar "$ref" LICENSE $(printf 'modules/%s ' $assembled) | tar -x -C "$top"
# Each module's release, as its manifest declares it, run from its folder into dist/<module>.
{
  echo '# Each module this package assembles, released as its manifest says (copr/srpm.sh).'
  echo 'set -euo pipefail'
  echo 'top=$PWD'
  for module in $assembled; do
    command=$(sed -n 's/^  release: *//p' "$top/modules/$module/MANIFEST.yaml")
    [ -n "$command" ] || { echo "srpm: $module declares no release command" >&2; exit 2; }
    echo "mkdir -p dist/$module && (cd modules/$module && NSTACK_DIST=\"\$top/dist/$module\" $command)"
  done
} > "$top/assemble.sh"
# The spec of that ref, its version written in: Copr reads it again from the source package alone.
{ echo "%global kitokey_version $version"; at modules/install/kitokey.spec; } > "$work/kitokey.spec"
# No relabelling of what is mounted: labels left alone.
podman run --rm --security-opt label=disable -v "$work:/work" -v "$out:/out" -w /work \
  registry.fedoraproject.org/fedora:44 bash -euo pipefail -c "
dnf install -y -q cargo rpm-build xz >/dev/null 2>&1
cd kitokey-$version && mkdir .cargo
manifests=\$(ls modules/*/Cargo.toml)
first=\${manifests%%\$'\n'*}
cargo vendor --locked --versioned-dirs --manifest-path \$first \$(for m in \${manifests#\$first}; do printf -- '-s %s ' \$m; done) \
  vendor 2>/dev/null > .cargo/config.toml
grep -q vendored-sources .cargo/config.toml
cd .. && XZ_OPT=-T0 tar -cJf kitokey-$version.tar.xz kitokey-$version
rpmbuild -bs kitokey.spec --define '_sourcedir /work' --define '_srcrpmdir /out' --quiet
"
ls "$out"/kitokey-"$version"-*.src.rpm
