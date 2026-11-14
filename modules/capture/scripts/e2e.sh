#!/usr/bin/env bash
# The module's end-to-end scenarios (cycle 01, D3), on sound made here and on devices made
# for the run: nobody is recorded. The run is sealed off the computer's sound (2026-11-13: on the
# desktop's graph, AirPods connected during a run were taken as the default input):
# - in a bubblewrap of its own (bwrap, Flatpak's sandbox): a /dev with no /dev/snd — no sound card
#   can be opened, through PipeWire or ALSA directly —, its own processes, and no desktop's
#   runtime folder nor system bus — the desktop's graph and Bluetooth cannot be reached;
# - on a PipeWire, WirePlumber and pipewire-pulse of its own, as private_graph.sh's, no hardware
#   monitored; a guard stops the run, NOT RUN, at anything there not its own.
# The desktop's defaults are never moved: read before and after, outside the seal, the same.
# Where PipeWire or bubblewrap is missing — a CI runner — it says so and runs nothing.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .evidence
say() { echo "$*" | tee -a .evidence/e2e.txt; }
# The run's logs, where NapkinStack reads them after it (nstack e2e capture).
evidence=${NSTACK_EVIDENCE:-$PWD/.evidence/e2e}

if [ -z "${KITOKEY_E2E_SEALED:-}" ]; then
  : > .evidence/e2e.txt
  command -v bwrap > /dev/null || { say "e2e: NOT RUN — bwrap (bubblewrap) is missing"; exit 77; }
  command -v pactl > /dev/null || { say "e2e: NOT RUN — pactl is missing"; exit 77; }
  desktop_defaults() { echo "$(pactl get-default-sink 2>/dev/null || echo none) $(pactl get-default-source 2>/dev/null || echo none)"; }
  desktop_before=$(desktop_defaults)
  # A /dev of its own, without /dev/snd; its own /proc and processes, all ended with it; the
  # desktop's runtime folder (its PipeWire, its pulse) and the system bus (Bluetooth) hidden.
  sealed=(bwrap --dev-bind / / --dev /dev --tmpfs /dev/shm --unshare-pid --proc /proc --die-with-parent)
  for hidden in "${XDG_RUNTIME_DIR:-}" "/run/user/$(id -u)" /run/dbus /run/pulse /run/pipewire; do
    [ -n "$hidden" ] && [ -d "$hidden" ] && sealed+=(--tmpfs "$hidden")
  done
  "${sealed[@]}" -- true 2>> .evidence/e2e.txt ||
    { say "e2e: NOT RUN — this system refuses the run a namespace of its own (bwrap)"; exit 77; }
  status=0
  "${sealed[@]}" --setenv KITOKEY_E2E_SEALED 1 -- bash "$(realpath "$0")" "$@" || status=$?
  say "The desktop's defaults, before and after the run: $desktop_before"
  if [ "$(desktop_defaults)" = "$desktop_before" ]; then
    say "  ok    the desktop's defaults the same"
  else
    say "  FAIL  the desktop's defaults now $(desktop_defaults): the run never speaks to them — a device plugged in meanwhile?"
    [ "$status" = 0 ] && status=1
  fi
  exit "$status"
fi

# Sealed from here. No sound card in sight: the first of the guard's assertions.
if [ -e /dev/snd ] || { command -v aplay > /dev/null && ! aplay -l 2>&1 | grep -q 'no soundcards'; }; then
  say "e2e: NOT RUN — a sound card is in sight inside the seal: $(ls /dev/snd 2>/dev/null | tr '\n' ' ')"; exit 77
fi
say "Sealed: /dev holds $(ls /dev | tr '\n' ' ')"

for tool in pipewire wireplumber pipewire-pulse pactl ffmpeg ffprobe pw-play pw-dump pw-cli pw-loopback pw-metadata python3; do
  command -v "$tool" >/dev/null || { say "e2e: NOT RUN — $tool is missing"; exit 77; }
done

cargo build --release --locked 2>&1 | tail -3 >> .evidence/e2e.txt
# Its clocks are measured against the devices' own, which a loaded machine stamps late: a
# quiet one is waited for, a few minutes at most — the build just made counts too (#281).
quiet=$(( $(nproc) > 1 ? $(nproc) / 2 : 1 ))   # one processor: under 1
for _ in $(seq 60); do [ "$(cut -d. -f1 /proc/loadavg)" -lt "$quiet" ] && break; sleep 5; done
if [ "$(cut -d. -f1 /proc/loadavg)" -ge "$quiet" ]; then
  say "e2e: NOT RUN — the machine is busy (load $(cut -d' ' -f1 /proc/loadavg), $(nproc) processors): a clock cannot be measured on it"; exit 77
fi
bin=$(realpath "${CARGO_TARGET_DIR:-$PWD/target}/release/kitokey-capture")   # where cargo built it
work=$(mktemp -d)
# Its logs in the run's evidence, this run's alone, never the user's (ADR-0007).
export KITOKEY_LOG_DIR="$evidence/logs"
rm -rf "$KITOKEY_LOG_DIR"

