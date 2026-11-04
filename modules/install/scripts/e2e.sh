#!/usr/bin/env bash
# The package as a user meets it, for the system KITOKEY_TARGET names (unset: the system it runs on): installed
# with dnf or apt in a container of that system that never saw Kitokey, each program run, removed —
# meetings and settings stay. With no argument, made from stand-in programs (no other module
# built); given a package, that one. Needs podman and the network.
set -euo pipefail
shopt -s nullglob
here=$(cd "$(dirname "$0")/.." && pwd)
# As the other modules' e2e where their system is missing: said, and exit 77 — not run here, never a failure.
command -v podman >/dev/null || { echo "e2e: NOT RUN — podman is missing here"; exit 77; }
# Unset, the system this runs on, as it names itself (fedora-44, ubuntu-24.04).
target=${KITOKEY_TARGET:-$(. /etc/os-release && echo "$ID-$VERSION_ID")}
package=${1:+$(realpath "$1")}
if [ -z "$package" ]; then
  work=$(mktemp -d)
  trap 'rm -rf "$work"' EXIT
  for program in shell/kitokey capture/kitokey-capture transcription/kitokey-transcription summary/kitokey-summary; do
    mkdir -p "$work/$(dirname "$program")"
    printf '#!/bin/sh\necho "%s: usage: …" >&2\nexit 2\n' "$(basename "$program")" > "$work/$program"
    chmod +x "$work/$program"
  done
  printf '[Desktop Entry]\nType=Application\nName=Kitokey\nComment[fr]=x\nExec=kitokey\nIcon=com.napkinstack.Kitokey\n' > "$work/shell/com.napkinstack.Kitokey.desktop"
  printf 'png' > "$work/shell/com.napkinstack.Kitokey.png"
  echo 0.0.0 > "$work/shell/VERSION"
  mkdir -p "$work/install"
  KITOKEY_TARGET=$target NSTACK_DIST="$work/install" bash "$here/scripts/release.sh" >/dev/null
  made=("$work"/install/*.rpm "$work"/install/*.deb)
  package=${made[0]}
fi
# What any package must install, and nothing else: the spec's %files, its macros expanded.
expected=$(sed -n '/^%files/,$p' "$here/kitokey.spec" | sed '1d; /^$/d; s|%{_bindir}|/usr/bin|; s|%{_datadir}|/usr/share|' | sort | tr '\n' ' ')
case $target in
  fedora-[0-9]*)
    image="registry.fedoraproject.org/fedora:${target#fedora-}"
    add='dnf install -y -q /tmp/kitokey.rpm'; remove='dnf remove -y -q kitokey'; file=/tmp/kitokey.rpm
    listed='rpm -ql kitokey'; scripts='rpm -q --scripts kitokey'
    old='dnf install -y -q rpm-build && mkdir -p /tmp/old && printf "%s\n" "Name: wholeheard" "Version: 0.2.0" "Release: 2" "Summary: x" "License: x" "BuildArch: noarch" "%description" "x" "%install" "install -Dm755 /dev/null %{buildroot}/usr/bin/wholeheard" "%files" "/usr/bin/wholeheard" > /tmp/old/w.spec && rpmbuild -bb --quiet --define "_rpmdir /tmp/old" /tmp/old/w.spec && rpm -i /tmp/old/noarch/wholeheard-0.2.0-2.noarch.rpm'
    gone='! rpm -q wholeheard >/dev/null' ;;
  ubuntu-[0-9]*)
    image="docker.io/library/ubuntu:${target#ubuntu-}"
    add='apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq /tmp/kitokey.deb'
    remove='apt-get remove -y -qq kitokey'; file=/tmp/kitokey.deb
    listed='dpkg -L kitokey'
    scripts='ls /var/lib/dpkg/info/kitokey.preinst /var/lib/dpkg/info/kitokey.postinst /var/lib/dpkg/info/kitokey.prerm /var/lib/dpkg/info/kitokey.postrm'
    old='mkdir -p /tmp/old/DEBIAN /tmp/old/usr/bin && install -m755 /dev/null /tmp/old/usr/bin/wholeheard && printf "%s\n" "Package: wholeheard" "Version: 0.2.0" "Architecture: all" "Maintainer: x <x@example.invalid>" "Description: x" > /tmp/old/DEBIAN/control && dpkg-deb --build --root-owner-group /tmp/old /tmp/wholeheard.deb >/dev/null && dpkg -i /tmp/wholeheard.deb'
    gone='! dpkg-query -W -f "\${db:Status-Status}" wholeheard 2>/dev/null | grep -qx installed' ;;
  *) echo "e2e: no package is made for '$target'" >&2; exit 2 ;;
esac
podman run --rm --security-opt label=disable -v "$package:$file:ro" -v "$here/scripts/judge-installed.sh:/judge.sh:ro" -e add="$add" -e remove="$remove" -e listed="$listed" -e scripts="$scripts" -e expected="$expected" -e old="$old" -e gone="$gone" "$image" bash -euo pipefail -c '
failed=0
check() { if eval "$2"; then echo "  ok    $1"; else echo "  FAIL  $1"; failed=1; fi; }
mkdir -p $HOME/.config/kitokey $HOME/Kitokey/2026-10-18_10-00
echo "sessions = \"$HOME/Kitokey\"" > $HOME/.config/kitokey/settings.toml
echo "{}" > $HOME/Kitokey/2026-10-18_10-00/session.json
# Wholeheard there first, as a user of it has it (D86): a stand-in of its last version.
check "a Wholeheard 0.2.0 is there before" "eval \"\$old\" >/tmp/old.log 2>&1 || { tail -5 /tmp/old.log; false; }"
check "installed, every dependency found" "eval \"\$add\" >/tmp/add.log 2>&1 || { tail -5 /tmp/add.log; false; }"
check "it takes Wholeheard'"'"'s place: Wholeheard removed, its program gone" "eval \"\$gone\" && ! test -e /usr/bin/wholeheard"
for program in kitokey kitokey-capture kitokey-transcription kitokey-summary; do
  check "$program on the path, every library found" "command -v $program >/dev/null && ! ldd \$(command -v $program) 2>/dev/null | grep -q \"not found\""
  # A program says its usage; the shell, a window with no display here, goes as far as GTK.
  check "$program answers as itself" "{ timeout 10 $program 2>&1 || true; } | grep -qE \"^$program: usage|initialize GTK\""
done
check "the tray'"'"'s library is installed with it — the shell opens it at run time" "ldconfig -p | grep -E \"lib(ayatana-)?appindicator3\.so\.1\" >/dev/null"
check "pw-metadata is installed with it — capture runs it" "command -v pw-metadata >/dev/null"
check "it installs the spec'"'"'s files and the directories above them, nothing else, and runs no script" "bash /judge.sh \"\$listed\" \"\$scripts\""
check "the menu sees Kitokey, under its id" "grep -q ^Exec=kitokey /usr/share/applications/com.napkinstack.Kitokey.desktop"
check "removed" "eval \"\$remove\" >/dev/null 2>&1 && ! command -v kitokey >/dev/null"
check "the meetings and the settings stay" "test -f $HOME/.config/kitokey/settings.toml && test -f $HOME/Kitokey/2026-10-18_10-00/session.json"
[ "$failed" = 0 ] && echo "e2e: passed" || { echo "e2e: FAILED"; exit 1; }
'
