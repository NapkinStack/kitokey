#!/usr/bin/env bash
# The module's e2e (scripts/e2e.sh) in a GNOME session of its own, never shown: what attest runs.
#
# A headless gnome-shell — a virtual monitor, the AppIndicator extension, its notifications, its
# portal — on a session bus of its own; a system bus of its own from the start, with nobody on it:
# logind, GDM and polkit are never asked, the user's session never registered nor locked; a
# PipeWire of its own with no hardware, where the run's microphone and output are made. Nothing
# of the user's desktop is reached: their screen, bus, notifications and sound stay theirs.
# Every process started carries the run's XDG_RUNTIME_DIR, and is ended by it.
#
#     scripts/headless.sh            (scripts/e2e.sh alone runs on the user's desktop, as before)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)

if [ "${1:-}" != --inside ]; then
  missing=""
  for program in gnome-shell dbus-run-session dbus-daemon gsettings pipewire wireplumber pipewire-pulse; do
    command -v "$program" >/dev/null || missing="$missing $program"
  done
  [ -d /usr/share/gnome-shell/extensions/appindicatorsupport@rgcjonas.gmail.com ] || missing="$missing AppIndicator-extension"
  if [ -n "$missing" ]; then
    mkdir -p "$here/../.evidence"
    echo "e2e: NOT RUN — no GNOME to run it in here, missing:$missing" | tee "$here/../.evidence/e2e.txt"; exit 77
  fi
  # A partial run's names checked before the session is made (scripts/e2e_only.py): a typo refused
  # at once, named, as e2e.sh would refuse it minutes later.
  if ! asked=$(python3 -I "$here/e2e_only.py" --asked "$here/e2e_window.py" 2>&1); then
    mkdir -p "$here/../.evidence"
    echo "$asked" | tee "$here/../.evidence/e2e.txt"; exit 3
  fi
  root=${XDG_CACHE_HOME:-$HOME/.cache}/kitokey-headless/run-$$
  mkdir -p "$root/run" "$root/user/.config" "$root/user/.local/share" "$root/user/.local/state" "$root/user/.cache" "$root/tmp"
  chmod 700 "$root/run"
  ended() {
    local p signal
    for signal in TERM KILL; do
      for p in /proc/[0-9]*; do
        grep -qzx "XDG_RUNTIME_DIR=$root/run" "$p/environ" 2>/dev/null && kill -s "$signal" "${p#/proc/}" 2>/dev/null || true
      done
      [ "$signal" = KILL ] || sleep 2
    done
    ! mountpoint -q "$root/run/doc" || fusermount3 -u "$root/run/doc" || true
    rm -rf "$root"
  }
  trap ended EXIT
  # Of the user's environment, their tools and caches and NapkinStack's, and the scenarios a partial
  # run plays (KITOKEY_E2E_ONLY, scripts/e2e_only.py); nothing of their desktop.
  mapfile -t kept < <(env | grep -E '^(NSTACK_[A-Z_]*|CARGO_TARGET_DIR|KITOKEY_SCREENSHOTS|KITOKEY_E2E_ONLY)=' || true)
  setsid env -i PATH="$PATH" USER="$USER" LOGNAME="$USER" LANG="${LANG:-C.UTF-8}" "${kept[@]}" \
    CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
    UV_CACHE_DIR="${UV_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/uv}" \
    HOME="$root/user" XDG_RUNTIME_DIR="$root/run" TMPDIR="$root/tmp" \
    XDG_CONFIG_HOME="$root/user/.config" XDG_DATA_HOME="$root/user/.local/share" \
    XDG_STATE_HOME="$root/user/.local/state" XDG_CACHE_HOME="$root/user/.cache" \
    XDG_SESSION_TYPE=wayland XDG_CURRENT_DESKTOP=GNOME KITOKEY_HEADLESS=1 \
    DBUS_SYSTEM_BUS_ADDRESS="unix:path=$root/run/system_bus_socket" \
    dbus-run-session -- "$0" --inside "$@" &
  status=0; wait $! || status=$?
  exit "$status"
fi
shift
# Only ever inside the session the first half made: run by hand, the settings below would be the user's.
case "${KITOKEY_HEADLESS:-}:${XDG_RUNTIME_DIR:-}" in
  1:*/kitokey-headless/run-*/run) ;;
  *) echo "e2e: NOT RUN — --inside is for the session headless.sh makes, never the user's" >&2; exit 2 ;;
esac
run=$XDG_RUNTIME_DIR; logs=$HOME/logs; mkdir -p "$logs"
cat > "$run/system-bus.conf" <<EOF
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig><type>system</type><listen>${DBUS_SYSTEM_BUS_ADDRESS}</listen><auth>EXTERNAL</auth>
<policy context="default"><allow send_destination="*" eavesdrop="true"/><allow eavesdrop="true"/><allow own="*"/></policy></busconfig>
EOF
dbus-daemon --config-file="$run/system-bus.conf" --nofork --nopidfile 2> "$logs/system-bus.log" &
until [ -S "$run/system_bus_socket" ]; do sleep 0.1; done

