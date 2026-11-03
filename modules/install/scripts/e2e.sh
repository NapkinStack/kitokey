#!/usr/bin/env bash
# The package as a user meets it, for the system WHOLEHEARD_TARGET names (unset: the system it runs on): installed
# with dnf or apt in a container of that system that never saw Wholeheard, each program run, removed —
# meetings and settings stay. With no argument, made from stand-in programs (no other module
# built); given a package, that one. Needs podman and the network.
set -euo pipefail
shopt -s nullglob
here=$(cd "$(dirname "$0")/.." && pwd)
# As the other modules' e2e where their system is missing: said, and exit 77 — not run here, never a failure.
command -v podman >/dev/null || { echo "e2e: NOT RUN — podman is missing here"; exit 77; }
# Unset, the system this runs on, as it names itself (fedora-44, ubuntu-24.04).
target=${WHOLEHEARD_TARGET:-$(. /etc/os-release && echo "$ID-$VERSION_ID")}
package=${1:+$(realpath "$1")}
if [ -z "$package" ]; then
  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
  for program in shell/wholeheard capture/wholeheard-capture transcription/wholeheard-transcription summary/wholeheard-summary; do
    mkdir -p "$work/$(dirname "$program")"
    printf '#!/bin/sh\necho "%s: usage: …" >&2\nexit 2\n' "$(basename "$program")" > "$work/$program"
    chmod +x "$work/$program"
  done
  printf '[Desktop Entry]\nType=Application\nName=Wholeheard\nComment[fr]=x\nExec=wholeheard\nIcon=dev.wholeheard.Wholeheard\n' > "$work/shell/dev.wholeheard.Wholeheard.desktop"
  printf 'png' > "$work/shell/dev.wholeheard.Wholeheard.png"
  echo 0.0.0 > "$work/shell/VERSION"
  mkdir -p "$work/install"
  WHOLEHEARD_TARGET=$target NSTACK_DIST="$work/install" bash "$here/scripts/release.sh" >/dev/null
  made=("$work"/install/*.rpm "$work"/install/*.deb)
  package=${made[0]}
fi
# What any package must install, and nothing else: the spec's %files, its macros expanded.
expected=$(sed -n '/^%files/,$p' "$here/wholeheard.spec" | sed '1d; /^$/d; s|%{_bindir}|/usr/bin|; s|%{_datadir}|/usr/share|' | sort | tr '\n' ' ')
case $target in
  fedora-[0-9]*)
    image="registry.fedoraproject.org/fedora:${target#fedora-}"
    add='dnf install -y -q /tmp/wholeheard.rpm'; remove='dnf remove -y -q wholeheard'; file=/tmp/wholeheard.rpm
    listed='rpm -ql wholeheard'; scripts='rpm -q --scripts wholeheard' ;;
  ubuntu-[0-9]*)
    image="docker.io/library/ubuntu:${target#ubuntu-}"
    add='apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq /tmp/wholeheard.deb'
    remove='apt-get remove -y -qq wholeheard'; file=/tmp/wholeheard.deb
    listed='dpkg -L wholeheard'
    scripts='ls /var/lib/dpkg/info/wholeheard.preinst /var/lib/dpkg/info/wholeheard.postinst /var/lib/dpkg/info/wholeheard.prerm /var/lib/dpkg/info/wholeheard.postrm' ;;
  *) echo "e2e: no package is made for '$target'" >&2; exit 2 ;;
esac
podman run --rm --security-opt label=disable -v "$package:$file:ro" -v "$here/scripts/judge-installed.sh:/judge.sh:ro" -e add="$add" -e remove="$remove" -e listed="$listed" -e scripts="$scripts" -e expected="$expected" "$image" bash -euo pipefail -c '
failed=0
check() { if eval "$2"; then echo "  ok    $1"; else echo "  FAIL  $1"; failed=1; fi; }

# Create testuser, then get its home directory (no literal home path in source)
id -u testuser >/dev/null 2>&1 || useradd -m -s /bin/bash testuser
TEST_HOME="$(getent passwd testuser | cut -d: -f6)"

# Snapshot home BEFORE install — what the user already has
mkdir -p "$TEST_HOME/.config/wholeheard" "$TEST_HOME/.local/share/wholeheard" "$TEST_HOME/.local/state/wholeheard" "$TEST_HOME/.config/autostart"
pre_config=$(ls -1 "$TEST_HOME/.config/wholeheard" 2>/dev/null | sort)
pre_share=$(ls -1 "$TEST_HOME/.local/share/wholeheard" 2>/dev/null | sort)
pre_state=$(ls -1 "$TEST_HOME/.local/state/wholeheard" 2>/dev/null | sort)
pre_autostart=$(ls -1 "$TEST_HOME/.config/autostart" 2>/dev/null | sort)

# Create pre-existing settings and meetings (simulate S1: user already has data)
mkdir -p "$TEST_HOME/Wholeheard/2026-10-18_10-00"
echo "sessions = \\\"$TEST_HOME/Wholeheard\\\"" > "$TEST_HOME/.config/wholeheard/settings.toml"
echo '{"id": "2026-10-18_10-00"}' > "$TEST_HOME/Wholeheard/2026-10-18_10-00/session.json"

check "installed, every dependency found" "eval \"\$add\" >/tmp/add.log 2>&1 || { tail -5 /tmp/add.log; false; }"
for program in wholeheard wholeheard-capture wholeheard-transcription wholeheard-summary; do
  check "$program on the path, every library found" "command -v $program >/dev/null && ! ldd \$(command -v $program) 2>/dev/null | grep -q \"not found\""
  # A program says its usage; the shell, a window with no display here, goes as far as GTK.
  check "$program answers as itself" "{ timeout 10 $program 2>&1 || true; } | grep -qE \"^$program: usage|initialize GTK\""
done
check "the tray'\''s library is installed with it — the shell opens it at run time" "ldconfig -p | grep -E \"lib(ayatana-)?appindicator3\\.so\\.1\" >/dev/null"
check "pw-metadata is installed with it — capture runs it" "command -v pw-metadata >/dev/null"
check "it installs the spec'\''s files and the directories above them, nothing else, and runs no script" "bash /judge.sh \"\$listed\" \"\$scripts\""
check "the menu sees Wholeheard, under its id" "grep -q ^Exec=wholeheard /usr/share/applications/dev.wholeheard.Wholeheard.desktop"

# REMOVE
check "removed" "eval \"\$remove\" >/dev/null 2>&1 && ! command -v wholeheard >/dev/null"

# Verify all 4 programs + desktop entry + icon are gone from system paths
for program in wholeheard wholeheard-capture wholeheard-transcription wholeheard-summary; do
  check "$program binary removed" "! command -v $program >/dev/null"
done
check "desktop entry removed" "! test -f /usr/share/applications/dev.wholeheard.Wholeheard.desktop"
check "icon removed" "! test -f /usr/share/icons/hicolor/512x512/apps/dev.wholeheard.Wholeheard.png"

# Compare leftover with docs/privacy.md — only these user dirs should remain
post_config=$(ls -1 "$TEST_HOME/.config/wholeheard" 2>/dev/null | sort)
post_share=$(ls -1 "$TEST_HOME/.local/share/wholeheard" 2>/dev/null | sort)
post_state=$(ls -1 "$TEST_HOME/.local/state/wholeheard" 2>/dev/null | sort)
post_autostart=$(ls -1 "$TEST_HOME/.config/autostart" 2>/dev/null | sort)

check "~/.config/wholeheard unchanged (privacy.md)" "[ \"\$pre_config\" = \"\$post_config\" ]"
check "~/.local/share/wholeheard unchanged (privacy.md)" "[ \"\$pre_share\" = \"\$post_share\" ]"
check "~/.local/state/wholeheard unchanged (privacy.md)" "[ \"\$pre_state\" = \"\$post_state\" ]"
check "~/.config/autostart unchanged (privacy.md)" "[ \"\$pre_autostart\" = \"\$post_autostart\" ]"

# The meetings and the settings stay
check "the meetings and the settings stay" "test -f \$TEST_HOME/.config/wholeheard/settings.toml && test -f \$TEST_HOME/Wholeheard/2026-10-18_10-00/session.json"

# S2: Re-install and verify it reads settings/meetings
check "re-installed, every dependency found" "eval \"\$add\" >/tmp/add2.log 2>&1 || { tail -5 /tmp/add2.log; false; }"
for program in wholeheard wholeheard-capture wholeheard-transcription wholeheard-summary; do
  check "$program on the path after re-install" "command -v $program >/dev/null"
  check "$program answers as itself after re-install" "{ timeout 10 $program 2>&1 || true; } | grep -qE \"^$program: usage|initialize GTK\""
done
# Verify it reads the pre-existing settings and meetings
check "reads pre-existing settings" "grep -q \"sessions = \\\\\\\"\$TEST_HOME/Wholeheard\\\\\\\"\" \$TEST_HOME/.config/wholeheard/settings.toml"
check "reads pre-existing meetings" "test -f \$TEST_HOME/Wholeheard/2026-10-18_10-00/session.json"

[ "\$failed" = 0 ] && echo "e2e: passed" || { echo "e2e: FAILED"; exit 1; }
'