# The run's own graph, as private_graph.sh's. A socket's path is short: never under a long folder.
graph=$(mktemp -d /tmp/wh-XXXXXX)
export XDG_RUNTIME_DIR=$graph/rt PIPEWIRE_RUNTIME_DIR=$graph/rt XDG_CONFIG_HOME=$graph/cfg XDG_STATE_HOME=$graph/state
export XDG_CACHE_HOME=$graph/cache XDG_DATA_HOME=$graph/data DBUS_SESSION_BUS_ADDRESS=disabled:
unset PIPEWIRE_REMOTE PULSE_SERVER DISPLAY WAYLAND_DISPLAY
mkdir -p "$graph/rt" "$graph/cfg/wireplumber/wireplumber.conf.d" "$graph/cfg/pipewire/pipewire-pulse.conf.d"
chmod 700 "$graph/rt"
cat > "$graph/cfg/wireplumber/wireplumber.conf.d/90-no-hardware.conf" <<CONF
wireplumber.profiles = { main = { monitor.alsa = disabled, monitor.bluez = disabled, monitor.bluez-midi = disabled, monitor.alsa-midi = disabled, monitor.v4l2 = disabled, monitor.libcamera = disabled, support.logind = disabled, support.reserve-device = disabled, support.portal-permissionstore = disabled } }
CONF
# pipewire-pulse's fallback sink, `auto_null`, off where its own configuration says it is: only the
# run's sinks in this graph (2026-11-13, a guard stopping every run at it).
cat > "$graph/cfg/pipewire/pipewire-pulse.conf.d/90-no-always-sink.conf" <<CONF
pulse.properties = { pulse.cmd.always-sink = false }
CONF
pipewire > "$graph/pipewire.log" 2>&1 & daemon=$!
for _ in $(seq 20); do [ -S "$graph/rt/pipewire-0" ] && break; sleep 0.25; done
wireplumber > "$graph/wireplumber.log" 2>&1 & manager=$!
pipewire-pulse > "$graph/pipewire-pulse.log" 2>&1 & pulse=$!
for _ in $(seq 20); do pactl info > /dev/null 2>&1 && break; sleep 0.25; done
restore() {
  status=$?
  # Cleaning is never cut short: `timeout` signals the script, then its whole group, a second TERM.
  trap '' TERM INT HUP
  pkill -P $$ 2>/dev/null || true
  # The graph's daemons by their PIDs, killed outright if they have not ended within two seconds.
  kill "$pulse" "$manager" "$daemon" 2>/dev/null || true
  for _ in $(seq 20); do
    alive=0; for p in "$pulse" "$manager" "$daemon"; do kill -0 "$p" 2>/dev/null && alive=1; done
    [ "$alive" = 0 ] && break; sleep 0.1
  done
  kill -KILL "$pulse" "$manager" "$daemon" 2>/dev/null || true
  wait "$pulse" "$manager" "$daemon" 2>/dev/null || true
  rm -rf "$work" "$graph"
  if [ -e "$work.real" ]; then
    say "e2e: NOT RUN — a real device appeared: $(cat "$work.real")"
    # The guard's trial passes only stopped by its intruder: stopped by anything else, or not at all, it fails.
    if [ -n "${E2E_INTRUDER:-}" ] && ! grep -q "intruder_${E2E_INTRUDER%%:*}" "$work.real"; then
      say "  FAIL  the guard stopped the run, not at intruder_${E2E_INTRUDER%%:*}"; rm -f "$work.real" "$work.trial"; exit 1
    fi
    rm -f "$work.real" "$work.trial"; exit 77
  fi
  if [ -e "$work.trial" ]; then say "  FAIL  $(cat "$work.trial")"; rm -f "$work.trial"; exit 1; fi
  if [ -n "${E2E_INTRUDER:-}" ]; then say "  FAIL  the guard let intruder_${E2E_INTRUDER%%:*} through"; exit 1; fi
  exit "$status"
}
trap restore EXIT
# A TERM is a stop, never a skip: the guard's own says NOT RUN through "$work.real", read in restore.
trap 'exit 143' TERM
trap 'exit 130' INT
trap 'exit 129' HUP

# The guard, the whole run long, failing closed: the graph read (an answer late or empty stops the
# run), no device in it, no source or sink but the run's own `kitokey_e2e_*`, and no process of the
# run — every process in the seal is — holding anything under /dev/snd (`pcm*`, `control*`). One seen:
# every capture and the graph's daemon killed at once; the script ends at its next step (a scenario's
# `sleep` runs out first, the sound already cut), NOT RUN — exit 77, as a run that cannot run here
# (.nstack/handbook/09-platform.md).
real_devices() {
  PATH="${trial_bin:-}$PATH" python3 - <<'GUARD'
import json, os, subprocess, sys
seen = []
try:
    dump = json.loads(subprocess.run(["pw-dump"], capture_output=True, text=True, timeout=3).stdout or "null")
except (subprocess.TimeoutExpired, ValueError):
    dump = None
if not dump:
    seen.append("the run's graph did not answer pw-dump: nothing seen is not nothing there")
# A device of any kind, or a source or sink not the run's own: streams (Stream/…) are the programs'.
for obj in dump or []:
    props = (obj.get("info") or {}).get("props") or {}
    name = str(props.get("node.name") or props.get("device.name") or obj.get("id"))
    device = obj.get("type", "").endswith(":Device")
    stranger = str(props.get("media.class", "")).startswith("Audio/") and not name.startswith("kitokey_e2e_")
    if device or stranger:
        seen.append(f"{name} ({props.get('media.class') or obj.get('type')})")
for pid in filter(str.isdigit, os.listdir("/proc")):
    try:
        held = [os.readlink(f"/proc/{pid}/fd/{fd}") for fd in os.listdir(f"/proc/{pid}/fd")]
    except OSError:
        continue
    seen += [f"{card} held by {pid}" for card in held if card.startswith("/dev/snd/")]
print(", ".join(seen))
sys.exit(1 if seen else 0)
GUARD
}
stop_all() {
  echo "$1" > "$work.real"
  for pid in /proc/[0-9]*; do
    [ "$(readlink "$pid/exe" 2>/dev/null)" = "$bin" ] && kill -KILL "${pid#/proc/}" 2>/dev/null
  done
  kill -KILL "$daemon" 2>/dev/null || true
  kill -TERM $$
}
# The device trial's intruder: no Device factory runs without hardware or a bus (the installed SPA
# ones are api.alsa.{pcm,acp,compress.offload}.device, api.bluez5.device, api.libcamera.device), and
# `api.alsa.enum.udev` in the seal made none (2026-11-14). The guard's pw-dump answers for it then: the
# graph's own dump, and the Device as pw-dump prints one once the trial brings it.
if [[ ${E2E_INTRUDER:-} == device:* ]]; then
  mkdir -p "$graph/trial" && trial_bin=$graph/trial:
  cat > "$graph/trial/pw-dump" <<SHIM
#!/bin/sh
$(command -v pw-dump) | python3 -c '
import json, os, sys
dump = json.load(sys.stdin)
if os.path.exists("$graph/intruder_device"):
    dump.append({"id": 100000, "type": "PipeWire:Interface:Device", "info": {"props": {"device.name": "intruder_device"}}})
json.dump(dump, sys.stdout)'
SHIM
  chmod +x "$graph/trial/pw-dump"
fi
if ! seen=$(real_devices); then stop_all "$seen"; fi
( while sleep 0.5; do seen=$(real_devices) || { stop_all "$seen"; break; }; done ) & guard=$!
# The guard's own trial: E2E_INTRUDER=<sink|source|device>:<seconds> brings one not the run's that
# long after the start, as a headset or a card appearing — the run must end NOT RUN, exit 77, naming it
# (`intruder_sink`, `intruder_source`, `intruder_device`); stopped by anything else, or not, it fails.
# The trial ends at once when it proves nothing: its intruder not in the guard's pw-dump within 3 s,
# or there and the run not stopped 3 s later — never a whole run to learn it.
if [ -n "${E2E_INTRUDER:-}" ]; then
  (
    kind=${E2E_INTRUDER%%:*}
    sleep "${E2E_INTRUDER#*:}"
    case $kind in
      sink) pactl load-module module-null-sink sink_name=intruder_sink > /dev/null ;;
      source) pw-cli create-node adapter factory.name=support.null-audio-sink node.name=intruder_source \
                media.class=Audio/Source/Virtual object.linger=true audio.position=[FL,FR] > /dev/null ;;
      device) touch "$graph/intruder_device" ;;
    esac
    there=
    for _ in $(seq 12); do
      if PATH="${trial_bin:-}$PATH" pw-dump 2>/dev/null | grep -q "\"intruder_$kind\""; then there=1; break; fi
      sleep 0.25
    done
    if [ -z "$there" ]; then   # the guard may have stopped the graph first: then it saw it
      [ -e "$work.real" ] || { echo "the trial's intruder did not appear: intruder_$kind" > "$work.trial"; kill -TERM $$; }
      exit 0
    fi
    sleep 3
    echo "the guard let intruder_$kind through" > "$work.trial"; kill -TERM $$
  ) &
