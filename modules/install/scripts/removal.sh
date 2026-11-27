#!/usr/bin/env bash
# D80, in the e2e's container, the package installed: Kitokey started as its user starts it, a
# meeting written, the package removed — nothing it installed is left, the user's home is as it
# was, and what Kitokey left there is what docs/privacy.md names —; installed again, Kitokey
# reads the meetings and the settings it left. Run as root by e2e.sh, which passes:
#   $add $remove $listed  the system's commands      $kept  privacy.md's paths, `~/…`, one a line
#                                                    $kept_fr  the same, read from its French part
#   $tools                what drives the window     $driver  WebKitWebDriver's path
# What a user does and how a start is read: as-user.sh, beside it.
set -uo pipefail
source /as-user.sh
first_use
kept_then=$(snapshot)
check "Kitokey is not running when the package is removed" "! pgrep -x kitokey >/dev/null"
files=$(eval "$listed" | while read -r path; do [ -d "$path" ] || echo "$path"; done)

check "removed" "eval \"\$remove\" >/tmp/remove.log 2>&1 || { tail -5 /tmp/remove.log; false; }"
left=$(for path in $files; do [ -e "$path" ] && echo "$path"; done)
check "no file the package installed is left" "test -z \"\$left\" || { echo \"        got: \$left\"; false; }"
check "the user's home is as it was: the meetings and the settings, untouched" "test \"\$(snapshot)\" = \"\$kept_then\""
# What is new in the home since Kitokey came: under a path privacy.md names, or the meetings'
# folder the user chose (« your meetings folder »), or the desktop's own folders any program with a window makes
# — the caches of Mesa's shaders, fontconfig and GStreamer, dconf, accessibility's bus, the
# document portal, GVfs, IBus —, everything under each; and the system's keyring's folder, which
# Kitokey's start makes open, itself alone — empty: no key was given (privacy.md names its key there).
allowed=$(printf '%s\n' "$kept" | sed "s|^~|.|"; echo ".${chosen#"$home"}"; printf '%s\n' ./.cache/mesa_shader_cache ./.cache/fontconfig ./.cache/gstreamer-1.0 ./.cache/dconf ./.cache/at-spi ./.cache/doc ./.cache/gvfsd ./.gvfs ./.config/ibus)
alone=./.local/share/keyrings
new=$(comm -13 <(echo "$before") <(cd "$home" && find . -mindepth 1 | sort))
stray=$(while read -r path; do
  [ -n "$path" ] || continue
  named=no
  [ "$path" = "$alone" ] && named=yes
  while read -r name; do
    case $path/ in "$name"/* ) named=yes ;; esac
    case $name/ in "$path"/* ) named=yes ;; esac   # a folder above a named one
  done <<< "$allowed"
  [ $named = yes ] || echo "$path"
done <<< "$new")
check "docs/privacy.md names the same Linux paths in French as in English" "test \"\$kept\" = \"\$kept_fr\" || { echo \"        got: \$kept_fr\"; false; }"
check "what Kitokey left in the home is what docs/privacy.md names" "test -z \"\$stray\" || { echo \"        got: \$stray\" | head -12; false; }"

check "installed again" "eval \"\$add\" >/tmp/add-again.log 2>&1 || { tail -5 /tmp/add-again.log; false; }"
reads_again
exit $failed
