#!/usr/bin/env bash
# D80, in the e2e's container, the package installed: Kitokey started as its user starts it, a
# meeting written, the package removed — nothing it installed is left, the user's home is as it
# was, and what Kitokey left there is what docs/privacy.md names —; installed again, Kitokey
# reads the meetings and the settings it left. Run as root by e2e.sh, which passes:
#   $add $remove $listed  the system's commands      $kept  privacy.md's paths, `~/…`, one a line
#   $tools                what drives the window     $driver  WebKitWebDriver's path
set -uo pipefail
failed=0
check() { if eval "$2"; then echo "  ok    $1"; else echo "  FAIL  $1"; failed=1; fi; }
eval "$tools" >/tmp/tools.log 2>&1 || { tail -5 /tmp/tools.log; echo "  FAIL  what drives the window installed"; exit 1; }
useradd -m meet
home=$(getent passwd meet | cut -d: -f6)
# As the applications' menu starts it, on a display and a session bus of its own. Automation is
# what tauri-driver turns on; WebKit's own sandbox needs namespaces a container does not give.
start() {
  runuser -u meet -- env HOME="$home" PYTHONUNBUFFERED=1 TAURI_WEBVIEW_AUTOMATION=true WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 \
    xvfb-run -a dbus-run-session -- timeout 180 python3 /start-as-user.py "$driver" >/tmp/start.out 2>/tmp/start.log
  grep '^{' /tmp/start.out | tail -1
}
# The home as a list: each path, its kind, and each file's content.
snapshot() {
  (cd "$home" && find . -mindepth 1 -printf '%p %y\n' | sort && find . -type f -exec sha256sum {} + | sort -k2)
}
welcomes() { cat "$home"/.local/state/kitokey/logs/shell.*.jsonl 2>/dev/null | grep -c '"shell.welcome_shown"'; }
before=$(cd "$home" && find . -mindepth 1 | sort)

first=$(start)
folder=$(python3 -c 'import json, sys; print(json.loads(sys.argv[1])["folder"])' "$first" 2>/dev/null)
check "its first start, as its user: the window opens and says where the meetings go" "test -n \"\$folder\" || { tail -3 /tmp/start.out; grep -v -E \"dbus-daemon|portal|fuse|^\$\" /tmp/start.log | tail -8; false; }"
check "the first start shows the welcome" "test \"\$(welcomes)\" = 1"
check "the first start writes the settings" "test -f \$home/.config/kitokey/settings.toml"
# The user chooses where the meetings go, as Settings writes it: only the settings read again can
# tell a later start of it — the default is the folder above.
chosen="$home/Meetings"
runuser -u meet -- sh -c "printf 'sessions = \"%s\"\n' '$chosen' >> '$home/.config/kitokey/settings.toml'"
# A meeting as capture leaves one (contract session-audio/v2), and its minutes beside it.
meeting="$chosen/2026-11-04_10-00"
runuser -u meet -- mkdir -p "$meeting"
runuser -u meet -- sh -c "printf '%s\n' '{\"contract\": \"session-audio/v2\", \"started\": \"2026-11-04T10:00:00+01:00\", \"stopped\": \"2026-11-04T10:30:00+01:00\", \"tracks\": {\"local\": {\"device\": \"Mic\"}}}' > '$meeting/session.json'
printf '# A meeting\n' > '$meeting/minutes.md'"
kept_then=$(snapshot)
files=$(eval "$listed" | while read -r path; do [ -d "$path" ] || echo "$path"; done)

check "removed" "eval \"\$remove\" >/tmp/remove.log 2>&1 || { tail -5 /tmp/remove.log; false; }"
left=$(for path in $files; do [ -e "$path" ] && echo "$path"; done)
check "no file the package installed is left" "test -z \"\$left\" || { echo \"        got: \$left\"; false; }"
check "the user's home is as it was: the meetings and the settings, untouched" "test \"\$(snapshot)\" = \"\$kept_then\""
# What is new in the home since Kitokey came: under a path privacy.md names, or the meetings'
# folder the user chose (« your meetings folder »), or the desktop's own folders any program with a window makes
# — the caches of Mesa's shaders, fontconfig and GStreamer, dconf, accessibility's bus, the
# document portal, GVfs, IBus — and the system's keyring, which Kitokey opens at its start: none
# is Kitokey's own (privacy.md names its key there).
allowed=$(printf '%s\n' "$kept" | sed "s|^~|.|"; echo ".${chosen#"$home"}"; printf '%s\n' ./.cache/mesa_shader_cache ./.cache/fontconfig ./.cache/gstreamer-1.0 ./.cache/dconf ./.cache/at-spi ./.cache/doc ./.cache/gvfsd ./.gvfs ./.config/ibus ./.local/share/keyrings)
new=$(comm -13 <(echo "$before") <(cd "$home" && find . -mindepth 1 | sort))
stray=$(while read -r path; do
  [ -n "$path" ] || continue
  named=no
  while read -r name; do
    case $path/ in "$name"/* ) named=yes ;; esac
    case $name/ in "$path"/* ) named=yes ;; esac   # a folder above a named one
  done <<< "$allowed"
  [ $named = yes ] || echo "$path"
done <<< "$new")
check "what Kitokey left in the home is what docs/privacy.md names" "test -z \"\$stray\" || { echo \"        got: \$stray\" | head -12; false; }"

check "installed again" "eval \"\$add\" >/tmp/add-again.log 2>&1 || { tail -5 /tmp/add-again.log; false; }"
again=$(start)
check "started again, it reads the settings it left: the meetings' folder the user chose" "python3 -c 'import json, sys; sys.exit(json.loads(sys.argv[1])[\"folder\"] != sys.argv[2])' \"\$again\" \"\$chosen\""
check "started again, it lists the meeting it left there" "python3 -c 'import json, sys; sys.exit(sys.argv[2] not in json.loads(sys.argv[1])[\"meetings\"])' \"\$again\" \"\$meeting\""
check "started again, no welcome a second time" "test \"\$(welcomes)\" = 1"
exit $failed