fi

out=$(pactl load-module module-null-sink sink_name=kitokey_e2e_out)
mic=$(pactl load-module module-null-sink sink_name=kitokey_e2e_mic)
pactl set-default-sink kitokey_e2e_out
pactl set-default-source kitokey_e2e_mic.monitor

# "me" is a 440 Hz note; "others" is an 880 Hz one, with fourteen seconds of nothing in it.
tone() { ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=$1:duration=$2" -af volume=0.5 -ar 48000 -ac 2 "$3"; }
tone 440 40 "$work/me.wav"; tone 880 8 "$work/others-a.wav"; tone 880 8 "$work/others-b.wav"

measure() { python3 scripts/e2e_measure.py "$@" 2>&1 | tee -a .evidence/e2e.txt; return "${PIPESTATUS[0]}"; }
# Whether the run of an id logged these steps, in this order among its own, no level, and never a path of the run's folder.
logged() {
  python3 - "$work" "$@" <<'PY'
import json, os, pathlib, sys
work, trace, wanted = sys.argv[1], sys.argv[2], sys.argv[3:]
text = "".join(path.read_text() for path in sorted(pathlib.Path(os.environ["KITOKEY_LOG_DIR"]).glob("capture.*.jsonl")))
steps = [line["message"] for line in map(json.loads, text.splitlines()) if line.get("span", {}).get("trace") == trace]
mine = iter(steps)
# The levels, ten a second, are said to the shell and never logged.
sys.exit(0 if all(step in mine for step in wanted) and work not in text and "capture.level" not in steps else f"{trace}: {steps}")
PY
}
failed=0
check() { if "$@"; then say "  ok    $*"; else say "  FAIL  $*"; failed=1; fi; }

say "A. a recording stopped as a user stops it"
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
sleep 0.5
"$bin" record "$work/sessions" --as v1 > "$work/a.events" 2> "$work/a.errors" &
recorder=$!
sleep 1; pw-play "$work/others-a.wav"                    # 8 s of the others
sleep 4; folder=$(ls -d "$work"/sessions/*/ | head -1)
cp "$folder/others.wav" "$work/copy.wav"; copied_at=$(date +%s.%N)   # read while it is written
sleep 10                                                 # 14 s of nothing: "silent" is owed
pw-play "$work/others-b.wav"; sleep 1                    # and "heard" when it comes back
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
sed 's/^/    /' "$work/a.events" "$work/a.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/a.errors"
check measure events "$work/a.events"
check measure stopped "$folder" 440 880
check measure read-while-written "$work/copy.wav" "$folder/session.json" "$copied_at" 5
# Each default is named for the device it stands for, not as "default_input" or "default_output".
check grep -q '"me":"kitokey_e2e_mic' "$work/a.events"
check grep -q '"others":"kitokey_e2e_out' "$work/a.events"

say "B. a recording killed"
kill "$mic_player" 2>/dev/null || true
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
sleep 0.5
"$bin" record "$work/killed" --as v1 > "$work/b.events" 2>/dev/null &
recorder=$!
sleep 1; pw-play "$work/others-a.wav" & others_player=$!
sleep 7
kill -KILL "$recorder"; killed_at=$(date +%s.%N); wait "$recorder" 2>/dev/null || true
kill "$mic_player" "$others_player" 2>/dev/null || true
folder=$(ls -d "$work"/killed/*/ | head -1)
sed 's/^/    /' "$work/b.events" >> .evidence/e2e.txt
check measure killed "$folder" "$killed_at" 5

say "C. the two files on one clock"
# The same sound into both sources: what differs between the two files is the program's doing.
pactl set-default-source kitokey_e2e_out.monitor
"$bin" record "$work/clock" --as v1 > "$work/c.events" 2>/dev/null &
recorder=$!
sleep 2; pw-play "$work/others-a.wav"; sleep 1
kill -INT "$recorder"; wait "$recorder" || true
check measure same-clock "$(ls -d "$work"/clock/*/ | head -1)" 0.1

say "D. whoever listened is gone"
# The reader takes "started" and leaves: every later event is said to a closed pipe.
mkfifo "$work/d.pipe"
"$bin" record "$work/alone" --as v1 > "$work/d.pipe" 2> "$work/d.errors" &
recorder=$!
head -n 1 "$work/d.pipe" > "$work/d.events"
sleep 13                                                 # "silent" is owed at ten seconds, to nobody
kill -TERM "$recorder" 2>/dev/null || true; left_as=0; wait "$recorder" || left_as=$?
sed 's/^/    /' "$work/d.events" "$work/d.errors" >> .evidence/e2e.txt
check test "$left_as" = 0 -a ! -s "$work/d.errors"
check grep -q '"stopped"' "$(ls -d "$work"/alone/*/ | head -1)session.json"

# The same three, each source as delivered (session-audio/v2, ADR-0004): what `record` writes
# unasked — E asks nothing, the others ask `--as v2`, which is the same. Their notes are
# made to tell a recorder that changes them: the second channel at half the first's level, so
# that channels mixed or swapped show, both resting away from zero, so that centring does, and
# in 24 bits, so that a track cut to 16 does.
tone() { ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=$1:duration=$2" -af "volume=0.5,pan=stereo|c0=c0|c1=0.5*c0,dcshift=0.02" -ar 48000 -c:a pcm_s24le "$3"; }
tone 440 40 "$work/me.wav"; tone 880 8 "$work/others-a.wav"; tone 880 8 "$work/others-b.wav"
# How many gaps of zeros the run of an id logged — and where the first was, in seconds.
zeros() {
  python3 - "$work" "$@" <<'PY'
import json, os, pathlib, sys
work, trace, count = sys.argv[1], sys.argv[2], int(sys.argv[3])
lines = [json.loads(line) for path in pathlib.Path(os.environ["KITOKEY_LOG_DIR"]).glob("capture.*.jsonl") for line in path.read_text().splitlines()]
gaps = [line for line in lines if line.get("span", {}).get("trace") == trace and line["message"] == "capture.zeros"]
first = float(sys.argv[4]) if len(sys.argv) > 4 else None
ok = len(gaps) == count and all(gap["track"] == "remote" for gap in gaps) and (first is None or abs(gaps[0]["at"] - first) < 0.5)
sys.exit(0 if ok else f"{trace}: {gaps}")
PY
}
measure2() { python3 scripts/e2e_measure_v2.py "$@" 2>&1 | tee -a .evidence/e2e.txt; return "${PIPESTATUS[0]}"; }
pactl set-default-source kitokey_e2e_mic.monitor

say "E. each source as delivered: a recording stopped as a user stops it"
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
sleep 0.5
KITOKEY_TRACE=e2e-e "$bin" record "$work/v2" > "$work/e.events" 2> "$work/e.errors" &
recorder=$!
sleep 1; pw-play "$work/others-a.wav"
folder=$(ls -d "$work"/v2/*/ | head -1)
cp "$folder/local.wav" "$work/e-local.wav"; cp "$folder/remote.wav" "$work/e-remote.wav"   # what the WAVs hold then
sleep 14                                                 # nothing plays: "silent" is owed
pw-play "$work/others-b.wav"; sleep 1
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
kill "$mic_player" 2>/dev/null || true
sed 's/^/    /' "$work/e.events" "$work/e.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/e.errors"
check measure events "$work/e.events"
# Kept as FLAC once stopped (session-audio/v3, D87): the notes measured in them as in the WAVs.
check measure2 stopped "$folder" 440 880
check measure2 kept "$folder" "$work/e-local.wav" "$work/e-remote.wav"
check logged e2e-e capture.opened capture.opened capture.started capture.silent capture.heard capture.stopped capture.kept capture.kept

say "E2. kept as FLAC: killed while it compresses, and a second signal (D87)"
# Three recordings of a minute at once, each stopped, then: killed as its first FLAC is being written,
# killed as it is renamed, and a second signal as it is being written. Where the moment is caught
# depends on the machine: whatever is left is judged whole.
tone 440 75 "$work/me-long.wav"; tone 880 75 "$work/others-75.wav"
pw-play --target kitokey_e2e_mic "$work/me-long.wav" & mic_player=$!
pw-play "$work/others-75.wav" & others_player=$!
sleep 0.5
for run in part flac again; do
  KITOKEY_TRACE=e2e-e2-$run "$bin" record "$work/kept-$run" > /dev/null 2> "$work/e2-$run.errors" &
  eval "recorder_$run=$!"
done
sleep 65
signallers=()
for run in part flac again; do
  folder=$(ls -d "$work/kept-$run"/*/ | head -1)
  pid=$(eval echo "\$recorder_$run")
  case $run in
    part) python3 scripts/e2e_measure_v2.py signal-at "$folder" "$pid" SIGKILL local.flac.part & ;;
    flac) python3 scripts/e2e_measure_v2.py signal-at "$folder" "$pid" SIGKILL local.flac & ;;
    again) python3 scripts/e2e_measure_v2.py signal-at "$folder" "$pid" SIGTERM local.flac.part & ;;
  esac
  signallers+=("$!")
