#!/usr/bin/env bash
# What the Copr project (`@<group>/kitokey`) builds for, said from here before anything else asks it
# (D78): each Fedora release Copr still supports, on x86_64, the builds' internet access off — the
# crates are in the source package. With the token COPR_CONFIG holds, the text of
# https://copr.fedorainfracloud.org/api/, a secret of the forge. Made in a Fedora container.
# Usage: copr/project.sh <project>
set -euo pipefail
project=${1:?usage: copr/project.sh <project>}
: "${COPR_CONFIG:?the text of the API page of Copr, a secret the forge holds}"
podman run --rm -e COPR_CONFIG -e project="$project" registry.fedoraproject.org/fedora:44 bash -euo pipefail -c '
dnf install -y -q copr-cli >/dev/null 2>&1
mkdir -p ~/.config && printf "%s\n" "$COPR_CONFIG" > ~/.config/copr
chroots=$(copr-cli list-chroots | grep -E "^fedora-[0-9]+-x86_64$")
copr-cli modify "$project" --enable-net off $(printf -- "--chroot %s " $chroots) >/dev/null
echo "copr: $project builds for $(echo $chroots), without the network" >&2
'