# The session's own extension: the desktop shown rather than the overview, a window shown given
# the focus as a click gives it, screenshots over the bus allowed (unsafe mode: this session's).
helper=$XDG_DATA_HOME/gnome-shell/extensions/headless@kitokey
mkdir -p "$helper"
printf '{"uuid": "headless@kitokey", "name": "Kitokey e2e", "description": "The e2e session", "shell-version": ["%s"]}\n' \
  "$(gnome-shell --version | grep -o '[0-9]*' | head -1)" > "$helper/metadata.json"
cat > "$helper/extension.js" <<'EOF'
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
export default class Headless extends Extension {
    enable() {
        global.context.unsafe_mode = true;
        this._started = Main.layoutManager.connect('startup-complete', () => Main.overview.hide());
        this._created = global.display.connect('window-created', (_display, window) => {
            const shown = window.connect('shown', () => { window.disconnect(shown); Main.activateWindow(window); });
        });
        this._attention = global.display.connect('window-demands-attention', (_display, window) => Main.activateWindow(window));
    }
    disable() {
        global.context.unsafe_mode = false;
        Main.layoutManager.disconnect(this._started);
        global.display.disconnect(this._created);
        global.display.disconnect(this._attention);
    }
}
EOF
# Its settings, in its own dconf: the extensions on, no lock, and the shortcut granted as the
# user's desktop grants it once asked — never asked here: the portal's dialogs would take the focus.
gsettings set org.gnome.shell disable-user-extensions false
gsettings set org.gnome.shell enabled-extensions "['appindicatorsupport@rgcjonas.gmail.com', 'headless@kitokey']"
gsettings set org.gnome.shell welcome-dialog-last-shown-version 999
gsettings set org.gnome.desktop.lockdown disable-lock-screen true
gsettings set org.gnome.desktop.screensaver lock-enabled false
gsettings set org.gnome.desktop.session idle-delay 0
gsettings set org.gnome.settings-daemon.global-shortcuts applications "['com.napkinstack.Kitokey']"
gsettings set org.gnome.settings-daemon.global-shortcuts.application:/org/gnome/settings-daemon/global-shortcuts/com.napkinstack.Kitokey/ \
  shortcuts "[('record', {'shortcuts': <['<Alt><Super>m']>, 'description': <'Start recording / Stop recording'>})]"

# The accessibility bus, started here: the session bus fails to start it (Fedora: "Permission denied").
/usr/libexec/at-spi-bus-launcher --launch-immediately 2> "$logs/a11y.log" &
# Sound with no hardware: WirePlumber's "policy" profile monitors no ALSA, Bluetooth nor camera.
pipewire 2> "$logs/pipewire.log" &
until [ -S "$run/pipewire-0" ]; do sleep 0.1; done
wireplumber --profile policy 2> "$logs/wireplumber.log" &
pipewire-pulse 2> "$logs/pipewire-pulse.log" &

gnome-shell --wayland --no-x11 --headless --virtual-monitor 1920x1080 --wayland-display kitokey-0 > "$logs/shell.log" 2>&1 &
export WAYLAND_DISPLAY=kitokey-0
watcher="org.kde.StatusNotifierWatcher /StatusNotifierWatcher org.kde.StatusNotifierWatcher"
for _ in $(seq 300); do
  busctl --user get-property $watcher RegisteredStatusNotifierItems >/dev/null 2>&1 && pactl info >/dev/null 2>&1 && break
  sleep 0.1
done
/usr/libexec/at-spi2-registryd 2> "$logs/registryd.log" &
# A keyboard, as a person's session has one: a remote-desktop session's. Without one, no window is
# ever told it has the focus — GTK learns it from the keyboard entering it. The system's Python,
# where PyGObject is: the first on the path may be a tool's own (nstack attest puts its own first).
/usr/bin/python3 - > "$logs/keyboard" 2> "$logs/keyboard.log" <<'EOF' &
from gi.repository import Gio, GLib
bus = Gio.bus_get_sync(Gio.BusType.SESSION)
def call(path, interface, method, args=None, reply=None):
    return bus.call_sync("org.gnome.Mutter.RemoteDesktop", path, interface, method, args, reply, Gio.DBusCallFlags.NONE, -1, None)
session = call("/org/gnome/Mutter/RemoteDesktop", "org.gnome.Mutter.RemoteDesktop", "CreateSession", None, GLib.VariantType("(o)"))[0]
call(session, "org.gnome.Mutter.RemoteDesktop.Session", "Start")
for pressed in (True, False):
    call(session, "org.gnome.Mutter.RemoteDesktop.Session", "NotifyKeyboardKeysym", GLib.Variant("(ub)", (0xffe1, pressed)))
print("ready", flush=True)
GLib.MainLoop().run()
EOF
for _ in $(seq 100); do [ -s "$logs/keyboard" ] && break; sleep 0.1; done
if [ ! -s "$logs/keyboard" ]; then
  mkdir -p "$here/../.evidence"
  echo "e2e: FAILED — the session has no keyboard, so no window would have the focus:" | tee "$here/../.evidence/e2e.txt"
  sed 's/^/    /' "$logs/keyboard.log" | tee -a "$here/../.evidence/e2e.txt"; exit 1
fi

status=0
"$here/e2e.sh" "$@" || status=$?
# The session's own log beside the run's evidence, never in it: what GNOME says of a session with
# no logind nor accounts is no line of Kitokey's, and NapkinStack audits every log there.
cp "$logs/shell.log" "$here/../.evidence/headless-shell.log" 2>/dev/null || true
exit "$status"