done
sleep 0.5
kill -INT "$recorder_part" "$recorder_flac" "$recorder_again"
wait "$recorder_part" "$recorder_flac" 2>/dev/null || true
again_as=0; wait "$recorder_again" || again_as=$?
# Its signallers alone: the graph's daemons are children of this script too, and never end.
wait "${signallers[@]}" 2>/dev/null || true
kill "$mic_player" "$others_player" 2>/dev/null || true
for run in part flac again; do
  folder=$(ls -d "$work/kept-$run"/*/ | head -1)
  say "    $run: $(ls "$folder" | tr '\n' ' ')"
  check measure2 whole "$folder"
done
check test "$again_as" = 0 -a ! -s "$work/e2-again.errors"
check test -z "$(ls "$(ls -d "$work"/kept-again/*/ | head -1)" | grep '\.part$')"

say "F. each source as delivered: a recording killed"
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
sleep 0.5
"$bin" record "$work/v2-killed" --as v2 > /dev/null 2>&1 &
recorder=$!
sleep 1; pw-play "$work/others-a.wav" & others_player=$!
sleep 7
kill -KILL "$recorder"; killed_at=$(date +%s.%N); wait "$recorder" 2>/dev/null || true
kill "$mic_player" "$others_player" 2>/dev/null || true
check measure2 killed "$(ls -d "$work"/v2-killed/*/ | head -1)" "$killed_at" 5

say "G. each source as delivered: the clock places the same sound at the same instant"
pactl set-default-source kitokey_e2e_out.monitor
"$bin" record "$work/v2-clock" --as v2 > /dev/null 2>&1 &
recorder=$!
sleep 2; pw-play "$work/others-a.wav"; sleep 1
kill -INT "$recorder"; wait "$recorder" || true
check measure2 same-instant "$(ls -d "$work"/v2-clock/*/ | head -1)" 0.005

say "H. each source as delivered: a source that stops, and comes back — twice"
pactl set-default-source kitokey_e2e_mic.monitor
tone 880 30 "$work/others-long.wav"
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
pw-play "$work/others-long.wav" & others_player=$!
sleep 0.5
KITOKEY_TRACE=e2e-h "$bin" record "$work/v2-stops" --as v2 > /dev/null 2>&1 &
recorder=$!
sleep 4; python3 scripts/e2e_measure_v2.py pause "$recorder" remote 6   # six seconds without the remote source
# Back for three seconds, then gone again: each gap is logged, the second as the first (#285).
sleep 3; python3 scripts/e2e_measure_v2.py pause "$recorder" remote 4
sleep 3
kill -INT "$recorder"; wait "$recorder" || true
kill "$mic_player" "$others_player" 2>/dev/null || true
check measure2 stopped-source "$(ls -d "$work"/v2-stops/*/ | head -1)"
check logged e2e-h capture.started capture.zeros capture.stopped
check zeros e2e-h 2

say "I. each source as delivered: a source mute from its start"
# Its stream paused as soon as it is there: it delivers nothing, and its zeros are logged once.
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
sleep 0.5
KITOKEY_TRACE=e2e-i "$bin" record "$work/v2-mute" --as v2 > /dev/null 2>&1 &
recorder=$!
for _ in $(seq 40); do python3 scripts/e2e_measure_v2.py pause "$recorder" remote 5 2>/dev/null && break; sleep 0.05; done
pw-play "$work/others-a.wav" & others_player=$!
sleep 3
kill -INT "$recorder"; wait "$recorder" || true
kill "$mic_player" "$others_player" 2>/dev/null || true
check zeros e2e-i 1 0

