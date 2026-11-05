#!/usr/bin/env bash
# D78's fourth criterion, around Copr's build of the next version (.github/workflows/copr.yml):
# `before`, the version Copr holds now, installed from it in a Fedora 44 container that stays, and
# used as a user does; `after`, once Copr built KITOKEY_VERSION, `dnf upgrade` to it — the
# meetings and settings stay. Copr holding no version yet, there is nothing to upgrade from: said,
# and not run. Usage: copr/upgrade.sh before|after <project>
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
phase=${1:?usage: copr/upgrade.sh before|after <project>}
copr=${2:?usage: copr/upgrade.sh before|after <project>}
name=kitokey-upgrade
# The next version, the one Copr builds: the release tag's.
version=${KITOKEY_VERSION:?KITOKEY_VERSION names the version Copr builds}
# What drives the window, as e2e.sh installs it on Fedora.
tools='dnf install -y -q xorg-x11-server-Xvfb xorg-x11-xauth webkitgtk6.0 dbus-daemon python3 procps-ng findutils'
inside() { podman exec -e tools="$tools" -e driver=/usr/bin/WebKitWebDriver -e version="$version" "$name" "$@"; }
case $phase in
  before)
    podman rm -f "$name" >/dev/null 2>&1 || true
    # No relabelling of what is mounted: labels left alone. An init as the first process, to reap
    # what ends: under a bare `sleep`, a Kitokey that quit stays a zombie, and is seen as running.
    podman run -d --init --name "$name" --security-opt label=disable -v "$here/scripts/as-user.sh:/as-user.sh:ro" \
      -v "$here/scripts/start-as-user.py:/start-as-user.py:ro" -v "$here/copr/upgrade-inside.sh:/upgrade-inside.sh:ro" \
      registry.fedoraproject.org/fedora:44 sleep infinity >/dev/null
    inside bash -c "dnf install -y -q dnf-plugins-core >/dev/null && dnf copr enable -y '$copr' >/dev/null"
    # Copr unreachable is a failure, not an empty project.
    held=$(inside dnf repoquery -q kitokey) || { echo "upgrade: FAIL — Copr could not be asked what it holds" >&2; podman rm -f "$name" >/dev/null; exit 1; }
    if [ -z "$held" ]; then
      echo "upgrade: NOT RUN — Copr holds no kitokey yet: the first version has none to upgrade from"
      podman rm -f "$name" >/dev/null
      exit 0
    fi
    inside bash -c 'dnf install -y -q kitokey >/tmp/add.log 2>&1 || { tail -5 /tmp/add.log; exit 1; }'
    inside bash /upgrade-inside.sh before ;;
  after)
    podman container exists "$name" || { echo "upgrade: NOT RUN — nothing was installed before the build"; exit 0; }
    status=0
    inside bash /upgrade-inside.sh after || status=$?
    podman rm -f "$name" >/dev/null
    exit $status ;;
  *) echo "usage: copr/upgrade.sh before|after <project>" >&2; exit 2 ;;
esac
