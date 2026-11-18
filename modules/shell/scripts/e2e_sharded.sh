#!/usr/bin/env bash
# The module's e2e cut in shards played side by side (#862), each in a GNOME session of its own
# made by scripts/headless.sh: the tray's part; main()'s own scenarios, H to R; and the scenarios
# main() calls, in KITOKEY_E2E_SHARDS lists balanced by their measured times (scripts/e2e_times.txt).
# Their text put back in one, in the order a whole run says it; every shard's logs judged together,
# once, as a whole run's are (D29); one verdict. Run by headless.sh for a full run.
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
cd "$here/.."
out=.evidence
evidence=${NSTACK_EVIDENCE:-$PWD/$out/e2e}
mkdir -p "$out" "$evidence"
: > "$out/e2e.txt"
say() { echo "$*" | tee -a "$out/e2e.txt"; }
began=$SECONDS

# Built once, before the shards share it: each one's own build then finds it done.
cargo build --locked 2>&1 | tail -1 >> "$out/e2e.txt"
mapfile -t lists < <(python3 -I scripts/e2e_shards.py lists "${KITOKEY_E2E_SHARDS:-3}" scripts/e2e_window.py scripts/e2e_times.txt)
# Beside the sessions headless.sh makes, never under /tmp: the shards' screenshots and logs.
shards=$(mktemp -d "${XDG_CACHE_HOME:-$HOME/.cache}/kitokey-shards-XXXXXX")
trap 'rm -rf "$shards"' EXIT
say "e2e: in $((${#lists[@]} + 2)) shards side by side (#862): the tray's part, main()'s own scenarios, and the others in ${#lists[@]}"

# Each shard's environment: its folder for its text and evidence, and what it plays.
play() { # number, then the shard's own variables
  local number=$1
  shift
  mkdir -p "$shards/$number"
  env "$@" KITOKEY_E2E_SHARDS=1 KITOKEY_E2E_OUT="$shards/$number" NSTACK_EVIDENCE="$shards/$number/e2e" \
    bash scripts/headless.sh > "$shards/$number/output.txt" 2>&1 &
  pids+=("$!")
}
pids=()
play 0 KITOKEY_E2E_PART=tray KITOKEY_E2E_ONLY=
play 1 KITOKEY_E2E_PART=main KITOKEY_E2E_ONLY=
for number in "${!lists[@]}"; do
  play $((number + 2)) KITOKEY_E2E_PART= KITOKEY_E2E_ONLY="${lists[$number]}"
done
statuses=()
for pid in "${pids[@]}"; do
  status=0
  wait "$pid" || status=$?
  statuses+=("$status")
done

# The shards' text, in a whole run's order.
texts=()
for number in "${!pids[@]}"; do texts+=("$shards/$number/e2e.txt"); done
python3 -I scripts/e2e_shards.py merge scripts/e2e_window.py "${texts[@]}" | tee -a "$out/e2e.txt"

# Each shard's verdict: a shard ran whole when its last line says its checks passed. Not run on one
# — no GNOME, no WebDriver — is not run on all; any other end fails the run.
failed=0
not_run=0
for number in "${!pids[@]}"; do
  last=$(tail -n 1 "$shards/$number/e2e.txt" 2>/dev/null || true)
  case "${statuses[$number]}:$last" in
    3:*"its checks passed"*) say "  ok    shard $number: $last" ;;
    77:*) say "  NOT RUN  shard $number: $last"; not_run=1 ;;
    *) say "  FAIL  shard $number (exit ${statuses[$number]}): $last"; failed=1 ;;
  esac
done

# Their evidence together: every shard's logs judged at once, a whole run's (nothing « not owed »),
# their screenshots in one folder, each session's own log beside the evidence.
mkdir -p "$evidence/shots"
for number in "${!pids[@]}"; do
  cp -r "$shards/$number/e2e/shots/." "$evidence/shots/" 2>/dev/null || true
  cp "$shards/$number/headless-shell.log" "$out/headless-shell-$number.log" 2>/dev/null || true
  cp "$shards/$number/headless-a11y.log" "$out/headless-a11y-$number.log" 2>/dev/null || true
done
logs=0
env -u KITOKEY_E2E_ONLY -u KITOKEY_E2E_PART uv run --quiet --with pyyaml==6.0.3 python scripts/e2e_logs.py "$shards" "$evidence" MANIFEST.yaml \
  > "$shards/logs.txt" 2>&1 || logs=$?
tee -a "$out/e2e.txt" < "$shards/logs.txt"
if [ "$logs" = 0 ]; then say "  ok    logs_read (every shard's logs)"; else say "  FAIL  logs_read (every shard's logs)"; failed=1; fi
say "    (the run, its shards side by side: $((SECONDS - began)) s)"
if [ "$not_run" = 1 ] && [ "$failed" = 0 ]; then
  say "e2e: NOT RUN — a shard could not run here"
  exit 77
fi
[ "$failed" = 0 ] && say "e2e: passed" || say "e2e: FAILED"
exit "$failed"