say "J. a recording that follows the devices (D31): switched to others of their own form, and one taken away"
# A microphone of one channel at 16 kHz and an output of six at 44.1 kHz: the tracks keep the
# form they opened with. Every note is played on a device of the run, never on the system's.
mic_b=$(pactl load-module module-null-sink sink_name=kitokey_e2e_mic_b rate=16000 channels=1)
out_b=$(pactl load-module module-null-sink sink_name=kitokey_e2e_out_b rate=44100 channels=6 channel_map=surround-51)
tone 660 4 "$work/660.wav"; tone 1320 4 "$work/1320.wav"; tone 440 4 "$work/440.wav"; tone 880 4 "$work/880.wav"
both() { pw-play --target "$1" "$work/$2.wav" & local p=$!; pw-play --target "$3" "$work/$4.wav"; wait "$p"; }
KITOKEY_TRACE=e2e-j "$bin" record "$work/v2-moved" --as v2 > "$work/j.events" 2> "$work/j.errors" &
recorder=$!
# The first form, beside it: its lines are the same.
"$bin" record "$work/v1-moved" --as v1 > "$work/j1.events" 2> /dev/null &
recorder_v1=$!
sleep 1; both kitokey_e2e_mic 440 kitokey_e2e_out 880
pactl set-default-source kitokey_e2e_mic_b.monitor; pactl set-default-sink kitokey_e2e_out_b
sleep 1; both kitokey_e2e_mic_b 660 kitokey_e2e_out_b 1320
cp "$(ls -d "$work"/v2-moved/*/ | head -1)session.json" "$work/j-during.json"   # read while it records
# The microphone chosen back; the output recorded taken away: the system names the next.
pactl set-default-source kitokey_e2e_mic.monitor
pactl unload-module "$mic_b"
pactl unload-module "$out_b"
sleep 1; pactl set-default-sink kitokey_e2e_out
sleep 1; both kitokey_e2e_mic 440 kitokey_e2e_out 880
sleep 1
kill -INT "$recorder" "$recorder_v1"; stopped_as=0; wait "$recorder" || stopped_as=$?; wait "$recorder_v1" || true
grep -v '"level"' "$work/j.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/j.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/j.errors"
check measure2 followed "$(ls -d "$work"/v2-moved/*/ | head -1)" "$work/j.events" \
  "kitokey_e2e_mic_b:660,kitokey_e2e_mic:440" "kitokey_e2e_out_b:1320,kitokey_e2e_out:880" "$work/j-during.json"
check measure2 moves-said "$work/j1.events" "kitokey_e2e_mic_b,kitokey_e2e_mic" "kitokey_e2e_out_b,kitokey_e2e_out"
check logged e2e-j capture.started capture.device capture.device capture.stopped

say "K. a call heard on an output other than the default is recorded there (D41)"
# The call's own output, beside the default; the stand-in call, one process as a call is: it
# records the microphone and plays what it hears on that output. Nobody is recorded.
out_c=$(pactl load-module module-null-sink sink_name=kitokey_e2e_call)
tone 1100 4 "$work/1100.wav"; tone 660 4 "$work/660.wav"; tone 440 4 "$work/440.wav"
pw-loopback --capture=kitokey_e2e_mic.monitor --playback=kitokey_e2e_call --name=kitokey-e2e-call & call=$!
sleep 1
KITOKEY_TRACE=e2e-k "$bin" record "$work/v2-call" --as v2 > "$work/k.events" 2> "$work/k.errors" &
recorder=$!
sleep 3; pw-play --target kitokey_e2e_mic "$work/1100.wav"   # heard by the call, played on its output
# The call goes back to the default output: so does the remote side, within two seconds.
playing=$(python3 scripts/e2e_measure_v2.py playing "$call")
pw-metadata -n default "$playing" target.object kitokey_e2e_out > /dev/null
sleep 2; pw-play --target kitokey_e2e_call "$work/660.wav"   # two seconds after: the remote side followed, not on it
sleep 1; pw-play --target kitokey_e2e_out "$work/440.wav"
sleep 1
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
pactl unload-module "$out_c"
grep -v '"level"' "$work/k.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/k.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/k.errors"
check measure2 call-followed "$(ls -d "$work"/v2-call/*/ | head -1)" "$work/k.events" kitokey_e2e_call kitokey_e2e_out 1100 660 440
check logged e2e-k capture.started capture.device capture.device capture.stopped

say "L. the microphone a call holds, and a headset's never opened actively (D66, ADR-0015)"
# A source of the run's, fed by a device of the run's, made as WirePlumber makes a headset's virtual
# input: stand_in NAME [more properties of the source]. No Bluetooth card: its script is not at work.
mic_c=$(pactl load-module module-null-sink sink_name=kitokey_e2e_mic_c)
stand_in() {
  pw-loopback --name="$1-loopback" \
    --capture-props="{ node.name=$1_in media.class=Stream/Input/Audio/Internal target.object=${feed:-kitokey_e2e_mic_c} stream.capture.sink=true node.passive=true node.dont-fallback=true node.linger=true ${2:-} }" \
    --playback-props="{ node.name=$1 node.description=$1 media.class=Audio/Source ${2:-} }" &
}
call_on() { pw-record --target "$1" "$work/call-$$.wav" & }
tone 660 30 "$work/660-long.wav"
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & mic_player=$!

# A call on a microphone other than the default: recorded, named, and left when it leaves.
stand_in kitokey_e2e_other; other=$!
sleep 1
KITOKEY_TRACE=e2e-l "$bin" record "$work/v2-mic" --as v2 > "$work/l.events" 2> "$work/l.errors" &
recorder=$!
sleep 2; call_on kitokey_e2e_other; call=$!
sleep 4; folder=$(ls -d "$work"/v2-mic/*/ | head -1); cp "$folder/session.json" "$work/l-during.json"
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
sleep 3
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
kill "$other" 2>/dev/null || true; wait "$other" 2>/dev/null || true
grep -v '"level"' "$work/l.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/l.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/l.errors"
check grep -q '"device":"kitokey_e2e_other"' "$work/l.events"
check grep -q 'kitokey_e2e_other' "$work/l-during.json"
check grep -q '"device":"kitokey_e2e_mic' "$work/l.events"    # back on the default when it left
check measure2 said "$work/l.events" me kitokey_e2e_other kitokey_e2e_mic    # another takes over at once: no device lost

# The default is a headset's, and no call: the microphone waits, nothing opens it. A call takes it:
# a passive stream, which it carries. It leaves: capture lets go, and the source no longer runs.
# The source's state is what is judged, so its sink is played to only while a call is on: a stand-in
# runs whenever it is, and a real headset does not. WirePlumber's own script, and the profile it would
# switch, are judged on real AirPods alone (D66).
kill "$mic_player" 2>/dev/null || true; wait "$mic_player" 2>/dev/null || true
stand_in kitokey_e2e_headset "bluez5.loopback=true"; headset=$!
sleep 1
pactl set-default-source kitokey_e2e_headset
KITOKEY_TRACE=e2e-m "$bin" record "$work/v2-headset" --as v2 > "$work/m.events" 2> "$work/m.errors" &
recorder=$!
sleep 3
folder=$(ls -d "$work"/v2-headset/*/ | head -1)
check grep -q 'unavailable.*waits' "$work/m.events"
check test ! -e "$folder/local.wav"
check measure2 not-following "$recorder"
check measure2 not-running kitokey_e2e_headset 2
call_on kitokey_e2e_headset; call=$!
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & mic_player=$!
sleep 3
check grep -q '"device":"kitokey_e2e_headset"' "$work/m.events"
check measure2 passive "$recorder"
check measure2 running kitokey_e2e_headset
check test -e "$folder/local.wav"
sleep 2
kill "$mic_player" 2>/dev/null || true; wait "$mic_player" 2>/dev/null || true
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
sleep 3     # what WirePlumber's restore takes, 2 s, and a reading of the graph
check measure2 not-following "$recorder"
check measure2 not-running kitokey_e2e_headset 2
check grep -q '"device":null' "$work/m.events"
pw-play --target kitokey_e2e_out "$work/880.wav"
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
kill "$headset" 2>/dev/null || true; wait "$headset" 2>/dev/null || true
pactl set-default-source kitokey_e2e_mic.monitor
pactl unload-module "$mic_c"; mic_c=
grep -v '"level"' "$work/m.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/m.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/m.errors"
check measure2 waited "$folder"
check logged e2e-m capture.unavailable capture.started capture.opened capture.device capture.me_waits capture.stopped
# Never an address, in a line of the output, a session or a log: a stand-in has none, this holds the path.
check test -z "$(grep -lE '([0-9A-Fa-f]{2}[:_-]){5}[0-9A-Fa-f]{2}' "$work/m.events" "$folder/session.json" "$KITOKEY_LOG_DIR"/capture.*.jsonl 2>/dev/null)"

