#!/usr/bin/env bash
# Copr builds the source package (D78) in the project named (`@<group>/kitokey`), with the token
# COPR_CONFIG holds — the text of https://copr.fedorainfracloud.org/api/, a secret of the forge —: for each
# release copr/project.sh gave the project. Fails when a release's build fails, naming it. Writes the releases built, one a line, to <out-file>.
# Usage: copr/build.sh <project> <the source package> <out-file>. Made in a Fedora container.
set -euo pipefail
usage='usage: copr/build.sh <project> <the source package> <out-file>'
project=${1:?$usage}
srpm=$(realpath "${2:?$usage}")
out=${3:?$usage}
: "${COPR_CONFIG:?the text of the API page of Copr, a secret the forge holds}"
# No relabelling of what is mounted: labels left alone.
podman run --rm --security-opt label=disable -e COPR_CONFIG -e project="$project" -v "$srpm:/kitokey.src.rpm:ro" \
  registry.fedoraproject.org/fedora:44 bash -euo pipefail -c '
dnf install -y -q copr-cli >/dev/null 2>&1
mkdir -p ~/.config && printf "%s\n" "$COPR_CONFIG" > ~/.config/copr
build=$(copr-cli build "$project" /kitokey.src.rpm --nowait | sed -n "s/^Created builds: //p")
echo "copr: build $build" >&2
copr-cli watch-build "$build" >&2 || true
python3 - "$build" <<"EOF_PY"
import sys
from copr.v3 import Client
failed = []
for chroot in Client.create_from_config_file().build_chroot_proxy.get_list(int(sys.argv[1])):
    print(f"copr: {chroot.name}: {chroot.state}", file=sys.stderr)
    if chroot.state == "succeeded":
        print(chroot.name.split("-")[1])
    else:
        failed.append(chroot.name)
if failed:
    sys.exit("copr: the build failed for " + ", ".join(failed))
EOF_PY
' > "$out"
