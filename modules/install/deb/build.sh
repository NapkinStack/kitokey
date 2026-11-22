# Run in an Ubuntu container by scripts/release.sh: the .deb of the five programs, its library
# dependencies read by dpkg-shlibdeps from what each program links — the libraries installed
# here for that, as a user's system has them —, and what is run or opened at run time, not
# linked, named by hand: pw-metadata for capture, the tray's library for the shell.
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq --no-install-recommends dpkg-dev binutils \
  libwebkit2gtk-4.1-0 libgtk-3-0t64 librsvg2-2 libpipewire-0.3-0t64 libasound2t64 libvulkan1 >/dev/null
root=/tmp/package
for program in shell/kitokey capture/kitokey-capture transcription/kitokey-transcription summary/kitokey-summary export/kitokey-export; do
  install -Dm755 "/artefacts/$program" "$root/usr/bin/${program#*/}"
done
# Lighter without their symbols; a program that is no binary is left as it is.
for program in "$root"/usr/bin/*; do strip --strip-unneeded "$program" 2>/dev/null || :; done
install -Dm644 /artefacts/shell/com.napkinstack.Kitokey.desktop "$root/usr/share/applications/com.napkinstack.Kitokey.desktop"
install -Dm644 /artefacts/shell/com.napkinstack.Kitokey.png "$root/usr/share/icons/hicolor/512x512/apps/com.napkinstack.Kitokey.png"
# What the programs owe the authors of what they ship (D102), as windows/notices.py names it:
# Kitokey's LICENSE and install's notices, mounted by scripts/release.sh, and each module's page.
install -Dm644 /LICENSE "$root/usr/share/doc/kitokey/LICENSE.txt"
install -Dm644 /notices/MPL-SOURCES.txt "$root/usr/share/doc/kitokey/MPL-SOURCES.txt"
install -Dm644 /artefacts/shell/THIRD-PARTY-NOTICES.html "$root/usr/share/doc/kitokey/THIRD-PARTY-NOTICES-kitokey.html"
install -Dm644 /artefacts/capture/THIRD-PARTY-NOTICES.html "$root/usr/share/doc/kitokey/THIRD-PARTY-NOTICES-kitokey-capture.html"
install -Dm644 /artefacts/transcription/THIRD-PARTY-NOTICES.html "$root/usr/share/doc/kitokey/THIRD-PARTY-NOTICES-kitokey-transcription.html"
install -Dm644 /artefacts/summary/THIRD-PARTY-NOTICES.html "$root/usr/share/doc/kitokey/THIRD-PARTY-NOTICES-kitokey-summary.html"
install -Dm644 /artefacts/export/THIRD-PARTY-NOTICES.html "$root/usr/share/doc/kitokey/THIRD-PARTY-NOTICES-kitokey-export.html"
# dpkg-shlibdeps reads a source package's control file where it runs: one is made for it.
mkdir -p /tmp/shlibs/debian
printf 'Source: kitokey\n\nPackage: kitokey\nArchitecture: any\n' > /tmp/shlibs/debian/control
binaries=()
for program in "$root"/usr/bin/*; do
  [ "$(head -c 4 "$program")" = $'\x7fELF' ] && binaries+=("$program")
done
linked=""
if [ ${#binaries[@]} -gt 0 ]; then
  linked=$(cd /tmp/shlibs && dpkg-shlibdeps -O "${binaries[@]}" | sed -n 's/^shlibs:Depends=//p')
fi
depends="${linked:+$linked, }pipewire-bin, libayatana-appindicator3-1 | libappindicator3-1"
mkdir -p "$root/DEBIAN"
# Filled in by bash, never sed: no character of a dependency can break the line it goes in.
shopt -u patsub_replacement 2>/dev/null || :   # bash 5.2: an & in what replaces is that, not the match
control=$(< /deb/control.in)
control=${control//@VERSION@/$version}
control=${control//@ARCH@/$(dpkg --print-architecture)}
control=${control//@SIZE@/$(du -sk "$root/usr" | cut -f1)}
control=${control//@DEPENDS@/$depends}
printf '%s\n' "$control" > "$root/DEBIAN/control"
dpkg-deb --build --root-owner-group "$root" "/out/kitokey_${version}_$(dpkg --print-architecture).deb" >/dev/null