say "N. what takes the session manager away, on a private graph of its own (D66)"
check bash scripts/private_graph.sh "$bin"

say "O. a microphone named at the start (D74)"
mic_c=$(pactl load-module module-null-sink sink_name=kitokey_e2e_mic_c)
mic_d=$(pactl load-module module-null-sink sink_name=kitokey_e2e_mic_d)
tone 1100 30 "$work/1100-long.wav"
stand_in kitokey_e2e_named; named=$!
feed=kitokey_e2e_mic_d stand_in kitokey_e2e_other; other=$!
stand_in kitokey_e2e_headset "bluez5.loopback=true"; headset=$!
feed=kitokey_e2e_mic_d stand_in kitokey_e2e_x_AA_BB_CC_DD_EE_FF; addressed=$!
sleep 1

# The list: every source, by its name and description; no output, nothing opened; no address in any line.
listed_as=0; "$bin" microphones > "$work/o.list" 2> "$work/o.list-errors" || listed_as=$?
# The run's own alone in the evidence: the computer's microphones are named after its owner at times.
grep 'kitokey_e2e_' "$work/o.list" | sed 's/^/    /' >> .evidence/e2e.txt
check test "$listed_as" = 0 -a ! -s "$work/o.list-errors"
check grep -q '"name":"kitokey_e2e_named"' "$work/o.list"
check grep -q '"name":"kitokey_e2e_headset"' "$work/o.list"
check grep -q '"description":"kitokey_e2e_x_unknown"' "$work/o.list"
check test -z "$(grep 'kitokey_e2e_out' "$work/o.list")"
check test -z "$(grep -E '([0-9A-Fa-f]{2}[:_-]){5}[0-9A-Fa-f]{2}' "$work/o.list")"
check test -z "$(pactl list short source-outputs)"

# Named: recorded from its start; neither a call holding another nor a new default moves it.
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & named_player=$!
pw-play --target kitokey_e2e_mic_d "$work/1100-long.wav" & other_player=$!
KITOKEY_TRACE=e2e-o1 "$bin" record "$work/v2-named" --microphone kitokey_e2e_named > "$work/o1.events" 2> "$work/o1.errors" &
recorder=$!
sleep 2; call_on kitokey_e2e_other; call=$!
sleep 3; pactl set-default-source kitokey_e2e_other
sleep 3
folder=$(ls -d "$work"/v2-named/*/ | head -1)
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
pactl set-default-source kitokey_e2e_mic.monitor
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
grep -v '"level"' "$work/o1.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/o1.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/o1.errors"
check grep -q '"me":"kitokey_e2e_named"' "$work/o1.events"
check measure2 said "$work/o1.events" me
check measure2 changes "$folder"
check measure2 local-pitch "$folder" 660
check test -z "$(grep microphone_gone "$work/o1.events")"
kill "$named_player" "$other_player" 2>/dev/null || true; wait "$named_player" "$other_player" 2>/dev/null || true

# A headset's named: no stream at all while nothing runs it, a call on another none the less; the
# sentence says so. A call takes it: a passive stream. It leaves: let go (ADR-0015).
KITOKEY_TRACE=e2e-o2 "$bin" record "$work/v2-named-headset" --microphone kitokey_e2e_headset > "$work/o2.events" 2> "$work/o2.errors" &
recorder=$!
sleep 2; call_on kitokey_e2e_other; call=$!
sleep 2
folder=$(ls -d "$work"/v2-named-headset/*/ | head -1)
check grep -q 'unavailable.*chosen.*waits' "$work/o2.events"
check test ! -e "$folder/local.wav"
check measure2 not-following "$recorder"
check measure2 not-running kitokey_e2e_headset 2
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
call_on kitokey_e2e_headset; call=$!
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & named_player=$!
sleep 3
check grep -q '"device":"kitokey_e2e_headset"' "$work/o2.events"
check measure2 passive "$recorder"
check measure2 running kitokey_e2e_headset
sleep 2
kill "$named_player" 2>/dev/null || true; wait "$named_player" 2>/dev/null || true
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
sleep 3
check measure2 not-following "$recorder"
check measure2 not-running kitokey_e2e_headset 2
check grep -q '"device":null' "$work/o2.events"
pw-play --target kitokey_e2e_out "$work/880.wav"
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
grep -v '"level"' "$work/o2.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/o2.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/o2.errors"
check measure2 waited "$folder"
check test -z "$(grep microphone_gone "$work/o2.events")"

