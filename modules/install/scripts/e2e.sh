#!/usr/bin/env bash
# The package as a user meets it, for the system KITOKEY_TARGET names (unset: the system it runs on): installed
# with dnf or apt in a container of that system that never saw Kitokey, each program run, removed —
# meetings and settings stay. With no argument, made from stand-in programs (no other module
# built); given a package, that one; given `copr:<owner>/kitokey`, the one Copr built, installed
# with `dnf copr enable` (D78) — on Fedora alone. Needs podman and the network.
set -euo pipefail
shopt -s nullglob
here=$(cd "$(dirname "$0")/.." && pwd)
# As the other modules' e2e where their system is missing: said, and exit 77 — not run here, never a failure.
command -v podman >/dev/null || { echo "e2e: NOT RUN — podman is missing here"; exit 77; }
# Unset, the system this runs on, as it names itself (fedora-44, ubuntu-24.04).
target=${KITOKEY_TARGET:-$(. /etc/os-release && echo "$ID-$VERSION_ID")}
copr=
case ${1:-} in
  copr:*/*) copr=${1#copr:}; package= ;;
  *) package=${1:+$(realpath "$1")} ;;
esac
# Given a package, Kitokey itself is started in the container, as its user (D80): stand-ins cannot be.
real=${package:+1}${copr:+1}
# Copr's is built from this checkout's source, the release tag's: each program says its version (D78).
version=$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["version"])' "$here/../shell/tauri.conf.json")
if [ -z "$package$copr" ]; then
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
# What any package must install, and nothing else: the spec's %files, its macros expanded. The
# licence is shipped by a package built from source (`from_source`): Copr's must hold it; a package
# given may — one made from the artefacts does not.
files=$(sed -n '/^%files/,$p' "$here/kitokey.spec" \
  | sed '1d; /^$/d; /^%if/d; /^%endif/d; s|%{_bindir}|/usr/bin|; s|%{_datadir}|/usr/share|; s|^%license LICENSE$|/usr/share/licenses/kitokey/LICENSE|' | sort)
licence=/usr/share/licenses/kitokey/LICENSE
if [ -n "$copr" ]; then optional=; else optional=$licence; files=$(grep -vx "$licence" <<< "$files"); fi
expected=$(tr '\n' ' ' <<< "$files")
# What Kitokey may leave in a user's home on Linux, as docs/privacy.md says it: its Linux paths,
# in English, then in French — the same, or one of the two pages is wrong.
privacy=$here/../../docs/privacy.md
# The bullet that starts with its words, and the lines that carry it on; the paths it names.
linux() { awk -v line="$1" 'index($0, line) == 1 {on = 1; print; next} on && /^    / {print; next} {on = 0}' "$privacy" | grep -o '`~[^`]*`' | tr -d '`' | sort -u; }
kept=$(linux '  - Linux:')
kept_fr=$(linux '  - Linux :')
case $target in
  fedora-[0-9]*)
    image="registry.fedoraproject.org/fedora:${target#fedora-}"
    add='dnf install -y -q /tmp/kitokey.rpm'; remove='dnf remove -y -q kitokey'; file=/tmp/kitokey.rpm
    [ -z "$copr" ] || add="dnf install -y -q dnf-plugins-core && dnf copr enable -y $copr && dnf install -y -q kitokey"
    listed='rpm -ql kitokey'; scripts='rpm -q --scripts kitokey'
    # The stand-in Wholeheard is made here, so signed by nobody: Fedora 45's rpm refuses that unless told.
    old='dnf install -y -q rpm-build && mkdir -p /tmp/old && printf "%s\n" "Name: wholeheard" "Version: 0.2.0" "Release: 2" "Summary: x" "License: x" "BuildArch: noarch" "%description" "x" "%install" "install -Dm755 /dev/null %{buildroot}/usr/bin/wholeheard" "%files" "/usr/bin/wholeheard" > /tmp/old/w.spec && rpmbuild -bb --quiet --define "_rpmdir /tmp/old" /tmp/old/w.spec && rpm -i --nosignature /tmp/old/noarch/wholeheard-0.2.0-2.noarch.rpm'
    gone='! rpm -q wholeheard >/dev/null'
    # What drives the window: a display, a session bus, WebKit's driver (GTK 4's drives 4.1's, as on the workstation).
    tools='dnf install -y -q xorg-x11-server-Xvfb xorg-x11-xauth webkitgtk6.0 dbus-daemon python3 procps-ng findutils'
    driver=/usr/bin/WebKitWebDriver ;;
  ubuntu-[0-9]*)
    [ -z "$copr" ] || { echo "e2e: Copr builds for Fedora only" >&2; exit 2; }
    image="docker.io/library/ubuntu:${target#ubuntu-}"
    add='apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq /tmp/kitokey.deb'
    remove='apt-get remove -y -qq kitokey'; file=/tmp/kitokey.deb
    listed='dpkg -L kitokey'
    scripts='ls /var/lib/dpkg/info/kitokey.preinst /var/lib/dpkg/info/kitokey.postinst /var/lib/dpkg/info/kitokey.prerm /var/lib/dpkg/info/kitokey.postrm'
    old='mkdir -p /tmp/old/DEBIAN /tmp/old/usr/bin && install -m755 /dev/null /tmp/old/usr/bin/wholeheard && printf "%s\n" "Package: wholeheard" "Version: 0.2.0" "Architecture: all" "Maintainer: x <x@example.invalid>" "Description: x" > /tmp/old/DEBIAN/control && dpkg-deb --build --root-owner-group /tmp/old /tmp/wholeheard.deb >/dev/null && dpkg -i /tmp/wholeheard.deb'
    gone='! dpkg-query -W -f "\${db:Status-Status}" wholeheard 2>/dev/null | grep -qx installed'
    tools='DEBIAN_FRONTEND=noninteractive apt-get install -y -qq xvfb xauth webkit2gtk-driver dbus python3 procps findutils'
    driver=/usr/bin/WebKitWebDriver ;;
  *) echo "e2e: no package is made for '$target'" >&2; exit 2 ;;
esac
given=(); [ -z "$package" ] || given=(-v "$package:$file:ro")
podman run --rm --security-opt label=disable ${given[@]+"${given[@]}"} -v "$here/scripts/judge-installed.sh:/judge.sh:ro" \
  -v "$here/scripts/removal.sh:/removal.sh:ro" -v "$here/scripts/as-user.sh:/as-user.sh:ro" -v "$here/scripts/start-as-user.py:/start-as-user.py:ro" \
  -e add="$add" -e remove="$remove" -e listed="$listed" -e scripts="$scripts" -e expected="$expected" -e optional="$optional" -e old="$old" -e gone="$gone" \
  -e real="$real" -e copr="$copr" -e version="$version" -e kept="$kept" -e kept_fr="$kept_fr" -e tools="$tools" -e driver="$driver" "$image" bash -euo pipefail -c '
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
  [ -z "$copr" ] || check "$program says the tag'"'"'s version, $version" "test \"\$($program --version)\" = \"$program $version\""
done
check "the tray'"'"'s library is installed with it — the shell opens it at run time" "ldconfig -p | grep -E \"lib(ayatana-)?appindicator3\.so\.1\" >/dev/null"
check "pw-metadata is installed with it — capture runs it" "command -v pw-metadata >/dev/null"
check "it installs the spec'"'"'s files and the directories above them, nothing else, and runs no script" "bash /judge.sh \"\$listed\" \"\$scripts\""
check "the menu sees Kitokey, under its id" "grep -q ^Exec=kitokey /usr/share/applications/com.napkinstack.Kitokey.desktop"
if [ -n "$real" ]; then
  # Kitokey started as its user, removed, installed again (D80).
  bash /removal.sh || failed=1
else
  check "removed" "eval \"\$remove\" >/dev/null 2>&1 && ! command -v kitokey >/dev/null"
  check "the meetings and the settings stay" "test -f $HOME/.config/kitokey/settings.toml && test -f $HOME/Kitokey/2026-10-18_10-00/session.json"
fi
[ "$failed" = 0 ] && echo "e2e: passed" || { echo "e2e: FAILED"; exit 1; }
'
