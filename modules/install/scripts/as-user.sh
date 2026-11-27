#!/usr/bin/env bash
# Sourced in the e2e's container, as root, Kitokey installed: what a user of it does — its first
# start, a folder of their own for the meetings, one meeting there — and how a later start is read.
# Used by removal.sh (D80) and copr/upgrade-inside.sh (D78). Needs:
#   $tools  what drives the window, installed here     $driver  WebKitWebDriver's path
failed=0
check() { if eval "$2"; then echo "  ok    $1"; else echo "  FAIL  $1"; failed=1; fi; }
eval "$tools" >/tmp/tools.log 2>&1 || { tail -5 /tmp/tools.log; echo "  FAIL  what drives the window installed"; exit 1; }
# Made once: an upgrade sources this again in the same container.
id meet >/dev/null 2>&1 || useradd -m meet
home=$(getent passwd meet | cut -d: -f6)
# As the applications' menu starts it, on a display and a session bus of its own. Automation is
# what tauri-driver turns on; WebKit's own sandbox needs namespaces a container does not give.
start() {
  runuser -u meet -- env HOME="$home" PYTHONUNBUFFERED=1 TAURI_WEBVIEW_AUTOMATION=true WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 \
    xvfb-run -a dbus-run-session -- timeout 180 python3 /start-as-user.py "$driver" >/tmp/start.out 2>/tmp/start.log
  grep '^{' /tmp/start.out | tail -1
}
# The home as a list: each path, its kind, owner, group and mode, and each file's content.
snapshot() {
  (cd "$home" && find . -mindepth 1 -printf '%p %y %u %g %m\n' | sort && find . -type f -exec sha256sum {} + | sort -k2)
}
welcomes() { cat "$home"/.local/state/kitokey/logs/shell.*.jsonl 2>/dev/null | grep -c '"shell.welcome_shown"'; }
before=$(cd "$home" && find . -mindepth 1 | sort)
# Where the user keeps the meetings — not the default —, and the one they hold.
chosen="$home/Meetings"
meeting="$chosen/2026-11-04_10-00"

# Its first start, the folder the user chooses, a meeting written there.
first_use() {
  first=$(start)
  folder=$(python3 -c 'import json, sys; print(json.loads(sys.argv[1])["folder"])' "$first" 2>/dev/null)
  check "its first start, as its user: the window opens and says where the meetings go" "test -n \"\$folder\" || { tail -3 /tmp/start.out; grep -v -E \"dbus-daemon|portal|fuse|^\$\" /tmp/start.log | tail -8; false; }"
  check "the first start shows the welcome" "test \"\$(welcomes)\" = 1"
  check "the first start writes the settings" "test -f \$home/.config/kitokey/settings.toml"
  # The user chooses where the meetings go, as Settings writes it: only the settings read again can
  # tell a later start of it — the default is the folder above.
  runuser -u meet -- sh -c "printf 'sessions = \"%s\"\n' '$chosen' >> '$home/.config/kitokey/settings.toml'"
  # A meeting as capture leaves one (contract session-audio/v2), and its minutes beside it.
  runuser -u meet -- mkdir -p "$meeting"
  runuser -u meet -- sh -c "printf '%s\n' '{\"contract\": \"session-audio/v2\", \"started\": \"2026-11-04T10:00:00+01:00\", \"stopped\": \"2026-11-04T10:30:00+01:00\", \"tracks\": {\"local\": {\"device\": \"Mic\"}}}' > '$meeting/session.json'
  printf '# A meeting\n' > '$meeting/minutes.md'"
}
# Started again: the settings, the meeting and the welcome as the user left them.
reads_again() {
  again=$(start)
  check "started again, it reads the settings it left: the meetings' folder the user chose" "python3 -c 'import json, sys; sys.exit(json.loads(sys.argv[1])[\"folder\"] != sys.argv[2])' \"\$again\" \"\$chosen\""
  check "started again, it lists the meeting it left there" "python3 -c 'import json, sys; sys.exit(sys.argv[2] not in json.loads(sys.argv[1])[\"meetings\"])' \"\$again\" \"\$meeting\""
  check "started again, no welcome a second time" "test \"\$(welcomes)\" = 1"
}