# A name not there at the start: said, then Auto as if none were named.
KITOKEY_TRACE=e2e-o3 "$bin" record "$work/v2-named-none" --microphone kitokey_e2e_nobody > "$work/o3.events" 2> "$work/o3.errors" &
recorder=$!
sleep 3
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
folder=$(ls -d "$work"/v2-named-none/*/ | head -1)
grep -v '"level"' "$work/o3.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/o3.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0 -a ! -s "$work/o3.errors"
check test "$(head -1 "$work/o3.events" | grep -c microphone_gone)" = 1
check grep -q '"me":"kitokey_e2e_mic' "$work/o3.events"
check test -s "$folder/local.flac"
check logged e2e-o3 capture.microphone_gone capture.started capture.stopped

# A name that goes during the recording: said, Auto takes over, the track going on; back, taken again.
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & named_player=$!
KITOKEY_TRACE=e2e-o4 "$bin" record "$work/v2-named-gone" --microphone kitokey_e2e_named > "$work/o4.events" 2> "$work/o4.errors" &
recorder=$!
sleep 3
kill "$named" 2>/dev/null || true; wait "$named" 2>/dev/null || true
sleep 4
check test "$(grep -c microphone_gone "$work/o4.events")" = 1
check grep -q '"device":"kitokey_e2e_mic' "$work/o4.events"
stand_in kitokey_e2e_named; named=$!
sleep 4
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
folder=$(ls -d "$work"/v2-named-gone/*/ | head -1)
kill "$named_player" 2>/dev/null || true; wait "$named_player" 2>/dev/null || true
grep -v '"level"' "$work/o4.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/o4.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0
check test "$(grep -c microphone_gone "$work/o4.events")" = 1
check test "$(grep -c microphone_back "$work/o4.events")" = 1
check measure2 said "$work/o4.events" me kitokey_e2e_mic kitokey_e2e_named
check measure2 changes "$folder" kitokey_e2e_mic kitokey_e2e_named
check test -s "$folder/local.flac"
check logged e2e-o4 capture.started capture.microphone_gone capture.device capture.microphone_back capture.device capture.stopped

# A headset's named, gone, back with no call on it: said back, what stood in closes, no stream, the
# source not running. A call takes it then: passive, as ever.
KITOKEY_TRACE=e2e-o6 "$bin" record "$work/v2-named-back" --microphone kitokey_e2e_headset > "$work/o6.events" 2> "$work/o6.errors" &
recorder=$!
sleep 2
kill "$headset" 2>/dev/null || true; wait "$headset" 2>/dev/null || true
sleep 3
check test "$(grep -c microphone_gone "$work/o6.events")" = 1
check grep -q '"device":"kitokey_e2e_mic' "$work/o6.events"
stand_in kitokey_e2e_headset "bluez5.loopback=true"; headset=$!
sleep 3
check test "$(grep -c microphone_back "$work/o6.events")" = 1
check measure2 said "$work/o6.events" me kitokey_e2e_mic null
check measure2 not-following "$recorder"
check measure2 not-running kitokey_e2e_headset 2
call_on kitokey_e2e_headset; call=$!
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & named_player=$!
sleep 3
check grep -q '"device":"kitokey_e2e_headset"' "$work/o6.events"
check measure2 passive "$recorder"
kill "$named_player" 2>/dev/null || true; wait "$named_player" 2>/dev/null || true
kill "$call" 2>/dev/null || true; wait "$call" 2>/dev/null || true
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
grep -v '"level"' "$work/o6.events" | sed 's/^/    /' >> .evidence/e2e.txt
sed 's/^/    /' "$work/o6.errors" >> .evidence/e2e.txt
check test "$stopped_as" = 0

# A source whose name holds an address: the list gives an id without it, and the start takes it back.
addressed_id=$(python3 -c 'import json, sys; print(next(line["name"] for line in map(json.loads, open(sys.argv[1])) if line["description"] == "kitokey_e2e_x_unknown"))' "$work/o.list")
KITOKEY_TRACE=e2e-o5 "$bin" record "$work/v2-named-address" --microphone "$addressed_id" > "$work/o5.events" 2> "$work/o5.errors" &
recorder=$!
sleep 2
kill -INT "$recorder"; stopped_as=0; wait "$recorder" || stopped_as=$?
folder=$(ls -d "$work"/v2-named-address/*/ | head -1)
check test "$stopped_as" = 0 -a ! -s "$work/o5.errors"
check grep -q '"me":"kitokey_e2e_x_unknown"' "$work/o5.events"
check test -z "$(grep -lE '([0-9A-Fa-f]{2}[:_-]){5}[0-9A-Fa-f]{2}' "$work/o5.events" "$folder/session.json" "$KITOKEY_LOG_DIR"/capture.*.jsonl 2>/dev/null)"

kill "$named" "$other" "$headset" "$addressed" 2>/dev/null || true; wait "$named" "$other" "$headset" "$addressed" 2>/dev/null || true
pactl unload-module "$mic_c"; mic_c=
pactl unload-module "$mic_d"; mic_d=
# O's devices gone from the graph and the default input the run's own again, before P names it: a
# device still being taken away leaves cpal no name but `default_input`.
pactl set-default-source kitokey_e2e_mic.monitor
for _ in $(seq 50); do
  pw-dump 2>/dev/null | grep -qE '"node.name": "kitokey_e2e_(mic_c|mic_d|named|other|headset|x_)' || break
  sleep 0.1
done
sleep 0.5

say "P. listening without recording (D88): each source heard, nothing written, stopped within a second — asked, or its input closed; a headset said"
# What the run's folder holds, file by file and size: listening must leave it as it was. It starts in a
# folder of its own, its temporary files there too: they must stay empty.
files() { find "$work" -mindepth 1 \( -path "$work/p.*" -prune \) -o -type f -printf '%P %s\n' -o -printf '%P/\n' | sort; }
# Whether it holds what it held before listening; what differs said, nothing else.
unchanged() { diff <(echo "$before") <(files) | sed 's/^/    /' | tee -a .evidence/e2e.txt | grep -q . && return 1 || return 0; }
mkdir -p "$work/p-cwd"
# listen_for RUN [ARGS]: started on a pipe's input, its output in p.RUN.events; the pipe open on 3.
listen_for() {
  mkfifo "$work/p.$1.in"
  (cd "$work/p-cwd" && KITOKEY_TRACE="e2e-p-$1" TMPDIR="$work/p-cwd" exec "$bin" listen "${@:2}") \
    < "$work/p.$1.in" > "$work/p.$1.events" 2> "$work/p.$1.errors" &
  listener=$!
  exec 3> "$work/p.$1.in"
}
# stopped_within PID SECONDS: the program ends well within that long of what was just asked.
stopped_within() {
  local asked=$1 as=0; wait "$2" || as=$?
  python3 -c 'import sys, time; sys.exit(0 if time.time() - float(sys.argv[1]) < float(sys.argv[2]) else "took " + str(time.time() - float(sys.argv[1])))' "$asked" "$3" && test "$as" = 0
}
# heard RUN ME OTHERS: each source named first, `me` no headset's; each level above the floor; no line a
# recording alone says.
heard() {
  python3 - "$work/p.$1.events" "$2" "$3" <<'PY'
import json, sys
lines = [json.loads(line) for line in open(sys.argv[1])]
devices = [(line["source"], line["device"]) for line in lines if line["event"] == "device"]
loudest = {source: max([line["level"] for line in lines if line["event"] == "level" and line["source"] == source] or [0]) for source in ("me", "others")}
said = {line["event"] for line in lines}
named = [source for source, _ in devices[:2]] == ["me", "others"] and all(device.startswith(want) for (_, device), want in zip(devices, sys.argv[2:]))
ok = named and lines[0].get("headset") is False and min(loudest.values()) > 0.5 and said <= {"device", "level"}
print(f"    devices {devices}, loudest {loudest}, said {sorted(said)}")
sys.exit(0 if ok else "not heard as it should be")
PY
}
before=$(files)
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
pw-play --target kitokey_e2e_out "$work/others-long.wav" & others_player=$!
sleep 0.5
listen_for stop
sleep 3
asked=$(date +%s.%N); echo stop >&3
check stopped_within "$asked" "$listener" 1
exec 3>&-
check heard stop kitokey_e2e_mic kitokey_e2e_out
check test ! -s "$work/p.stop.errors"
# A termination signal, as the shell closing the panel.
listen_for term
sleep 2
asked=$(date +%s.%N); kill -TERM "$listener"
check stopped_within "$asked" "$listener" 1
exec 3>&-
check heard term kitokey_e2e_mic kitokey_e2e_out
# Its input closed, as by a shell that crashed while the panel showed: no microphone outlives it.
listen_for eof
sleep 2
asked=$(date +%s.%N); exec 3>&-
check stopped_within "$asked" "$listener" 1
check heard eof kitokey_e2e_mic kitokey_e2e_out
kill "$mic_player" "$others_player" 2>/dev/null || true; wait "$mic_player" "$others_player" 2>/dev/null || true
check unchanged
check test -z "$(ls -A "$work/p-cwd")"
check test -z "$(pactl list short source-outputs)"
check logged e2e-p-stop capture.opened capture.opened capture.listening capture.listened
# The default input changed while it listens: a later `device` of `me`, which never says `headset`
# (`headset_said_once`, below).
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
listen_for moved
sleep 2
pactl set-default-source kitokey_e2e_out.monitor
sleep 3
asked=$(date +%s.%N); echo stop >&3
check stopped_within "$asked" "$listener" 1
exec 3>&-
pactl set-default-source kitokey_e2e_mic.monitor
kill "$mic_player" 2>/dev/null || true; wait "$mic_player" 2>/dev/null || true
moved_said() {
  python3 - "$work/p.moved.events" <<'PY' 2>&1 | tee -a .evidence/e2e.txt; return "${PIPESTATUS[0]}"
import json, sys
mine = [line for line in map(json.loads, open(sys.argv[1])) if line["event"] == "device" and line["source"] == "me"]
print(f"    the default input moved while listening: me's devices {[line['device'] for line in mine]}")
sys.exit(0 if len(mine) >= 2 and mine[1]["device"] != mine[0]["device"] else "no later device of me")
PY
}
check moved_said
# A headset's default, no call: a recording waits (M); listening opens it actively — it runs from
# capture's stream alone —, and lets it go when stopped (ADR-0015's amendment).
mic_c=$(pactl load-module module-null-sink sink_name=kitokey_e2e_mic_c)
stand_in kitokey_e2e_headset "bluez5.loopback=true"; headset=$!
sleep 1
pactl set-default-source kitokey_e2e_headset
check measure2 not-running kitokey_e2e_headset 1
listen_for headset
sleep 2
check grep -q '"device":"kitokey_e2e_headset","event":"device","headset":true,"source":"me"' "$work/p.headset.events"
check measure2 running kitokey_e2e_headset
asked=$(date +%s.%N); echo stop >&3
check stopped_within "$asked" "$listener" 1
exec 3>&-
sleep 1
check measure2 not-running kitokey_e2e_headset 2
kill "$headset" 2>/dev/null || true; wait "$headset" 2>/dev/null || true
pactl set-default-source kitokey_e2e_mic.monitor
# A microphone named, taken away while it listens: what it says then, and it still stops as asked.
stand_in kitokey_e2e_named; named=$!
pw-play --target kitokey_e2e_mic_c "$work/660-long.wav" & named_player=$!
pw-play --target kitokey_e2e_mic "$work/me.wav" & mic_player=$!
sleep 1
listen_for named --microphone kitokey_e2e_named
sleep 2
kill "$named" 2>/dev/null || true; wait "$named" 2>/dev/null || true
sleep 3
asked=$(date +%s.%N); echo stop >&3
check stopped_within "$asked" "$listener" 1
exec 3>&-
kill "$named_player" "$mic_player" 2>/dev/null || true; wait "$named_player" "$mic_player" 2>/dev/null || true
# As recording/v1 says: no line says it, `me`'s level falls to 0 and stays there, still said.
gone_unsaid() {
  python3 - "$work/p.named.events" <<'PY' 2>&1 | tee -a .evidence/e2e.txt; return "${PIPESTATUS[0]}"
import json, sys
lines = [json.loads(line) for line in open(sys.argv[1])]
said = [line for line in lines if line["event"] != "level"]
me = [line["level"] for line in lines if line["event"] == "level" and line["source"] == "me"]
print(f"    named, gone while listening: {said}; me's levels by half-seconds {[max(me[n:n + 5]) for n in range(0, len(me), 5)]}")
ok = [line["event"] for line in said] == ["device", "device"] and max(me[:15]) > 0.5 and len(me) > 40 and max(me[-15:]) == 0
sys.exit(0 if ok else "not as recording/v1 says")
PY
}
check gone_unsaid
sed 's/^/    errors: /' "$work/p.named.errors" >> .evidence/e2e.txt
pactl unload-module "$mic_c"; mic_c=
grep -v '"level"' "$work"/p.*.events | sed 's/^/    /' >> .evidence/e2e.txt
# `headset` is said on listening's first `device` of `me` alone (recording/v1): never on a later one,
# never on `others`', never by a recording — every scenario's lines read.
headset_said_once() {
  python3 - "$work" <<'PY' 2>&1 | tee -a .evidence/e2e.txt; return "${PIPESTATUS[0]}"
import json, pathlib, sys
wrong, listened = [], 0
for path in sorted(pathlib.Path(sys.argv[1]).glob("*.events")):
    lines = [json.loads(line) for line in path.read_text().splitlines() if line.startswith("{")]
    listening = path.name.startswith("p.")
    first = next((n for n, line in enumerate(lines) if line.get("event") == "device" and line.get("source") == "me"), None) if listening else None
    for n, line in enumerate(lines):
        if "headset" in line and not (listening and n == first):
            wrong.append(f"{path.name}:{n + 1} {line}")
    listened += listening
by_file = {}
for w in wrong:
    by_file[w.split(":")[0]] = by_file.get(w.split(":")[0], 0) + 1
print(f"    headset: {len(wrong)} line(s) where recording/v1 does not say it, in {listened} listening runs and every recording's"
      + (f", by file {by_file}" if by_file else "") + "".join(f"\n      {w}" for w in wrong[:5]))
sys.exit(1 if wrong or not listened else 0)
PY
}
check headset_said_once
check unchanged

kill "$guard" 2>/dev/null || true; wait "$guard" 2>/dev/null || true
[ "$failed" = 0 ] && say "e2e: passed" || say "e2e: FAILED"
exit "$failed"
