"""The window's end-to-end scenario: Kitokey started with `--window` by tauri-driver, the page
read as WebKitGTK shows it, over WebDriver. Sessions are made here; nothing is recorded.

    python3 scripts/e2e_window.py <kitokey-binary> <work-folder>

Prints one line per check, `ok` or `FAIL`, and exits 1 on any failure.
"""

from __future__ import annotations

import datetime as dt
import hashlib
import http.server
import json
import math
import os
import re
import shutil
import signal
import socket
import struct
import subprocess
import sys
import threading
import time
import traceback
import urllib.request
import wave
from pathlib import Path

from selenium import webdriver
from selenium.common.exceptions import (
    ElementNotInteractableException, NoSuchWindowException, StaleElementReferenceException, TimeoutException, WebDriverException,
)
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.options import ArgOptions
from selenium.webdriver.remote.client_config import ClientConfig
from selenium.webdriver.support.expected_conditions import staleness_of
from selenium.webdriver.support.select import Select
from selenium.webdriver.support.wait import WebDriverWait
from urllib3.exceptions import HTTPError, ReadTimeoutError

BINARY, WORK = Path(sys.argv[1]), Path(sys.argv[2])
# kitokey-summary played here: it says what it was given — its arguments, the key in its
# environment, the context it would send (D59), read where the real one reads it. A test refuses a
# key at openrouter.ai and answers elsewhere; minutes are written, naming their context as the
# real ones do, the way it was called kept beside them. Nothing leaves the machine. Where a scenario
# stands a service in ($STANDIN, D60), a request for a provider elsewhere than this computer is sent
# to it, as the real one would send it there, and the minutes name the host that answered.
SUMMARY = WORK / "kitokey-summary"
SUMMARY.write_text(
    """#!/bin/sh
here=$(dirname "$0")
printf '%s\\n' "$*" > "$here/summary-args"
printf '%s' "${KITOKEY_PROVIDER_KEY:-none}" > "$here/summary-key"
if [ "$1" != --test ]; then
  provider=local
  case "$3" in http://127.0.0.1*|http://localhost*) ;; *) provider=$(python3 -c "$HOST" "$3"); [ -n "$STANDIN" ] && python3 -c "$SEND" "$STANDIN/write";; esac
  [ -e "$here/slow-minutes" ] && sleep 3
  [ -e "$here/fail-minutes" ] && { echo '{"event": "failed", "code": "answer.not_minutes"}'; echo "kitokey-summary: the provider's answer is not minutes" >&2; exit 1; }
  [ -e "$here/fail-plainly" ] && { echo "kitokey-summary: a failure no code names" >&2; exit 1; }
  context=$(python3 -c "$CONTEXT" "$1")
  printf '%s' "$context" > "$here/summary-context"
  echo '{"event": "sending", "host": "h", "model": "m", "characters": 1}'
  named=""; [ -n "$STANDIN" ] && named="provider: $provider\\n"
  printf -- "---\\ncontract: minutes/v1\\n${named}model: %s\\ncontext: %s\\n---\\n\\n## A\\n\\n## B\\n\\n## C\\n" "$5" "$context" > "$1/minutes.md"
  echo '{"event": "written", "file": "x"}'
  exit 0
fi
[ -e "$here/test-code" ] && { echo '{"event": "sending", "host": "h", "model": "m", "characters": 1}'; echo '{"event": "failed", "code": "'"$(cat "$here/test-code")"'"}'; echo; echo "an earlier line" >&2; echo "kitokey-summary: raw words of the provider" >&2; exit 1; }
case "$*" in *openrouter.ai*) echo "kitokey-summary: the provider at openrouter.ai refused the key" >&2; exit 1;; esac
echo '{"event": "sending"}'
echo '{"event": "answered"}'
"""
)
SUMMARY.chmod(0o755)
# kitokey-transcription played here: a whole transcript, naming the model it was given — a
# service's by its host, as the real one names it; where a scenario stands the service in
# ($STANDIN, D60), its request is sent to it, as the real one would send the sound there.
HEARING = WORK / "kitokey-transcription"
HEARING.write_text(
    """#!/bin/sh
printf '%s\\n' "$*" > "$(dirname "$0")/hearing-args"
# Tester (D43, D63): the key it was given kept beside; a model that gives no times refused, as the real one says it.
if [ "$1" = --test ]; then
  printf '%s' "${KITOKEY_PROVIDER_KEY:-none}" > "$(dirname "$0")/hearing-key"
  [ -e "$(dirname "$0")/test-code" ] && { echo '{"event": "started"}'; echo '{"event": "failed", "code": "'"$(cat "$(dirname "$0")/test-code")"'"}'; echo; echo "an earlier line" >&2; echo "kitokey-transcription: raw words of the service" >&2; exit 1; }
  exit 0
fi
[ -e "$(dirname "$0")/fail-hearing" ] && { echo "kitokey-transcription: the engine stopped" >&2; exit 1; }
engine=whisper.cpp
model=$(basename "$3" .bin)
if [ "$2" = --provider ]; then
  engine=$(python3 -c "$HOST" "$3"); model=$5
  [ -n "$STANDIN" ] && python3 -c "$SEND" "$STANDIN/hear"
fi
[ -e "$(dirname "$0")/slow-hearing" ] && sleep 3
context=$(python3 -c "$CONTEXT" "$1")
printf '{"type":"header","contract":"transcript/v1","engine":"%s","model":"%s","language":null,"context":%s}\\n{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Refait avec %s."}\\n{"type":"complete","duration":1.0}\\n' "$engine" "$model" "$context" "$model" > "$1/transcript.jsonl"
"""
)
HEARING.chmod(0o755)
# The context of a session's folder as both steps read it — trimmed, none when nothing is left —,
# quoted as JSON, for the two programs played above.
os.environ["CONTEXT"] = (
    "import json, pathlib, sys; f = pathlib.Path(sys.argv[1], 'context.txt'); "
    "t = f.read_text().strip() if f.exists() else ''; print(json.dumps(t or None, ensure_ascii=False))"
)
# The host of a provider's address, and a request sent to a stand-in service, for the two above.
os.environ["HOST"] = "import sys, urllib.parse; print(urllib.parse.urlsplit(sys.argv[1]).hostname)"
os.environ["SEND"] = "import sys, urllib.request; urllib.request.urlopen(urllib.request.Request(sys.argv[1], data=b'x', method='POST'), timeout=5)"

# kitokey-capture played here: it says it started, then each source's level ten times a second,
# until it is told to stop. No device, nobody recorded.
RECORDER = WORK / "kitokey-capture"
RECORDER.write_text(
    """#!/bin/bash
here=$(dirname "$0")
# The system's microphones, as the scenario lists them (D74): none, the list is not offered.
if [ "$1" = microphones ]; then
  # Slow to answer, when the scenario says so: a choice is made before the list is read (D74).
  [ -e "$here/microphones-slow" ] && sleep 1
  if [ -e "$here/microphones-unread" ]; then echo "the system's microphones could not be read" >&2; exit 1; fi
  [ -e "$here/microphones" ] && cat "$here/microphones"
  exit 0
fi
# Listening for « Vérifier le son » (D88): a real stream on the run's microphone, held as capture holds
# its own, read in the graph by the scenario; each side's level ten times a second — the meeting's loud
# while the scenario says it hears the test sound —; ended by « stop », its input closed or a signal.
if [ "$1" = listen ]; then
  printf '%s\n' "$*" > "$here/listen-args"
  rm -f "$here/listen-ended" "$here/listen-told"
  # A program named as capture is, so that its stream is Kitokey's own for the shell's watcher (D30):
  # a copy of pw-cat, told to record — its name no longer says it.
  mkdir -p "$here/own"; [ -x "$here/own/kitokey-capture" ] || cp "$(command -v pw-cat)" "$here/own/kitokey-capture"
  "$here/own/kitokey-capture" --record --target kitokey_e2e_check_mic "$here/listen.wav" & held=$!
  ended() { kill "$held" 2>/dev/null; wait "$held" 2>/dev/null; date +%s.%N > "$here/listen-ended"; exit 0; }
  trap ended TERM INT
  headset=false; [ -e "$here/listen-headset" ] && headset=true
  echo '{"event":"device","source":"me","device":"Kitokey e2e mic","headset":'$headset'}'
  echo '{"event":"device","source":"others","device":"Kitokey e2e output"}'
  while :; do
    read -t 0.1 line; got=$?
    if [ "$got" = 0 ]; then echo "$line" > "$here/listen-told"; ended; fi
    [ "$got" -gt 128 ] || ended
    loud=0.05; [ -e "$here/listen-loud" ] && loud=0.8
    echo '{"event":"level","source":"me","level":0.4}'
    echo '{"event":"level","source":"others","level":'$loud'}'
  done
fi
printf '%s\\n' "$*" > "$here/recorder-args"
folder="$2/2026-10-18_12-00"; mkdir -p "$folder"
printf '{"contract":"session-audio/v2","started":"2026-10-18T12:00:00+02:00","tracks":{}}' > "$folder/session.json"
me='"Mic"'
# No microphone to open, when the scenario says so (D49).
if [ -e "$2/waiting-me" ]; then echo '{"event":"unavailable","source":"me","reason":"none"}'; me='"unavailable"'; fi
echo '{"event":"started","folder":"'"$folder"'","sources":{"me":'"$me"',"others":"Speaker"}}'
n=0
while ! read -t 0.1 line; do
  n=$((n + 1))
  # Levels at silence as long as the scenario says so; what capture says of a silence, when it does (D49).
  if [ -e "$2/hush" ]; then echo '{"event":"level","source":"me","level":0}'; echo '{"event":"level","source":"others","level":0}'
  else
    [ -e "$2/waiting-me" ] || echo '{"event":"level","source":"me","level":0.'$((n % 9 + 1))'}'
    echo '{"event":"level","source":"others","level":0.5}'
  fi
  if [ -e "$2/nothing-me" ]; then rm "$2/nothing-me"; echo '{"event":"silent","source":"me","nothing":true}'; fi
  if [ -e "$2/silent-others" ]; then rm "$2/silent-others"; echo '{"event":"silent","source":"others"}'; fi
  if [ -e "$2/heard-me" ]; then rm "$2/heard-me"; echo '{"event":"heard","source":"me"}'; fi
  if [ -e "$2/heard-others" ]; then rm "$2/heard-others"; echo '{"event":"heard","source":"others"}'; fi
  # A device changed, or gone, when the scenario says so (D31, D33).
  if [ -e "$2/move" ]; then rm "$2/move"; echo '{"event":"device","source":"me","device":"AirPods Pro"}'; fi
  if [ -e "$2/lose" ]; then rm "$2/lose"; echo '{"event":"device","source":"others","device":null}'; fi
  # The microphone named gone, Auto taking over, and back (D74).
  if [ -e "$2/mic-gone" ]; then rm "$2/mic-gone"; echo '{"event":"microphone_gone"}'; echo '{"event":"device","source":"me","device":"Default"}'; fi
  if [ -e "$2/mic-back" ]; then d=$(cat "$2/mic-back"); rm "$2/mic-back"; echo '{"event":"microphone_back"}'; echo '{"event":"device","source":"me","device":"'"$d"'"}'; fi
  # A headset's back with no call on it: said back, what stood in closes, no stream.
  if [ -e "$2/mic-back-waits" ]; then rm "$2/mic-back-waits"; echo '{"event":"microphone_back"}'; echo '{"event":"device","source":"me","device":null}'; fi
  # Another device that reads as the one chosen, which is not back.
  if [ -e "$2/mic-same" ]; then d=$(cat "$2/mic-same"); rm "$2/mic-same"; echo '{"event":"device","source":"me","device":"'"$d"'"}'; fi
done
printf '{"contract":"session-audio/v2","started":"2026-10-18T12:00:00+02:00","stopped":"2026-10-18T12:00:05+02:00","tracks":{}}' > "$folder/session.json"
echo '{"event":"stopped","folder":"'"$folder"'","seconds":5.0}'
"""
)
RECORDER.chmod(0o755)
FAILED = []
# What Tester says of each cause its program names (D72), as `src/words.rs` says it in French.
TEST_CAUSES = [
    ("provider.refused_key", "Le fournisseur a refusé la clé\u00a0: saisissez-la de nouveau dans les réglages de Kitokey."),
    ("provider.unknown_model", "Le fournisseur ne connaît pas ce modèle ou cette adresse\u00a0: choisissez-les de nouveau dans les réglages de Kitokey."),
    ("provider.no_times", "Ce modèle ne donne pas les temps de ce qu’il entend\u00a0: choisissez-en un qui donne ceux de chaque segment dans les réglages de Kitokey."),
    ("provider.no_credit", "Le fournisseur a refusé faute de crédit\u00a0: ajoutez du crédit chez lui, puis réessayez."),
    ("provider.too_many", "Le fournisseur refuse pour trop de demandes, ou un quota épuisé\u00a0: patientez, ou vérifiez votre forfait chez lui."),
    ("provider.down", "Le fournisseur répond qu’il est en panne\u00a0: réessayez plus tard. Sur cet ordinateur, ce modèle est peut-être trop gros pour la mémoire\u00a0: choisissez-en un plus petit."),
    ("provider.unreachable", "Le fournisseur n’a pas pu être joint\u00a0: vérifiez qu’il tourne, ou le réseau, puis réessayez."),
]
MONTHS_FR = "janvier février mars avril mai juin juillet août septembre octobre novembre décembre".split()
# The page's ground, as `ui/window.css` sets it in each theme.
GROUND = {"light": "rgb(242, 244, 247)", "dark": "rgb(21, 23, 28)"}


def check(what: str, held: bool, got: object = "") -> None:
    print(f"  {'ok  ' if held else 'FAIL'}  {what}" + ("" if held else f"\n        got: {got}"))
    if not held:
        FAILED.append(what)


def shot(browser, name: str) -> None:
    """The page as it is, saved where KITOKEY_SCREENSHOTS says: for an eye against D11's boards. One
    whose name says its theme is measured: a light one light, a dark one dark (D89), its mean
    lightness read by ImageMagick, 0 black to 1 white."""
    folder = os.environ.get("KITOKEY_SCREENSHOTS")
    if folder:
        Path(folder).mkdir(parents=True, exist_ok=True)
        browser.save_screenshot(str(Path(folder) / f"{name}.png"))
        theme = next((theme for theme in ("light", "dark") if theme in name.split("-")), None)
        if theme:
            said = subprocess.run(["magick", str(Path(folder) / f"{name}.png"), "-colorspace", "Gray", "-format", "%[fx:mean]", "info:"],
                                  capture_output=True, text=True).stdout.strip()
            mean = float(said) if said.replace(".", "", 1).isdigit() else None
            check(f"{name}: the screenshot is {theme}", mean is not None and (mean >= 0.6 if theme == "light" else mean <= 0.4), said)


def choose(browser, element_id: str, value: str) -> None:
    """A choice in a list of the page as it is: drawn again under the driver — a choice just
    saved draws it anew —, the list is found afresh."""
    for _ in range(5):
        try:
            Select(browser.find_element(By.ID, element_id)).select_by_value(value)
            return
        except StaleElementReferenceException:
            time.sleep(0.2)
    Select(browser.find_element(By.ID, element_id)).select_by_value(value)


def press(browser, element_id: str) -> None:
    """A click on the page as it is: drawn again under the driver, the element is found afresh;
    found before it can be clicked — shown, laid out — it is clicked a moment later (#510)."""
    for _ in range(10):
        try:
            browser.find_element(By.ID, element_id).click()
            return
        except (StaleElementReferenceException, ElementNotInteractableException):
            time.sleep(0.2)
    browser.find_element(By.ID, element_id).click()


def free(browser, element_id: str, case: str) -> None:
    """A Tester's button offered again before the next case's click: a click on it disabled is
    lost without an error, its code never asked, the last sentence read again (#799)."""
    if not wait_for(browser, lambda b: b.find_element(By.ID, element_id).is_enabled(), 10):
        check(f"Tester still busy before {case}", False, f"#{element_id} disabled for 10 s")


# Each click a button of the page took, counted by its id: a disabled one takes none.
CLICKS_COUNTED = """if (!window.e2eClicks) {
  window.e2eClicks = {};
  document.addEventListener("click", (event) => {
    const id = event.target.closest("button")?.id;
    if (id) window.e2eClicks[id] = (window.e2eClicks[id] ?? 0) + 1;
  }, true);
}
return window.e2eClicks[arguments[0]] ?? 0;"""


def tester_click(browser, element_id: str, case: str) -> None:
    """A click on Tester confirmed taken — its handler starts a test at once —, made once more if
    lost; lost twice, a named failure (#799). Not the "testing" sentence: a test of the fake
    program ends before a poll would read it."""
    for _ in range(2):
        before = browser.execute_script(CLICKS_COUNTED, element_id)
        press(browser, element_id)
        if wait_for(browser, lambda b: b.execute_script(CLICKS_COUNTED, element_id) > before, 1):
            return
    check(f"Tester never started for {case}", False, f"#{element_id} clicked twice, no click taken")


def context_opened(browser):
    """A meeting's context, opened from its settings row when it is closed (D89)."""
    toggle = browser.find_element(By.ID, "context-toggle")
    if toggle.get_attribute("aria-expanded") != "true":
        toggle.click()
    return browser.find_element(By.ID, "context-text")


def redo_pick(browser, panel: str, step: str, index: int) -> None:
    """A model chosen in a Refaire… panel's « Avancé » (D91): opened if it is closed, then its list —
    the first entry Settings' own."""
    toggle = f"{panel}-advanced"
    wait_for(browser, lambda b: b.find_elements(By.ID, toggle))
    if browser.find_element(By.ID, toggle).get_attribute("aria-expanded") != "true":
        press(browser, toggle)
    Select(browser.find_element(By.ID, f"{panel}-{step}-model")).select_by_index(index)


def redo_offered(browser, panel: str, step: str) -> list[str]:
    """What a Refaire… panel's « Avancé » offers for a step, opened if it is closed."""
    toggle = f"{panel}-advanced"
    wait_for(browser, lambda b: b.find_elements(By.ID, toggle))
    if browser.find_element(By.ID, toggle).get_attribute("aria-expanded") != "true":
        press(browser, toggle)
    return browser.execute_script("return [...document.getElementById(arguments[0]).options].map(o => o.textContent)", f"{panel}-{step}-model")


def tokens(theme: str) -> str:
    """A theme's colours as `ui/window.css` sets them: the light ones first, the dark ones under
    `prefers-color-scheme: dark`."""
    css = (Path(__file__).parent.parent / "ui" / "window.css").read_text()
    start = css.index(":root {", 0 if theme == "light" else css.index("prefers-color-scheme: dark")) + len(":root {")
    return css[start:css.index("}", start)].replace("\n", " ").strip()


def shot_themed(browser, name: str, theme: str) -> None:
    """A screenshot in its theme: the system's, or, where the desktop shows the other one, the theme's
    colours put on the page — a dark one on a light desktop as a light one on a dark desktop (D89)."""
    forced = (theme == "dark") != browser.execute_script("return matchMedia('(prefers-color-scheme: dark)').matches")
    if forced:
        browser.execute_script("document.documentElement.style.cssText = arguments[0] + '; color-scheme: ' + arguments[1]", tokens(theme), theme)
        time.sleep(0.3)
    shot(browser, name)
    if forced:
        browser.execute_script("document.documentElement.style.cssText = ''")


def stop_from_small(browser) -> None:
    """Arrêter in the small window: the window goes with the recording, sometimes before the
    driver has answered the click."""
    try:
        browser.find_element(By.ID, "small-stop").click()
    except NoSuchWindowException:
        pass


def read(browser, reads):
    """What the page says, as it is: drawn again under the driver, it is read afresh (#409)."""
    for _ in range(5):
        try:
            return reads(browser)
        except StaleElementReferenceException:
            time.sleep(0.2)
    return reads(browser)


def wait_for(browser, holds, seconds: float = 10) -> bool:
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        # The page drawn again between finding an element and reading it: not yet, look again.
        try:
            if holds(browser):
                return True
        except StaleElementReferenceException:
            pass
        time.sleep(0.1)
    return False


def free_port() -> int:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def session(folder: Path, started: dt.datetime, minutes: float | None, files: dict[str, int], form: str = "v2") -> None:
    folder.mkdir(parents=True)
    record = {"contract": f"session-audio/{form}", "started": started.isoformat(timespec="seconds")}
    if minutes is not None:
        record["stopped"] = (started + dt.timedelta(minutes=minutes)).isoformat(timespec="seconds")
    record["sources" if form == "v1" else "tracks"] = {}
    (folder / "session.json").write_text(json.dumps(record))
    for name, size in files.items():
        (folder / name).parent.mkdir(parents=True, exist_ok=True)
        with open(folder / name, "wb") as made:
            made.truncate(size)


def instances() -> list[int]:
    """The processes of the Kitokey under test."""
    found = []
    for process in Path("/proc").iterdir():
        try:
            if process.name.isdigit() and os.readlink(process / "exe") == str(BINARY.resolve()):
                found.append(int(process.name))
        except OSError:
            pass
    return found


def tray_entries() -> tuple[str, str, list[tuple[int, str]]]:
    """The tray's menu of the Kitokey under test, as the desktop reads it: its bus, its path, and
    each entry's number and words (D64)."""
    watcher = ["org.kde.StatusNotifierWatcher", "/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher"]
    listed = subprocess.run(["busctl", "--user", "get-property", *watcher, "RegisteredStatusNotifierItems"],
                            capture_output=True, text=True).stdout
    ours = {str(pid) for pid in instances()}
    # The program's own buses: the watcher lists an item as `bus@path`, or since GNOME's
    # AppIndicator 66 as its path alone; the bus is the one of ours that answers for its menu.
    names = subprocess.run(["busctl", "--user", "list", "--no-legend"], capture_output=True, text=True).stdout
    buses = [words[0] for words in (line.split() for line in names.splitlines())
             if len(words) > 1 and words[0].startswith(":") and words[1] in ours]
    for item in (word.strip('"') for word in listed.split() if "kitokey" in word):
        path = item.split("@", 1)[-1]
        for bus in buses:
            answered = subprocess.run(["busctl", "--user", "--json=short", "call", bus, f"{path}/Menu", "com.canonical.dbusmenu",
                                       "GetLayout", "iias", "0", "--", "-1", "0"], capture_output=True, text=True)
            if answered.returncode == 0:
                break
        else:
            continue
        layout = answered.stdout
        entries: list[tuple[int, str]] = []

        def walk(node) -> None:
            node = node["data"] if isinstance(node, dict) and "data" in node else node
            number, props, children = node
            if "label" in props:
                entries.append((number, props["label"]["data"]))
            for child in children:
                walk(child)

        walk(json.loads(layout)["data"][1])
        return bus, path, entries
    return "", "", []


def tray_labels() -> list[str]:
    """The words of the tray's menu of the Kitokey under test (D64)."""
    return [label for _, label in tray_entries()[2]]


def tray_click(words: str) -> bool:
    """Clicks the tray's entry whose words begin so, as the desktop does (D60)."""
    bus, path, entries = tray_entries()
    number = next((n for n, label in entries if label.startswith(words)), None)
    if number is None:
        return False
    return subprocess.run(["busctl", "--user", "call", bus, f"{path}/Menu", "com.canonical.dbusmenu", "Event", "isvu",
                           str(number), "clicked", "s", "", "0"], capture_output=True).returncode == 0

def tray_colour() -> str:
    """The tray's icon of the Kitokey under test as the desktop draws it: "blue" at rest, "red" while
    recording (D89), from the image the item names — the toolkit writes it to a file and gives its
    folder and name (IconThemePath, IconName); "" when none is read."""
    bus, path, _ = tray_entries()
    if not bus:
        return ""
    def prop(name: str) -> str:
        said = subprocess.run(["busctl", "--user", "get-property", bus, path, "org.kde.StatusNotifierItem", name],
                              capture_output=True, text=True).stdout.split(" ", 1)
        return said[1].strip().strip('"') if len(said) == 2 else ""
    folder, name = prop("IconThemePath"), prop("IconName")
    found = [f for f in (Path(name), Path(folder) / f"{name}.png", Path(folder) / name) if name and f.is_file()]
    if not found:
        return ""
    mean = subprocess.run(["magick", str(found[0]), "-background", "white", "-alpha", "remove", "-resize", "1x1!",
                           "-format", "%[fx:int(255*r)] %[fx:int(255*g)] %[fx:int(255*b)]", "info:"],
                          capture_output=True, text=True).stdout.split()
    if len(mean) != 3:
        return ""
    r, g, b = (int(v) for v in mean)
    return "red" if r > b + 40 else "blue" if b > r + 40 else f"{r},{g},{b}"


class Window:
    """Kitokey with its window, for as long as the `with` lasts."""

    def __init__(self, sessions: Path, language: str, more: str = "", ollama: str = "http://127.0.0.1:9", small: bool = False,
                 welcomed: bool = True, args: tuple[str, ...] = ("--window",)) -> None:
        config = WORK / f"config-{language}-{sessions.name}"
        (config / "kitokey").mkdir(parents=True, exist_ok=True)
        self.file = config / "kitokey" / "settings.toml"
        # The full window, as the scenarios before D35 knew it; the small one, opened first, where asked.
        self.small = small
        opened = "" if small else 'window = "full"\n'
        # Set up already (D58): a scenario with no meeting yet is not met by the first run's assistant.
        greeted = "welcomed = true\n" if welcomed else ""
        self.args = list(args)
        self.file.write_text(f'sessions = "{sessions}"\n{opened}{greeted}{more}')
        self.env = dict(os.environ, XDG_CONFIG_HOME=str(config), XDG_DATA_HOME=str(WORK / "data"), XDG_STATE_HOME=str(WORK / "state"), LANGUAGE="")
        # An Ollama and a keyring of the scenario's, never the user's own.
        self.env.update(LANG=language, LC_ALL=language, KITOKEY_OLLAMA=ollama, KITOKEY_KEYRING="memory", KITOKEY_SUMMARY=str(SUMMARY), KITOKEY_TRANSCRIPTION=str(HEARING), KITOKEY_CAPTURE=str(RECORDER))

    def __enter__(self) -> webdriver.Remote:
        # One Kitokey per session: one still leaving takes this start's arguments and this one
        # leaves, its session never opened (#510). Waited for.
        until = time.monotonic() + 10
        while instances() and time.monotonic() < until:
            time.sleep(0.1)
        port, native = free_port(), free_port()
        self.driver_process = subprocess.Popen(
            ["tauri-driver", "--port", str(port), "--native-port", str(native)],
            env=self.env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        options = ArgOptions()
        options.set_capability("browserName", "wry")
        options.set_capability("tauri:options", {"application": str(BINARY), "args": self.args})
        # The driver answers once it is up: until then, connections are refused or cut.
        # A command the driver never answers breaks off after a minute, on the line that sent it —
        # four minutes for a GET or a DELETE, which urllib3 tries again three times —, never the
        # run's twenty minutes, silent (#510).
        address = f"http://127.0.0.1:{port}"
        for _ in range(50):
            try:
                self.browser = webdriver.Remote(address, options=options, client_config=ClientConfig(address, timeout=60))
                break
            except ReadTimeoutError:
                self.driver_process.terminate()
                self.driver_process.wait(timeout=10)
                raise RuntimeError(f"tauri-driver opened no session in a minute; Kitokey's processes: {instances()}") from None
            except (WebDriverException, HTTPError):
                time.sleep(0.2)
        else:
            self.driver_process.terminate()
            self.driver_process.wait(timeout=10)
            raise SystemExit("tauri-driver did not answer")
        # Whichever window opened: the small one at rest, or the full one's sections named.
        wait_for(self.browser, lambda b: b.find_elements(By.ID, "small-record") or b.execute_script("return (document.getElementById('settings-link').getAttribute('aria-label') || '') !== ''"))
        return self.browser

    def __exit__(self, *_) -> None:
        try:
            self.browser.quit()
        finally:
            self.driver_process.terminate()
            self.driver_process.wait(timeout=10)
            # A session that never answered leaves the Kitokey under test behind: ended, or it takes
            # the next start's arguments and the next session never opens (#510). Only this binary's.
            for pid in instances():
                try:
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass


def asking_less_motion(window: Window) -> Window:
    """A window whose desktop asks for less motion, in the scenario's own settings, never the user's:
    WebKitGTK reads `org.gnome.desktop.interface enable-animations`, from a key file with this backend
    (`gtk-3.0/settings.ini` alone is not enough: the query stays false under Wayland)."""
    settings = Path(window.env["XDG_CONFIG_HOME"]) / "glib-2.0" / "settings"
    settings.mkdir(parents=True, exist_ok=True)
    (settings / "keyfile").write_text("[org/gnome/desktop/interface]\nenable-animations=false\n")
    window.env.update(GSETTINGS_BACKEND="keyfile")
    return window


# The sidebar's primary actions shown, by their ids: one at a time (D69).
PRIMARIES = "return [...document.querySelectorAll('#sidebar .primary')].filter(b => b.offsetParent).map(b => b.id)"
LESS_MOTION = "return matchMedia('(prefers-reduced-motion: reduce)').matches"
# From here on, whatever starts to move, in the running page.
WATCH_MOTION = ("window.__moved = []; for (const kind of ['animationstart', 'transitionrun'])"
                " document.addEventListener(kind, (e) => window.__moved.push(`${kind} ${e.animationName ?? e.propertyName} ${e.target.className || e.target.id}`), true)")
# What moved so far, and every element or pseudo-element that would move by its computed style.
MOVING = ("const moves = [...window.__moved], some = (t) => t.split(',').some((d) => parseFloat(d) > 0);"
          "for (const e of document.querySelectorAll('*')) for (const p of [null, '::before', '::after']) {"
          " const c = getComputedStyle(e, p), named = `${e.tagName}.${e.className.baseVal ?? e.className}#${e.id}${p ?? ''}`;"
          " if (c.animationName !== 'none' && some(c.animationDuration)) moves.push(`animation ${c.animationName} ${named}`);"
          " if (c.transitionProperty !== 'none' && some(c.transitionDuration)) moves.push(`transition ${c.transitionProperty} ${c.transitionDuration} ${named}`); }"
          "return moves")


# The run's own output, made by e2e.sh: the window's sound goes there, never to the speakers.
QUIET = os.environ.get("PULSE_SINK", "")
# What WirePlumber remembers of an application's sound, by its name: the user's own Kitokey
# shares it, so a run that changes a volume there changes theirs.
REMEMBERED = Path(os.environ.get("XDG_STATE_HOME") or Path.home() / ".local" / "state") / "wireplumber" / "stream-properties"


def remembered() -> str | None:
    """The volume the system keeps for Kitokey's sound, as its line says it."""
    if not REMEMBERED.exists():
        return None
    return next((line for line in REMEMBERED.read_text().splitlines() if line.startswith("Output/Audio:application.name:kitokey=")), None)


def same_volume(one: str | None, other: str | None) -> bool:
    """The same volume remembered: the system writes the line's keys in no fixed order."""
    read = lambda line: line and json.loads(line.split("=", 1)[1])
    return read(one) == read(other)


def played_to(window: "Window") -> set[str]:
    """The outputs the sound of this window — its web view, under its driver — is linked to."""
    def under(pid: int) -> bool:
        while pid > 1:
            if pid == window.driver_process.pid:
                return True
            try:
                pid = int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[1])
            except (OSError, IndexError, ValueError):
                return False
        return False

    objects = json.loads(subprocess.run(["pw-dump"], capture_output=True, text=True, check=True).stdout)
    nodes = {o["id"]: (o.get("info") or {}).get("props") or {} for o in objects if o.get("type") == "PipeWire:Interface:Node"}
    streams = {i for i, props in nodes.items() if props.get("media.class") == "Stream/Output/Audio"
               and under(int(props.get("application.process.id") or 0))}
    return {nodes.get(o["info"]["input-node-id"], {}).get("node.name", "?") for o in objects
            if o.get("type") == "PipeWire:Interface:Link" and o["info"]["output-node-id"] in streams}


def kitokey_streams(window: "Window") -> list[str]:
    """The streams of Kitokey's processes — the window's, under its driver, and what it started —, by class."""
    def under(pid: int) -> bool:
        while pid > 1:
            if pid == window.driver_process.pid:
                return True
            try:
                pid = int(Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[1])
            except (OSError, IndexError, ValueError):
                return False
        return False

    objects = json.loads(subprocess.run(["pw-dump"], capture_output=True, text=True, check=True).stdout)
    clients = {o["id"]: (o.get("info") or {}).get("props") or {} for o in objects if o.get("type") == "PipeWire:Interface:Client"}
    found = []
    for o in objects:
        props = (o.get("info") or {}).get("props") or {}
        if o.get("type") != "PipeWire:Interface:Node" or not str(props.get("media.class", "")).startswith("Stream/"):
            continue
        pid = props.get("application.process.id") or clients.get(props.get("client.id"), {}).get("application.process.id") or 0
        if under(int(pid)):
            found.append(props["media.class"])
    return found


def rows(browser) -> list[tuple[str, str, str]]:
    """Each listed session: its group's heading, its first line, and what it holds — a mark by its name."""
    return browser.execute_script(
        """return [...document.querySelectorAll('.group')].flatMap(g => [...g.querySelectorAll('.session')].map(s =>
             [g.querySelector('h2').textContent, s.querySelector('.day').textContent,
              [...s.querySelectorAll('.length, .mark, .chip')].filter(c => !c.hidden).map(c => c.getAttribute('aria-label') ?? c.textContent).join('|')]))"""
    )


def week_of(day: dt.datetime) -> dt.datetime:
    """Monday 00:00 of the week a day is in, on this machine's clock — as the page reckons it."""
    start = day.replace(hour=0, minute=0, second=0, microsecond=0)
    return start - dt.timedelta(days=start.weekday())


def group(day: dt.datetime | None, now: dt.datetime) -> str:
    """The heading the page puts a session under, in French — the page's own rule, here."""
    if day is None:
        return "Date inconnue"
    this_week = week_of(now)
    if day >= this_week:
        return "Cette semaine"
    if day >= this_week - dt.timedelta(days=7):
        return "La semaine dernière"
    if (day.year, day.month) == (now.year, now.month):
        return f"Plus tôt en {MONTHS_FR[day.month - 1]}"
    return f"{MONTHS_FR[day.month - 1].capitalize()} {day.year}"


def model_offered(now: dt.datetime) -> None:
    """T. The default model missing: the window offers the tray's own fetch where it says so (D36)."""
    print("T. the transcription model missing: offered in the window too, where it says so (D36)")
    served, right = WORK / "served", os.urandom(300_000)
    served.mkdir()
    (served / "model.bin").write_bytes(os.urandom(300_000))  # served first: not the published file
    asked = []

    # Served slowly — 30 kB every 0.3 s, three seconds a fetch — so that its coming is seen.
    class Slow(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=str(served), **kwargs)

        def log_message(self, *_):
            asked.append(self.path)

        def copyfile(self, source, sink):
            while chunk := source.read(30_000):
                sink.write(chunk)
                sink.flush()
                time.sleep(0.3)

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Slow)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    host = f"127.0.0.1:{server.server_address[1]}"
    meetings, data = WORK / "fetching", WORK / "data-fetching"
    session(meetings / "2026-10-25_10-00", now - dt.timedelta(hours=1), 5, {"local.wav": 2000})
    heard = meetings / "2026-10-25_10-00" / "transcript.jsonl"
    fetched = data / "kitokey" / "models" / "ggml-large-v3-turbo-q8_0.bin"
    window = Window(meetings, "fr_FR.UTF-8")
    window.env.update(XDG_DATA_HOME=str(data), KITOKEY_MODEL_PUBLISHED=f"http://{host}/model.bin 300000 {hashlib.sha256(right).hexdigest()}")
    said = "return document.getElementById('model-state')?.innerText || ''"
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        offered = wait_for(browser, lambda b: b.find_elements(By.ID, "model-fetch"))
        label = browser.find_element(By.ID, "model-fetch").text if offered else ""
        check("Settings offer the missing model, with its size and host", offered and label == f"Télécharger le modèle (300 ko, {host})", label)
        check("nothing is fetched before the click", asked == [], asked)
        # Coming back to the window draws it anew, by design: the window's focus waited for first —
        # the desktop being in use, it may come late (#510).
        wait_for(browser, lambda b: b.execute_script("return document.hasFocus()"), 5)
        time.sleep(0.5)
        button = browser.find_element(By.ID, "model-fetch")
        time.sleep(3.5)
        check("in Settings too, the offer is not drawn again while nothing changes", not staleness_of(button)(browser))
        browser.find_element(By.ID, "model-fetch").click()
        coming = wait_for(browser, lambda b: "En cours de téléchargement" in b.execute_script(said) and not b.find_elements(By.ID, "model-fetch"), 3)
        check("once clicked, it is said coming, and not offered again", coming, browser.execute_script(said))
        again = wait_for(browser, lambda b: b.find_elements(By.ID, "model-fetch"), 15)
        check("a file that is not the published one: nothing kept, offered again", again and not fetched.exists() and asked != [], asked)
        (served / "model.bin").write_bytes(right)
        press(browser, "model-fetch")
        ready = wait_for(browser, lambda b: "Prêt" in b.execute_script(said), 15)
        check("fetched from Settings, the model is ready there", ready and fetched.read_bytes() == right, browser.execute_script(said))
        time.sleep(1)
        check("with none in line, no transcript is made", not heard.exists())
        # The model gone again, a meeting waits for it: offered on its page, where it waits.
        fetched.unlink()
        browser.execute_script("location.hash = '#/meeting/2026-10-25_10-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-transcript"))
        press(browser, "make-transcript")
        waits = "return document.getElementById('transcript-state')?.innerText || ''"
        offered = wait_for(browser, lambda b: "En attente du modèle Whisper" in b.execute_script(waits) and b.find_elements(By.CSS_SELECTOR, "#transcript-state #model-fetch"))
        check("a meeting waiting for the model offers it on its page", offered, browser.execute_script(waits))
        # The tray's entry that fetches it follows the language chosen, at once (D64).
        speak = lambda code: browser.execute_script("return window.__TAURI__.core.invoke('settings_saved', {change: {language: arguments[0]}})", code)
        tray = lambda b, said: any(label.startswith(said) for label in tray_labels())
        offered = wait_for(browser, lambda b: tray(b, "Télécharger le modèle ("), 5)
        speak("en")
        english = wait_for(browser, lambda b: tray(b, "Download the model ("), 5)
        speak("")
        check("the tray's entry that fetches it speaks the language chosen, at once", offered and english
              and wait_for(browser, lambda b: tray(b, "Télécharger le modèle"), 5), tray_labels())
        # Offered, it stays the same button across the page's readings: a focus on it is kept.
        button = browser.find_element(By.ID, "model-fetch")
        time.sleep(3.5)
        check("the offer is not drawn again while nothing changes", not staleness_of(button)(browser))
        browser.find_element(By.ID, "model-fetch").click()
        coming = wait_for(browser, lambda b: "en cours de téléchargement" in b.execute_script(waits) and not b.find_elements(By.ID, "model-fetch"), 3)
        check("while it comes, the meeting says so and offers it no more", coming, browser.execute_script(waits))
        made = wait_for(browser, lambda b: heard.exists(), 15)
        check("fetched from the meeting, its transcript is made", made and fetched.read_bytes() == right)
    # A model the user named, not on this computer: theirs, nothing offered.
    named = Window(meetings, "fr_FR.UTF-8", f'model = "{WORK / "mine.bin"}"\n')
    named.env.update(XDG_DATA_HOME=str(WORK / "data-named"), KITOKEY_MODEL_PUBLISHED=window.env["KITOKEY_MODEL_PUBLISHED"])
    with named as browser:
        browser.find_element(By.ID, "settings-link").click()
        drawn = wait_for(browser, lambda b: "Pas sur cet ordinateur" in b.execute_script(said))
        time.sleep(1)
        check("a model the user named is not offered: theirs", drawn and not browser.find_elements(By.ID, "model-fetch")
              and "proposé" not in browser.execute_script(said), browser.execute_script(said))
        before = len(asked)
        browser.execute_script("window.__TAURI__.core.invoke('model_fetch')")
        time.sleep(1.5)
        check("asked all the same, nothing is fetched for it", len(asked) == before and not (WORK / "data-named" / "kitokey" / "models").exists(), asked)
    server.shutdown()


def models_listed(now: dt.datetime) -> None:
    """U. Settings offer a short list of models to download, each said by what it is good for (D37)."""
    print("U. Settings: a short list of models to download, each said by what it is good for (D37)")
    served, turbo, small = WORK / "served-list", os.urandom(200_000), os.urandom(100_000)
    served.mkdir()
    (served / "turbo.bin").write_bytes(turbo)
    (served / "small.bin").write_bytes(os.urandom(100_000))  # served first: not the published file
    asked = []

    # Served slowly — 20 kB every 0.3 s — so that how far it is can be read.
    class Slow(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=str(served), **kwargs)

        def log_message(self, *_):
            asked.append(self.path)

        def copyfile(self, source, sink):
            while chunk := source.read(20_000):
                sink.write(chunk)
                sink.flush()
                time.sleep(0.3)

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Slow)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    at = f"http://127.0.0.1:{server.server_address[1]}"
    meetings, data = WORK / "listing", WORK / "data-listing"
    session(meetings / "2026-10-25_11-00", now - dt.timedelta(hours=1), 5, {"local.wav": 2000})
    heard = meetings / "2026-10-25_11-00" / "transcript.jsonl"
    models = data / "kitokey" / "models"
    window = Window(meetings, "fr_FR.UTF-8")
    window.env.update(XDG_DATA_HOME=str(data), KITOKEY_MODEL_PUBLISHED=";".join([
        f"{at}/turbo.bin 200000 {hashlib.sha256(turbo).hexdigest()}",
        f"{at}/small.bin 100000 {hashlib.sha256(small).hexdigest()}",
    ]))
    row = "return document.querySelector('[data-offered=\"{}\"]')?.innerText || ''"
    turbo_row, small_row = row.format("ggml-large-v3-turbo-q8_0.bin"), row.format("ggml-small-q8_0.bin")
    offers = "return [...document.querySelectorAll('#offered button, #model-fetch')].filter(b => !b.disabled).length"
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        listed = wait_for(browser, lambda b: b.execute_script(turbo_row) and b.execute_script(small_row))
        said = (browser.execute_script(turbo_row), browser.execute_script(small_row))
        check("Settings list Whisper large v3 turbo and Whisper small, each with its size, its host and what it is good for",
              listed and "Whisper large v3 turbo" in said[0] and "200 ko" in said[0] and "Recommandé" in said[0]
              and "Whisper small" in said[1] and "ordinateur modeste" in said[1]
              and all(at.split("/")[2] in line for line in said), said)
        check("each not on this computer is offered", "Télécharger" in said[0] and "Télécharger" in said[1], said)
        check("nothing is fetched before a click", asked == [], asked)
        chosen = browser.execute_script("return document.getElementById('model').selectedOptions[0]?.text || ''")
        check("the model chosen and missing is said so in the choice, not shown as another", "pas encore" in chosen, chosen)
        # A meeting waiting for the model chosen, the small one coming: its offer waits too.
        browser.execute_script("location.hash = '#/meeting/2026-10-25_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-transcript"))
        press(browser, "make-transcript")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#transcript-state #model-fetch"))
        browser.execute_script("window.__TAURI__.core.invoke('model_offered_fetch', {file: 'ggml-small-q8_0.bin'})")
        waits = wait_for(browser, lambda b: b.execute_script("return document.getElementById('model-fetch')?.disabled === true"), 5)
        check("another model coming, a meeting waiting for the chosen one offers it greyed", waits)
        wait_for(browser, lambda b: b.execute_script("return document.getElementById('model-fetch')?.disabled === false"), 15)
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.execute_script(small_row) and "Télécharger" in b.execute_script(small_row))
        # Clickable once the fetch before it has let go — greyed until then, a click on it is none.
        wait_for(browser, lambda b: b.execute_script("return document.querySelector('[data-offered=\"ggml-small-q8_0.bin\"] button')?.disabled === false"), 10)
        browser.execute_script("document.querySelector('[data-offered=\"ggml-small-q8_0.bin\"] button').click()")
        far = wait_for(browser, lambda b: "%" in b.execute_script(small_row), 5)
        check("being downloaded, how far is said on its row", far, browser.execute_script(small_row))
        check("one fetch at a time: nothing else is offered meanwhile", browser.execute_script(offers) == 0, browser.execute_script(offers))
        again = wait_for(browser, lambda b: "Télécharger" in b.execute_script(small_row) and "%" not in b.execute_script(small_row), 15)
        check("a file that is not the published one: nothing kept, offered again", again and not (models / "ggml-small-q8_0.bin").exists())
        (served / "small.bin").write_bytes(small)
        notified = WORK / "notified-small"
        monitor = subprocess.Popen(
            ["dbus-monitor", "--session", "interface='org.freedesktop.Notifications',member='Notify'"],
            stdout=open(notified, "w"), stderr=subprocess.DEVNULL,
        )
        try:
            time.sleep(0.5)
            browser.execute_script("document.querySelector('[data-offered=\"ggml-small-q8_0.bin\"] button').click()")
            here = wait_for(browser, lambda b: "Sur cet ordinateur" in b.execute_script(small_row), 15)
            check("fetched, its row says it is on this computer", here and (models / "ggml-small-q8_0.bin").read_bytes() == small, browser.execute_script(small_row))
            told = wait_for(browser, lambda b: "Choisissez-le" in notified.read_text(), 5)
        finally:
            monitor.terminate()
            monitor.wait(timeout=10)
        check("its notice says to choose it, never that a transcript begins",
              told and "La transcription est en cours" not in notified.read_text() and not heard.exists(), notified.read_text()[-400:])
        names = [o.text for o in browser.find_elements(By.CSS_SELECTOR, "#model option")]
        check("it is offered among the models to choose", any(name.startswith("Whisper small") for name in names), names)
        check("the choice stays the user's", "ggml-small" not in window.file.read_text()
              and "pas encore" in browser.execute_script("return document.getElementById('model').selectedOptions[0]?.text || ''"))
        # The turbo's row and D36's offer are the same fetch.
        browser.execute_script("document.querySelector('[data-offered=\"ggml-large-v3-turbo-q8_0.bin\"] button').click()")
        both = wait_for(browser, lambda b: "%" in b.execute_script(turbo_row) and not b.find_elements(By.ID, "model-fetch"), 5)
        check("the turbo's row and the missing model's offer are one fetch", both, browser.execute_script(turbo_row))
        ready = wait_for(browser, lambda b: "Sur cet ordinateur" in b.execute_script(turbo_row) and "Prêt" in (b.execute_script("return document.getElementById('model-state')?.innerText || ''")), 20)
        check("fetched, the model chosen is ready", ready and (models / "ggml-large-v3-turbo-q8_0.bin").read_bytes() == turbo)
    server.shutdown()


def list_refreshed(now: dt.datetime) -> None:
    """W. The list of models is read again only signed by the decider's key: until it is built in, none is (D40, D56)."""
    print("W. Settings: no list of models read again until the decider's key is built in, and it is said (D40, D56)")
    # When registry::PUBLIC_KEY is set, this scenario changes: the button is back, its host shown,
    # and an unsigned list served here is refused. The signed path is the shell's tests'
    # (tests/registry_fetch.rs), with a key of their own: no window can hold a key a test made.
    asked = []

    class List(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            asked.append(self.path)
            self.send_response(404)
            self.send_header("Content-Length", "0")
            self.end_headers()

        def log_message(self, *_):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), List)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    at = f"127.0.0.1:{server.server_address[1]}"
    meetings = WORK / "refreshed"
    meetings.mkdir()
    said = "return document.getElementById('offered-refreshed')?.innerText || ''"
    window = Window(meetings, "fr_FR.UTF-8")
    window.env.update(KITOKEY_REGISTRY=f"http://{at}/models.json")
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        waits = wait_for(browser, lambda b: "attend la clé qui la signe" in b.execute_script(said), 10)
        check("Settings say the list waits on the key that signs it, and the models stay offered",
              waits and "restent proposés" in browser.execute_script(said)
              and browser.find_elements(By.CSS_SELECTOR, '[data-offered="ggml-small-q8_0.bin"]'),
              browser.execute_script(said))
        check("no button offers to read it again", not browser.find_elements(By.ID, "offered-refresh"))
        # Asked all the same, it is refused, and logged so, with nothing asked of anyone.
        browser.execute_script("window.__TAURI__.core.invoke('registry_refresh').then(() => { window.listRead = 'read'; }, () => { window.listRead = 'refused'; })")
        answered = wait_for(browser, lambda b: b.execute_script("return window.listRead || ''"), 10)
        check("asked all the same, it is refused, and nothing crosses the network",
              answered and browser.execute_script("return window.listRead") == "refused" and asked == [],
              (browser.execute_script("return window.listRead"), asked))
    server.shutdown()


def speech_offered(now: dt.datetime) -> None:
    """X. On a Mac, Apple's SpeechAnalyzer is offered among who hears, its model fetched on the click (D42)."""
    print("X. Settings: Apple's SpeechAnalyzer offered where this Mac offers it, its model fetched on the click (D42)")
    # A program in place of kitokey-speech: this Mac offers it, Apple's model not here yet.
    program, asked = WORK / "kitokey-speech", WORK / "speech-asked"
    program.write_text(f"""#!/bin/sh
echo "$@" >> '{asked}'
case "$1" in
  status) echo '{{"available":true,"installed":false,"supported":true}}' ;;
  install) sleep 1; echo '{{"installed":true}}' ;;
esac
""")
    program.chmod(0o755)
    meetings = WORK / "speech"
    meetings.mkdir()
    pick = "return [...document.querySelectorAll('#model option')].map(o => o.value + '=' + o.text).join('|')"
    window = Window(meetings, "fr_FR.UTF-8")
    window.env.update(KITOKEY_SPEECH=str(program))
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        offered = wait_for(browser, lambda b: "speechanalyzer=" in b.execute_script(pick), 10)
        check("SpeechAnalyzer is offered among who hears, its model said not on this Mac yet",
              offered and "SpeechAnalyzer d’Apple, sur ce Mac — le modèle d’Apple n’est pas encore sur ce Mac" in browser.execute_script(pick),
              browser.execute_script(pick))
        check("its model is not fetched when Settings open", asked.read_text().split() == ["status", "fr-FR"], asked.read_text())
        Select(browser.find_element(By.ID, "model")).select_by_value("speechanalyzer")
        shown = wait_for(browser, lambda b: b.find_element(By.ID, "speech-fetch").is_displayed(), 5)
        check("chosen, it is not who hears yet: the download is offered, macOS fetching it from Apple",
              shown and "engine" not in window.file.read_text() and "install" not in asked.read_text()
              and "depuis Apple" in browser.find_element(By.ID, "speech-fetch").text, window.file.read_text())
        browser.find_element(By.ID, "speech-fetch").click()
        chosen = wait_for(browser, lambda b: 'engine = "speechanalyzer"' in window.file.read_text(), 10)
        check("on the click, macOS fetches it; then SpeechAnalyzer is who hears",
              chosen and "install fr-FR" in asked.read_text(), (asked.read_text(), window.file.read_text()))
    # Where no kitokey-speech answers — Linux, an older Mac — nothing is offered.
    window = Window(WORK / "speech", "fr_FR.UTF-8")
    window.env.update(KITOKEY_SPEECH=str(WORK / "nowhere"))
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "model"))
        time.sleep(1)
        check("where SpeechAnalyzer does not answer, it is not offered", "speechanalyzer" not in browser.execute_script(pick),
              browser.execute_script(pick))


def service_named(now: dt.datetime) -> None:
    """Z. Who hears chosen as the minutes' (D43): a service named, its host agreed to first, tested."""
    print("Z. Settings: a service named to hear, its host agreed to before anything is sent (D43)")
    meetings = WORK / "service"
    meetings.mkdir()
    # A meeting a service heard: its header names the service's host as the engine.
    session(meetings / "heard", dt.datetime.now().astimezone() - dt.timedelta(hours=1), 2, {"local.wav": 1000})
    (meetings / "heard" / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"api.groq.com","model":"whisper-large-v3","language":"fr","context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","voice":null,"start":0.5,"end":1.5,"text":"Bonjour."}\n{"type":"complete","duration":2.0}\n'
    )
    # OpenRouter's stand-in: it lists its transcription models, by what they do (D63).
    listed = []

    class Listing(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            listed.append(self.path)
            body = json.dumps({"data": [{"id": "openai/whisper-large-v3-turbo"}, {"id": "vendor/scribe"}]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Listing)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    stand = f"http://127.0.0.1:{server.server_address[1]}/api/v1"
    window = Window(meetings, "fr_FR.UTF-8")
    window.env.update(KITOKEY_HEARING=stand)
    said = "return document.getElementById('transcription-said')?.innerText || ''"
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "model"))
        check("the service's fields are hidden until it is chosen", not browser.find_element(By.ID, "transcription-service").is_displayed())
        Select(browser.find_element(By.ID, "model")).select_by_value("service")
        shown = wait_for(browser, lambda b: b.find_element(By.ID, "transcription-service").is_displayed(), 5)
        check("chosen, its address, its model and its key are asked", shown and browser.find_elements(By.ID, "transcription-key"))
        # An address that is none, or a name that only starts as this computer's: said, nothing saved.
        browser.find_element(By.ID, "transcription-provider").send_keys("hears.example/v1")
        browser.find_element(By.ID, "transcription-save").click()
        bad = wait_for(browser, lambda b: "l’adresse commence par https://" in b.execute_script(said), 5)
        browser.find_element(By.ID, "transcription-provider").clear()
        browser.find_element(By.ID, "transcription-provider").send_keys("https://localhost.hears.example/v1")
        browser.find_element(By.ID, "transcription-model").send_keys("whisper-1")
        browser.find_element(By.ID, "transcription-save").click()
        elsewhere = wait_for(browser, lambda b: "acceptez d’abord" in b.execute_script(said), 5)
        check("an address that is none, or a host that only starts as this computer's, is said and not saved",
              bad and elsewhere and "transcription_provider" not in window.file.read_text(), browser.execute_script(said))
        browser.find_element(By.ID, "transcription-provider").clear()
        browser.find_element(By.ID, "transcription-model").clear()
        # A host elsewhere, not agreed to: nothing saved, the agreement asked.
        browser.find_element(By.ID, "transcription-provider").send_keys("https://hears.example/v1")
        browser.find_element(By.ID, "transcription-model").send_keys("whisper-1")
        agreement = browser.find_element(By.ID, "transcription-agreed").find_element(By.XPATH, "..").text
        browser.find_element(By.ID, "transcription-save").click()
        refused = wait_for(browser, lambda b: "acceptez d’abord" in b.execute_script(said), 5)
        check("a host elsewhere is not saved before the user agreed, the host named in the agreement",
              refused and "hears.example" in agreement and "transcription_provider" not in window.file.read_text(), (agreement, browser.execute_script(said)))
        # Agreed: saved, and who hears is that service.
        browser.find_element(By.ID, "transcription-agreed").click()
        browser.find_element(By.ID, "transcription-save").click()
        saved = wait_for(browser, lambda b: 'engine = "service"' in window.file.read_text(), 5)
        check("agreed, the service is saved as who hears, its host kept as agreed",
              saved and 'transcription_agreed = "hears.example"' in window.file.read_text() and "whisper-1" in window.file.read_text(), window.file.read_text())
        # Saved, the model no longer needed: Settings are drawn anew at the next reading of the makings,
        # perhaps between finding the button and clicking it. Clicked in the page, as it is now.
        browser.execute_script("document.getElementById('transcription-test').click()")
        tested = wait_for(browser, lambda b: b.execute_script(said).startswith("Il entend, avec les temps de chaque segment"), 10)
        check("Tester says in one sentence what the service answered: it hears, with each segment's times", tested, browser.execute_script(said))
        # The page drawn anew under a slow answer, as a model come or the window back do: nothing lost, no second ask.
        browser.execute_script("const real = window.__TAURI__.core; window.answer = null; window.asked = 0; window.answered = 0; window.held = window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => { if (asked !== 'transcription_tested') return real.invoke(asked, ...rest); window.asked += 1; return new Promise((resolve) => { window.answer = () => real.invoke(asked, ...rest).then((told) => { resolve(told); setTimeout(() => { window.answered += 1; }); }); }); } }; window.restore = () => { window.__TAURI__.core = real; };")
        browser.execute_script("window.__old = document.getElementById('transcription-said'); document.getElementById('transcription-test').click()")
        wait_for(browser, lambda b: b.execute_script("return window.answer !== null"), 5)
        browser.execute_script("refresh()")
        drawn = wait_for(browser, lambda b: b.execute_script("return !window.__old.isConnected && document.getElementById('transcription-said') !== null"), 5)
        check("drawn anew while it asks: Tester stays disabled", drawn and browser.execute_script("return document.getElementById('transcription-test').disabled"))
        browser.execute_script("window.answer()")
        kept = wait_for(browser, lambda b: b.execute_script(said).startswith("Il entend, avec les temps de chaque segment"), 10)
        check("drawn anew while it asks: the answer is said on the page as it is", kept and not browser.execute_script("return document.getElementById('transcription-test').disabled") and browser.execute_script("return window.asked") == 1, browser.execute_script(said))
        browser.execute_script("window.restore(); window.__old = document.getElementById('transcription-said'); refresh()")
        wait_for(browser, lambda b: b.execute_script("return !window.__old.isConnected && document.getElementById('transcription-said') !== null"), 5)
        hears = "Il entend, avec les temps de chaque segment"
        testing = "return document.getElementById('transcription-test').disabled"
        check("drawn anew after it answered: the answer stays", browser.execute_script(said).startswith(hears), browser.execute_script(said))
        check("drawn anew after it answered: Tester can be asked again", not browser.execute_script(testing))
        # Enregistrer while Tester asks: the sentence said, then the answer; the button given back.
        browser.execute_script("window.answer = null; window.__TAURI__.core = window.held; document.getElementById('transcription-test').click()")
        wait_for(browser, lambda b: b.execute_script("return window.answer !== null"), 5)
        browser.execute_script("document.getElementById('transcription-save').click()")
        saved_said = wait_for(browser, lambda b: "hears.example" in b.execute_script(said), 5)
        check("Enregistrer while Tester asks: said, Tester still disabled", saved_said and browser.execute_script(testing), browser.execute_script(said))
        browser.execute_script("window.answer()")
        check("then Tester's answer is said, and it can be asked again", wait_for(browser, lambda b: b.execute_script(said).startswith(hears) and not b.execute_script(testing), 10), browser.execute_script(said))
        browser.execute_script("window.restore()")
        # Enregistrer's own sentence: it outlives a redraw too.
        browser.execute_script("document.getElementById('transcription-save').click()")
        wait_for(browser, lambda b: "hears.example" in b.execute_script(said), 5)
        kept_said = browser.execute_script(said)
        browser.execute_script("window.__old = document.getElementById('transcription-said'); refresh()")
        wait_for(browser, lambda b: b.execute_script("return !window.__old.isConnected && document.getElementById('transcription-said') !== null"), 5)
        check("Enregistrer's sentence: it stays when drawn anew", browser.execute_script(said) == kept_said, browser.execute_script(said))
        # Another address saved while Tester asks about the first: its answer is not said of the second.
        address_to = "const a = document.getElementById('transcription-provider'); a.value = arguments[0]; a.dispatchEvent(new Event('input')); document.getElementById('transcription-agreed').click(); document.getElementById('transcription-save').click()"
        browser.execute_script("window.answer = null; window.__TAURI__.core = window.held; document.getElementById('transcription-test').click()")
        wait_for(browser, lambda b: b.execute_script("return window.answer !== null"), 5)
        browser.execute_script(address_to, "https://other.example/v1")
        moved = wait_for(browser, lambda b: "other.example" in b.execute_script(said) and 'transcription_agreed = "other.example"' in window.file.read_text(), 5)
        browser.execute_script("window.answer()")
        back = wait_for(browser, lambda b: not b.execute_script(testing), 10)
        check("another service saved while Tester asks: the answer is not said, the button given back",
              moved and back and "other.example" in browser.execute_script(said) and not browser.execute_script(said).startswith(hears), browser.execute_script(said))
        browser.execute_script("window.restore()")
        browser.execute_script(address_to, "https://hears.example/v1")
        wait_for(browser, lambda b: 'transcription_agreed = "hears.example"' in window.file.read_text(), 5)
        # Another model saved while Tester asks about the first: the same.
        model_to = "const m = document.getElementById('transcription-model'); m.value = arguments[0]; m.dispatchEvent(new Event('input')); document.getElementById('transcription-save').click()"
        browser.execute_script("window.answer = null; window.__TAURI__.core = window.held; document.getElementById('transcription-test').click()")
        wait_for(browser, lambda b: b.execute_script("return window.answer !== null"), 5)
        browser.execute_script(model_to, "whisper-2")
        moved = wait_for(browser, lambda b: 'transcription_model = "whisper-2"' in window.file.read_text(), 5)
        browser.execute_script("window.answer()")
        back = wait_for(browser, lambda b: not b.execute_script(testing), 10)
        check("another model saved while Tester asks: the answer is not said, the button given back",
              moved and back and not browser.execute_script(said).startswith(hears), browser.execute_script(said))
        browser.execute_script("window.restore()")
        browser.execute_script(model_to, "whisper-1")
        wait_for(browser, lambda b: 'transcription_model = "whisper-1"' in window.file.read_text(), 5)
        # What was saved cannot be read to bind the answer: said, the button given back.
        browser.execute_script("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'settings_shown' ? Promise.reject('REFUS') : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; }; document.getElementById('transcription-test').click()")
        told = wait_for(browser, lambda b: b.execute_script(said) == "REFUS" and not b.execute_script(testing), 5)
        browser.execute_script("window.restore()")
        check("what was saved not read: said, the button given back", told, browser.execute_script(said))
        # Who heard a meeting, said where it is read: the service's host, never « sur cet ordinateur ».
        browser.execute_script("location.hash = '#/meeting/heard'")
        makers = lambda b: [p.text for p in b.find_elements(By.CSS_SELECTOR, "#meeting .maker")]
        named = wait_for(browser, lambda b: "Écrit par whisper-large-v3, via api.groq.com" in makers(b), 5)
        check("a meeting a service heard names the service, never this computer", named and not any("sur cet ordinateur" in m for m in makers(browser)), makers(browser))
        browser.execute_script("location.hash = '#/settings'")
        note = wait_for(browser, lambda b: "reçoit le son de vos réunions" in b.find_element(By.ID, "settings").text, 5)
        check("Settings say a transcription service receives the meetings' sound", note and "L’audio ne quitte jamais" not in browser.find_element(By.ID, "settings").text)
        check("another address and back: the last answer is gone", browser.execute_script(said) == "", browser.execute_script(said))
        # Offered by name, as the minutes' services are (D63).
        offered = lambda b, id_: b.execute_script("return [...document.querySelectorAll(arguments[0])].map(o => [o.value, o.textContent])", f"#{id_} option")
        names = [said_ for value, said_ in offered(browser, "model") if value.startswith("service:")]
        check("the services that hear are offered by name, as the minutes' are: OpenRouter, OpenAI", names == ["OpenRouter", "OpenAI"], names)
        # Another choice: what was said of the last goes at once, on the same page.
        browser.execute_script("document.getElementById('transcription-test').click()")
        wait_for(browser, lambda b: b.execute_script(said).startswith(hears), 10)
        Select(browser.find_element(By.ID, "model")).select_by_value("service:0")
        check("another service chosen: the last answer goes at once", browser.execute_script(said) == "", browser.execute_script(said))
        Select(browser.find_element(By.ID, "model")).select_by_value("service")
        # Another service chosen while Tester asks: its button at once, the old answer never said, even drawn anew.
        browser.execute_script("window.answer = null; window.answered = 0; window.__TAURI__.core = window.held; document.getElementById('transcription-test').click()")
        wait_for(browser, lambda b: b.execute_script("return window.answer !== null"), 5)
        Select(browser.find_element(By.ID, "model")).select_by_value("service:0")
        check("another service chosen while Tester asks: its button given back at once", not browser.execute_script(testing))
        browser.execute_script("window.answer()")
        wait_for(browser, lambda b: b.execute_script("return window.answered") >= 1, 10)
        check("the answer for the service left is not said", browser.execute_script(said) == "", browser.execute_script(said))
        browser.execute_script("window.restore(); window.__old = document.getElementById('transcription-said'); refresh()")
        wait_for(browser, lambda b: b.execute_script("return !window.__old.isConnected && document.getElementById('transcription-said') !== null"), 5)
        check("drawn anew after: nor then", browser.execute_script(said) == "", browser.execute_script(said))
        listed.clear()
        Select(browser.find_element(By.ID, "model")).select_by_value("service:0")
        models = wait_for(browser, lambda b: [s for _, s in offered(b, "transcription-models")] == ["openai/whisper-large-v3-turbo", "vendor/scribe", "Un autre modèle…"], 5)
        check("OpenRouter chosen: its transcription models listed from it, its address filled and not asked",
              models and listed == ["/api/v1/models?output_modalities=transcription"] and not browser.find_element(By.ID, "transcription-provider").is_displayed()
              and not browser.find_element(By.ID, "transcription-model").is_displayed(), (offered(browser, "transcription-models"), listed))
        Select(browser.find_element(By.ID, "transcription-models")).select_by_value("")
        typed = wait_for(browser, lambda b: b.find_element(By.ID, "transcription-model").is_displayed()
                         and b.execute_script("return document.activeElement.id") == "transcription-model", 3)
        check("« Un autre modèle… »: a model typed by hand", typed)
        Select(browser.find_element(By.ID, "transcription-models")).select_by_value("vendor/scribe")
        browser.find_element(By.ID, "transcription-key").send_keys("sk-stand-in-not-a-key")
        if not browser.find_element(By.ID, "transcription-agreed").is_selected():
            browser.find_element(By.ID, "transcription-agreed").click()
        press(browser, "transcription-save")
        saved = wait_for(browser, lambda b: f'transcription_provider = "{stand}"' in window.file.read_text(), 5)
        text = window.file.read_text()
        check("saved: OpenRouter's address, the model chosen in its list, the key never in the settings",
              saved and 'transcription_model = "vendor/scribe"' in text and "sk-stand-in" not in text, text)
        (WORK / "hearing-key").unlink(missing_ok=True)
        press(browser, "transcription-test")
        tested = wait_for(browser, lambda b: b.execute_script(said).startswith("Il entend, avec les temps de chaque segment"), 10)
        check("Tester, on the service chosen from the list, says it hears with each segment's times — the key the keyring kept given to it",
              tested and (WORK / "hearing-key").exists() and (WORK / "hearing-key").read_text() == "sk-stand-in-not-a-key", browser.execute_script(said))
        # Each cause its own sentence (D72); the raw text never, outside developer mode.
        code = WORK / "test-code"
        try:
            for cause, sentence in TEST_CAUSES:
                free(browser, "transcription-test", f"a service, {cause}")
                code.write_text(cause)
                tester_click(browser, "transcription-test", f"a service, {cause}")
                held = wait_for(browser, lambda b: b.execute_script(said) == sentence, 10)
                check(f"Tester of a service, {cause}: its own sentence, what to do, no raw text", held, browser.execute_script(said))
                # A second test in flight once the sentence is read: the next case waits for it (#799).
                tester_click(browser, "transcription-test", f"a service, {cause} again")
            free(browser, "transcription-test", "a service, a code unknown")
            code.write_text("provider.of_a_later_version")
            tester_click(browser, "transcription-test", "a service, a code unknown")
            unknown = wait_for(browser, lambda b: b.execute_script(said).startswith("Le modèle ne répond pas"), 10)
            check("a code unknown is read as no code: today's sentence", unknown and "vérifiez son adresse, son nom et sa clé" in browser.execute_script(said), browser.execute_script(said))
        finally:
            code.unlink(missing_ok=True)
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 5)
        browser.execute_script("location.hash = '#/settings'")
        kept = wait_for(browser, lambda b: b.find_elements(By.ID, "model") and b.find_element(By.ID, "model").get_attribute("value") == "service:0"
                        and b.find_element(By.ID, "transcription-models").get_attribute("value") == "vendor/scribe", 5)
        check("drawn again: OpenRouter chosen, its model in its list", kept)
        Select(browser.find_element(By.ID, "model")).select_by_value("service:1")
        named = wait_for(browser, lambda b: [s for _, s in offered(b, "transcription-models")] == ["whisper-1", "Un autre modèle…"], 5)
        check("OpenAI chosen: the model it publishes with segments, named in the repository — not its own list",
              named and len(listed) == 2, (offered(browser, "transcription-models"), listed))
        # In developer mode, the sentence and the program's last line, its name off.
        code.write_text(TEST_CAUSES[-1][0])
        try:
            before = browser.find_element(By.ID, "transcription-said")
            press(browser, "developer")
            WebDriverWait(browser, 5).until(staleness_of(before))
            free(browser, "transcription-test", "a service in developer mode")
            tester_click(browser, "transcription-test", "a service in developer mode")
            raw = wait_for(browser, lambda b: "raw words of the service" in b.execute_script(said), 10)
            told = browser.execute_script(said)
            check("Tester of a service in developer mode: the cause, then the raw line alone",
                  raw and told.startswith(TEST_CAUSES[-1][1]) and "kitokey-transcription:" not in told and "earlier line" not in told, told)
        finally:
            code.unlink(missing_ok=True)
    server.shutdown()


def ollama_pulled(now: dt.datetime) -> None:
    """V. The minutes' recommended model fetched by Ollama, on the user's click (D37)."""
    print("V. Settings: qwen3:8b fetched by Ollama on the user's click; without Ollama, said where to get it (D37)")
    held, pulls, ways = ["mistral:7b"], [], ["cut", "refused", "whole"]

    # An Ollama of the scenario's: its models, and a pull that is cut, then refused, then whole —
    # each said in lines, slowly, as Ollama streams it.
    class Ollama(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_GET(self):
            body = json.dumps({"models": [{"name": name} for name in held]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self):
            pulls.append(json.loads(self.rfile.read(int(self.headers["content-length"]))))
            way = ways.pop(0)
            self.send_response(200)
            self.send_header("content-type", "application/x-ndjson")
            self.end_headers()
            self.wfile.write(b'{"status":"pulling manifest"}\n')
            for done in range(0, 1001, 100):
                if way == "cut" and done == 300:
                    return
                if way == "refused" and done == 300:
                    self.wfile.write(b'{"error":"pull model manifest: file does not exist"}\n')
                    return
                self.wfile.write(json.dumps({"status": "pulling a1b2", "digest": "sha256:a1b2", "total": 1000, "completed": done}).encode() + b"\n")
                self.wfile.flush()
                time.sleep(0.3)
            held.append("qwen3:8b")
            self.wfile.write(b'{"status":"success"}\n')

        def log_message(self, *_):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Ollama)
    threading.Thread(target=server.serve_forever, daemon=True).start()

    # Whisper small served slowly, and not as published: a fetch that comes, then is refused.
    class Slow(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200)
            self.send_header("content-length", "100000")
            self.end_headers()
            for _ in range(10):
                self.wfile.write(b"x" * 10_000)
                self.wfile.flush()
                time.sleep(0.3)

        def log_message(self, *_):
            pass

    whisper = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Slow)
    threading.Thread(target=whisper.serve_forever, daemon=True).start()
    at = f"http://127.0.0.1:{whisper.server_address[1]}"
    meetings = WORK / "pulling"
    meetings.mkdir()
    window = Window(meetings, "fr_FR.UTF-8", ollama=f"http://127.0.0.1:{server.server_address[1]}")
    window.env.update(XDG_DATA_HOME=str(WORK / "data-pulling"), KITOKEY_MODEL_PUBLISHED=f"{at}/turbo.bin 1 {'0' * 64};{at}/small.bin 100000 {'0' * 64}")
    row = "return document.querySelector('[data-offered=\"qwen3:8b\"]')?.innerText || ''"
    pull = "document.querySelector('[data-offered=\"qwen3:8b\"] button').click()"
    offers = "return [...document.querySelectorAll('#offered button, #model-fetch')].filter(b => !b.disabled).length"
    notified = WORK / "notified-pull"
    monitor = subprocess.Popen(
        ["dbus-monitor", "--session", "interface='org.freedesktop.Notifications',member='Notify'"],
        stdout=open(notified, "w"), stderr=subprocess.DEVNULL,
    )
    try:
        with window as browser:
            browser.find_element(By.ID, "settings-link").click()
            listed = wait_for(browser, lambda b: "Télécharger" in b.execute_script(row))
            said = browser.execute_script(row)
            check("qwen3:8b is offered for the minutes, its size and what it is good for said", listed and "qwen3:8b" in said and "5 Go" in said and "comptes rendus" in said, said)
            check("nothing is fetched before a click", pulls == [], pulls)
            # A Whisper model coming: the pull waits — its row greyed, and asked all the same, Ollama is not.
            browser.execute_script("document.querySelector('[data-offered=\"ggml-small-q8_0.bin\"] button').click()")
            wait_for(browser, lambda b: "%" in b.execute_script(row.replace("qwen3:8b", "ggml-small-q8_0.bin")), 5)
            greyed = browser.execute_script("return document.querySelector('[data-offered=\"qwen3:8b\"] button')?.disabled === true")
            browser.execute_script("window.__TAURI__.core.invoke('ollama_pull_offered')")
            time.sleep(1)
            check("a Whisper model coming, the pull waits: its row greyed, Ollama not asked", greyed and pulls == [], (greyed, pulls))
            wait_for(browser, lambda b: "Télécharger" in b.execute_script(row.replace("qwen3:8b", "ggml-small-q8_0.bin")) and "%" not in b.execute_script(row.replace("qwen3:8b", "ggml-small-q8_0.bin")), 15)
            wait_for(browser, lambda b: b.execute_script("return document.querySelector('[data-offered=\"qwen3:8b\"] button')?.disabled === false"), 5)
            for way in ("cut", "refused"):
                browser.execute_script(pull)
                far = wait_for(browser, lambda b: "%" in b.execute_script(row), 5)
                check(f"{way}: how far is said while Ollama fetches it, nothing else offered meanwhile", far and browser.execute_script(offers) == 0, browser.execute_script(row))
                again = wait_for(browser, lambda b: "Télécharger" in b.execute_script(row), 15)
                check(f"{way}: said, and offered again", again and "n’a pas été téléchargé" in notified.read_text(), notified.read_text()[-300:])
            browser.execute_script(pull)
            here = wait_for(browser, lambda b: "Sur cet ordinateur" in b.execute_script(row), 20)
            check("whole: on this computer, Ollama asked for qwen3:8b", here and pulls[-1].get("model") == "qwen3:8b", (browser.execute_script(row), pulls))
            first = browser.execute_script("return document.querySelector('#minutes optgroup option')?.textContent || ''")
            check("it is offered first among the minutes' models", "qwen3:8b" in first, first)
            check("its notice says it is ready for the minutes", wait_for(browser, lambda b: "comptes rendus" in notified.read_text(), 5), notified.read_text()[-300:])
    finally:
        monitor.terminate()
        monitor.wait(timeout=10)
        whisper.shutdown()
    # No Ollama answering: where to get it is said, and nothing offered — Kitokey never installs it.
    with Window(meetings, "fr_FR.UTF-8") as browser:
        browser.find_element(By.ID, "settings-link").click()
        absent = wait_for(browser, lambda b: "ollama.com" in b.execute_script(row))
        check("no Ollama: the row says it is needed and where to get it, with no button",
              absent and not browser.find_elements(By.CSS_SELECTOR, '[data-offered="qwen3:8b"] button'), browser.execute_script(row))
    server.shutdown()


def first_run(now: dt.datetime) -> None:
    """Y. The first start (D58): the full window opens on one question, then the setting said plainly."""
    print("Y. the first start: one question, the setting said plainly, private by default (D58)")
    welcome = "return !document.getElementById('welcome').hidden && document.getElementById('welcome').innerText || ''"
    focused = "return [document.activeElement.tagName, document.activeElement.name || document.activeElement.id, document.activeElement.value || ''].join(':')"
    picked = "return [...document.querySelectorAll('#welcome input[type=radio]:checked')].map(i => i.value).join(',')"
    summary = "return document.getElementById('welcome-summary')?.innerText || ''"
    keys = lambda browser, *sent: ActionChains(browser).send_keys(*sent).perform()

    def started(name: str, language: str = "fr_FR.UTF-8", dark: bool = False, ollama: str = "http://127.0.0.1:9") -> "Window":
        meetings = WORK / f"welcome-{name}"
        meetings.mkdir()
        # No `window` line: the small one is the one left, as at a real first start (D35).
        window = Window(meetings, language, ollama=ollama, small=True, welcomed=False, args=())
        window.env.update(XDG_DATA_HOME=str(WORK / f"data-welcome-{name}"), KITOKEY_SPEECH=str(WORK / "nowhere"))
        # The dark theme asked of GTK, whatever the desktop shows; the light one cannot be asked
        # of a dark desktop without changing the user's setting — its colours are put on the page
        # for its screenshots instead, below.
        if dark:
            window.env.update(GTK_THEME="Adwaita:dark")
        return window

    def seen(browser, name: str) -> None:
        time.sleep(0.4)  # the card's arrival, 180 ms, ended
        shot_themed(browser, name, "light" if "light" in name.split("-") else "dark")

    def until_question(browser) -> bool:
        return wait_for(browser, lambda b: "Vos réunions parlent-elles" in b.execute_script(welcome) or "Do your meetings" in b.execute_script(welcome), 10)

    # 1. Skipped: everything on this computer, the meetings opened.
    window = started("skip")
    with window as browser:
        shown = until_question(browser)
        check("started without --window, not welcomed, no meeting: the full window opens on the assistant",
              shown and not browser.find_elements(By.ID, "small-record"), browser.execute_script(welcome))
        said = browser.execute_script(welcome)
        check("one question, its three answers in plain words, its sentence typeset as French has it",
              "Réglons Kitokey en une question." in said and "Vos réunions parlent-elles de sujets confidentiels ?" in said
              and all(a in said for a in ("Oui, souvent", "Rarement ou jamais", "Je ne sais pas", "Passer — tout sur cet ordinateur", "Suivant")), said)
        check("nothing is chosen for the user", browser.execute_script(picked) == "", browser.execute_script(picked))
        check("each answer is a radio of one group, named by the question",
              browser.execute_script("const g = document.querySelector('#welcome [role=radiogroup]'); return !!g && !!g.getAttribute('aria-labelledby') && g.querySelectorAll('input[type=radio][name=private]').length === 3"))
        check("no sidebar of the meetings while it is answered", not browser.find_element(By.ID, "sidebar").is_displayed())
        marked = wait_for(browser, lambda b: "welcomed = true" in window.file.read_text(), 5)
        check("welcomed is written as soon as it is shown, nothing else", marked and "minutes_provider" not in window.file.read_text(), window.file.read_text())
        browser.set_window_size(420, 400)
        time.sleep(0.5)
        wide = browser.execute_script("return document.documentElement.scrollWidth <= document.documentElement.clientWidth")
        check("at the window's smallest size nothing overflows sideways", wide)
        seen(browser, "d58-1-question-light-small")
        browser.set_window_size(1100, 800)
        time.sleep(0.3)
        card = browser.execute_script("const c = document.querySelector('#welcome .welcome'), s = getComputedStyle(c); return [c.getBoundingClientRect().width / document.documentElement.clientWidth, parseFloat(s.paddingLeft)]")
        check("at the window's opening size the card takes two thirds of it, its margin 20 px at most", 0.6 <= card[0] <= 0.7 and card[1] <= 20, card)
        seen(browser, "d58-1-question-light")
        # Passer, by the keyboard: Tab from the question reaches it after the answers.
        keys(browser, Keys.TAB)
        first = browser.execute_script(focused)
        keys(browser, Keys.TAB)
        skip = browser.execute_script(focused)
        check("Tab reaches the answers, then Passer", first.startswith("INPUT:private") and skip == "BUTTON:welcome-skip:", (first, skip))
        # Drawn once the window has the system's focus: waited for, the desktop being in use (#510).
        # Asked for, then waited for: another window of the desktop may hold the system's focus.
        browser.switch_to.window(browser.current_window_handle)
        ring = wait_for(browser, lambda b: b.execute_script("const s = getComputedStyle(document.activeElement); return s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) > 0"), 10)
        check("the focus is seen", ring)
        keys(browser, Keys.ENTER)
        listed = wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed() and b.find_element(By.ID, "sidebar").is_displayed(), 10)
        text = window.file.read_text()
        check("Passer: the minutes by Ollama on this computer, the meetings opened",
              listed and 'minutes_provider = "http://127.0.0.1:9/v1"' in text and 'minutes_model = "qwen3:8b"' in text, text)
        check("the window left is not changed: the small one stays the one reopened (D35)", "window =" not in text, text)
        check("nothing is fetched on Passer", not (WORK / "data-welcome-skip" / "kitokey" / "models").exists())
    # Started again, welcomed: no assistant.
    window.args = ["--window"]
    with window as browser:
        time.sleep(1)
        check("welcomed, it never comes back at a start", browser.execute_script(welcome) == "" and browser.find_elements(By.ID, "small-record"))

    # 2. Private, by the keyboard: the summary, then Plus tard.
    window = started("private")
    with window as browser:
        until_question(browser)
        keys(browser, Keys.TAB, Keys.SPACE)
        check("Space chooses the answer focused", browser.execute_script(picked) == "often", browser.execute_script(picked))
        keys(browser, Keys.ARROW_DOWN)
        keys(browser, Keys.ARROW_DOWN)
        check("the arrows go through the answers", browser.execute_script(picked) == "unsure", browser.execute_script(picked))
        keys(browser, Keys.TAB, Keys.TAB, Keys.ENTER)
        shown = wait_for(browser, lambda b: "Mode privé" in b.execute_script(summary), 5)
        said = browser.execute_script(summary)
        check("« Je ne sais pas »: the private mode, who transcribes, who writes, the reminder, each said plainly",
              shown and "Vos réunions ne quittent pas cet ordinateur." in said and "Transcription" in said and "Whisper large v3 turbo" in said
              and "Compte rendu" in said and "Qwen3 8b" in said and "Rappel" in said and "ggml" not in said, said)
        check("Ollama not answering: the minutes wait for it, with ollama.com", "ollama.com" in said, said)
        check("the model missing: offered with its size and host",
              "Télécharger le modèle (874 Mo)" in said.replace(" Mo", " Mo") and "Depuis huggingface.co, vérifié avant d’être utilisé." in said, said)
        check("nothing is saved before the last screen", "minutes_provider" not in window.file.read_text(), window.file.read_text())
        seen(browser, "d58-3-summary-private-light")
        press(browser, "welcome-later")
        listed = wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 10)
        check("Plus tard: saved private, the meetings opened, the model left unfetched",
              listed and 'minutes_model = "qwen3:8b"' in window.file.read_text() and not (WORK / "data-welcome-private" / "kitokey" / "models").exists(),
              window.file.read_text())

    # 3. Rarely, then a service: connected, chosen in Settings.
    window = started("service")
    with window as browser:
        until_question(browser)
        keys(browser, Keys.TAB, Keys.ARROW_DOWN)
        check("the arrows choose « Rarement ou jamais »", browser.execute_script(picked) == "rarely", browser.execute_script(picked))
        keys(browser, Keys.TAB, Keys.TAB, Keys.ENTER)
        asked = wait_for(browser, lambda b: "Qui écrit le compte rendu ?" in b.execute_script(welcome), 5)
        said = browser.execute_script(welcome)
        check("then who writes the minutes, its two answers, no address nor key asked",
              asked and "Cet ordinateur" in said and "Un service en ligne" in said and not browser.find_elements(By.CSS_SELECTOR, "#welcome input:not([type=radio])"), said)
        seen(browser, "d58-2-writer-light")
        keys(browser, Keys.TAB, Keys.ARROW_DOWN)
        check("« Un service en ligne » chosen by the keyboard", browser.execute_script(picked) == "service", browser.execute_script(picked))
        keys(browser, Keys.TAB, Keys.TAB, Keys.ENTER)
        shown = wait_for(browser, lambda b: "Mode connecté" in b.execute_script(summary), 5)
        said = browser.execute_script(summary)
        check("the connected mode: the minutes online, the transcription on this computer",
              shown and "la transcription reste sur cet ordinateur" in said and "Choisir mon service" in said, said)
        seen(browser, "d58-3-summary-connected-light")
        press(browser, "welcome-go")
        opened = wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed()
                          and b.execute_script("return document.activeElement.id") == "minutes", 10)
        check("Choisir mon service opens Settings at the minutes, nothing saved for them yet",
              opened and "minutes_provider" not in window.file.read_text(), (browser.execute_script("return document.activeElement.id"), window.file.read_text()))
        # Refaire le réglage: Annuler changes nothing; confirmed, it is saved.
        before = window.file.read_text()
        press(browser, "welcome-again")
        again = until_question(browser)
        check("Refaire le réglage opens the assistant, with Annuler", again and "Annuler" in browser.execute_script(welcome) and "Passer" not in browser.execute_script(welcome))
        press(browser, "welcome-skip")
        back = wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 5)
        check("Annuler: back to Settings, nothing changed", back and window.file.read_text() == before, window.file.read_text())
        press(browser, "welcome-again")
        until_question(browser)
        browser.execute_script("document.querySelector('#welcome input[value=often]').click()")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Mode privé" in b.execute_script(summary), 5)
        check("nothing changes before the last screen is confirmed", window.file.read_text() == before, window.file.read_text())
        press(browser, "welcome-later")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 10)
        check("confirmed, it is saved", 'minutes_model = "qwen3:8b"' in window.file.read_text(), window.file.read_text())
        check("and the reminder left on, as it was", "reminder = false" not in window.file.read_text(), window.file.read_text())
        # Private changes who transcribes and who writes, and nothing else: from a service that
        # hears, with the reminder off, a model and a folder of the user's, confirmed private.
        lines = [line for line in window.file.read_text().splitlines()
                 if not line.startswith(("engine", "transcription_", "reminder", "model ="))]
        mine = window.file.parent / "my-model.bin"
        window.file.write_text("\n".join(lines + [
            'engine = "service"', 'transcription_provider = "http://127.0.0.1:9/v1"',
            'transcription_model = "a-model"', 'transcription_agreed = "127.0.0.1:9"',
            "reminder = false", f'model = "{mine}"']) + "\n")
        kept = [line for line in window.file.read_text().splitlines()
                if line.startswith(("transcription_", "reminder", "model =", "sessions"))]
        # Refaire, confirmed connected, from a setting that saving private would change — a service
        # that hears: Settings at the minutes, nothing changed by itself.
        press(browser, "settings-link")
        wait_for(browser, lambda b: b.find_elements(By.ID, "welcome-again") and b.find_element(By.ID, "welcome-again").is_displayed(), 10)
        press(browser, "welcome-again")
        until_question(browser)
        browser.execute_script("document.querySelector('#welcome input[value=rarely]').click()")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#welcome input[name=writer]"), 5)
        browser.execute_script("document.querySelector('#welcome input[value=service]').click()")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Mode connecté" in b.execute_script(summary), 5)
        staged = window.file.read_text()
        press(browser, "welcome-go")
        opened = wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed()
                          and b.execute_script("return document.activeElement.id") == "minutes", 10)
        check("Refaire, confirmed connected: Settings at the minutes, nothing changed by itself",
              opened and window.file.read_text() == staged, window.file.read_text())
        press(browser, "settings-link")
        wait_for(browser, lambda b: b.find_elements(By.ID, "welcome-again") and b.find_element(By.ID, "welcome-again").is_displayed(), 10)
        press(browser, "welcome-again")
        until_question(browser)
        browser.execute_script("document.querySelector('#welcome input[value=often]').click()")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Mode privé" in b.execute_script(summary), 5)
        press(browser, "welcome-later")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 10)
        after = window.file.read_text()
        check("private from a service: the engine back to this computer",
              'engine = "whisper"' in after, after)
        check("and nothing else: the service's fields, the reminder, the model and the folder as they were",
              all(line in after.splitlines() for line in kept), (kept, after))

    # 4. The dark theme, in English.
    window = started("dark", "en_GB.UTF-8", dark=True)
    with window as browser:
        until_question(browser)
        dark = browser.execute_script("return matchMedia('(prefers-color-scheme: dark)').matches")
        check("the system's dark theme is the page's", dark and browser.execute_script("return getComputedStyle(document.body).backgroundColor") == GROUND["dark"])
        said = browser.execute_script(welcome)
        check("in English", "Do your meetings discuss confidential matters?" in said and "Skip — everything on this computer" in said, said)
        seen(browser, "d58-1-question-dark")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Private mode" in b.execute_script(summary), 5)
        check("Next with nothing chosen is private", "Private mode" in browser.execute_script(summary), browser.execute_script(summary))
        seen(browser, "d58-3-summary-private-dark")
        press(browser, "welcome-back")
        until_question(browser)
        browser.execute_script("document.querySelector('#welcome input[value=rarely]').click()")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#welcome input[name=writer]"), 5)
        seen(browser, "d58-2-writer-dark")
        browser.execute_script("document.querySelector('#welcome input[value=service]').click()")
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Connected mode" in b.execute_script(summary), 5)
        seen(browser, "d58-3-summary-connected-dark")

    # At the window's smallest size: the view 420 px at most, nothing wider than it.
    wide = lambda b: b.execute_script("const e = document.documentElement; return window.innerWidth <= 420 && e.scrollWidth <= e.clientWidth")

    # 5. From the applications' menu, not welcomed: the assistant all the same; left midway, its
    # answers are not kept — only that it was shown.
    window = started("menu")
    window.args = ["--window"]
    with window as browser:
        shown = until_question(browser)
        check("started with --window, not welcomed, no meeting: the full window opens on the assistant",
              shown and not browser.find_elements(By.ID, "small-record") and len(browser.window_handles) == 1,
              (browser.window_handles, browser.execute_script(welcome)))
        wait_for(browser, lambda b: "welcomed = true" in window.file.read_text(), 5)
        before = window.file.read_text()
        browser.set_window_size(420, 400)
        browser.execute_script("document.querySelector('#welcome input[value=rarely]').click()")
        press(browser, "welcome-next")
        reached = wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#welcome input[name=writer]"), 5)
        time.sleep(0.5)
        check("its second question at the window's smallest size: nothing overflows sideways", reached and wide(browser))
        browser.execute_script("document.querySelector('#welcome input[value=service]').click()")
        press(browser, "welcome-next")
        reached = wait_for(browser, lambda b: "Mode connecté" in b.execute_script(summary), 5)
        time.sleep(0.5)
        check("its last screen, connected, at the smallest size: nothing overflows sideways", reached and wide(browser))
    check("left midway, nothing of its answers is written", window.file.read_text() == before, window.file.read_text())

    # 6. The model here already: Terminer, nothing to fetch.
    window = started("here")
    models = WORK / "data-welcome-here" / "kitokey" / "models"
    models.mkdir(parents=True)
    (models / "ggml-large-v3-turbo-q8_0.bin").write_bytes(b"\0" * 16)
    with window as browser:
        until_question(browser)
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Mode privé" in b.execute_script(summary), 5)
        said = browser.execute_script(summary)
        # Read once the card has arrived (180 ms from transparent): `.text` is the text seen.
        wait_for(browser, lambda b: b.find_element(By.ID, "welcome-go").text != "", 2)
        done = browser.find_element(By.ID, "welcome-go").text
        check("the model here: Terminer, no fetch offered", done == "Terminer" and "Télécharger" not in said, (done, said))
        browser.set_window_size(420, 400)
        time.sleep(0.5)
        check("its last screen, private, at the smallest size: nothing overflows sideways", done == "Terminer" and wide(browser))
        browser.set_window_size(1100, 800)
        press(browser, "welcome-go")
        listed = wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 10)
        check("Terminer: saved private, the meetings opened, nothing fetched",
              listed and 'minutes_model = "qwen3:8b"' in window.file.read_text() and [m.name for m in models.iterdir()] == ["ggml-large-v3-turbo-q8_0.bin"],
              (window.file.read_text(), list(models.iterdir())))

    # 7. The model missing, Ollama answering without the recommended one: « Télécharger le modèle »
    # fetches it as Settings do, the window at the meetings meanwhile — refused, cut, then whole.
    right, ways, asked, posted = os.urandom(300_000), ["refused", "cut", "whole"], [], []

    class Model(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_GET(self):
            way = ways.pop(0)
            asked.append(way)
            sent = os.urandom(len(right)) if way == "refused" else right
            self.send_response(200)
            self.send_header("content-length", str(len(right)))
            self.end_headers()
            for at in range(0, len(right), 30_000):
                if way == "cut" and at == 150_000:
                    return
                self.wfile.write(sent[at:at + 30_000])
                self.wfile.flush()
                time.sleep(0.3)

        def log_message(self, *_):
            pass

    class Ollama(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            body = json.dumps({"models": [{"name": "mistral:7b"}]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self):
            posted.append(self.path)
            self.send_response(500)
            self.end_headers()

        def log_message(self, *_):
            pass

    servers = [http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) for handler in (Model, Ollama)]
    for server in servers:
        threading.Thread(target=server.serve_forever, daemon=True).start()
    host, ollama = f"127.0.0.1:{servers[0].server_address[1]}", f"http://127.0.0.1:{servers[1].server_address[1]}"
    fetched = WORK / "data-welcome-fetch" / "kitokey" / "models" / "ggml-large-v3-turbo-q8_0.bin"
    state = "return document.getElementById('model-state')?.innerText || ''"
    notified = WORK / "notified-welcome"
    monitor = subprocess.Popen(
        ["dbus-monitor", "--session", "interface='org.freedesktop.Notifications',member='Notify'"],
        stdout=open(notified, "w"), stderr=subprocess.DEVNULL,
    )

    def last_screen(browser) -> str:
        """From Settings, Refaire le réglage gone through to its last screen, private."""
        press(browser, "settings-link")
        wait_for(browser, lambda b: b.find_elements(By.ID, "welcome-again") and b.find_element(By.ID, "welcome-again").is_displayed(), 10)
        offered = wait_for(browser, lambda b: b.find_elements(By.ID, "model-fetch"), 5)
        press(browser, "welcome-again")
        until_question(browser)
        press(browser, "welcome-next")
        wait_for(browser, lambda b: "Mode privé" in b.execute_script(summary), 5)
        return offered

    try:
        window = started("fetch", ollama=ollama)
        window.env.update(KITOKEY_MODEL_PUBLISHED=f"http://{host}/model.bin {len(right)} {hashlib.sha256(right).hexdigest()}")
        with window as browser:
            until_question(browser)
            press(browser, "welcome-next")
            wait_for(browser, lambda b: "Mode privé" in b.execute_script(summary), 5)
            wait_for(browser, lambda b: b.find_element(By.ID, "welcome-go").text != "", 2)
            said, label = browser.execute_script(summary), browser.find_element(By.ID, "welcome-go").text
            check("Ollama answering without qwen3:8b: the minutes wait for it, Settings fetch it — no ollama.com",
                  "Ollama n’a pas encore qwen3:8b" in said and "ollama.com" not in said, said)
            check("the model missing: Télécharger, its size and host shown, nothing fetched before the click",
                  label == "Télécharger le modèle (300 ko)" and f"Depuis {host}, vérifié avant d’être utilisé." in said and asked == [], (label, said, asked))
            for round_, way in enumerate(("refused", "cut", "whole")):
                if round_:
                    offered = last_screen(browser)
                    check(f"before the {way} one: Settings offer it again, as after their own", offered)
                    wait_for(browser, lambda b: b.find_element(By.ID, "welcome-go").text != "", 2)
                # What is told after the click: each notice is seen twice on the bus, sent and relayed.
                mark = len(notified.read_text())
                press(browser, "welcome-go")
                away = wait_for(browser, lambda b: b.execute_script("return location.hash") in ("#/", "") and b.find_element(By.ID, "list").is_displayed() and asked == ["refused", "cut", "whole"][:round_ + 1], 5)
                check(f"{way}: the click fetches it, the window at the meetings while it comes",
                      away and not fetched.exists() and "Le modèle" not in notified.read_text()[mark:], (asked, notified.read_text()[mark:]))
                if way != "whole":
                    told = wait_for(browser, lambda b: "Le modèle n’a pas été téléchargé" in notified.read_text()[mark:], 15)
                    check(f"{way}: said as Settings say it, the model not put in its place", told and not fetched.exists(), notified.read_text()[mark:][-300:])
            text = window.file.read_text()
            check("saved private before it is fetched: the minutes by the Ollama answering, qwen3:8b",
                  f'minutes_provider = "{ollama}/v1"' in text and 'minutes_model = "qwen3:8b"' in text, text)
            press(browser, "settings-link")
            coming = wait_for(browser, lambda b: "En cours de téléchargement" in b.execute_script(state), 5)
            # Refaire le réglage answered while it comes: the model come, the page read again
            # meanwhile, the answer stays the user's.
            press(browser, "welcome-again")
            until_question(browser)
            browser.execute_script("document.querySelector('#welcome input[value=rarely]').click()")
            # Come; the page read again, as a model come or a fetch ended does: its drawing then
            # given a second to end. Settings left, nothing of them is there to ask for it.
            came = wait_for(browser, lambda b: fetched.exists(), 15)
            browser.execute_script("refresh()")
            time.sleep(1)
            check("the model come while the assistant is answered: the answer chosen stays, never drawn over",
                  came and browser.execute_script(picked) == "rarely", browser.execute_script(picked))
            press(browser, "welcome-skip")
            ready = wait_for(browser, lambda b: "Prêt" in b.execute_script(state), 15)
            check("whole: said coming in Settings, then ready there, checked against its fingerprint",
                  coming and ready and fetched.read_bytes() == right and wait_for(browser, lambda b: "Modèle prêt" in notified.read_text(), 5),
                  browser.execute_script(state))
            check("Ollama asked nothing but what it holds", posted == [], posted)

        # 8. Connected, no service chosen afterwards: the first meeting's minutes say none is chosen,
        # and open Settings.
        window = started("unchosen")
        with window as browser:
            until_question(browser)
            browser.execute_script("document.querySelector('#welcome input[value=rarely]').click()")
            press(browser, "welcome-next")
            wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#welcome input[name=writer]"), 5)
            browser.execute_script("document.querySelector('#welcome input[value=service]').click()")
            press(browser, "welcome-next")
            wait_for(browser, lambda b: "Mode connecté" in b.execute_script(summary), 5)
            press(browser, "welcome-go")
            wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 10)
            first = WORK / "welcome-unchosen" / "2026-10-28_09-00"
            session(first, now - dt.timedelta(hours=1), 3, {"local.wav": 1000})
            (first / "transcript.jsonl").write_text(
                '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
                '{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Bonjour."}\n{"type":"complete","duration":1.0}\n'
            )
            browser.execute_script(f"location.hash = '#/meeting/{first.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes"), 10)
            press(browser, "make-minutes")
            told = wait_for(browser, lambda b: "Aucun modèle choisi pour les comptes rendus" in notified.read_text(), 10)
            opened = wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 5)
            check("connected, no service chosen after: the first meeting's minutes say none is, and open Settings",
                  told and opened and not (first / "minutes.md").exists() and "minutes_provider" not in window.file.read_text(),
                  (notified.read_text()[-300:], window.file.read_text()))
    finally:
        monitor.terminate()
        monitor.wait(timeout=10)
        for server in servers:
            server.shutdown()


def meeting_language(now: dt.datetime) -> None:
    """AC. The language meetings are spoken in, and each meeting's own (D65)."""
    print("AC. the meetings' language in Settings, a meeting's own on its page, told to the transcription (D65)")
    meetings = WORK / "speaking"
    heard = meetings / "2026-10-27_14-00"
    session(heard, now - dt.timedelta(days=1), 0.1, {"local.wav": 1000})
    model = WORK / "speaking-model.bin"
    model.write_bytes(b"\0" * 16)
    window = Window(meetings, "fr_FR.UTF-8", f'reminder = false\nmodel = "{model}"\n')
    args = WORK / "hearing-args"
    told = lambda: args.read_text() if args.exists() else ""
    with window as browser:
        press(browser, "settings-link")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meetings-language"), 10)
        offered = [o.text for o in Select(browser.find_element(By.ID, "meetings-language")).options]
        label = browser.execute_script("return document.querySelector('label[for=meetings-language]')?.textContent")
        check("Settings offer the meetings' language, detected by default, French and English first, labelled",
              offered[:3] == ["Détectée par la transcription", "Français", "English"] and label == "Langue des réunions"
              and Select(browser.find_element(By.ID, "meetings-language")).first_selected_option.text == "Détectée par la transcription", (offered, label))
        choose(browser, "meetings-language", "en")
        saved = wait_for(browser, lambda b: 'meeting_language = "en"' in window.file.read_text(), 5)
        check("chosen, it is the settings' default", saved, window.file.read_text())
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meeting-language"), 10)
        own = Select(browser.find_element(By.ID, "meeting-language"))
        check("a meeting's page offers its own language, the settings' by default — the language they name said",
              own.first_selected_option.text == "Celle des Réglages — English" and not (heard / "language").exists(), own.first_selected_option.text)
        choose(browser, "meeting-language", "de")
        written = wait_for(browser, lambda b: (heard / "language").exists() and (heard / "language").read_text() == "de\n", 5)
        check("chosen on its page, the meeting's language file holds its code alone", written)
        args.unlink(missing_ok=True)
        press(browser, "make-transcript")
        made = wait_for(browser, lambda b: (heard / "transcript.jsonl").exists() and "--language" in told(), 15)
        check("its transcript is heard in the meeting's own language", made and "--language de" in told(), told())
        choose(browser, "meeting-language", "")
        removed = wait_for(browser, lambda b: not (heard / "language").exists(), 5)
        check("« Celle des Réglages »: the file removed", removed)
        args.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('transcript_redo', {name: arguments[0], model: arguments[1]})", heard.name, str(model))
        again = wait_for(browser, lambda b: "--language" in told(), 15)
        check("made again with none of its own: heard in the settings' language", again and "--language en" in told(), told())
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meetings-language"), 10)
        choose(browser, "meetings-language", "")
        wait_for(browser, lambda b: "meeting_language =" not in window.file.read_text(), 5)
        args.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('transcript_redo', {name: arguments[0], model: arguments[1]})", heard.name, str(model))
        detected = wait_for(browser, lambda b: told() != "", 15)
        check("« Détectée »: no language told, the engine detects it", detected and "--language" not in told() and "meeting_language =" not in window.file.read_text(), (told(), window.file.read_text()))
        refused = [browser.execute_script(
            "return window.__TAURI__.core.invoke('language_set', {name: arguments[0], language: arguments[1]}).then(() => 'kept', (why) => String(why))", heard.name, text)
            for text in ("Deutsch", "zz")]
        check("a text that is not a code of ISO 639-1 is refused, nothing written", "kept" not in refused and not (heard / "language").exists(), refused)
        browser.execute_script(f"location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 5)
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meeting-language"), 10)
        # By script, in one step: the page may be drawn again between finding the list and reading it.
        said = browser.execute_script("const p = document.getElementById('meeting-language'); return p.options[p.selectedIndex].text")
        check("the settings detecting it, the page says « détectée »", said == "Celle des Réglages — détectée", said)
    # SpeechAnalyzer hears (a program in its place, as X has it): it detects none, and hears French
    # and English alone — the system's when nothing names another (D64).
    program = WORK / "speaking-speech"
    program.write_text("#!/bin/sh\necho '{\"available\":true,\"installed\":true,\"supported\":true}'\n")
    program.chmod(0o755)
    window = Window(meetings, "fr_FR.UTF-8", f'reminder = false\nmodel = "{model}"\nengine = "speechanalyzer"\nmeeting_language = "de"\n')
    window.env.update(KITOKEY_SPEECH=str(program))
    with window as browser:
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meeting-language"), 10)
        offered = [o.text for o in Select(browser.find_element(By.ID, "meeting-language")).options]
        check("SpeechAnalyzer hearing: the page says the system's language, never « détectée », and offers French and English alone",
              offered == ["Celle des Réglages — Français", "Français", "English"], offered)
    with window as browser:
        press(browser, "settings-link")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meetings-language"), 10)
        chosen = browser.execute_script("const p = document.getElementById('meetings-language'); return p.selectedIndex < 0 ? '' : p.options[p.selectedIndex].text")
        check("Settings too: the meetings' language SpeechAnalyzer does not hear stays chosen, said as heard",
              chosen == "Deutsch — SpeechAnalyzer entend celle du système", chosen)
    # Its own language written while Whisper heard, one SpeechAnalyzer does not hear: said as heard.
    (heard / "language").write_text("de\n")
    with window as browser:
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "meeting-language"), 10)
        own = Select(browser.find_element(By.ID, "meeting-language")).first_selected_option.text
        check("a meeting's own language SpeechAnalyzer does not hear: kept, said with the one it hears", own == "Deutsch — SpeechAnalyzer entend Français", own)
    (heard / "language").unlink()


def app_language(now: dt.datetime) -> None:
    """AF. The language Kitokey speaks, chosen in Settings, the system's by default (D64)."""
    print("AF. the app's language chosen in Settings: spoken at once and after a restart, never the meetings' (D64)")
    meetings = WORK / "speaking-app"
    heard = meetings / "2026-10-28_09-00"
    session(heard, now - dt.timedelta(days=1), 0.1, {"local.wav": 1000})
    model = WORK / "speaking-app-model.bin"
    model.write_bytes(b"\0" * 16)
    # SpeechAnalyzer heard here as on a Mac with its model (as X has it): told a language, never detecting one.
    program, asked = WORK / "speaking-app-speech", WORK / "speaking-app-asked"
    program.write_text(f"""#!/bin/sh
echo "$@" >> '{asked}'
echo '{{"available":true,"installed":true,"supported":true}}'
""")
    program.chmod(0o755)
    window = Window(meetings, "fr_FR.UTF-8", f'reminder = false\nmodel = "{model}"\nengine = "speechanalyzer"\n')
    window.env.update(KITOKEY_SPEECH=str(program))
    args = WORK / "hearing-args"
    told = lambda: args.read_text() if args.exists() else ""
    french = ["Ouvrir Kitokey", "Démarrer un enregistrement", "Transcrire le dernier enregistrement",
              "Rédiger le compte rendu de la dernière réunion", "Quitter Kitokey"]
    english = ["Open Kitokey", "Start recording", "Transcribe the last recording", "Write the minutes of the last recording", "Quit Kitokey"]
    sections = lambda b: b.execute_script("return [document.getElementById('settings-link').getAttribute('aria-label'), document.documentElement.lang]")
    redo = lambda b: b.execute_script("window.__TAURI__.core.invoke('transcript_redo', {name: arguments[0], model: arguments[1]})", heard.name, str(model))

    def by_settings(b, ready: str) -> str:
        """The transcript made again by whoever the settings name to hear (D59): what it was told."""
        args.unlink(missing_ok=True)
        count = notified.read_text().count(ready)
        b.execute_script("window.__TAURI__.core.invoke('context_redo', {name: arguments[0]})", heard.name)
        wait_for(b, lambda b: told() != "" and notified.read_text().count(ready) > count, 15)
        return told()

    notified = WORK / "notified-language"
    monitor = subprocess.Popen(["dbus-monitor", "--session", "interface='org.freedesktop.Notifications',member='Notify'"],
                               stdout=open(notified, "w"), stderr=subprocess.DEVNULL)
    try:
        with window as browser:
            menu = wait_for(browser, lambda b: tray_labels() == french, 10)
            check("started in French, the system's: the window and the tray's menu speak it",
                  menu and sections(browser) == ["Réglages", "fr"], (tray_labels(), sections(browser)))
            press(browser, "settings-link")
            wait_for(browser, lambda b: b.find_elements(By.ID, "app-language"), 10)
            pick = Select(browser.find_element(By.ID, "app-language"))
            label = browser.execute_script("return document.querySelector('label[for=app-language] .name')?.textContent")
            check("Settings offer the language, the system's by default, each other in its own",
                  [o.text for o in pick.options] == ["Langue du système", "Français", "English"]
                  and pick.first_selected_option.get_attribute("value") == "" and label == "Langue de Kitokey",
                  ([o.text for o in pick.options], label))
            redo(browser)
            wait_for(browser, lambda b: "Transcription prête" in notified.read_text(), 15)
            before = by_settings(browser, "Transcription prête")
            spoke = len(asked.read_text())
            choose(browser, "app-language", "en")
            saved = wait_for(browser, lambda b: 'language = "en"' in window.file.read_text(), 5)
            named = lambda b: b.execute_script("return document.querySelector('label[for=app-language] .name')?.textContent")
            # The page drawn again once SpeechAnalyzer has said what it offers: a moment after the sidebar.
            spoken = wait_for(browser, lambda b: sections(b) == ["Settings", "en"] and named(b) == "Language of Kitokey", 5)
            label = named(browser)
            check("English chosen: written, and the window speaks it at once — the choice keeps the focus",
                  saved and spoken and label == "Language of Kitokey"
                  and browser.execute_script("return document.activeElement.id") == "app-language",
                  (window.file.read_text(), sections(browser), label))
            check("the tray's menu speaks it at once, with no restart", wait_for(browser, lambda b: tray_labels() == english, 5), tray_labels())
            mark = len(notified.read_text())
            after = by_settings(browser, "Transcript ready")
            check("a notification speaks it", "Transcript ready" in notified.read_text()[mark:] and "Transcription prête" not in notified.read_text()[mark:],
                  notified.read_text()[mark:][-300:])
            check("SpeechAnalyzer hears the meeting in the system's language, before and after: never the app's",
                  "--engine speechanalyzer --language fr" in before and after == before, (before, after))
            check("and is asked of the system's language alone", "status fr-FR" in asked.read_text()[spoke:] and "en-" not in asked.read_text(), asked.read_text())
            main = browser.current_window_handle
            browser.execute_script("document.getElementById('small-link').click()")
            wait_for(browser, lambda b: len(b.window_handles) == 2, 5)
            browser.switch_to.window([h for h in browser.window_handles if h != main][0])
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-record"), 5)
            small = [browser.find_element(By.ID, "small-record").get_attribute("aria-label"), browser.execute_script("return document.documentElement.lang")]
            check("the bar speaks it", small == ["Start a recording", "en"], small)
            try:
                browser.find_element(By.ID, "small-full").click()
            except NoSuchWindowException:
                pass  # the small window goes with the click
            browser.switch_to.window(main)
            wait_for(browser, lambda b: len(b.window_handles) == 1, 5)
            # The assistant, opened again from Settings (D58): its questions in the language chosen.
            browser.execute_script("location.hash = '#/welcome/again'")
            asking = "return document.getElementById('welcome-heading')?.textContent"
            asked_in = wait_for(browser, lambda b: b.execute_script(asking) == "Do your meetings discuss confidential matters?", 5)
            check("the assistant speaks it", asked_in, browser.execute_script(asking))
            browser.execute_script("location.hash = '#/'")
        # A service hears from now on, one on this computer: agreed, no key (D43).
        window.file.write_text(window.file.read_text().replace('engine = "speechanalyzer"', 'engine = "service"')
                               + 'transcription_provider = "http://127.0.0.1:9/v1"\ntranscription_model = "hears"\n')
        with window as browser:
            menu = wait_for(browser, lambda b: tray_labels() == english, 10)
            check("started again, the system still French: it speaks the one chosen", menu and sections(browser) == ["Settings", "en"],
                  (tray_labels(), sections(browser)))
            press(browser, "settings-link")
            wait_for(browser, lambda b: b.find_elements(By.ID, "app-language"), 10)
            kept = Select(browser.find_element(By.ID, "app-language")).first_selected_option.text
            check("Settings say the one chosen", kept == "English", kept)
            before = by_settings(browser, "Transcript ready")
            choose(browser, "app-language", "")
            back = wait_for(browser, lambda b: "language" not in window.file.read_text() and sections(b) == ["Réglages", "fr"], 5)
            menu = wait_for(browser, lambda b: tray_labels() == french, 5)
            check("the system's again: the key gone, French at once in the window and the menu", back and menu,
                  (window.file.read_text(), sections(browser), tray_labels()))
            after = by_settings(browser, "Transcription prête")
            check("a service is told the same before and after: no language, the meetings' naming none",
                  "--provider http://127.0.0.1:9/v1" in before and after == before and "--language" not in after, (before, after))
            refused = browser.execute_script(
                "return window.__TAURI__.core.invoke('settings_saved', {change: {language: 'de'}}).then(() => 'kept', (why) => String(why))")
            check("a language it does not speak is refused, nothing written", refused != "kept" and "language" not in window.file.read_text(), refused)
    finally:
        monitor.terminate()
        monitor.wait(timeout=10)


def confidential_every_way(now: dt.datetime) -> None:
    """AI. A meeting said confidential (D60), its transcript made again every way the criterion names
    — from the tray, by Réessayer, by « Refaire avec ce contexte » — and its minutes from the tray:
    Whisper and Ollama here, nothing asked of the services the settings name."""
    print("AI. a confidential meeting made again from the tray, by Réessayer, by Refaire avec ce contexte: here alone (D60)")
    counted = {"hear": 0, "write": 0}

    class Service(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_POST(self):
            counted[self.path.strip("/")] += 1
            self.rfile.read(int(self.headers.get("content-length") or 0))
            self.send_response(200)
            self.send_header("content-length", "0")
            self.end_headers()

        def log_message(self, *_):
            pass

    class Ollama(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_GET(self):
            body = json.dumps({"models": [{"name": "qwen3:8b"}]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    servers = [http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) for handler in (Service, Ollama)]
    for server in servers:
        threading.Thread(target=server.serve_forever, daemon=True).start()
    standin, base = (f"http://127.0.0.1:{server.server_port}" for server in servers)
    model = WORK / "every-way-model.bin"
    model.write_bytes(b"\0" * 16)
    named = (f'reminder = false\nmodel = "{model}"\n'
             'engine = "service"\ntranscription_provider = "https://hears.example/v1"\ntranscription_model = "whisper-1"\n'
             'transcription_agreed = "hears.example"\nminutes_provider = "https://writes.example/v1"\nminutes_model = "gpt-4.1-mini"\n'
             'minutes_agreed = "writes.example"\n')
    meetings = WORK / "every-way"
    meeting = meetings / "2026-10-28_15-00"
    session(meeting, now - dt.timedelta(hours=3), 1, {"local.wav": 1000})
    (meeting / "confidential").write_bytes(b"")
    (meeting / "context.txt").write_text("Budget.")
    hearing, writing = WORK / "hearing-args", WORK / "summary-args"
    heard_here = lambda: hearing.exists() and "--provider" not in hearing.read_text()

    def header() -> dict:
        try:
            return json.loads((meeting / "transcript.jsonl").read_text().splitlines()[0])
        except (FileNotFoundError, IndexError, ValueError):
            return {}

    window = Window(meetings, "fr_FR.UTF-8", named, ollama=base)
    window.env.update(STANDIN=standin)
    try:
        with window as browser:
            wait_for(browser, lambda b: tray_labels() != [], 10)
            # From the tray: the last recording's transcript, the engine named a service.
            hearing.unlink(missing_ok=True)
            clicked = tray_click("Transcrire le dernier enregistrement")
            made = wait_for(browser, lambda b: hearing.exists() and header().get("engine") == "whisper.cpp", 15)
            check("from the tray: Whisper hears it here, the service named in the settings asked nothing",
                  clicked and made and heard_here() and counted["hear"] == 0, (hearing.read_text() if hearing.exists() else "", counted))
            # By Réessayer: a making that failed, asked again — once the tray's is over: one being
            # made is refused, as the page's button would be.
            wait_for(browser, lambda b: b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.transcript === null && m.transcripts_waiting.length === 0)"), 15)
            (WORK / "fail-hearing").touch()
            hearing.unlink(missing_ok=True)
            browser.execute_script("window.__TAURI__.core.invoke('context_redo', {name: arguments[0]})", meeting.name)
            failed = wait_for(browser, lambda b: hearing.exists() and b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.transcript === null && m.transcripts_waiting.length === 0)"), 15)
            (WORK / "fail-hearing").unlink()
            hearing.unlink(missing_ok=True)
            browser.execute_script("window.__TAURI__.core.invoke('transcript_again', {name: arguments[0]})", meeting.name)
            again = wait_for(browser, lambda b: hearing.exists(), 15)
            check("by Réessayer: here again, nothing asked of the service", failed and again and heard_here() and counted["hear"] == 0,
                  (hearing.read_text() if hearing.exists() else "", counted))
            # From the tray: the last recording's minutes.
            wait_for(browser, lambda b: b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.minutes === null && m.minutes_waiting.length === 0)"), 15)
            writing.unlink(missing_ok=True)
            clicked = tray_click("Rédiger le compte rendu de la dernière réunion")
            written = wait_for(browser, lambda b: writing.exists(), 15)
            check("its minutes from the tray: Ollama here, no agreement asked, nothing sent",
                  clicked and written and f"--provider {base}/v1" in writing.read_text() and counted == {"hear": 0, "write": 0}
                  and not browser.execute_script("return location.hash").startswith("#/agree"), (writing.read_text() if writing.exists() else "", counted))
            # By « Refaire avec ce contexte »: the transcript and the minutes, here.
            wait_for(browser, lambda b: b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.transcript === null && m.minutes === null && m.minutes_waiting.length === 0)"), 15)
            hearing.unlink(missing_ok=True)
            writing.unlink(missing_ok=True)
            browser.execute_script("window.__TAURI__.core.invoke('context_redo', {name: arguments[0]})", meeting.name)
            redone = wait_for(browser, lambda b: hearing.exists() and writing.exists(), 20)
            check("by Refaire avec ce contexte: Whisper and Ollama here, nothing asked of either service",
                  redone and heard_here() and f"--provider {base}/v1" in writing.read_text() and counted == {"hear": 0, "write": 0},
                  (hearing.read_text() if hearing.exists() else "", writing.read_text() if writing.exists() else "", counted))
    finally:
        for server in servers:
            server.shutdown()


def own_models(now: dt.datetime) -> None:
    """BC. A meeting's own models: chosen for it alone, used when it is made and made again (D91)."""
    print("BC. a meeting's own models: written whole, used to make and make again, Settings unchanged, a model gone said (D91)")
    meetings = WORK / "own-models"
    meeting = meetings / "2026-11-11_09-00"
    session(meeting, now - dt.timedelta(hours=2), 1, {"local.wav": 1000})
    data = WORK / "data-own-models"
    folder = data / "kitokey" / "models"
    folder.mkdir(parents=True)
    own = folder / "ggml-tiny-own.bin"
    own.write_bytes(b"\0" * 16)
    model = WORK / "own-models-settings.bin"
    model.write_bytes(b"\0" * 16)
    local = "http://localhost:11434/v1"
    named = f'reminder = false\nmodel = "{model}"\nminutes_provider = "{local}"\nminutes_model = "settings-writer"\n'
    hearing, writing = WORK / "hearing-args", WORK / "summary-args"
    read = lambda path: path.read_text() if path.exists() else ""
    idle = "return window.__TAURI__.core.invoke('makings').then(m => m.transcript === null && m.minutes === null && m.transcripts_waiting.length === 0 && m.minutes_waiting.length === 0)"
    shown = "return window.__TAURI__.core.invoke('meeting_models_shown', {name: arguments[0]})"
    window = Window(meetings, "fr_FR.UTF-8", named)
    window.env.update(XDG_DATA_HOME=str(data))
    with window as browser:
        settings_before = window.file.read_text()
        chosen = {"transcription": {"engine": "whisper", "model": own.name}, "minutes": {"provider": local, "model": "mistral-own"}}
        browser.execute_script("return window.__TAURI__.core.invoke('meeting_models_chosen', {name: arguments[0], chosen: arguments[1]})", meeting.name, chosen)
        written = wait_for(browser, lambda b: (meeting / "models.json").exists(), 5)
        check("chosen for the meeting: models.json written whole, as the contract shows it, nothing beside it",
              written and json.loads((meeting / "models.json").read_text()) == chosen
              and sorted(p.name for p in meeting.iterdir()) == ["local.wav", "models.json", "session.json"], sorted(p.name for p in meeting.iterdir()))
        hearing.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('transcript_asked', {name: arguments[0]})", meeting.name)
        heard = wait_for(browser, lambda b: str(own) in read(hearing) and (meeting / "transcript.jsonl").exists(), 15)
        check("made: heard by the meeting's own Whisper model, not Settings'", heard and str(model) not in read(hearing), read(hearing))
        wait_for(browser, lambda b: b.execute_script(idle), 15)
        writing.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('minutes_asked', {name: arguments[0]})", meeting.name)
        wrote = wait_for(browser, lambda b: "mistral-own" in read(writing) and (meeting / "minutes.md").exists(), 15)
        check("its minutes written by the meeting's own model", wrote and "settings-writer" not in read(writing), read(writing))
        wait_for(browser, lambda b: b.execute_script(idle), 15)
        hearing.unlink(missing_ok=True)
        writing.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('context_redo', {name: arguments[0]})", meeting.name)
        again = wait_for(browser, lambda b: str(own) in read(hearing) and "mistral-own" in read(writing), 20)
        check("made again (Refaire avec ce contexte): the meeting's own models both", again, (read(hearing), read(writing)))
        wait_for(browser, lambda b: b.execute_script(idle), 15)
        check("Settings unchanged by the meeting's choice", window.file.read_text() == settings_before, window.file.read_text())
        said = browser.execute_script(shown, meeting.name)
        check("its choice shown, none gone", said == {"chosen": chosen, "gone": []}, said)
        # Its page says which models made its transcript and its minutes (criterion 3).
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        makers = lambda b: b.execute_script("return [...document.querySelectorAll('.maker')].map(m => m.textContent)")
        both = wait_for(browser, lambda b: len(makers(b)) >= 2, 10)
        texts = " | ".join(makers(browser)).lower()
        check("its page says the models that made its transcript and its minutes", both and "whisper tiny own" in texts and "mistral own" in texts, texts)
        # A Refaire… that picks Settings' own model — outside Kitokey's folder of models — is still offered
        # it, whatever the meeting chose: the models listed are Settings', never the meeting's overlay.
        hearing.unlink(missing_ok=True)
        picked = browser.execute_script(
            "return window.__TAURI__.core.invoke('transcript_redo', {name: arguments[0], model: arguments[1]}).then(() => 'made', (why) => String(why))",
            meeting.name, str(model))
        redone = wait_for(browser, lambda b: str(model) in read(hearing), 15)
        check("Refaire… with Settings' own model, beside the meeting's choice: made with it", picked == "made" and redone, (picked, read(hearing)))
        wait_for(browser, lambda b: b.execute_script(idle), 20)
        # A choice the contract refuses is refused, the file left as it was.
        for wrong in ({"transcription": {"engine": "whisper", "model": "../../etc/passwd"}}, {"minutes": {"provider": local, "model": ""}}):
            refused = browser.execute_script(
                "return window.__TAURI__.core.invoke('meeting_models_chosen', {name: arguments[0], chosen: arguments[1]}).then(() => 'kept', (why) => String(why))",
                meeting.name, wrong)
            check(f"a choice the contract refuses is refused, nothing written: {wrong}",
                  refused != "kept" and json.loads((meeting / "models.json").read_text()) == chosen, refused)
        # Another host than Settings' to write: Settings' writes it, and it is said gone.
        other = {**chosen, "minutes": {"provider": "https://other.example/v1", "model": "m"}}
        browser.execute_script("return window.__TAURI__.core.invoke('meeting_models_chosen', {name: arguments[0], chosen: arguments[1]})", meeting.name, other)
        said = browser.execute_script(shown, meeting.name)
        check("a host to write that Settings do not name: said gone", said["gone"] == ["minutes"], said)
        writing.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('minutes_asked', {name: arguments[0]})", meeting.name)
        wrote = wait_for(browser, lambda b: "settings-writer" in read(writing), 15)
        check("its minutes then written by Settings' model, no agreement asked for that host",
              wrote and "other.example" not in read(writing) and not browser.execute_script("return location.hash").startswith("#/agree"), read(writing))
        wait_for(browser, lambda b: b.execute_script(idle), 15)
        browser.execute_script("return window.__TAURI__.core.invoke('meeting_models_chosen', {name: arguments[0], chosen: arguments[1]})", meeting.name, chosen)
        # The model removed: Settings' choice stands in, and it is said.
        own.unlink()
        said = browser.execute_script(shown, meeting.name)
        check("its model removed: said gone", said["gone"] == ["transcription"], said)
        hearing.unlink(missing_ok=True)
        browser.execute_script("window.__TAURI__.core.invoke('context_redo', {name: arguments[0]})", meeting.name)
        stood = wait_for(browser, lambda b: str(model) in read(hearing), 15)
        check("made again with its model gone: Settings' model stands in", stood, read(hearing))
        wait_for(browser, lambda b: b.execute_script(idle), 20)
        # None left: the file removed.
        browser.execute_script("return window.__TAURI__.core.invoke('meeting_models_chosen', {name: arguments[0], chosen: {}})", meeting.name)
        removed = wait_for(browser, lambda b: not (meeting / "models.json").exists(), 5)
        check("none chosen: models.json removed", removed)


LONG = "hf.co/unsloth/Mistral-Small-3.2-24B-Instruct-2506-GGUF:UD-Q4_K_XL-very-long-name"
LONG_SAID = "Mistral Small 3.2 24B Instruct 2506 GGUF UD Q4 K XL Very Long Name"
# A list's chosen entry: whether it fits the list as drawn, and what is said whole under it, if anything —
# inside the window's width.
def long_resized(browser, where: str, chosen: str) -> None:
    """A long name chosen in Refaire… at 830 px, where it fits; then the window narrowed to 420 px and
    widened back, never crossing 840 (the page's own break): said whole under its list once the list is
    too narrow for it, at 12 px, and no longer once it fits again — whatever the window does after."""
    pick = "redo-panel-minutes-model"
    Select(browser.find_element(By.ID, pick)).select_by_visible_text(chosen)
    time.sleep(0.5)
    got = browser.execute_script(LONG_SHOWN, pick)
    check(f"{where}: chosen at 830 px, the long name fits its list, nothing said under it", got["fit"] and got["said"] is None, got)
    browser.set_window_size(420, 700)
    narrowed = wait_for(browser, lambda b: b.execute_script(LONG_SHOWN, pick)["said"] is not None, 5)
    got = browser.execute_script(LONG_SHOWN, pick)
    check(f"{where}: the window narrowed to 420 px after the choice — the name said whole under its list, 12 px, inside the window",
          narrowed and not got["fit"] and got["said"] == got["text"] and got["size"] == "12px" and got["inside"], got)
    browser.set_window_size(830, 800)
    widened = wait_for(browser, lambda b: b.execute_script(LONG_SHOWN, pick)["said"] is None, 5)
    got = browser.execute_script(LONG_SHOWN, pick)
    check(f"{where}: widened back to 830 px — it fits again, nothing said under it", widened and got["fit"], got)


LONG_SHOWN = ("const s = document.getElementById(arguments[0]); const f = getComputedStyle(s); const c = document.createElement('canvas').getContext('2d');"
              "c.font = `${f.fontSize} ${f.fontFamily}`; const text = s.options[s.selectedIndex].textContent;"
              "const u = document.getElementById(arguments[0] + '-whole'); const shown = u && u.getClientRects().length > 0;"
              "return {text, fit: Math.ceil(c.measureText(text).width) + 36 <= s.clientWidth, said: shown ? u.textContent : null, size: u ? getComputedStyle(u).fontSize : null,"
              " inside: !shown || (u.getBoundingClientRect().right <= innerWidth && u.scrollWidth <= u.clientWidth + 1)}")


def models_advanced(now: dt.datetime) -> None:
    """BD. « Avancé » (D91, board 6): closed by default in the bar opened and in Refaire…, the meeting's
    own models chosen there — Settings' first —, the next recording's at rest, kept with the meeting,
    Settings unchanged; a model gone said on the page; a confidential meeting offered no service."""
    print("BD. « Avancé » in the bar and in Refaire…: closed, Settings' first, the meeting's own models kept, Settings unchanged, a model gone said, no service when confidential (D91)")

    class Tags(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            # A name far longer than the bar is wide, as the explorer met one (D91's UX pass).
            body = json.dumps({"models": [{"name": "qwen3:14b"}, {"name": "mistral:7b"}, {"name": LONG}]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    tags = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Tags)
    threading.Thread(target=tags.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{tags.server_address[1]}"
    here = f"{base}/v1"
    data = WORK / "data-advanced"
    folder = data / "kitokey" / "models"
    folder.mkdir(parents=True)
    tiny = folder / "ggml-tiny-adv.bin"
    tiny.write_bytes(b"\0" * 16)
    model = WORK / "ggml-adv-settings.bin"
    model.write_bytes(b"\0" * 16)
    named = (f'after = "transcript"\nreminder = false\nmodel = "{model}"\nminutes_provider = "{here}"\nminutes_model = "qwen3:14b"\n'
             'transcription_provider = "https://hears.example/v1"\ntranscription_model = "whisper-1"\n')
    meetings = WORK / "advanced"
    meetings.mkdir()
    meeting = meetings / "2026-10-18_12-00"
    hearing, writing = WORK / "hearing-args", WORK / "summary-args"
    read = lambda path: path.read_text() if path.exists() else ""
    own = lambda: json.loads((meeting / "models.json").read_text()) if (meeting / "models.json").exists() else None
    shown = ("const t = document.getElementById(arguments[0] + '-advanced'); const p = document.getElementById(arguments[0] + '-advanced-panel');"
             "const o = (id) => { const s = document.getElementById(arguments[0] + id); return s ? [...s.options].map(x => x.textContent) : null; };"
             "const v = (id) => { const s = document.getElementById(arguments[0] + id); return s ? s.options[s.selectedIndex]?.textContent : null; };"
             "return {expanded: t?.getAttribute('aria-expanded'), hidden: p?.hidden, hear: o('-transcription-model'), write: o('-minutes-model'),"
             " heard: v('-transcription-model'), written: v('-minutes-model'), note: p?.querySelector('.says')?.textContent, said: p?.querySelector('.note')?.textContent}")
    idle = "return window.__TAURI__.core.invoke('makings').then(m => m.transcript === null && m.minutes === null && m.transcripts_waiting.length === 0 && m.minutes_waiting.length === 0)"
    window = Window(meetings, "fr_FR.UTF-8", named, ollama=base, small=True)
    window.env.update(XDG_DATA_HOME=str(data))
    try:
        with window as browser:
            before = window.file.read_text()
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 10)
            press(browser, "small-more")
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-advanced") and b.find_element(By.ID, "small-advanced").is_displayed(), 5)
            got = browser.execute_script(shown, "small")
            check("the bar opened: « Avancé » closed by default", got["expanded"] == "false" and got["hidden"] is True, got)
            press(browser, "small-advanced")
            opened = wait_for(browser, lambda b: b.execute_script(shown, "small")["hidden"] is False, 5)
            got = browser.execute_script(shown, "small")
            check("opened at rest: the transcription's models — Settings' first, short, then those installed and the service Settings set — and the minutes', Settings' never twice",
                  opened and got["hear"] == ["Réglages\u00a0: Whisper adv settings", "Whisper tiny adv", "whisper-1, via hears.example"]
                  and got["write"] == ["Réglages\u00a0: Qwen3 14b", LONG_SAID + " — sur cet ordinateur", "Mistral 7b — sur cet ordinateur"]
                  and got["note"] == "Pour cette réunion seule ; les Réglages ne changent pas.", got)
            # What « Avancé » holds said on it; the note before the lists; the lists at the board's 14 px, none cut.
            looks = browser.execute_script(
                "const p = document.getElementById('small-advanced-panel'); const c = document.createElement('canvas').getContext('2d');"
                "const fit = (s) => { const f = getComputedStyle(s); c.font = `${f.fontSize} ${f.fontFamily}`;"
                " return Math.ceil(c.measureText(s.options[s.selectedIndex].textContent).width) + 36 <= s.clientWidth; };"
                "const lists = [...p.querySelectorAll('select')];"
                "return {label: document.getElementById('small-advanced').textContent, first: p.firstElementChild.className,"
                " sizes: lists.map(s => getComputedStyle(s).fontSize), fit: lists.map(fit)}")
            check("« Avancé » says it holds the models; « pour cette réunion seule » above the lists; the lists at 14 px, their choice whole",
                  looks == {"label": "Avancé — modèles", "first": "says", "sizes": ["14px", "14px"], "fit": [True, True]}, looks)
            Select(browser.find_element(By.ID, "small-minutes-model")).select_by_visible_text(LONG_SAID + " — sur cet ordinateur")
            whole = wait_for(browser, lambda b: b.execute_script(LONG_SHOWN, "small-minutes-model")["said"] is not None, 3)
            got = browser.execute_script(LONG_SHOWN, "small-minutes-model")
            check("fr, the bar: a model's name longer than the list is wide is said whole under it, at 12 px, never only cut",
                  whole and not got["fit"] and got["said"] == got["text"] and got["size"] == "12px" and got["inside"], got)
            # « Avancé » closed and opened again — drawn anew with the choice kept: said whole again.
            press(browser, "small-advanced")
            wait_for(browser, lambda b: b.execute_script(shown, "small")["hidden"] is True, 3)
            press(browser, "small-advanced")
            again = wait_for(browser, lambda b: b.execute_script(shown, "small")["hidden"] is False
                             and b.execute_script(LONG_SHOWN, "small-minutes-model")["said"] == LONG_SAID + " — sur cet ordinateur", 5)
            check("fr, the bar: « Avancé » closed and opened again, the long name chosen still said whole under its list",
                  again, browser.execute_script(LONG_SHOWN, "small-minutes-model"))
            Select(browser.find_element(By.ID, "small-transcription-model")).select_by_visible_text("Whisper tiny adv")
            Select(browser.find_element(By.ID, "small-minutes-model")).select_by_visible_text("Mistral 7b — sur cet ordinateur")
            kept = wait_for(browser, lambda b: b.execute_script("return window.__TAURI__.core.invoke('models_offered', {name: null}).then(o => o.chosen)")
                            == {"transcription": {"engine": "whisper", "model": tiny.name}, "minutes": {"provider": here, "model": "mistral:7b"}}, 5)
            check("chosen at rest: kept for the next recording, nothing written, Settings unchanged",
                  kept and not any(meetings.iterdir()) and window.file.read_text() == before, sorted(p.name for p in meetings.iterdir()))
            line = "const k = document.getElementById('small-models-kept'); return k && k.getClientRects().length > 0 ? k.textContent : null"
            described = "return document.getElementById('small-advanced').getAttribute('aria-describedby')"
            check("« Avancé » open: nothing said under the row of the models kept, nor described on it",
                  browser.execute_script(line) is None and browser.execute_script(described) is None, [browser.execute_script(line), browser.execute_script(described)])
            next_models = window.file.parent / "next-models.json"
            check("chosen at rest: kept on disk beside the settings", next_models.exists(), sorted(p.name for p in window.file.parent.iterdir()))
            press(browser, "small-advanced")
            keptSaid = "Modèles gardés : tiny adv · Mistral 7b"
            said = wait_for(browser, lambda b: b.execute_script(line) == keptSaid, 3)
            check("« Avancé » closed with a choice for the next recording: said under the row, naming them, as the context kept is", said, browser.execute_script(line))
            told = browser.execute_script("const k = document.getElementById('small-models-kept'); const c = document.getElementById('small-context-kept');"
                                          "return {size: getComputedStyle(k).fontSize, context: getComputedStyle(c).fontSize,"
                                          " described: document.getElementById('small-advanced').getAttribute('aria-describedby')}")
            check("the models kept: 12 px as the context's line, and given to a screen reader with « Avancé »",
                  told == {"size": "12px", "context": "12px", "described": "small-models-kept"}, told)
            # Kitokey quit and started again: the models chosen at rest are still the next recording's, and said.
            window.__exit__()
            browser = window.__enter__()
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 10)
            again = wait_for(browser, lambda b: b.execute_script(line) == keptSaid, 5)
            chosen_again = browser.execute_script("return window.__TAURI__.core.invoke('models_offered', {name: null}).then(o => o.chosen)")
            check("started again: the models chosen at rest still kept for the next recording, and said",
                  again and chosen_again == {"transcription": {"engine": "whisper", "model": tiny.name}, "minutes": {"provider": here, "model": "mistral:7b"}},
                  [browser.execute_script(line), chosen_again])
            press(browser, "small-record")
            started = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
            taken = wait_for(None, lambda b: own() == {"transcription": {"engine": "whisper", "model": tiny.name}, "minutes": {"provider": here, "model": "mistral:7b"}}, 5)
            check("started: the choice made at rest is the new meeting's models.json", started and taken, own())
            left = browser.execute_script("return window.__TAURI__.core.invoke('models_offered', {name: null}).then(o => o.chosen)")
            check("taken by this meeting: none kept for the next, nothing left on disk", left == {} and not next_models.exists(), [left, next_models.exists()])
            check("recording: « Avancé » described by nothing kept", browser.execute_script(described) is None, browser.execute_script(described))
            # Drawn again for this meeting, closed (board 6): the options opened, then « Avancé ».
            wait_for(browser, lambda b: b.execute_script(shown, "small")["expanded"] == "false", 5)
            time.sleep(0.5)
            if not browser.execute_script("return document.getElementById('small-options').hidden === false"):
                browser.execute_script("document.getElementById('small-more').click()")
            browser.execute_script("document.getElementById('small-advanced').click()")
            recording = wait_for(browser, lambda b: b.execute_script(shown, "small")["heard"] == "Whisper tiny adv", 5)
            got = browser.execute_script(shown, "small")
            check("recording: « Avancé » shows this meeting's own models", recording and got["written"] == "Mistral 7b — sur cet ordinateur", got)
            Select(browser.find_element(By.ID, "small-minutes-model")).select_by_index(0)
            back = wait_for(None, lambda b: own() == {"transcription": {"engine": "whisper", "model": tiny.name}}, 5)
            check("recording: Settings' chosen again for the minutes, that part of models.json gone", back, own())
            late = browser.execute_async_script(
                "const done = arguments[arguments.length - 1]; window.__TAURI__.core.invoke('next_models_kept', {chosen: arguments[0]}).then(() => done('kept'), (why) => done('refused'))",
                {"transcription": {"engine": "speechanalyzer"}})
            nothing = browser.execute_script("return window.__TAURI__.core.invoke('models_offered', {name: null}).then(o => o.chosen)")
            check("recording: no models are kept for the next — that one's own are its meeting's", late == "refused" and nothing == {}, [late, nothing])
            offered = browser.execute_script(shown, "small")
            check("recording: the service Settings set is offered while the meeting is not confidential",
                  "whisper-1, via hears.example" in (offered["hear"] or []), offered)
            browser.execute_script("document.getElementById('small-confidential').click()")
            redrawn = wait_for(browser, lambda b: (b.execute_script(shown, "small")["hear"] or ["via"]) and not any("via" in one for one in b.execute_script(shown, "small")["hear"])
                               and b.execute_script(shown, "small")["expanded"] == "true", 5)
            check("said confidential while « Avancé » is open: drawn again at once, open, no service offered", redrawn, browser.execute_script(shown, "small"))
            browser.execute_script("document.getElementById('small-confidential').click()")
            wait_for(browser, lambda b: not (meeting / "confidential").exists() and "whisper-1, via hears.example" in (b.execute_script(shown, "small")["hear"] or []), 5)
            # Said confidential by the window's switch while a list of « Avancé » has the focus: drawn again,
            # the focus kept on that list (showSmall's rule).
            browser.execute_script("document.getElementById('small-transcription-model').focus()")
            browser.execute_script("window.__TAURI__.core.invoke('confidential_set', {name: arguments[0], on: true})", meeting.name)
            redrawn = wait_for(browser, lambda b: not any("via" in one for one in (b.execute_script(shown, "small")["hear"] or ["via"])), 5)
            time.sleep(0.3)
            kept_focus = browser.execute_script("return document.activeElement.id")
            check("said confidential from the window while a list has the focus: « Avancé » drawn again, the focus kept on that list",
                  redrawn and kept_focus == "small-transcription-model", kept_focus)
            browser.execute_script("window.__TAURI__.core.invoke('confidential_set', {name: arguments[0], on: false})", meeting.name)
            wait_for(browser, lambda b: "whisper-1, via hears.example" in (b.execute_script(shown, "small")["hear"] or []), 5)
            hearing.unlink(missing_ok=True)
            time.sleep(0.6)
            stop_from_small(browser)
            heard = wait_for(None, lambda b: str(tiny) in read(hearing) and (meeting / "transcript.jsonl").exists(), 15)
            check("stopped: its transcript heard by the model chosen in the bar", heard and str(model) not in read(hearing), read(hearing))
            check("Settings unchanged by any of it", window.file.read_text() == before, window.file.read_text())
            # Stopped by ■: nothing kept for the next, nothing said kept — as the shell has it.
            wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10)
            # Stopped with « Avancé » open on that meeting's models: drawn again for the next recording,
            # closed, Settings' first — never left open on the meeting that ended (#766, P5).
            rest = wait_for(browser, lambda b: b.execute_script(shown, "small")["heard"] == "Réglages : Whisper adv settings", 3)
            got = browser.execute_script(shown, "small")
            check("stopped by ■ with « Avancé » open: drawn again for the next recording, closed, Settings' first",
                  rest and got["expanded"] == "false" and got["written"] == "Réglages : Qwen3 14b", got)
            none_kept = lambda b: b.execute_script("return window.__TAURI__.core.invoke('models_offered', {name: null}).then(o => o.chosen)")
            quiet = wait_for(browser, lambda b: b.execute_script(line) is None, 3)
            check("stopped by ■: no models kept for the next, and none said kept", quiet and none_kept(browser) == {}, [browser.execute_script(line), none_kept(browser)])
            # Chosen again at rest, then started and stopped from the tray's icon: the same.
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            if browser.execute_script("return document.getElementById('small-options').hidden"):
                browser.execute_script("document.getElementById('small-more').click()")
            if browser.execute_script(shown, "small")["expanded"] != "true":
                browser.execute_script("document.getElementById('small-advanced').click()")
            wait_for(browser, lambda b: b.execute_script(shown, "small")["hidden"] is False, 5)
            time.sleep(0.3)
            Select(browser.find_element(By.ID, "small-minutes-model")).select_by_visible_text("Mistral 7b — sur cet ordinateur")
            wait_for(browser, lambda b: none_kept(b) != {}, 5)
            browser.execute_script("document.getElementById('small-advanced').click()")
            said = wait_for(browser, lambda b: b.execute_script(line) == "Modèle gardé\u00a0: Mistral 7b", 3)
            check("chosen again at rest, « Avancé » closed: said kept, naming the one", said, browser.execute_script(line))
            made = meetings / "2026-10-18_11-30"
            meeting.rename(made)
            try:
                tray_click("Démarrer")
                wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
                time.sleep(0.6)
                # « Avancé » opened on this meeting's models, then stopped from the tray: the bar's own
                # stop is not the only one that draws it again for the next (#766, P5).
                wait_for(browser, lambda b: b.execute_script(shown, "small")["expanded"] == "false", 5)
                time.sleep(0.5)
                if browser.execute_script("return document.getElementById('small-options').hidden"):
                    browser.execute_script("document.getElementById('small-more').click()")
                browser.execute_script("document.getElementById('small-advanced').click()")
                ours = wait_for(browser, lambda b: b.execute_script(shown, "small")["written"] == "Mistral 7b — sur cet ordinateur"
                                and b.execute_script(shown, "small")["expanded"] == "true", 5)
                check("recording from the tray: « Avancé » open on this meeting's models", ours, browser.execute_script(shown, "small"))
                tray_click("Arrêter l’enregistrement")
                wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10)
                rest = wait_for(browser, lambda b: b.execute_script(shown, "small")["written"] == "Réglages\u00a0: Qwen3 14b", 3)
                got = browser.execute_script(shown, "small")
                check("stopped from the tray with « Avancé » open: drawn again for the next recording, closed, Settings' first",
                      rest and got["expanded"] == "false" and got["heard"] == "Réglages\u00a0: Whisper adv settings", got)
                quiet = wait_for(browser, lambda b: b.execute_script(line) is None, 3)
                check("started and stopped from the tray's icon: no models kept for the next, and none said kept",
                      quiet and none_kept(browser) == {}, [browser.execute_script(line), none_kept(browser)])
                # Quit and started again after a recording took them: nothing comes back.
                wait_for(browser, lambda b: b.execute_script(idle), 15)
                window.__exit__()
                browser = window.__enter__()
                wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 10)
                time.sleep(1)
                check("started again after a recording took them: no models kept, none said, nothing on disk",
                      none_kept(browser) == {} and browser.execute_script(line) is None and not next_models.exists(),
                      [none_kept(browser), browser.execute_script(line), next_models.exists()])
                got = browser.execute_script(shown, "small")
                check("started again: « Avancé » closed, Settings' first in both lists",
                      got["expanded"] == "false" and got["heard"] == "Réglages\u00a0: Whisper adv settings" and got["written"] == "Réglages\u00a0: Qwen3 14b", got)
                wait_for(None, lambda b: (meeting / "transcript.jsonl").exists(), 15)
                wait_for(browser, lambda b: b.execute_script(idle), 15)
            finally:
                shutil.rmtree(meeting, ignore_errors=True)
                made.rename(meeting)
        # Refaire…: its « Avancé » closed, the meeting's own models in it; chosen there, kept once Refaire is pressed.
        full = Window(meetings, "fr_FR.UTF-8", named, ollama=base)
        full.env.update(XDG_DATA_HOME=str(data))
        with full as browser:
            before = full.file.read_text()
            browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"), 10)
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            press(browser, "redo-transcript")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel-advanced"), 5)
            got = browser.execute_script(shown, "redo-transcript-panel")
            check("Refaire… of the transcript: « Avancé » closed by default, the meeting's own model chosen in it",
                  got["expanded"] == "false" and got["hidden"] is True and got["heard"] == "Whisper tiny adv"
                  and got["written"] == "Réglages\u00a0: Qwen3 14b", got)
            # Annuler: nothing changed.
            press(browser, "redo-transcript-panel-advanced")
            Select(browser.find_element(By.ID, "redo-transcript-panel-minutes-model")).select_by_visible_text("Mistral 7b — sur cet ordinateur")
            press(browser, "redo-transcript-panel-cancel")
            time.sleep(0.5)
            check("Annuler: the meeting's models as they were", own() == {"transcription": {"engine": "whisper", "model": tiny.name}}, own())
            press(browser, "redo-transcript")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel-advanced"), 5)
            press(browser, "redo-transcript-panel-advanced")
            Select(browser.find_element(By.ID, "redo-transcript-panel-transcription-model")).select_by_index(0)
            Select(browser.find_element(By.ID, "redo-transcript-panel-minutes-model")).select_by_visible_text("Mistral 7b — sur cet ordinateur")
            hearing.unlink(missing_ok=True)
            press(browser, "redo-transcript-do")
            redone = wait_for(None, lambda b: str(model) in read(hearing), 15)
            check("Refaire with Settings' model chosen in « Avancé »: made again by it, the choice kept for the meeting",
                  redone and own() == {"minutes": {"provider": here, "model": "mistral:7b"}}, (read(hearing), own()))
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            # Its minutes first, by its own model: then Refaire… on them.
            browser.execute_script("window.__TAURI__.core.invoke('minutes_asked', {name: arguments[0]})", meeting.name)
            wait_for(None, lambda b: (meeting / "minutes.md").exists(), 15)
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            browser.execute_script("location.hash = '#/'")
            browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"), 10)
            writing.unlink(missing_ok=True)
            # Below the page's break at 840 px from here, so that a resize never draws the page anew.
            browser.set_window_size(830, 800)
            time.sleep(1.5)
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"), 10)
            press(browser, "redo-minutes")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-panel-advanced"), 5)
            got = browser.execute_script(shown, "redo-panel")
            check("Refaire… of the minutes: « Avancé » closed, the meeting's own minutes model in it",
                  got["expanded"] == "false" and got["written"] == "Mistral 7b — sur cet ordinateur", got)
            press(browser, "redo-panel-advanced")
            long_resized(browser, "fr, Refaire…", LONG_SAID + " — sur cet ordinateur")
            Select(browser.find_element(By.ID, "redo-panel-minutes-model")).select_by_visible_text("Mistral 7b — sur cet ordinateur")
            gone_ = wait_for(browser, lambda b: b.execute_script(LONG_SHOWN, "redo-panel-minutes-model")["said"] is None, 3)
            check("a name that fits: nothing said under the list", gone_, browser.execute_script(LONG_SHOWN, "redo-panel-minutes-model"))
            press(browser, "redo-do")
            wrote = wait_for(None, lambda b: "mistral:7b" in read(writing), 15)
            check("Refaire with « Avancé » left closed: made again by the meeting's own model", wrote and "qwen3:14b" not in read(writing), read(writing))
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            browser.set_window_size(1100, 800)
            time.sleep(1.5)
            # Its own model fails: the page names it, and where to choose another — Refaire… › Avancé, not Settings.
            failure = "const w = document.querySelector('#minutes-state .what'); return w ? w.textContent : ''"
            # The notification, read before the page: the same sentence.
            notified = WORK / "notified-own-minutes"
            monitor = subprocess.Popen(["dbus-monitor", "--session", "interface='org.freedesktop.Notifications',member='Notify'"],
                                       stdout=open(notified, "w"), stderr=subprocess.DEVNULL)
            time.sleep(0.5)
            (WORK / "fail-minutes").touch()
            try:
                press(browser, "redo-minutes")
                wait_for(browser, lambda b: b.find_elements(By.ID, "redo-do"), 5)
                press(browser, "redo-do")
                failed = wait_for(browser, lambda b: "pas été fait" in b.execute_script(failure), 15)
                told = browser.execute_script(failure)
                check("the meeting's own minutes model fails: the page names it, and Refaire… › Avancé to choose another",
                      failed and "Mistral 7b" in told and "Refaire…\u00a0› Avancé" in told and "réglages" not in told, told)
                heard_own = wait_for(None, lambda b: "pas été refait" in notified.read_text(), 10)
                since = len(notified.read_text())
                said_own = notified.read_text()
                check("and its notification says the same: the model named, Refaire… › Avancé, never Settings",
                      heard_own and "Mistral 7b, choisi pour cette réunion" in said_own and "Refaire…\u00a0› Avancé" in said_own
                      and "réglages" not in said_own, said_own[-500:])
                # Settings' model fails: Settings' wording, as before.
                (meeting / "models.json").unlink()
                wait_for(browser, lambda b: b.execute_script(idle), 15)
                browser.execute_script("location.hash = '#/'")
                browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
                wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"), 10)
                press(browser, "redo-minutes")
                wait_for(browser, lambda b: b.find_elements(By.ID, "redo-do"), 5)
                writing.unlink(missing_ok=True)
                press(browser, "redo-do")
                failed = wait_for(browser, lambda b: "qwen3:14b" in read(writing) and "pas été fait" in b.execute_script(failure), 15)
                told = browser.execute_script(failure)
                check("Settings' minutes model fails: Settings' wording, no « Avancé »",
                      failed and "dans les réglages de Kitokey" in told and "Avancé" not in told, told)
                heard_settings = wait_for(None, lambda b: "pas été refait" in notified.read_text()[since:], 10)
                said_settings = notified.read_text()[since:]
                check("and its notification keeps Settings' wording",
                      heard_settings and "dans les réglages de Kitokey" in said_settings and "Avancé" not in said_settings, said_settings[-500:])
            finally:
                monitor.terminate()
                (WORK / "fail-minutes").unlink()
                (meeting / "models.json").write_text(json.dumps({"minutes": {"provider": here, "model": "mistral:7b"}}))
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            check("Settings unchanged by Refaire…", full.file.read_text() == before, full.file.read_text())
            # Refaire… of the transcript: the transcript alone, by the meeting's models — its minutes untouched.
            minutes_before = (meeting / "minutes.md").read_text()
            hearing.unlink(missing_ok=True)
            writing.unlink(missing_ok=True)
            browser.execute_script("location.hash = '#/'")
            browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"), 10)
            press(browser, "redo-transcript")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-do"), 5)
            press(browser, "redo-transcript-do")
            heard = wait_for(None, lambda b: str(model) in read(hearing), 15)
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            time.sleep(2)
            check("Refaire… of the transcript makes the transcript alone, its minutes untouched",
                  heard and not writing.exists() and (meeting / "minutes.md").read_text() == minutes_before, (read(hearing), read(writing)))
            # Settings unreadable: Refaire… still opens, from those last read, and says why.
            kept_settings = full.file.read_text()
            full.file.write_text("this is [not toml")
            press(browser, "redo-transcript")
            opened = wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel-advanced"), 5)
            if opened:
                press(browser, "redo-transcript-panel-advanced")
            got = browser.execute_script(shown, "redo-transcript-panel")
            full.file.write_text(kept_settings)
            check("Settings unreadable: Refaire… opens all the same, offering what was last read, and says it",
                  opened and got["heard"] == "Réglages\u00a0: Whisper adv settings" and bool(got["said"]), got)
            if opened:
                press(browser, "redo-transcript-panel-cancel")
            else:
                browser.execute_script("location.hash = '#/'")
                browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
                wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"), 10)
            # A model gone: said on the page, Settings' standing in.
            (meeting / "models.json").write_text(json.dumps({"transcription": {"engine": "whisper", "model": tiny.name}}))
            tiny.unlink()
            browser.execute_script("location.hash = '#/'")
            browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
            said = wait_for(browser, lambda b: b.find_elements(By.ID, "models-gone") and b.find_element(By.ID, "models-gone").is_displayed(), 10)
            gone_said = browser.execute_script("return document.getElementById('models-gone')?.textContent")
            check("a model chosen for the meeting and gone: said on its page, Settings' standing in",
                  said and gone_said == "Le modèle de transcription choisi pour cette réunion n’est plus là\u00a0: celui des Réglages le remplace.", gone_said)
            press(browser, "redo-transcript")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel-advanced"), 5)
            press(browser, "redo-transcript-panel-advanced")
            got = browser.execute_script(shown, "redo-transcript-panel")
            check("and in « Avancé »: Settings' shown, the model gone said", got["heard"] == "Réglages\u00a0: Whisper adv settings"
                  and "n’est plus là" in (got["said"] or ""), got)
            press(browser, "redo-transcript-panel-cancel")
            # Confidential, Settings hearing and writing by services: « Avancé » names no service at all —
            # Settings' entry says who makes it here —; a service chosen is said unused, and refused.
            serviced = kept_settings + ('engine = "service"\ntranscription_agreed = "hears.example"\n'
                                        'minutes_agreed = "writes.example"\n')
            serviced = serviced.replace(f'minutes_provider = "{here}"', 'minutes_provider = "https://writes.example/v1"').replace('minutes_model = "qwen3:14b"', 'minutes_model = "gpt-4.1-mini"')
            full.file.write_text(serviced)
            (meeting / "confidential").write_bytes(b"")
            (meeting / "models.json").write_text(json.dumps({"transcription": {"engine": "service", "provider": "https://hears.example/v1", "model": "whisper-1"}}))
            press(browser, "redo-transcript")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel-advanced"), 5)
            press(browser, "redo-transcript-panel-advanced")
            got = browser.execute_script(shown, "redo-transcript-panel")
            check("confidential, Settings naming services: no entry names one — Settings' says Whisper and Ollama here",
                  got["hear"] and got["write"] and not any("via" in one for one in got["hear"] + got["write"])
                  and got["hear"][0] == "Réglages\u00a0: Whisper adv settings" and got["write"][0].startswith("Réglages\u00a0: "), got)
            check("a service the confidential meeting chose: not shown chosen, said unused while confidential",
                  got["heard"] == got["hear"][0] and "n’est pas utilisé tant qu’elle est confidentielle" in (got["said"] or ""), got)
            press(browser, "redo-transcript-panel-cancel")
            refused = browser.execute_async_script(
                "const done = arguments[arguments.length - 1]; window.__TAURI__.core.invoke('meeting_models_chosen', {name: arguments[0], chosen: arguments[1]}).then(() => done('kept'), () => done('refused'))",
                meeting.name, {"minutes": {"provider": "https://writes.example/v1", "model": "gpt-4.1-mini"}})
            check("a service chosen for a confidential meeting is refused, nothing written",
                  refused == "refused" and own() == {"transcription": {"engine": "service", "provider": "https://hears.example/v1", "model": "whisper-1"}}, [refused, own()])
            (meeting / "confidential").unlink()
            # Settings naming no one to write the minutes: their entry says so in words.
            full.file.write_text("\n".join(line for line in kept_settings.splitlines() if not line.startswith(("minutes_provider", "minutes_model"))) + "\n")
            press(browser, "redo-transcript")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel-advanced"), 5)
            press(browser, "redo-transcript-panel-advanced")
            got = browser.execute_script(shown, "redo-transcript-panel")
            check("Settings naming no minutes model: their entry reads « Réglages : aucun modèle choisi »",
                  (got["write"] or [None])[0] == "Réglages\u00a0: aucun modèle choisi", got)
            press(browser, "redo-transcript-panel-cancel")
            full.file.write_text(kept_settings)
        # In English, the bar and Refaire…: the long name said whole as well.
        english = Window(meetings, "en_US.UTF-8", named, ollama=base, small=True)
        english.env.update(XDG_DATA_HOME=str(data))
        with english as browser:
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 10)
            press(browser, "small-more")
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-advanced") and b.find_element(By.ID, "small-advanced").is_displayed(), 5)
            press(browser, "small-advanced")
            wait_for(browser, lambda b: b.execute_script(shown, "small")["hidden"] is False, 5)
            Select(browser.find_element(By.ID, "small-minutes-model")).select_by_visible_text(LONG_SAID + " — on this computer")
            whole = wait_for(browser, lambda b: b.execute_script(LONG_SHOWN, "small-minutes-model")["said"] is not None, 3)
            got = browser.execute_script(LONG_SHOWN, "small-minutes-model")
            check("en, the bar: the long name said whole under its list, at 12 px",
                  whole and not got["fit"] and got["said"] == got["text"] and got["size"] == "12px" and got["inside"], got)
            Select(browser.find_element(By.ID, "small-minutes-model")).select_by_index(0)
            wait_for(browser, lambda b: b.execute_script("return window.__TAURI__.core.invoke('models_offered', {name: null}).then(o => o.chosen)") == {}, 5)
        english_full = Window(meetings, "en_US.UTF-8", named, ollama=base)
        english_full.env.update(XDG_DATA_HOME=str(data))
        with english_full as browser:
            browser.set_window_size(830, 800)
            time.sleep(1.5)
            browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"), 10)
            wait_for(browser, lambda b: b.execute_script(idle), 15)
            press(browser, "redo-minutes")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-panel-advanced"), 5)
            press(browser, "redo-panel-advanced")
            long_resized(browser, "en, Redo…", LONG_SAID + " — on this computer")
            press(browser, "redo-panel-cancel")
    finally:
        tags.shutdown()


def advanced_latest_answer(now: dt.datetime) -> None:
    """BF. « Avancé » in the bar: an ask's answer come after a later ask's never draws over it (#815).
    The asks made as a recording starts are answered late — Ollama past models_offered's 2 s —, the
    one made when « Avancé » is opened at once: the late ones, come last, would close it under the user."""
    print("BF. the bar's « Avancé » opened while the start's asks are still answered: it stays open, the latest ask drawn (#815)")
    late = threading.Event()
    asked_late = []

    class Tags(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            # Asked while `late` is set: answered after the shell stopped waiting, Ollama then none.
            if late.is_set():
                asked_late.append(time.monotonic())
                time.sleep(3)
            body = json.dumps({"models": [{"name": "qwen3:14b"}, {"name": "mistral:7b"}]}).encode()
            try:
                self.send_response(200)
                self.send_header("content-length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            except OSError:
                pass

        def log_message(self, *_):
            pass

    tags = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Tags)
    threading.Thread(target=tags.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{tags.server_address[1]}"
    named = f'after = "transcript"\nreminder = false\nminutes_provider = "{base}/v1"\nminutes_model = "qwen3:14b"\n'
    meetings = WORK / "advanced-latest"
    meetings.mkdir()
    opened = "const p = document.getElementById('small-advanced-panel'); return p !== null && p.hidden === false"
    try:
        with Window(meetings, "fr_FR.UTF-8", named, ollama=base, small=True) as browser:
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 10)
            press(browser, "small-more")
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-advanced") and b.find_element(By.ID, "small-advanced").is_displayed(), 5)
            late.set()
            press(browser, "small-record")
            started = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
            # The start's asks all made: one at least come, and no other for half a second.
            wait_for(None, lambda b: asked_late, 10)
            wait_for(None, lambda b: asked_late and time.monotonic() - asked_late[-1] >= 0.5, 10)
            made = len(asked_late)
            late.clear()
            if not browser.execute_script("return document.getElementById('small-options').hidden === false"):
                press(browser, "small-more")
            press(browser, "small-advanced")
            shown = wait_for(browser, lambda b: b.execute_script(opened), 5)
            check("recording: « Avancé » opened while the start's asks are still answered", started and made >= 1 and shown, [started, made, shown])
            # The late answers come — the shell gives up on Ollama at 2 s, the stand-in answers at 3 s —:
            # the panel is still open once each has been answered, and after.
            answered = wait_for(None, lambda b: asked_late and time.monotonic() - asked_late[-1] >= 3.5, 10)
            closed = wait_for(browser, lambda b: not b.execute_script(opened), 1.5)
            check("the start's late answers, come after the one asked on opening: « Avancé » stays open, as the user left it (#815)",
                  answered and not closed and browser.execute_script("return document.getElementById('small-advanced').getAttribute('aria-expanded')") == "true",
                  browser.execute_script("const t = document.getElementById('small-advanced'); const p = document.getElementById('small-advanced-panel');"
                                         " return {expanded: t?.getAttribute('aria-expanded'), hidden: p?.hidden}"))
            press(browser, "small-stop")
            wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10)
    finally:
        tags.shutdown()


def minutes_copied(now: dt.datetime) -> None:
    """AJ. A meeting's minutes copied in one click, ready to paste where the user writes (D45)."""
    print("AJ. the minutes copied in one click: their Markdown, titled as the page names them, « Copié » said in place (D45)")
    meetings = WORK / "copying"
    written, none_, broken = (meetings / name for name in ("2026-10-28_09-00", "2026-10-28_10-00", "2026-10-28_11-00"))
    for folder in (written, none_, broken):
        session(folder, now - dt.timedelta(days=1), 10, {"local.wav": 1000})
        (folder / "transcript.jsonl").write_text(
            '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
            '{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Bonjour."}\n{"type":"complete","duration":1.0}\n')
    (written / "minutes.md").write_text(
        "---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: local\nmodel: qwen3:8b\ncontext: null\n---\n\n"
        "## 1.\n\nLe budget est validé.\n\n## 2.\n\n- Envoyer le devis — Inès — vendredi\n\n## 3.\n\n- Rien\n")
    (broken / "minutes.md").write_text("pas un compte rendu\n")
    expected = ("## Points saillants\n\nLe budget est validé.\n\n## Actions\n\n- Envoyer le devis — Inès — vendredi\n\n"
                "## Points clés\n\n- Rien\n")
    # The clipboard stood in: what the page writes is kept on the page, never the user's own clipboard.
    stand_in = ("window.__copied = []; const refuse = arguments[0];"
                "Object.defineProperty(navigator, 'clipboard', {configurable: true, value: {writeText: (text) =>"
                " refuse ? Promise.reject(new Error('refused')) : (window.__copied.push(text), Promise.resolve())}})")
    status = "return document.getElementById('minutes-copied')?.textContent ?? null"
    with Window(meetings, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.execute_script(f"location.hash = '#/meeting/{written.name}'")
        offered = wait_for(browser, lambda b: b.find_elements(By.ID, "copy-minutes"), 10)
        beside = browser.execute_script(
            "const c = document.getElementById('copy-minutes'), r = document.getElementById('redo-minutes');"
            "return c && r && c.parentElement === r.parentElement && c.nextElementSibling === r")
        check("a meeting with minutes offers « Copier le compte rendu », beside Refaire…",
              offered and beside and browser.find_element(By.ID, "copy-minutes").get_attribute("aria-label") == "Copier le compte rendu", beside)
        browser.execute_script(stand_in, False)
        press(browser, "copy-minutes")
        said = wait_for(browser, lambda b: b.execute_script(status) == "Copié", 3)
        copied = browser.execute_script("return window.__copied")
        check("clicked: their Markdown copied, without the front matter, titled as the page names them; « Copié » said in place",
              said and copied == [expected] and browser.find_element(By.ID, "minutes-copied").get_attribute("role") == "status", copied)
        gone = wait_for(browser, lambda b: b.execute_script(status) == "", 4)
        check("« Copié » said for a moment", gone, browser.execute_script(status))
        browser.execute_script(stand_in, True)
        press(browser, "copy-minutes")
        refused = wait_for(browser, lambda b: b.execute_script(status) == "Le compte rendu n’a pas pu être copié.", 3)
        check("the clipboard refused: one sentence says so, the minutes as they were",
              refused and browser.find_elements(By.CSS_SELECTOR, "#meeting .minutes .body h2"), browser.execute_script(status))
        for folder, why in ((none_, "without minutes"), (broken, "with minutes that do not read")):
            browser.execute_script(f"location.hash = '#/meeting/{folder.name}'")
            wait_for(browser, lambda b: b.execute_script("return document.getElementById('meeting').dataset.name") == folder.name
                     and b.find_elements(By.CSS_SELECTOR, "#meeting .minutes .none"), 10)
            check(f"a meeting {why} offers no copy", not browser.find_elements(By.ID, "copy-minutes"))


def announced(now: dt.datetime) -> None:
    """AM. The reminder said in the bar, the announcement one click away (D50, D90)."""
    print("AM. the reminder in a line under the bar's row for fifteen seconds, its announcement copied; then the row alone; the reminder off, neither (D50, D90)")
    line = "const a = document.getElementById('small-announce'); return [!!a && a.getClientRects().length > 0, a && a.textContent]"
    place = "return document.getElementById('small-stop').getBoundingClientRect().top"
    for theme in ("light", "dark"):
        taping = WORK / f"announcing-{theme}"
        taping.mkdir()
        window = Window(taping, "fr_FR.UTF-8", 'after = "nothing"\n', small=True)
        # The dark theme asked of GTK; the other, the desktop's own.
        if theme == "dark":
            window.env.update(GTK_THEME="Adwaita:dark")
        with window as browser:
            # The tray's icon at rest, then while recording (D89): the tile blue, then red.
            rest = wait_for(browser, lambda b: tray_colour() == "blue", 10)
            check("the tray's icon at rest: the mark on its blue tile", rest, tray_colour())
            browser.find_element(By.ID, "small-record").click()
            wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.execute_script(line)[0], 10)
            check("recording, the tray's icon turns red", wait_for(browser, lambda b: tray_colour() == "red", 5), tray_colour())
            shown = browser.execute_script(line)
            check("recording, the reminder says it", shown[0] and "Prévenez les participants que cette réunion est enregistrée." in shown[1], shown)
            order = browser.execute_script("const a = document.getElementById('small-announce'); return [a.parentElement.className, a.parentElement.previousElementSibling.className]")
            check("in the line under the bar's row, its timer and its levels", order == ["bar-lines", "bar-row"], order)
            # The bar grows to hold it: nothing to scroll.
            held = wait_for(browser, lambda b: b.execute_script("const e = document.scrollingElement; return e.scrollHeight <= e.clientHeight"), 3)
            check("nothing to scroll with it", held, browser.execute_script("const e = document.scrollingElement; return [e.scrollHeight, e.clientHeight]"))
            shot(browser, f"d50-announce-{theme}")
            browser.execute_script("window.copied_for_test = null; Object.defineProperty(navigator, 'clipboard', {configurable: true, value: {writeText: async (t) => { window.copied_for_test = t; }}})")
            browser.find_element(By.ID, "small-announce-copy").click()
            copied = wait_for(browser, lambda b: b.find_element(By.ID, "small-announce-copied").text == "Copié", 3)
            text = browser.execute_script("return window.copied_for_test")
            check("Copier l’annonce: the sentence on the clipboard, « Copié » in place", copied and text == "J’enregistre cette réunion pour en faire le compte rendu. Dites-le-moi si vous préférez que je ne le fasse pas.", text)
            before = browser.execute_script(place)
            gone = wait_for(browser, lambda b: not b.execute_script(line)[0], 20)
            check("after fifteen seconds it is gone", gone, browser.execute_script(line))
            check("gone, the bar is its row alone again: 440 × 44", wait_for(browser, lambda b: b.execute_script("return [innerWidth, innerHeight]") == [440, 44], 3), browser.execute_script("return [innerWidth, innerHeight]"))
            check("Arrêter has not moved", browser.execute_script(place) == before, [before, browser.execute_script(place)])
            stop_from_small(browser)
            check("stopped, the tray's icon is blue again", wait_for(None, lambda b: tray_colour() == "blue", 20), tray_colour())
    speaking = WORK / "announcing-en"
    speaking.mkdir()
    with Window(speaking, "fr_FR.UTF-8", 'after = "nothing"\nmeeting_language = "en"\n', small=True) as browser:
        browser.find_element(By.ID, "small-record").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.execute_script(line)[0], 10)
        # By script: a slow machine may be past the fifteen seconds, and what is judged here is the sentence.
        browser.execute_script("window.copied_for_test = null; Object.defineProperty(navigator, 'clipboard', {configurable: true, value: {writeText: async (t) => { window.copied_for_test = t; }}})")
        browser.execute_script("document.getElementById('small-announce-copy').click()")
        wait_for(browser, lambda b: b.execute_script("return window.copied_for_test"), 3)
        text = browser.execute_script("return window.copied_for_test")
        check("a meeting said in English: the announcement in English, the window in French", text == "I’m recording this meeting to write up its minutes. Tell me if you’d rather I didn’t.", text)
        # Its own language file comes before the settings', as a meeting's own word does.
        folder = next(p for p in speaking.iterdir() if (p / "session.json").exists())
        (folder / "language").write_text("fr\n")
        browser.execute_script("window.copied_for_test = null")
        time.sleep(0.5)
        browser.execute_script("document.getElementById('small-announce-copy').click()")
        wait_for(browser, lambda b: b.execute_script("return window.copied_for_test"), 3)
        text = browser.execute_script("return window.copied_for_test")
        check("the meeting's own language comes first", text.startswith("J’enregistre"), text)
        # Refused: said of the announcement, in place, nothing moved.
        before = browser.execute_script(place)
        browser.execute_script("Object.defineProperty(navigator, 'clipboard', {configurable: true, value: {writeText: () => Promise.reject(new Error('refused'))}})")
        browser.execute_script("document.getElementById('small-announce-copy').click()")
        refused = wait_for(browser, lambda b: b.find_element(By.ID, "small-announce-copied").text == "Pas copiée.", 3)
        check("a clipboard refused: said of the announcement, Arrêter where it was", refused and browser.execute_script(place) == before, browser.find_element(By.ID, "small-announce-copied").text)
        stop_from_small(browser)
    quiet = WORK / "announcing-off"
    quiet.mkdir()
    with Window(quiet, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True) as browser:
        browser.find_element(By.ID, "small-record").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop"), 10)
        time.sleep(1.5)
        check("the reminder off: neither the line nor the button shows", not browser.execute_script(line)[0], browser.execute_script(line))
        stop_from_small(browser)


def heard_checked(now: dt.datetime) -> None:
    """AN. Each side's meter in the bar's row and its health (D49, D90): a side silent two minutes
    said in a grey line, a microphone that sends nothing or a side with no device said at once."""
    print("AN. a thin meter per side in the row; a silence said at two minutes, never before, as information; a device's state at once; all inside the bar, ■ still (D49, D90)")
    place = "return document.getElementById('small-stop').getBoundingClientRect().top"
    said = "return [...document.querySelectorAll('.bar-lines .check-says')].map(s => s.getClientRects().length ? s.textContent : '')"
    quiet = "return [...document.querySelectorAll('#small .meter')].map(m => [m.classList.contains('quiet'), getComputedStyle(m.querySelector('.track')).borderTopStyle])"
    meters = ("return [...document.querySelectorAll('#small .bar-row .meter')].map(m => [m.querySelector('svg') !== null,"
              " m.querySelector('.track').getBoundingClientRect().height, m.querySelector('.track').getAttribute('aria-label')])")
    # Two minutes stood in: what the shell says of each silence, 110 seconds later; the shell's own count is tests/recording.rs's.
    later = ("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'recording_now'"
             " ? real.invoke(asked, ...rest).then((now) => ({ ...now, silent_for: now.silent_for.map((s) => s === null ? null : s + 110) })) : real.invoke(asked, ...rest) };"
             " window.restore = () => { window.__TAURI__.core = real; };")
    fits = ("const e = document.scrollingElement;"
            "const outside = [...document.querySelectorAll('#small .bar-row > *, #small .bar-lines > *, #small .meter')].filter(x => { const b = x.getBoundingClientRect();"
            " return b.height > 0 && (b.right > innerWidth + 0.5 || b.bottom > innerHeight + 0.5); }).map(x => x.className);"
            "return {scroll: [e.scrollHeight, e.clientHeight, e.scrollWidth, e.clientWidth], outside, size: [innerWidth, innerHeight]}")
    def fit(browser, what: str) -> None:
        holds = lambda got: got["scroll"][0] <= got["scroll"][1] and got["scroll"][2] <= got["scroll"][3] and not got["outside"]
        wait_for(browser, lambda b: holds(b.execute_script(fits)), 3)
        got = browser.execute_script(fits)
        check(f"{what}: nothing overflows or is cut, at {got['size']}", holds(got) and got["size"][0] == 440, got)
    expected = {
        "fr_FR.UTF-8": ("fr", "Votre micro", "Le son de la réunion",
                        "Rien n’en arrive\u00a0: vérifiez qu’il n’est pas coupé sur l’appareil lui-même, et qu’il est prêt.",
                        "La réunion\u00a0: rien depuis 2\u00a0min — l’enregistrement continue.", "Votre micro n’est pas enregistré pour l’instant."),
        "en_US.UTF-8": ("en", "Your microphone", "The meeting’s sound",
                        "Nothing comes from it: check it is not muted on the device itself, and that it is ready.",
                        "The meeting: nothing for 2 min — the recording goes on.", "Your microphone is not recorded for now."),
    }
    for language, (short, me, others, nothing, silent, waiting) in expected.items():
        for theme in ("light", "dark"):
            taping = WORK / f"checks-{short}-{theme}"
            taping.mkdir()
            (taping / "hush").touch()
            window = Window(taping, language, 'after = "nothing"\nreminder = false\n', small=True)
            if theme == "dark":
                window.env.update(GTK_THEME="Adwaita:dark")
            with window as browser:
                browser.find_element(By.ID, "small-record").click()
                wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
                got = browser.execute_script(meters)
                check("a thin meter per side in the row: its icon, a 4 px track, named by its side", got == [[True, 4, me], [True, 4, others]], got)
                check("at silence, nothing said, the bar its row alone", browser.execute_script(said) == ["", ""]
                      and wait_for(browser, lambda b: b.execute_script("return innerHeight") == 44, 3), [browser.execute_script(said), browser.execute_script("return innerHeight")])
                before = browser.execute_script(place)
                fit(browser, "listening")
                (taping / "hush").unlink()
                rose = wait_for(browser, lambda b: b.execute_script("return parseFloat(document.getElementById('level-others').style.width)") > 0, 3)
                check("levels risen: the meters move", rose)
                # Capture's word after ten seconds: the microphone that sends nothing said at once; the meeting's silence not yet.
                (taping / "nothing-me").touch()
                (taping / "silent-others").touch()
                told = wait_for(browser, lambda b: b.execute_script(said) == [nothing, ""], 3)
                time.sleep(1)
                check("a microphone that sends nothing is said at once, its meter dashed; a silence of ten seconds is not said",
                      told and browser.execute_script(said) == [nothing, ""] and browser.execute_script(quiet)[0] == [True, "dashed"]
                      and not browser.execute_script(quiet)[1][0], [browser.execute_script(said), browser.execute_script(quiet)])
                browser.execute_script(later)
                long = wait_for(browser, lambda b: b.execute_script(said) == [nothing, silent], 3)
                check("silent two minutes: said in words, its meter dashed, its icon crossed — grey, never as an error",
                      long and browser.execute_script(quiet)[1] == [True, "dashed"]
                      and browser.execute_script("const l = [...document.querySelectorAll('.bar-lines .check-says')][1]; return getComputedStyle(l).color === getComputedStyle(document.querySelector('#small .meter')).color && !l.closest('.warn')"),
                      browser.execute_script(said))
                fit(browser, "two sides said")
                check("■ has not moved", browser.execute_script(place) == before, [before, browser.execute_script(place)])
                # Nothing rewritten at each tick: a button or a line rewritten ten times a second loses clicks.
                browser.execute_script("window.written = 0; const seen = new MutationObserver((m) => { window.written += m.length; }); for (const e of document.querySelectorAll('.check-says')) seen.observe(e, {subtree: true, childList: true, characterData: true})")
                time.sleep(1)
                check("what does not change is not written again", browser.execute_script("return window.written") == 0, browser.execute_script("return window.written"))
                shot_themed(browser, f"d90-silent-{short}-{theme}", theme)
                press(browser, "small-more")
                fit(browser, "two sides said, the options open")
                shot_themed(browser, f"d90-silent-options-{short}-{theme}", theme)
                (taping / "heard-me").touch()
                back = wait_for(browser, lambda b: b.execute_script(said)[0] == "" and not b.execute_script(quiet)[0][0], 3)
                check("heard again: its line gone, its meter whole, ■ still", back and browser.execute_script(place) == before, browser.execute_script(said))
                browser.execute_script("window.restore()")
                stop_from_small(browser)
        # The microphone waiting (D66): said truthfully, at once.
        taping = WORK / f"checks-waiting-{short}"
        taping.mkdir()
        (taping / "waiting-me").touch()
        with Window(taping, language, 'after = "nothing"\nreminder = false\n', small=True) as browser:
            browser.find_element(By.ID, "small-record").click()
            shown = wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.execute_script(said) == [waiting, ""], 10)
            check("no microphone: said truthfully under the row, its meter dashed", shown and browser.execute_script(quiet)[0] == [True, "dashed"], browser.execute_script(said))
            fit(browser, "a side waiting")
            shot(browser, f"d90-waiting-{short}")
            stop_from_small(browser)
    # A user who asked for less motion, in a running window: nothing in the recording moves.
    with asking_less_motion(Window(WORK / "checks-waiting-fr", "fr_FR.UTF-8", 'after = "nothing"\n', small=True)) as browser:
        check("less motion asked: the page's query says so", browser.execute_script(LESS_MOTION))
        browser.execute_script(WATCH_MOTION)
        browser.find_element(By.ID, "small-record").click()
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, ".bar-lines .check-says") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        time.sleep(1)
        check("less motion asked: nothing in the recording moves", browser.execute_script(MOVING) == [], browser.execute_script(MOVING))
        stop_from_small(browser)


def microphone_chosen(now: dt.datetime) -> None:
    """AO. The microphone chosen by hand beside Auto, in the bar's options at rest (D74, D90)."""
    print("AO. Auto and the system's microphones, keyboard alone, either theme, inside the bar opened; kept, for the next recording; none offered where capture lists none; a new device opens nothing (D74)")
    listed = WORK / "microphones"
    unread = WORK / "microphones-unread"
    slow = WORK / "microphones-slow"
    args = WORK / "recorder-args"
    def lines(*pairs) -> str:
        return "".join(json.dumps({"event": "microphone", "name": name, "description": said}) + "\n" for name, said in pairs)
    long_name = "Ryzen HD Audio Controller Digital Microphone with a very long name indeed that goes on and on"
    two = [("alsa_laptop", "Digital Microphone"), ("bluez_input.AA_BB", long_name)]
    choices = "return [...document.querySelectorAll('#small-microphones .choice')].map(c => [c.querySelector('.name').textContent, c.querySelector('input').value, c.querySelector('input').checked])"
    state = ("const b = document.getElementById('small-microphone'); const p = document.getElementById('small-microphones'); const a = document.activeElement;"
             "return {shown: !!b && b.getClientRects().length > 0, open: !!b && b.getAttribute('aria-expanded') === 'true', panel: !!p && !p.hidden, name: b && b.textContent, active: a.id,"
             " value: a.matches('#small-microphones input') ? a.value : null}")
    # The row the keyboard is on lies inside the list, whole.
    whole = ("const a = document.activeElement.closest('.choice'); const r = document.getElementById('small-microphones').getBoundingClientRect();"
             "if (!a) return null; const b = a.getBoundingClientRect(); return [b.top >= r.top - 0.5 && b.bottom <= r.bottom + 0.5, b.top, b.bottom, r.top, r.bottom]")
    fits = ("const e = document.scrollingElement; const p = document.getElementById('small-microphones'); const r = p.getBoundingClientRect();"
            "const inside = r.left >= -0.5 && r.right <= innerWidth + 0.5 && r.top >= -0.5 && r.bottom <= innerHeight + 0.5;"
            "const cut = [...p.querySelectorAll('.name, .says')].filter(x => x.scrollWidth > x.clientWidth + 1 || x.getBoundingClientRect().right > r.right + 0.5).map(x => x.textContent);"
            "return {scroll: [e.scrollHeight, e.clientHeight, e.scrollWidth, e.clientWidth], inside, cut, wide: [p.scrollWidth, p.clientWidth], size: [innerWidth, innerHeight], rect: [r.left, r.top, r.right, r.bottom]}")
    def fit(browser, what: str) -> None:
        holds = lambda got: got["scroll"][0] <= got["scroll"][1] and got["scroll"][2] <= got["scroll"][3] and got["inside"] and not got["cut"] and got["wide"][0] <= got["wide"][1]
        # The bar grows to hold the list.
        wait_for(browser, lambda b: holds(b.execute_script(fits)), 3)
        got = browser.execute_script(fits)
        ok = holds(got)
        check(f"{what}: nothing overflows or is cut, at {got['size']}", ok, got)
    def keys(browser, *pressed) -> None:
        ActionChains(browser).send_keys(*pressed).perform()
        time.sleep(0.2)
    def options(browser) -> None:
        """The bar's options opened (D90): the microphone is there."""
        wait_for(browser, lambda b: b.find_element(By.ID, "small-more").is_displayed(), 8)
        if browser.find_element(By.ID, "small-more").get_attribute("aria-expanded") != "true":
            press(browser, "small-more")
        wait_for(browser, lambda b: b.execute_script(state)["shown"], 8)
    def pick(browser) -> None:
        options(browser)
        press(browser, "small-microphone")
    def to_the_name(browser) -> None:
        """By Tab alone, from where the window opened: the recording's gesture."""
        for _ in range(6):
            if browser.execute_script("return document.activeElement.id") == "small-microphone":
                return
            keys(browser, Keys.TAB)
    for language, (short, auto, title, gone, waits) in {
        "fr_FR.UTF-8": ("fr", "Micro\u00a0: Auto", "Micro\u00a0: Digital Microphone", "Le micro choisi n’est pas là\u00a0: Auto le remplace.", "Votre micro n’est pas enregistré pour l’instant."),
        "en_US.UTF-8": ("en", "Microphone: Auto", "Microphone: Digital Microphone", "The microphone you chose is not there: Auto stands in.", "Your microphone is not recorded for now."),
    }.items():
        for theme in ("light", "dark"):
            listed.write_text(lines(*two))
            args.unlink(missing_ok=True)
            taping = WORK / f"microphones-{short}-{theme}"
            taping.mkdir()
            window = Window(taping, language, 'after = "nothing"\n', small=True)
            if theme == "dark":
                window.env.update(GTK_THEME="Adwaita:dark")
            with window as browser:
                wait_for(browser, lambda b: b.find_element(By.ID, "small-more").is_displayed(), 8)
                # Read once the bar settled, bounded: « Plus d’options » shows before bar_fit has made it 44 high (#755).
                wait_for(browser, lambda b: b.execute_script("return innerHeight") == 44, 3)
                closed = browser.execute_script("return [innerWidth, innerHeight, document.getElementById('small-more').getAttribute('aria-label')]")
                options(browser)
                opened = wait_for(browser, lambda b: b.execute_script("return innerHeight") > 44, 3)
                got = browser.execute_script(state)
                check(f"{short} {theme}: closed, 440 × 44; « {closed[2]} » opens the options under the row, the bar growing", closed[:2] == [440, 44] and opened
                      and browser.find_element(By.ID, "small-more").get_attribute("aria-expanded") == "true", [closed, browser.execute_script("return innerHeight")])
                check(f"{short} {theme}: at rest the name says Auto, closed", auto in got["name"] and not got["panel"], got)
                to_the_name(browser)
                keys(browser, Keys.ENTER)
                wait_for(browser, lambda b: len(b.execute_script(choices)) == 3, 5)
                got = browser.execute_script(state)
                shown = browser.execute_script(choices)
                check(f"{short} {theme}: Enter opens Auto and the system's microphones, Auto checked and focused",
                      got["open"] and got["panel"] and [c[1] for c in shown] == ["", "alsa_laptop", "bluez_input.AA_BB"] and shown[0][0] == "Auto" and shown[0][2] and got["value"] == "", [got, shown])
                check(f"{short} {theme}: the focus is Auto's own row, not the name", browser.execute_script("return document.activeElement === document.querySelector('#small-microphones input[value=\\'\\']')"))
                fit(browser, f"{short} {theme}: the list open")
                shot_themed(browser, f"d74-list-{short}-{theme}", theme)
                keys(browser, Keys.ARROW_DOWN)
                check(f"{short} {theme}: an arrow moves to the next", browser.execute_script(state)["value"] == "alsa_laptop", browser.execute_script(state))
                keys(browser, Keys.ENTER)
                got = browser.execute_script(state)
                check(f"{short} {theme}: Enter closes, the focus back on the name, which says the choice",
                      not got["panel"] and got["active"] == "small-microphone" and title in got["name"], got)
                wait_for(browser, lambda b: 'microphone = "alsa_laptop"' in window.file.read_text(), 3)
                check(f"{short} {theme}: the choice is kept in the settings", 'microphone_description = "Digital Microphone"' in window.file.read_text(), window.file.read_text())
                # Escape leaves as it was; opened again, the choice is the one checked.
                keys(browser, Keys.ENTER)
                wait_for(browser, lambda b: b.execute_script(state)["panel"], 3)
                check(f"{short} {theme}: opened again, the one kept is checked", browser.execute_script(state)["value"] == "alsa_laptop", browser.execute_script(state))
                # Arrows only move: Escape after them keeps the choice as it was.
                keys(browser, Keys.ARROW_UP)
                check(f"{short} {theme}: an arrow moves back to Auto", browser.execute_script(state)["value"] == "", browser.execute_script(state))
                keys(browser, Keys.ESCAPE)
                got = browser.execute_script(state)
                check(f"{short} {theme}: Escape closes, the focus back on the name", not got["panel"] and got["active"] == "small-microphone", got)
                time.sleep(0.5)
                check(f"{short} {theme}: arrows then Escape leave the choice as it was", 'microphone = "alsa_laptop"' in window.file.read_text() and title in browser.execute_script(state)["name"], window.file.read_text())
                # Home and End move alone; Space chooses, keeps and closes, as Enter does.
                keys(browser, Keys.ENTER)
                check(f"{short} {theme}: opened for Home and End", wait_for(browser, lambda b: b.execute_script(state)["panel"], 3))
                keys(browser, Keys.END)
                check(f"{short} {theme}: End reaches the last row", browser.execute_script(state)["value"] == "bluez_input.AA_BB", browser.execute_script(state))
                keys(browser, Keys.HOME)
                check(f"{short} {theme}: Home reaches Auto", browser.execute_script(state)["value"] == "", browser.execute_script(state))
                time.sleep(0.5)
                check(f"{short} {theme}: Home and End choose nothing", 'microphone = "alsa_laptop"' in window.file.read_text() and browser.execute_script(state)["panel"], window.file.read_text())
                keys(browser, Keys.END, Keys.SPACE)
                got = browser.execute_script(state)
                check(f"{short} {theme}: Space chooses the row and closes, the focus on the name", not got["panel"] and got["active"] == "small-microphone" and long_name[:20] in got["name"], got)
                check(f"{short} {theme}: what Space chose is kept", wait_for(browser, lambda b: 'microphone = "bluez_input.AA_BB"' in window.file.read_text(), 3), window.file.read_text())
                keys(browser, Keys.ENTER)
                wait_for(browser, lambda b: b.execute_script(state)["panel"], 3)
                keys(browser, Keys.HOME, Keys.ARROW_DOWN, Keys.SPACE)
                check(f"{short} {theme}: Space chooses again the first microphone", wait_for(browser, lambda b: 'microphone = "alsa_laptop"' in window.file.read_text(), 3), window.file.read_text())
                # So many that they scroll: each row the keyboard reaches is seen whole, short names and long.
                for count, length in ((14, 0), (20, 100)):
                    many = [(f"m{n}", f"Microphone {n} " + "x" * max(0, length - 14)) for n in range(count)]
                    listed.write_text(lines(*two, *many))
                    keys(browser, Keys.ENTER)
                    check(f"{short} {theme}: {count} more listed", wait_for(browser, lambda b: len(b.execute_script(choices)) == count + 3, 5), browser.execute_script(choices))
                    cut = []
                    for _ in range(count + 2):
                        keys(browser, Keys.ARROW_DOWN)
                        seen = browser.execute_script(whole)
                        if not seen or not seen[0]:
                            cut.append([browser.execute_script(state)["value"], seen])
                    check(f"{short} {theme}: {count} names of {length or 'a few'} characters: each row reached by the keyboard is seen whole", not cut and browser.execute_script(state)["value"] == f"m{count - 1}", cut)
                    keys(browser, Keys.ESCAPE)
                    time.sleep(0.5)
                    check(f"{short} {theme}: {count} names gone through, Escape leaves the choice as it was", 'microphone = "alsa_laptop"' in window.file.read_text(), window.file.read_text())
                listed.write_text(lines(*two))
                # Escape in the options, the list closed: they close, the bar back to its row, the focus on its chevron.
                to_the_name(browser)
                keys(browser, Keys.ESCAPE)
                folded = wait_for(browser, lambda b: b.execute_script("return innerHeight") == 44, 3)
                check(f"{short} {theme}: Escape closes the options, 44 high again, the focus on « Plus d’options »",
                      folded and browser.execute_script("return [document.activeElement.id, document.activeElement.getAttribute('aria-expanded')]") == ["small-more", "false"],
                      browser.execute_script("return [innerHeight, document.activeElement.id]"))
                # The next recording takes it; capture says it gone: said, inside the window, the context open.
                press(browser, "small-record")
                wait_for(browser, lambda b: b.execute_script("return !document.querySelector('.small-live').hidden"), 8)
                check(f"{short} {theme}: the next recording names it", "--microphone alsa_laptop" in args.read_text(), args.read_text())
                (taping / "mic-gone").touch()
                press(browser, "small-more")
                said = "return document.querySelector('.check-says').textContent"
                wait_for(browser, lambda b: b.execute_script(said) != "", 5)
                check(f"{short} {theme}: gone, the small window says so", browser.execute_script(said) == gone, browser.execute_script(said))
                live = ("const e = document.scrollingElement; const out = [...document.querySelectorAll('.small-live > *')].filter(x => { const b = x.getBoundingClientRect(); return b.height > 0 && (b.right > innerWidth + 0.5 || b.bottom > innerHeight + 0.5); }).map(x => x.id || x.className);"
                        "return {scroll: [e.scrollHeight, e.clientHeight, e.scrollWidth, e.clientWidth], out}")
                # Settled, bounded: the bar grows to the sentence a moment after it is said (#755).
                settled = lambda got: got["scroll"][0] <= got["scroll"][1] and got["scroll"][2] <= got["scroll"][3] and not got["out"]
                wait_for(browser, lambda b: settled(b.execute_script(live)), 3)
                got = browser.execute_script(live)
                check(f"{short} {theme}: said with the context open, nothing overflows or is cut", got["scroll"][0] <= got["scroll"][1] and got["scroll"][2] <= got["scroll"][3] and not got["out"], got)
                shot_themed(browser, f"d74-gone-{short}-{theme}", theme)
                # Another of the same description is no return: the sentence stays, and it is true.
                (taping / "mic-same").write_text("Digital Microphone")
                time.sleep(0.6)
                check(f"{short} {theme}: a device of the same description is not the return", browser.execute_script(said) == gone, browser.execute_script(said))
                (taping / "mic-back").write_text("Digital Microphone")
                wait_for(browser, lambda b: b.execute_script(said) != gone, 5)
                check(f"{short} {theme}: back, it says so no more", browser.execute_script(said) == "", browser.execute_script(said))
                # A headset's, back with no call on it: not « not there », nor Auto: what is so.
                (taping / "mic-gone").touch()
                check(f"{short} {theme}: gone again, said again", wait_for(browser, lambda b: b.execute_script(said) == gone, 5), browser.execute_script(said))
                (taping / "mic-back-waits").touch()
                wait_for(browser, lambda b: b.execute_script(said) != gone, 5)
                check(f"{short} {theme}: back with no call, it says the microphone is not recorded for now", browser.execute_script(said) == waits, browser.execute_script(said))
                # Gone, and the recording ends so: the next one does not begin by saying it.
                (taping / "mic-gone").touch()
                check(f"{short} {theme}: gone a third time, said", wait_for(browser, lambda b: b.execute_script(said) == gone, 5), browser.execute_script(said))
                stop_from_small(browser)
                check(f"{short} {theme}: stopped, at rest", wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed() and not b.find_element(By.ID, "small-stop").is_displayed(), 8))
                # A press within half a second of the last is the same one, for the shell.
                time.sleep(0.8)
                press(browser, "small-record")
                check(f"{short} {theme}: the next recording runs", wait_for(browser, lambda b: b.execute_script("return !document.querySelector('.small-live').hidden"), 8))
                time.sleep(1)
                check(f"{short} {theme}: the next recording does not say the last one's gone", browser.execute_script(said) != gone, browser.execute_script(said))
                stop_from_small(browser)
                wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed() and not b.find_element(By.ID, "small-stop").is_displayed(), 8)
            # Kept for another start; Auto again takes the name away, from the file and the next start.
            with window as browser:
                options(browser)
                check(f"{short} {theme}: started again, the name says the choice kept", title in browser.execute_script(state)["name"], browser.execute_script(state))
                # A click chooses and closes, as a menu does — before the list read at the opening answers:
                # that answer, older than the choice, never takes it back.
                slow.touch()
                def click_row(index: int) -> None:
                    pick(browser)
                    wait_for(browser, lambda b: b.execute_script(state)["panel"], 3)
                    browser.find_elements(By.CSS_SELECTOR, "#small-microphones .choice")[index].click()
                    wait_for(browser, lambda b: not b.execute_script(state)["panel"], 3)
                    time.sleep(1.6)
                click_row(0)
                check(f"{short} {theme}: a click on Auto chooses it and closes, still Auto once the list has answered", auto in browser.execute_script(state)["name"], browser.execute_script(state))
                click_row(1)
                check(f"{short} {theme}: a click on a microphone before the list answers: still its name once it has", title in browser.execute_script(state)["name"], browser.execute_script(state))
                click_row(0)
                slow.unlink()
                check(f"{short} {theme}: Auto again, still Auto once the list has answered", auto in browser.execute_script(state)["name"], browser.execute_script(state))
                check(f"{short} {theme}: Auto takes the microphone out of the settings", wait_for(browser, lambda b: 'microphone = ' not in window.file.read_text(), 3), window.file.read_text())
                args.unlink(missing_ok=True)
                press(browser, "small-record")
                wait_for(browser, lambda b: args.exists(), 8)
                check(f"{short} {theme}: on Auto the recording names none", "--microphone" not in args.read_text(), args.read_text())
                wait_for(browser, lambda b: b.execute_script("return !document.querySelector('.small-live').hidden"), 8)
                stop_from_small(browser)
                wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed() and not b.find_element(By.ID, "small-stop").is_displayed(), 8)

    # What the fr light window above does not need twice: the other states, once.
    taping = WORK / "microphones-states"
    taping.mkdir()
    listed.write_text(lines(*two))
    with Window(taping, "fr_FR.UTF-8", 'after = "nothing"\nmicrophone = "gone_one"\nmicrophone_description = "Casque parti"\n', small=True) as browser:
        pick(browser)
        wait_for(browser, lambda b: len(b.execute_script(choices)) == 4, 5)
        shown = browser.execute_script(choices)
        check("a choice no longer listed stays shown, checked, and says it is absent",
              shown[-1][:2] == ["Casque parti", "gone_one"] and shown[-1][2] and "absent" in browser.execute_script("return [...document.querySelectorAll('#small-microphones .choice')].at(-1).textContent"), shown)
        # A new device, the list open and closed: nothing opens, nothing moves; it is there next time.
        keys(browser, Keys.ESCAPE)
        handles = len(browser.window_handles)
        listed.write_text(lines(*two, ("usb_new", "USB Headset")))
        time.sleep(3)
        got = browser.execute_script(state)
        check("a new device arriving opens no window, no dialog, no list", len(browser.window_handles) == handles and not got["panel"] and not browser.find_elements(By.CSS_SELECTOR, "dialog, [role=dialog], [role=alertdialog]"), got)
        pick(browser)
        wait_for(browser, lambda b: any(c[1] == "usb_new" for c in b.execute_script(choices)), 5)
        check("the new device is there when the name is opened", any(c[1] == "usb_new" for c in browser.execute_script(choices)), browser.execute_script(choices))
        keys(browser, Keys.ESCAPE)
        # So many that they scroll inside the list, the last one reached by the keyboard.
        listed.write_text(lines(*[(f"mic{n}", f"Microphone {n}") for n in range(14)]))
        pick(browser)
        wait_for(browser, lambda b: len(b.execute_script(choices)) == 16, 5)
        fit(browser, "fifteen microphones")
        reach = "const p = document.getElementById('small-microphones'); return [p.scrollHeight > p.clientHeight, getComputedStyle(p).overflowY]"
        check("fifteen microphones scroll inside the list", browser.execute_script(reach) == [True, "auto"], browser.execute_script(reach))
        keys(browser, Keys.ESCAPE)
    # Settings that cannot be written (D82): the choice is said not kept, and the earlier one stays —
    # in the name, in the list opened again, and for the next recording.
    taping = WORK / "microphones-unwritten"
    taping.mkdir()
    listed.write_text(lines(*two))
    args.unlink(missing_ok=True)
    window = Window(taping, "fr_FR.UTF-8", 'after = "nothing"\nmicrophone = "alsa_laptop"\nmicrophone_description = "Digital Microphone"\n', small=True)
    said = "return document.querySelector('.small-said').textContent"
    with window as browser:
        options(browser)
        kept = window.file.read_text()
        window.file.chmod(0o444)
        window.file.parent.chmod(0o555)
        try:
            pick(browser)
            wait_for(browser, lambda b: len(b.execute_script(choices)) == 3, 5)
            # The list read at the opening answered first: it never paints the name after the refusal.
            time.sleep(1.6)
            browser.find_elements(By.CSS_SELECTOR, "#small-microphones .choice")[2].click()
            check("a choice the settings cannot keep is said not kept",
                  wait_for(browser, lambda b: "n’a pas pu être gardé" in b.execute_script(said), 5), browser.execute_script(said))
            named = browser.execute_script(state)["name"]
            check("the name shows the earlier choice again", "Digital Microphone" in named and "Ryzen" not in named, named)
            if not browser.execute_script(state)["panel"]:
                pick(browser)
            wait_for(browser, lambda b: len(b.execute_script(choices)) == 3, 5)
            checked = [one[1] for one in browser.execute_script(choices) if one[2]]
            check("the list opened again checks the earlier choice", checked == ["alsa_laptop"], browser.execute_script(choices))
            keys(browser, Keys.ESCAPE)
            check("the settings are as they were", window.file.read_text() == kept, window.file.read_text())
            press(browser, "small-record")
            wait_for(browser, lambda b: args.exists(), 8)
            check("the next recording takes the earlier choice", "--microphone alsa_laptop" in args.read_text(), args.read_text())
            wait_for(browser, lambda b: b.execute_script("return !document.querySelector('.small-live').hidden"), 8)
            stop_from_small(browser)
            wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed() and not b.find_element(By.ID, "small-stop").is_displayed(), 8)
        finally:
            window.file.parent.chmod(0o755)
            window.file.chmod(0o644)
    # The list that cannot be read: said, Auto still there; none at all: nothing offered.
    taping = WORK / "microphones-unread-folder"
    taping.mkdir()
    unread.touch()
    with Window(taping, "fr_FR.UTF-8", 'after = "nothing"\n', small=True) as browser:
        pick(browser)
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#small-microphones [role=status]"), 5)
        said = browser.execute_script("return document.querySelector('#small-microphones [role=status]')?.textContent ?? ''")
        check("a list that cannot be read is said, Auto still offered", "Auto" in browser.execute_script(choices)[0][0] and "ne peut pas être lue" in said, said)
        fit(browser, "the list unread")
    unread.unlink()
    listed.unlink()
    taping = WORK / "microphones-none"
    taping.mkdir()
    args.unlink(missing_ok=True)
    with Window(taping, "fr_FR.UTF-8", 'after = "nothing"\n', small=True) as browser:
        time.sleep(1.5)
        check("where capture lists none — macOS, Windows —, no list is offered; the options still hold « Vérifier le son » (D90)",
              not browser.execute_script(state)["shown"] and browser.find_element(By.ID, "small-more").is_displayed(), browser.execute_script(state))
        press(browser, "small-record")
        wait_for(browser, lambda b: args.exists(), 8)
        check("and the recording names none", "--microphone" not in args.read_text(), args.read_text())
        wait_for(browser, lambda b: b.execute_script("return !document.querySelector('.small-live').hidden"), 8)
        press(browser, "small-more")
        opened = wait_for(browser, lambda b: b.find_element(By.ID, "small-context").is_displayed(), 3)
        check("recording, where capture lists none: no microphone in the options, nothing said of changing it (D90)",
              opened and not browser.find_element(By.ID, "small-microphone").is_displayed() and not browser.find_element(By.ID, "small-microphone-next").is_displayed())
        stop_from_small(browser)


def sound_checked(now: dt.datetime) -> None:
    """AZ. « Vérifier le son » (D88, board 7), from Settings: capture listens only while the panel shows."""
    print("AZ. « Vérifier le son » from Settings: each side's device and meter, the headset's sentence; a test sound played and said heard or not, never during a call; no stream of Kitokey's left within a second of closing; a recording started from it (D88)")
    sessions = WORK / "sessions-check"
    sessions.mkdir(exist_ok=True)
    mic = subprocess.run(["pactl", "load-module", "module-null-sink", "sink_name=kitokey_e2e_check_mic", "media.class=Audio/Source/Virtual"],
                         capture_output=True, text=True).stdout.strip()
    flag = lambda name: WORK / f"listen-{name}"
    panel = ("const d = document.getElementById('check-sound'); if (!d) return null; const t = (id) => document.getElementById(id).textContent;"
             " const w = (id) => document.getElementById(id).style.width;"
             " return {open: d.open, title: t('check-sound-title'), me: t('check-device-me'), others: t('check-device-others'),"
             " levels: [w('check-level-me'), w('check-level-others')], play: document.getElementById('check-play').disabled,"
             " active: d.contains(document.activeElement), shown: ['check-bluetooth', 'check-call', 'check-heard', 'check-missed', 'check-failed']"
             ".filter((id) => !document.getElementById(id).hidden).map((id) => [id, document.getElementById(id).textContent])}")
    fits = ("const d = document.getElementById('check-sound').getBoundingClientRect(); const e = document.scrollingElement;"
            "const cut = [...document.querySelectorAll('#check-sound button, #check-sound .check-said span')].filter(x => x.getBoundingClientRect().right > d.right + 0.5).map(x => x.textContent);"
            "return {inside: d.left >= -0.5 && d.right <= innerWidth + 0.5 && d.top >= -0.5 && d.bottom <= innerHeight + 0.5, cut, size: [innerWidth, innerHeight]}")
    def opened(browser) -> dict | None:
        press(browser, "check-sound-open")
        wait_for(browser, lambda b: (b.execute_script(panel) or {}).get("me"), 5)
        return browser.execute_script(panel)
    def closed_within(window: Window, browser, gesture) -> float | None:
        """The seconds from the gesture to the last of Kitokey's streams gone; none past three."""
        gesture(browser)
        asked = time.monotonic()
        while time.monotonic() - asked < 3:
            if not kitokey_streams(window):
                return time.monotonic() - asked
            time.sleep(0.05)
        return None
    def test_sound(window: Window, browser, heard: bool) -> tuple[set[str], dict]:
        """« Jouer un son de test »: the outputs the page played it to; heard on the meeting's side when the scenario says so."""
        flag("loud").unlink(missing_ok=True)
        press(browser, "check-play")
        played: set[str] = set()
        until = time.monotonic() + 2
        while time.monotonic() < until and not played:
            played = played_to(window)
            time.sleep(0.03)
        if heard and played:
            flag("loud").touch()
        wait_for(browser, lambda b: any(id_ in ("check-heard", "check-missed") for id_, _ in b.execute_script(panel)["shown"]), 5)
        flag("loud").unlink(missing_ok=True)
        return played, browser.execute_script(panel)
    try:
        for name in ("headset", "loud"):
            flag(name).unlink(missing_ok=True)
        flag("headset").touch()
        window = Window(sessions, "fr_FR.UTF-8")
        with window as browser:
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "check-sound-open"), 8)
            check("at rest, Settings shown, Kitokey holds no stream", kitokey_streams(window) == [], kitokey_streams(window))
            row = browser.execute_script("const b = document.getElementById('check-sound-open'); return [b.textContent, b.querySelectorAll('svg.glyph').length]")
            check("Settings offer « Vérifier le son », a labelled button with its icon", row == ["Vérifier le son", 1], row)
            got = opened(browser)
            check("the panel opens, named, the microphone and the meeting's sound each named by their device",
                  got and got["open"] and got["title"] == "Vérifier le son" and got["me"] == "Kitokey e2e mic" and got["others"] == "Kitokey e2e output", got)
            check("capture is asked to listen, the microphone of the settings — Auto here", (WORK / "listen-args").read_text() == "listen\n", (WORK / "listen-args").read_text())
            wait_for(browser, lambda b: b.execute_script(panel)["levels"] == ["40%", "5%"], 3)
            check("each meter moves with its side's level", browser.execute_script(panel)["levels"] == ["40%", "5%"], browser.execute_script(panel)["levels"])
            check("the microphone is open while the panel shows: a stream of Kitokey's", "Stream/Input/Audio" in kitokey_streams(window), kitokey_streams(window))
            said = dict(browser.execute_script(panel)["shown"])
            check("a Bluetooth headset's microphone: the panel says it, in one sentence",
                  said.get("check-bluetooth") == "Tant que ce panneau est ouvert, votre casque Bluetooth passe en qualité d’appel\u202f; il revient à la fermeture.", said)
            check("the focus is inside the panel, a modal one", browser.execute_script(panel)["active"])
            for width, theme in ((1920, "light"), (1920, "dark"), (420, "light"), (420, "dark")):
                browser.set_window_size(width, 760)
                time.sleep(0.3)
                got = browser.execute_script(fits)
                check(f"at {width} CSS px, {theme}: the panel is inside the window, nothing cut", got["inside"] and not got["cut"], got)
                shot_themed(browser, f"check-sound-{width}-{theme}", theme)
            browser.set_window_size(1100, 800)
            played, got = test_sound(window, browser, heard=False)
            check("« Jouer un son de test » plays on the run's output", played == {"kitokey_e2e_quiet"}, played)
            check("not heard on the meeting's side: said, and what to do",
                  dict(got["shown"]).get("check-missed") == "Kitokey n’a pas entendu le son de test. Vérifiez que le son sort de cet ordinateur et pas d’un autre appareil, puis rejouez-le.", got)
            gone = wait_for(browser, lambda b: "Stream/Output/Audio" not in kitokey_streams(window), 2)
            check("the test sound played, its stream is gone", gone, kitokey_streams(window))
            played, got = test_sound(window, browser, heard=True)
            check("heard on the meeting's side: said", dict(got["shown"]).get("check-heard") == "Kitokey entend le son de cet ordinateur." and "check-missed" not in dict(got["shown"]), got)
            shot_themed(browser, "check-sound-heard-light", "light")
            # A call holds a microphone: an application of its own, recording it.
            call = subprocess.Popen(["pw-record", "--target", "kitokey_e2e_check_mic", str(WORK / "call-check.wav")])
            try:
                held = wait_for(browser, lambda b: "check-call" in dict(b.execute_script(panel)["shown"]), 8)
                got = browser.execute_script(panel)
                check("a call holds a microphone: no test sound, the sentence of the board, alone — no headset's",
                      held and got["play"] and dict(got["shown"]).get("check-call") == "Un appel tient le micro\u00a0: pas de son de test, il partirait dans l’appel. Faites jouer un son dans l’appel et regardez la jauge."
                      and "check-bluetooth" not in dict(got["shown"]), got)
                shot_themed(browser, "check-sound-call-light", "light")
                refused = browser.execute_script("return window.__TAURI__.core.invoke('sound_check_mark')")
                check("the shell refuses a test sound while a call holds a microphone", refused == {"marked": False}, refused)
                # Closed and opened again, « Jouer un son de test » pressed at once, the call still held: nothing plays.
                press(browser, "check-sound-close")
                wait_for(browser, lambda b: b.execute_script("return document.getElementById('check-sound') === null"), 3)
                press(browser, "check-sound-open")
                browser.execute_script("document.getElementById('check-play').click()")
                time.sleep(1.0)
                outputs = [kind for kind in kitokey_streams(window) if kind == "Stream/Output/Audio"]
                check("pressed at once while a call holds a microphone: no sound, no stream on the output", outputs == [] and not played_to(window), [outputs, played_to(window)])
            finally:
                call.terminate()
                call.wait(timeout=5)
            freed = wait_for(browser, lambda b: not b.execute_script(panel)["play"], 8)
            check("the call gone, the test sound is offered again", freed and "check-call" not in dict(browser.execute_script(panel)["shown"]), browser.execute_script(panel))
            took = closed_within(window, browser, lambda b: press(b, "check-sound-close"))
            check("closed by ×: no stream of Kitokey's left within one second", took is not None and took <= 1.0, took)
            check("capture was told « stop »", (WORK / "listen-told").is_file() and (WORK / "listen-told").read_text() == "stop\n")
            # The streams go before the panel: it waits for capture's end, then leaves.
            gone_back = "return [document.getElementById('check-sound') === null, document.activeElement.id]"
            wait_for(browser, lambda b: b.execute_script(gone_back) == [True, "check-sound-open"], 3)
            back = browser.execute_script(gone_back)
            check("the panel is gone, the focus back on its button", back == [True, "check-sound-open"], back)
            opened(browser)
            took = closed_within(window, browser, lambda b: ActionChains(b).send_keys(Keys.ESCAPE).perform())
            check("closed by Escape: no stream of Kitokey's left within one second", took is not None and took <= 1.0, took)
            # A close sent before the shell counted the opening — the opening's answer held back 0.6 s —:
            # nothing of it is left once it answers.
            browser.execute_script("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) =>"
                                   " asked === 'sound_check_open' ? new Promise((done) => setTimeout(done, 600)).then(() => real.invoke(asked, ...rest))"
                                   " : real.invoke(asked, ...rest) }; window.restoreInvoke = () => { window.__TAURI__.core = real; };")
            press(browser, "check-sound-open")
            press(browser, "check-sound-close")
            closed_early = time.monotonic()
            time.sleep(0.8)
            none_left = wait_for(None, lambda b: kitokey_streams(window) == [], 1.0)
            check("closed before its opening was counted: no stream of Kitokey's left within a second of the opening's answer",
                  none_left, [kitokey_streams(window), time.monotonic() - closed_early])
            browser.execute_script("window.restoreInvoke()")
            ended = WORK / "listen-ended"
            # A recording started elsewhere — the tray — while the panel shows: capture stops listening
            # within a second, before it records, and the panel goes.
            (WORK / "recorder-args").unlink(missing_ok=True)
            opened(browser)
            ended.unlink(missing_ok=True)
            asked = time.time()
            tray_click("Démarrer")
            started = wait_for(None, lambda b: (WORK / "recorder-args").is_file() and ended.is_file(), 5)
            check("a recording started from the tray while the panel shows: capture stopped listening within a second, before it records",
                  started and float(ended.read_text()) - asked <= 1.0 and float(ended.read_text()) <= (WORK / "recorder-args").stat().st_mtime,
                  [started, ended.is_file() and float(ended.read_text()) - asked])
            check("the panel goes with its listening", wait_for(browser, lambda b: b.execute_script("return document.getElementById('check-sound') === null"), 3))
            # Stopped past the half-second in which a second press is the same gesture (toggle_recording).
            time.sleep(0.7)
            browser.execute_script("window.__TAURI__.core.invoke('record')")
            stopped = wait_for(browser, lambda b: not b.execute_script("return window.__TAURI__.core.invoke('recording_now').then(n => n.recording)"), 8)
            check("the recording from the tray stopped", stopped)
            # A panel that stops asking — its page gone, as a crash leaves it —: the shell gives its listening up.
            # Opened once the stopped recording has let go: until then the panel is refused, as it must be.
            for _ in range(8):
                (WORK / "listen-args").unlink(missing_ok=True)
                opened(browser)
                if wait_for(None, lambda b: (WORK / "listen-args").is_file(), 2):
                    break
                press(browser, "check-sound-close")
                time.sleep(1)
            ended.unlink(missing_ok=True)
            browser.execute_script("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) =>"
                                   " asked === 'sound_check_now' ? new Promise(() => {}) : real.invoke(asked, ...rest) }; window.restoreInvoke = () => { window.__TAURI__.core = real; };")
            asked = time.time()
            given_up = wait_for(None, lambda b: ended.is_file(), 5)
            check("a panel that stops asking: its listening given up within 2.5 s", given_up and float(ended.read_text()) - asked <= 2.5,
                  ended.is_file() and float(ended.read_text()) - asked)
            browser.execute_script("window.restoreInvoke()")
            check("asking again, the panel finds nothing open and goes", wait_for(browser, lambda b: b.execute_script("return document.getElementById('check-sound') === null"), 3))
            # A recording started from the panel: the panel goes, capture stops listening before it records.
            (WORK / "recorder-args").unlink(missing_ok=True)
            opened(browser)
            press(browser, "check-record")
            started = wait_for(browser, lambda b: (WORK / "recorder-args").is_file(), 8)
            check("« Démarrer un enregistrement » from the panel starts the recording, the panel closed",
                  started and (WORK / "recorder-args").read_text().startswith("record ") and browser.execute_script("return document.getElementById('check-sound') === null"))
            check("capture stopped listening before it records", (WORK / "listen-ended").is_file())
            wait_for(browser, lambda b: b.execute_script("return window.__TAURI__.core.invoke('recording_now').then(n => n.recording)"), 5)
            refused = browser.execute_script("return window.__TAURI__.core.invoke('sound_check_open')")
            check("never listening while a recording runs", refused == {"opened": False}, refused)
            browser.execute_script("window.__TAURI__.core.invoke('record')")
            wait_for(browser, lambda b: not b.execute_script("return window.__TAURI__.core.invoke('recording_now').then(n => n.recording)"), 8)
        # Its window destroyed with the panel open — the full window replaced by the bar —: the shell
        # ends the listening on the window's end, within a second; the page can say nothing then. The
        # driver's window is the one destroyed: what it says of that is not this check's.
        ended = WORK / "listen-ended"
        asked, gone = None, False
        try:
            with Window(sessions, "fr_FR.UTF-8") as browser:
                browser.execute_script("location.hash = '#/settings'")
                wait_for(browser, lambda b: b.find_elements(By.ID, "check-sound-open"), 8)
                opened(browser)
                ended.unlink(missing_ok=True)
                asked = time.time()
                try:
                    browser.execute_script("window.__TAURI__.core.invoke('small_window')")
                except (NoSuchWindowException, WebDriverException):
                    pass
                gone = wait_for(None, lambda b: ended.is_file(), 3)
        except (NoSuchWindowException, WebDriverException):
            gone = gone or ended.is_file()
        check("its window destroyed with the panel open: capture stops listening within a second",
              asked is not None and gone and float(ended.read_text()) - asked <= 1.0, asked and ended.is_file() and float(ended.read_text()) - asked)
        # The bar hidden with the panel open in it (D90 opens it there): the page hidden closes the panel
        # under WebKitGTK (`visibilitychange`), and capture stops listening within a second.
        with Window(sessions, "fr_FR.UTF-8", small=True) as browser:
            wait_for(browser, lambda b: b.find_element(By.ID, "small-hide").is_displayed(), 8)
            browser.execute_script("checkSound(words, null)")
            wait_for(browser, lambda b: (b.execute_script(panel) or {}).get("me"), 5)
            ended = WORK / "listen-ended"
            ended.unlink(missing_ok=True)
            asked = time.time()
            # The panel is modal: the bar's own « Masquer la barre » is asked as its button asks it.
            browser.execute_script("window.__TAURI__.core.invoke('bar_hide')")
            gone = wait_for(None, lambda b: ended.is_file(), 3)
            check("the bar hidden with the panel open: capture stops listening within a second",
                  gone and float(ended.read_text()) - asked <= 1.0, ended.is_file() and float(ended.read_text()) - asked)
        flag("headset").unlink(missing_ok=True)
        with Window(sessions, "en_US.UTF-8") as browser:
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "check-sound-open"), 8)
            got = opened(browser)
            check("in English: the panel's words; no headset, no Bluetooth sentence",
                  got and got["title"] == "Check the sound" and "check-bluetooth" not in dict(got["shown"]), got)
            note = browser.execute_script("return document.querySelector('#check-sound .check-note').textContent")
            check("the microphone's note, in English", note == "The microphone stays open as long as this panel is.", note)
            press(browser, "check-sound-close")
    finally:
        subprocess.run(["pactl", "unload-module", mic], capture_output=True)


def bar_window(now: dt.datetime) -> None:
    """AY. The bar as a window (D90, board 8): no title bar, « Masquer la barre »; hidden during a
    recording, it never comes back by itself — the tray's « Afficher la barre » brings it back."""
    print("AY. the bar without a title bar; hidden while recording, it stays hidden whatever happens, the tray stops and brings it back (D90)")
    taping = WORK / "bar-hidden"
    taping.mkdir()
    hidden = "return document.visibilityState"
    with Window(taping, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True) as browser:
        wait_for(browser, lambda b: b.find_element(By.ID, "small-hide").is_displayed(), 8)
        got = browser.execute_script("const h = document.getElementById('small-hide'); return [h.getAttribute('aria-label'), h.title, h.getBoundingClientRect().height, h.textContent.trim(), outerHeight - innerHeight]")
        check("« Masquer la barre »: an icon of 28 px, named by its words; no title bar over the row", got[:4] == ["Masquer la barre", "Masquer la barre", 28, ""] and got[4] == 0, got)
        check("shown, the tray offers no « Afficher la barre »", "Afficher la barre" not in tray_labels(), tray_labels())
        press(browser, "small-record")
        wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed() and tray_colour() == "red", 10)
        press(browser, "small-hide")
        gone = wait_for(browser, lambda b: b.execute_script(hidden) == "hidden", 5)
        check("hidden during a recording: the bar goes, the recording goes on, the tray's icon red", gone and tray_colour() == "red", browser.execute_script(hidden))
        # After the recording's entry, as board 8 sets it.
        check("the tray offers « Afficher la barre », after « Arrêter l’enregistrement »",
              wait_for(None, lambda b: tray_labels()[1:3] == ["Arrêter l’enregistrement", "Afficher la barre"], 5), tray_labels())
        # Whatever happens but a gesture: a device changes, the microphone goes and comes back.
        (taping / "move").touch()
        (taping / "mic-gone").touch()
        time.sleep(1)
        (taping / "mic-back").write_text("Mic")
        time.sleep(2)
        check("a device changed, the microphone gone and back: the bar stays hidden", browser.execute_script(hidden) == "hidden", browser.execute_script(hidden))
        # The tray's stop, the gesture the shortcut shares (toggle_recording): stopped, the bar still hidden.
        tray_click("Arrêter l’enregistrement")
        stopped = wait_for(None, lambda b: tray_colour() == "blue", 15)
        time.sleep(1)
        check("stopped from the tray, as the shortcut stops: the bar does not come back by itself", stopped and browser.execute_script(hidden) == "hidden", browser.execute_script(hidden))
        tray_click("Afficher la barre")
        back = wait_for(browser, lambda b: b.execute_script(hidden) == "visible", 5)
        check("« Afficher la barre » brings it back, at rest, and leaves the menu", back and browser.find_element(By.ID, "small-record").is_displayed()
              and wait_for(None, lambda b: "Afficher la barre" not in tray_labels(), 5), tray_labels())
        # At rest too: hidden, « Ouvrir Kitokey » opens the window left, the bar.
        press(browser, "small-hide")
        wait_for(browser, lambda b: b.execute_script(hidden) == "hidden", 5)
        tray_click("Ouvrir Kitokey")
        check("hidden at rest, « Ouvrir Kitokey » shows it again", wait_for(browser, lambda b: b.execute_script(hidden) == "visible", 5), browser.execute_script(hidden))
        # Hidden, then closed by another path — the full window opened on a page, as a notification
        # does —: the tray offers no bar that is gone.
        press(browser, "small-hide")
        wait_for(browser, lambda b: b.execute_script(hidden) == "hidden", 5)
        wait_for(None, lambda b: "Afficher la barre" in tray_labels(), 5)
        browser.execute_script("window.__TAURI__.core.invoke('full_window', {page: '#/settings'})")
        check("hidden, then closed for the full window on a page: « Afficher la barre » leaves the menu",
              wait_for(None, lambda b: "Afficher la barre" not in tray_labels(), 5), tray_labels())


def bar_opened(now: dt.datetime) -> None:
    """BA. The bar opened (D90, board 6): at rest, the microphone, « Vérifier le son » — D88's panel in
    the bar — and « Avancé » closed; recording, the context, the microphone recorded, « Confidentielle »
    and « Avancé », « Vérifier le son » not offered: the meters are in the row."""
    print("BA. the bar opened: at rest the microphone, « Vérifier le son » opening its panel in the bar, « Avancé » closed; recording the context, the microphone recorded, « Confidentielle » — keyboard alone, both themes, French and English, nothing overflowing (D90)")
    listed = WORK / "microphones"
    listed.write_text(json.dumps({"event": "microphone", "name": "alsa_laptop", "description": "Digital Microphone"}) + "\n")
    mic = subprocess.run(["pactl", "load-module", "module-null-sink", "sink_name=kitokey_e2e_check_mic", "media.class=Audio/Source/Virtual"],
                         capture_output=True, text=True).stdout.strip()
    focused = "const a = document.activeElement; return a.id || a.className"
    # Every element of the opened bar inside its window, no text cut, nothing to scroll; the window as
    # tall as the bar shows (bar_fit).
    fits = ("const e = document.scrollingElement; const o = document.getElementById('small-options');"
            "const out = [...o.querySelectorAll('*')].filter(x => { const r = x.getBoundingClientRect(); return r.width > 0 && r.height > 0"
            " && (r.left < -0.5 || r.right > innerWidth + 0.5 || r.bottom > innerHeight + 0.5); }).map(x => x.id || x.className.baseVal || x.className);"
            "const cut = [...o.querySelectorAll('button, p, .name, .says')].filter(x => x.getClientRects().length && x.scrollWidth > x.clientWidth + 1).map(x => x.textContent);"
            "return {scroll: [e.scrollHeight, e.clientHeight, e.scrollWidth, e.clientWidth], out, cut, size: [innerWidth, innerHeight],"
            " page: Math.ceil(document.getElementById('small').getBoundingClientRect().height)}")
    panel = ("const d = document.getElementById('check-sound'); if (!d || !d.open) return null; const r = d.getBoundingClientRect();"
             "const cut = [...d.querySelectorAll('button, .check-said span, h2')].filter(x => x.getClientRects().length && (x.getBoundingClientRect().right > r.right + 0.5 || x.scrollWidth > x.clientWidth + 1)).map(x => x.textContent);"
             "return {inside: r.left >= -0.5 && r.top >= -0.5 && r.right <= innerWidth + 0.5 && r.bottom <= innerHeight + 0.5, cut, height: Math.ceil(r.height), size: [innerWidth, innerHeight], active: d.contains(document.activeElement)}")
    def keys(browser, *pressed) -> None:
        ActionChains(browser).send_keys(*pressed).perform()
        time.sleep(0.2)
    def back_tab(browser) -> None:
        ActionChains(browser).key_down(Keys.SHIFT).send_keys(Keys.TAB).key_up(Keys.SHIFT).perform()
        time.sleep(0.2)
    def tabbed(browser, count: int) -> list[str]:
        reached = []
        for _ in range(count):
            keys(browser, Keys.TAB)
            reached.append(browser.execute_script(focused))
        return reached
    def to_more(browser) -> bool:
        for _ in range(8):
            if browser.execute_script(focused) == "small-more":
                return True
            keys(browser, Keys.TAB)
        return browser.execute_script(focused) == "small-more"
    def answered_late(browser, late: dict) -> None:
        # The shell's answers to the commands named, given that many ms late (0 for none): the page's own calls
        # go through it; `__invoked` calls the shell at once.
        browser.execute_script(
            "if (!window.__invoked) { const t = window.__TAURI__; const core = t.core; window.__invoked = core.invoke.bind(core);"
            " const late = Object.assign(Object.create(Object.getPrototypeOf(core)), core, {invoke: (c, a, o) => window.__late[c]"
            " ? new Promise((r) => setTimeout(r, window.__late[c])).then(() => window.__invoked(c, a, o)) : window.__invoked(c, a, o)});"
            " const top = Object.assign({}, t, {core: late}); try { window.__TAURI__ = top; } catch (x) {}"
            " if (window.__TAURI__ !== top) Object.defineProperty(window, '__TAURI__', {configurable: true, writable: true, value: top}); }"
            " window.__late = arguments[0];", late)
    def unfold(browser) -> bool:
        # The options opened whatever a step before left: each check after holds on its own.
        if browser.execute_script("return document.getElementById('small-options').hidden"):
            browser.execute_script("document.getElementById('small-more').click()")
        return wait_for(browser, lambda b: any(e.is_displayed() for e in b.find_elements(By.ID, "small-check")), 3)
    def fit(browser, what: str) -> None:
        holds = lambda got: (got["scroll"][0] <= got["scroll"][1] and got["scroll"][2] <= got["scroll"][3] and not got["out"] and not got["cut"]
                             and got["size"][0] == 440 and abs(got["size"][1] - got["page"]) <= 1)
        wait_for(browser, lambda b: holds(b.execute_script(fits)), 3)
        got = browser.execute_script(fits)
        check(f"{what}: 440 wide, as tall as it shows, nothing overflows or is cut", holds(got), got)
    try:
        for language, (short, said) in {
            "fr_FR.UTF-8": ("fr", {"check": "Vérifier le son", "advanced": "Avancé — modèles", "models": "Pour cette réunion seule\u202f; les Réglages ne changent pas.",
                                   "settings": ["Modèle de transcription", "Modèle du compte rendu"], "recorded": "Enregistré : Mic. Le changer vaut pour le prochain enregistrement."}),
            "en_US.UTF-8": ("en", {"check": "Check the sound", "advanced": "Advanced — models", "models": "For this meeting alone; Settings do not change.",
                                   "settings": ["Transcription model", "Minutes model"], "recorded": "Recorded: Mic. Changing it applies to the next recording."}),
        }.items():
            for theme in ("light", "dark"):
                name = f"{short} {theme}"
                taping = WORK / f"bar-opened-{short}-{theme}"
                taping.mkdir()
                window = Window(taping, language, 'after = "nothing"\nreminder = false\n', small=True)
                if theme == "dark":
                    window.env.update(GTK_THEME="Adwaita:dark")
                with window as browser:
                    wait_for(browser, lambda b: b.execute_script(focused) == "small-record", 10)
                    # At rest, by the keyboard alone: « Plus d’options », Enter.
                    reached = to_more(browser)
                    keys(browser, Keys.ENTER)
                    opened = wait_for(browser, lambda b: b.execute_script("return innerHeight") > 44 and any(e.is_displayed() for e in b.find_elements(By.ID, "small-check")), 3)
                    check(f"{name}: at rest, « Plus d’options » reached by Tab opens the options under the row by Enter", reached and opened, browser.execute_script("return innerHeight"))
                    if not opened:
                        continue
                    order = tabbed(browser, 5)
                    check(f"{name}: at rest, Tab goes from the row to the microphone, « Vérifier le son », then « Avancé »",
                          order == ["small-full", "small-hide", "small-microphone", "small-check", "small-advanced"], order)
                    got = browser.execute_script("const c = document.getElementById('small-check'); const a = document.getElementById('small-advanced');"
                                                 " return [c.textContent, c.querySelectorAll('svg.glyph').length, a.textContent, a.getAttribute('aria-expanded'),"
                                                 " document.getElementById('small-advanced-panel').hidden, a.getAttribute('aria-controls')]")
                    check(f"{name}: « {said['check']} », labelled with its icon; « {said['advanced']} », closed",
                          got == [said["check"], 1, said["advanced"], "false", True, "small-advanced-panel"], got)
                    fit(browser, f"{name}: at rest, opened")
                    shot_themed(browser, f"d90-opened-rest-{short}-{theme}", theme)
                    # « Avancé » (D91): opened by Enter, the transcription's and the minutes' models, for this meeting alone; closed by Enter.
                    keys(browser, Keys.ENTER)
                    advanced = ("const p = document.getElementById('small-advanced-panel'); const a = document.getElementById('small-advanced');"
                                " return p && [a.getAttribute('aria-expanded'), p.hidden, [...p.querySelectorAll('.models-row .name')].map(n => n.textContent),"
                                " p.querySelector('.says').textContent, document.activeElement.id]")
                    wait_for(browser, lambda b: (b.execute_script(advanced) or [None, True])[1] is False and b.execute_script(advanced)[4] == "small-advanced", 3)
                    got = browser.execute_script(advanced)
                    check(f"{name}: « {said['advanced']} » opened by Enter offers the transcription's and the minutes' models, for this meeting alone; the focus kept on it",
                          got == ["true", False, said["settings"], said["models"], "small-advanced"], got)
                    fit(browser, f"{name}: « {said['advanced']} » open")
                    shot_themed(browser, f"d90-opened-advanced-{short}-{theme}", theme)
                    keys(browser, Keys.ENTER)
                    got = browser.execute_script("return [document.getElementById('small-advanced').getAttribute('aria-expanded'), document.getElementById('small-advanced-panel').hidden]")
                    check(f"{name}: Enter closes « {said['advanced']} »", got == ["false", True] and browser.execute_script(focused) == "small-advanced", got)
                    # « Vérifier le son » by the keyboard: D88's panel, in the bar, the bar as tall as it.
                    back_tab(browser)
                    (WORK / "listen-args").unlink(missing_ok=True)
                    on = browser.execute_script(focused)
                    keys(browser, Keys.ENTER)
                    shown = wait_for(browser, lambda b: b.execute_script(panel) and (WORK / "listen-args").is_file(), 5)
                    wait_for(browser, lambda b: (b.execute_script(panel) or {}).get("inside"), 3)
                    got = browser.execute_script(panel)
                    check(f"{name}: Enter on « {said['check']} » opens its panel, capture listening", on == "small-check" and shown, [on, got])
                    check(f"{name}: the panel inside the bar, the bar as tall as it, nothing cut, the focus inside",
                          got and got["inside"] and not got["cut"] and got["size"][0] == 440 and got["size"][1] >= got["height"] and got["active"], got)
                    shot_themed(browser, f"d90-opened-check-{short}-{theme}", theme)
                    keys(browser, Keys.ESCAPE)
                    back = wait_for(browser, lambda b: b.execute_script("return document.getElementById('check-sound') === null") and b.execute_script(focused) == "small-check", 3)
                    check(f"{name}: Escape closes the panel, the focus back on « {said['check']} »", back, browser.execute_script(focused))
                    fit(browser, f"{name}: the panel closed, the bar back to its options")
                    # × by Enter: the panel goes, capture stops listening, the bar back to its options.
                    keys(browser, Keys.ENTER)
                    wait_for(browser, lambda b: (b.execute_script(panel) or {}).get("inside"), 5)
                    for _ in range(8):
                        if browser.execute_script(focused) == "check-sound-close":
                            break
                        keys(browser, Keys.TAB)
                    ended = WORK / "listen-ended"
                    ended.unlink(missing_ok=True)
                    keys(browser, Keys.ENTER)
                    back = wait_for(browser, lambda b: b.execute_script("return document.getElementById('check-sound') === null") and b.execute_script(focused) == "small-check", 3)
                    check(f"{name}: × by Enter closes the panel, capture stops listening, the focus back on « {said['check']} »",
                          back and wait_for(None, lambda b: ended.is_file(), 3), [browser.execute_script(focused), ended.is_file()])
                    fit(browser, f"{name}: closed by ×, the bar back to its options")
                    keys(browser, Keys.ESCAPE)
                    folded = wait_for(browser, lambda b: b.execute_script("return innerHeight") == 44 and b.execute_script(focused) == "small-more", 3)
                    check(f"{name}: Escape closes the options, 44 high, the focus on « Plus d’options »", folded, [browser.execute_script("return innerHeight"), browser.execute_script(focused)])
                    # « Avancé » left open, a recording started from the panel by the keyboard: the panel goes,
                    # the focus follows to ■, « Avancé » is closed in the new state.
                    keys(browser, Keys.ENTER)
                    wait_for(browser, lambda b: any(e.is_displayed() for e in b.find_elements(By.ID, "small-check")), 3)
                    tabbed(browser, 5)
                    keys(browser, Keys.ENTER)
                    back_tab(browser)
                    keys(browser, Keys.ENTER)
                    wait_for(browser, lambda b: (b.execute_script(panel) or {}).get("inside"), 5)
                    for _ in range(8):
                        if browser.execute_script(focused) == "check-record":
                            break
                        keys(browser, Keys.TAB)
                    keys(browser, Keys.ENTER)
                    started = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed() and b.execute_script("return document.getElementById('check-sound') === null"), 8)
                    held = wait_for(browser, lambda b: b.execute_script(focused) == "small-stop", 3)
                    check(f"{name}: recording started from the panel by Enter: the panel gone, the focus on ■", started and held, browser.execute_script(focused))
                    got = browser.execute_script("return [document.getElementById('small-advanced').getAttribute('aria-expanded'), document.getElementById('small-advanced-panel').hidden]")
                    check(f"{name}: « {said['advanced']} », open at rest, is closed once recording", got == ["false", True], got)
                    reached = to_more(browser)
                    keys(browser, Keys.ENTER)
                    opened = wait_for(browser, lambda b: b.find_element(By.ID, "small-context").is_displayed(), 3)
                    check(f"{name}: recording, « Plus d’options » opened by Enter", reached and opened, browser.execute_script(focused))
                    order = tabbed(browser, 6)
                    check(f"{name}: recording, Tab goes to the context, the microphone, « Confidentielle », then « Avancé »",
                          order == ["small-full", "small-hide", "small-context", "small-microphone", "small-confidential", "small-advanced"], order)
                    got = browser.execute_script("return [document.getElementById('small-check').getClientRects().length, document.getElementById('small-microphone-next').textContent]")
                    check(f"{name}: recording, « {said['check']} » is not offered; the microphone says the one recorded, and that a change is for the next",
                          got == [0, said["recorded"]], got)
                    fit(browser, f"{name}: recording, opened")
                    shot_themed(browser, f"d90-opened-recording-{short}-{theme}", theme)
                    keys(browser, Keys.ENTER)
                    fit(browser, f"{name}: recording, « {said['advanced']} » open")
                    got = browser.execute_script(advanced)
                    check(f"{name}: recording, « {said['advanced']} » opened by Enter offers this meeting's two models", got and got[:3] == ["true", False, said["settings"]], got)
                    stop_from_small(browser)
                    wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 8)
                    got = browser.execute_script("return [document.getElementById('small-advanced').getAttribute('aria-expanded'), document.getElementById('small-advanced-panel').hidden]")
                    check(f"{name}: « {said['advanced']} », open while recording, is closed once at rest", got == ["false", True], got)
                    # Escape in the row folds the options too (board 5, #757): from « Plus d’options » itself,
                    # the focus kept there; from the context's field, its text and the focus kept.
                    reached = to_more(browser)
                    keys(browser, Keys.ENTER)
                    wait_for(browser, lambda b: any(e.is_displayed() for e in b.find_elements(By.ID, "small-check")), 3)
                    keys(browser, Keys.ESCAPE)
                    folded = wait_for(browser, lambda b: b.execute_script("return innerHeight") == 44 and b.execute_script(focused) == "small-more", 3)
                    check(f"{name}: Escape on « Plus d’options », opened, folds the options, the focus kept on it",
                          reached and folded, [browser.execute_script("return innerHeight"), browser.execute_script(focused)])
                    unfolded = unfold(browser)
                    browser.execute_script("document.getElementById('small-next-context').focus()")
                    keys(browser, "Point")
                    # Whether the bar took the Escape, read on the document once its own listener has run.
                    browser.execute_script("window.__escaped = []; document.addEventListener('keydown', (e) => { if (e.key === 'Escape') window.__escaped.push(e.defaultPrevented); })")
                    keys(browser, Keys.ESCAPE)
                    # Folded to the row and the line saying the context kept: no longer 44 high.
                    folded = wait_for(browser, lambda b: b.execute_script("return document.getElementById('small-options').hidden"), 3)
                    got = browser.execute_script("const f = document.getElementById('small-next-context'); return [f.value, document.activeElement === f, window.__escaped]")
                    check(f"{name}: Escape in the row's context folds the options, taken by the bar, its text and the focus kept", unfolded and folded and got == ["Point", True, [True]],
                          [unfolded, browser.execute_script("return document.getElementById('small-options').hidden"), got])
                    # The options closed, Escape in the row changes nothing and is left to whatever else hears it.
                    browser.execute_script("window.__escaped = []")
                    before = browser.execute_script("return [innerHeight, document.getElementById('small-options').hidden]")
                    keys(browser, Keys.ESCAPE)
                    got = browser.execute_script("const f = document.getElementById('small-next-context');"
                                                 " return [innerHeight, document.getElementById('small-options').hidden, f.value, document.activeElement === f, window.__escaped]")
                    check(f"{name}: the options closed, Escape in the row's context changes nothing, the key not taken", got == [*before, "Point", True, [False]], [before, got])
                    browser.execute_script("document.getElementById('small-context-clear').click()")
                    # A recording started elsewhere while « Vérifier le son » shows in the bar (#754): the panel
                    # goes, capture stops listening, the focus follows to ■ — whichever of the panel's close and the
                    # bar's tick ends first, each order forced by answering one of them late.
                    for late in ({}, {"sound_check_close": 400}, {"recording_now": 300}, {"recording_now": 300, "sound_check_close": 400}):
                        unfold(browser)
                        browser.execute_script("document.getElementById('small-check').focus()")
                        keys(browser, Keys.ENTER)
                        wait_for(browser, lambda b: (b.execute_script(panel) or {}).get("active"), 5)
                        ended = WORK / "listen-ended"
                        ended.unlink(missing_ok=True)
                        answered_late(browser, late)
                        # Half a second after the last press: a second within it is the same one (D68).
                        time.sleep(0.6)
                        browser.execute_script("window.__invoked('record')")
                        started = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed() and b.execute_script("return document.getElementById('check-sound') === null"), 8)
                        # Held a while: a focus given back late by the panel's close must not take it away.
                        time.sleep(1)
                        got = [browser.execute_script(focused), ended.is_file()]
                        answered_late(browser, {})
                        check(f"{name}: a recording started elsewhere while « {said['check']} » shows, {' and '.join(f'{k} {v} ms late' for k, v in late.items()) or 'nothing late'}:"
                              " the panel gone, capture no longer listening, the focus on ■", started and got == ["small-stop", True], [started, got])
                        stop_from_small(browser)
                        wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed() and b.execute_script(focused) == "small-record", 8)
                    # Stopped elsewhere — the tray —, the focus on a control of the options recording, hidden at rest:
                    # it follows to ●.
                    tray = {"fr": ("Démarrer", "Arrêter"), "en": ("Start", "Stop")}[short]
                    offered = lambda words: wait_for(None, lambda b: any(label.startswith(words) for label in tray_labels()), 5) and tray_click(words)
                    for where in ("small-confidential", "small-context"):
                        time.sleep(0.6)
                        started = offered(tray[0]) and wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 8)
                        browser.execute_script("document.getElementById('small-more').click()")
                        wait_for(browser, lambda b: b.find_element(By.ID, where).is_displayed(), 3)
                        browser.execute_script(f"document.getElementById('{where}').focus()")
                        on = browser.execute_script(focused)
                        # Where the focus went meanwhile, said if it fails.
                        browser.execute_script("window.__went = []; document.addEventListener('focusin', (e) => window.__went.push(e.target.id || e.target.tagName))")
                        time.sleep(0.6)
                        stopped = offered(tray[1]) and wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 8)
                        time.sleep(1)
                        check(f"{name}: the focus on {where} recording, stopped from the tray: at rest, the focus on ●",
                              started and on == where and stopped and browser.execute_script(focused) == "small-record",
                              [started, on, stopped, browser.execute_script(focused), browser.execute_script("return [window.__went, document.hasFocus()]")])
    finally:
        listed.unlink(missing_ok=True)
        subprocess.run(["pactl", "unload-module", mic], capture_output=True)


def context_before_start(now: dt.datetime) -> None:
    """BB. The context typed before the start (D91, board 5): in the bar's row at rest, kept for the
    next recording until the user clears it — its ✕, or emptied —, said under the row while kept; Enter
    in it starts the recording; the new meeting's context.txt once its session.json is there, under
    D59's rules; another meeting being made does not close it; recording, the shell takes none."""
    print("BB. the context before the start: in the row at rest, kept until cleared and said so, Enter starts, the next meeting's context.txt, open while another is made (D91)")
    model = WORK / "before-model.bin"
    model.write_bytes(b"\0" * 16)
    settings = f'after = "transcript"\nreminder = false\nmodel = "{model}"\n'
    focused = "const a = document.activeElement; return a.id || a.className"
    field = ("const a = document.getElementById('small-next-context'); const k = document.getElementById('small-context-kept');"
             "const x = document.getElementById('small-context-clear'); const seen = (e) => !!e && e.getClientRects().length > 0;"
             "if (!a) return {shown: false, in_row: false, value: null};"
             "return {value: a.value, placeholder: a.placeholder, name: a.getAttribute('aria-label'), closed: a.readOnly || a.disabled, shown: seen(a),"
             " in_row: !!a.closest('.bar-row'), kept: k ? k.textContent : null, kept_shown: seen(k), clear: seen(x), clear_name: x ? x.getAttribute('aria-label') : null,"
             " described: (a.getAttribute('aria-describedby') || '').split(' '), height: innerHeight}")
    fits = ("const e = document.scrollingElement; const p = document.getElementById('small');"
            "const out = [...p.querySelectorAll('*')].filter(x => { const r = x.getBoundingClientRect(); return r.width > 0 && r.height > 0"
            " && (r.left < -0.5 || r.right > innerWidth + 0.5 || r.bottom > innerHeight + 0.5); }).map(x => x.id || x.className.baseVal || x.className);"
            "const cut = [...p.querySelectorAll('.bar-row button, .bar-lines p')].filter(x => x.getClientRects().length && x.scrollWidth > x.clientWidth + 1).map(x => x.textContent);"
            "const f = document.getElementById('small-next-context').getBoundingClientRect();"
            "return {scroll: [e.scrollHeight, e.clientHeight, e.scrollWidth, e.clientWidth], out, cut, size: [innerWidth, innerHeight], field: Math.round(f.width),"
            " page: Math.ceil(p.getBoundingClientRect().height)}")

    def keys(browser, *pressed) -> None:
        ActionChains(browser).send_keys(*pressed).perform()
        time.sleep(0.2)

    def asked(browser, command: str, args: dict | None = None):
        """A command of the shell, as the page asks it: its answer, or « refused: » and why."""
        return browser.execute_async_script(
            "const done = arguments[arguments.length - 1];"
            "window.__TAURI__.core.invoke(arguments[0], arguments[1]).then(done, (why) => done('refused: ' + JSON.stringify(why)))",
            command, args or {})

    def kept(browser) -> str:
        return asked(browser, "next_context")["text"]

    def fit(browser, what: str, wide: int = 120) -> None:
        holds = lambda got: (got["scroll"][0] <= got["scroll"][1] and got["scroll"][2] <= got["scroll"][3] and not got["out"] and not got["cut"]
                             and got["size"][0] == 440 and abs(got["size"][1] - got["page"]) <= 1 and got["field"] >= wide)
        wait_for(browser, lambda b: holds(b.execute_script(fits)), 3)
        got = browser.execute_script(fits)
        check(f"{what}: 440 wide, the field at least {wide} px, as tall as it shows, nothing overflows or is cut", holds(got), got)

    def tab_round(browser, count: int) -> list[str]:
        reached = []
        for _ in range(count):
            keys(browser, Keys.TAB)
            reached.append(browser.execute_script(focused))
        return reached

    said = "Budget du T4 avec Inès Durand et l’équipe achats."
    then = "Point d’étape avec Siobhán O’Neill."
    meetings = WORK / "before-fr-light"
    meetings.mkdir()
    meeting = meetings / "2026-10-18_12-00"
    with Window(meetings, "fr_FR.UTF-8", settings, small=True) as browser:
        wait_for(browser, lambda b: b.execute_script(focused) == "small-record", 10)
        wait_for(browser, lambda b: b.execute_script("return innerHeight") == 44, 3)
        got = browser.execute_script(field)
        check("fr light: at rest, the context's field is in the row, empty, named, open; no ✕, no sentence, the bar 44 high",
              got["in_row"] and got["shown"] and got["value"] == "" and got["placeholder"] == "Contexte de la réunion"
              and got["name"] == "De quoi parle la réunion ? Qui y participe ?" and not got["closed"]
              and not got["clear"] and not got["kept_shown"] and got["height"] == 44, got)
        if not got["shown"]:
            return
        fit(browser, "fr light: at rest, the field empty")
        order = tab_round(browser, 5)
        check("fr light: Tab goes round the row — options, whole window, hide, the context, then ● — as board 5 orders it from the context",
              order == ["small-more", "small-full", "small-hide", "small-next-context", "small-record"], order)
        back_tab = lambda b: ActionChains(b).key_down(Keys.SHIFT).send_keys(Keys.TAB).key_up(Keys.SHIFT).perform()
        back_tab(browser)
        time.sleep(0.2)
        keys(browser, said)
        got = browser.execute_script(field)
        check("fr light: typed at rest, the ✕ is offered, named, and under the row it is said kept for the next recording until cleared",
              got["value"] == said and got["clear"] and got["clear_name"] == "Effacer le contexte" and got["kept_shown"]
              and got["kept"] == "Gardé pour le prochain enregistrement, jusqu’à ce que vous l’effaciez." and "small-context-kept" in got["described"], got)
        held = wait_for(browser, lambda b: kept(b) == said, 3)
        check("fr light: typed at rest, the shell keeps it for the next recording; nothing is written anywhere yet",
              held and not any(meetings.iterdir()), [kept(browser), sorted(p.name for p in meetings.iterdir())])
        fit(browser, "fr light: at rest, a context kept, its sentence under the row")
        check("fr light: kept, the bar grows by its line", browser.execute_script("return innerHeight") > 44, browser.execute_script("return innerHeight"))
        time.sleep(0.3)
        shot_themed(browser, "d91-context-rest-fr-light", "light")
        order = tab_round(browser, 2)
        check("fr light: filled, Tab goes from the context to its ✕, then ●", order == ["small-context-clear", "small-record"], order)
        # One field: the options at rest hold none of their own.
        press(browser, "small-more")
        wait_for(browser, lambda b: any(e.is_displayed() for e in b.find_elements(By.ID, "small-check")), 3)
        got = browser.execute_script("return [document.getElementById('small-context').getClientRects().length, document.getElementById('small-next-context').value]")
        check("fr light: the options opened at rest hold no second field; the row's keeps the context", got == [0, said], got)
        press(browser, "small-more")
        # Kept by the shell, not by the page: the bar drawn again shows it.
        browser.refresh()
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 10)
        drawn = wait_for(browser, lambda b: b.execute_script(field)["value"] == said and b.execute_script(field)["kept_shown"], 3)
        check("fr light: the bar drawn again, the context kept is shown, and said", drawn, browser.execute_script(field))
        # Over D59's 400 characters, the shell refuses it, what was kept stays.
        refused = asked(browser, "next_context_kept", {"text": "é" * 401})
        check("fr light: over 400 characters, the shell refuses it and keeps what it had",
              str(refused).startswith("refused:") and kept(browser) == said, [refused, kept(browser)])
        # Enter in the field starts the recording: the new meeting's context.txt, whole, beside its session.json.
        browser.find_element(By.ID, "small-next-context").click()
        keys(browser, Keys.END, Keys.ENTER)
        started = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
        written = wait_for(None, lambda b: (meeting / "context.txt").is_file() and (meeting / "context.txt").read_text() == said, 5)
        check("fr light: Enter in the field starts the recording; the context typed before is the new meeting's context.txt, whole, beside its session.json, nothing else",
              started and written and sorted(p.name for p in meeting.iterdir()) == ["context.txt", "session.json"],
              sorted(p.name for p in meeting.iterdir()) if meeting.exists() else None)
        got = browser.execute_script(field)
        check("fr light: recording, the row holds no field, nothing is said kept", not got["shown"] and not got["kept_shown"] and not got["clear"], got)
        press(browser, "small-more")
        shown = wait_for(browser, lambda b: b.execute_script("return document.getElementById('small-context').value") == said, 3)
        check("fr light: recording, the options' field is this meeting's and shows it", shown, browser.execute_script("return document.getElementById('small-context').value"))
        check("fr light: taken by this meeting, nothing is kept for the next", kept(browser) == "", kept(browser))
        # Recording, a context for the next is refused — a key typed as the tray starts one would land in the meeting after.
        late = asked(browser, "next_context_kept", {"text": "Tapé trop tard"})
        check("fr light: recording, the shell takes no context for the next, and what it kept stays",
              str(late).startswith("refused:") and kept(browser) == "", [late, kept(browser)])
        # Stopped: its transcript is made, slowly — another meeting being made —, the field open for the next.
        (WORK / "slow-hearing").touch()
        (WORK / "hearing-args").unlink(missing_ok=True)
        try:
            # Past the half-second in which the shell takes a second press for the same gesture.
            time.sleep(0.6)
            stop_from_small(browser)
            wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10)
            making = wait_for(None, lambda b: (WORK / "hearing-args").is_file(), 5)
            got = browser.execute_script(field)
            check("fr light: back at rest, the row's field is empty — the next meeting's own — while the one before is made",
                  making and got["shown"] and got["value"] == "" and not got["kept_shown"] and not (meeting / "transcript.jsonl").exists(), [making, got])
            browser.find_element(By.ID, "small-next-context").click()
            keys(browser, then)
            got = browser.execute_script(field)
            still = not (meeting / "transcript.jsonl").exists()
            check("fr light: another meeting being made does not close the field: typed, kept for the next",
                  still and not got["closed"] and got["value"] == then and wait_for(browser, lambda b: kept(b) == then, 3), [still, got])
            heard = wait_for(None, lambda b: (meeting / "transcript.jsonl").is_file(), 15)
            header = json.loads((meeting / "transcript.jsonl").read_text().splitlines()[0]) if heard else {}
            check("fr light: the meeting's transcript heard the context typed before its start", header.get("context") == said, header)
        finally:
            (WORK / "slow-hearing").unlink(missing_ok=True)
        # The next recording, started from the tray, takes what was kept.
        meeting.rename(meetings / "2026-10-18_11-00")
        tray_click("Démarrer")
        started = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
        written = wait_for(None, lambda b: (meeting / "context.txt").is_file() and (meeting / "context.txt").read_text() == then, 5)
        check("fr light: the next recording, started from the tray, takes the context kept", started and written,
              (meeting / "context.txt").read_text() if (meeting / "context.txt").is_file() else None)
        time.sleep(0.6)
        stop_from_small(browser)
        wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10)
        wait_for(None, lambda b: (meeting / "transcript.jsonl").is_file(), 15)
        # Cleared by its ✕: nothing is kept for the next, the sentence gone, the bar 44 high, the focus in the field.
        browser.find_element(By.ID, "small-next-context").click()
        keys(browser, "Pour la suite")
        wait_for(browser, lambda b: kept(b) == "Pour la suite", 3)
        press(browser, "small-context-clear")
        cleared = wait_for(browser, lambda b: kept(b) == "" and b.execute_script("return innerHeight") == 44, 3)
        got = browser.execute_script(field)
        check("fr light: cleared by its ✕, nothing is kept, nothing said, 44 high, the focus in the field",
              cleared and got["value"] == "" and not got["kept_shown"] and not got["clear"] and browser.execute_script(focused) == "small-next-context", [kept(browser), got])
        # Emptied by the keyboard too.
        keys(browser, "Encore")
        wait_for(browser, lambda b: kept(b) == "Encore", 3)
        ActionChains(browser).key_down(Keys.CONTROL).send_keys("a").key_up(Keys.CONTROL).send_keys(Keys.BACKSPACE).perform()
        emptied = wait_for(browser, lambda b: kept(b) == "" and not b.execute_script(field)["kept_shown"], 3)
        check("fr light: emptied by the keyboard, nothing is kept", emptied, [kept(browser), browser.execute_script(field)])

    meetings = WORK / "before-en-dark"
    meetings.mkdir()
    window = Window(meetings, "en_US.UTF-8", settings, small=True)
    window.env.update(GTK_THEME="Adwaita:dark")
    typed = "Q4 budget with Inès Durand and the purchasing team."
    with window as browser:
        wait_for(browser, lambda b: b.execute_script(focused) == "small-record", 10)
        browser.find_element(By.ID, "small-next-context").click()
        keys(browser, typed)
        got = browser.execute_script(field)
        check("en dark: at rest, the row's field, its ✕, and the sentence kept for the next recording until cleared",
              got["placeholder"] == "Meeting context" and got["name"] == "What is the meeting about? Who takes part?" and got["clear_name"] == "Clear the context"
              and got["kept"] == "Kept for the next recording, until you clear it." and got["kept_shown"] and got["clear"] and not got["closed"], got)
        fit(browser, "en dark: at rest, a context kept")
        time.sleep(0.3)
        shot_themed(browser, "d91-context-rest-en-dark", "dark")
        wait_for(browser, lambda b: kept(b) == typed, 3)
    # Kitokey quit and started again: the context typed before the start is still kept for the next.
    with Window(meetings, "en_US.UTF-8", settings, small=True) as browser:
        wait_for(browser, lambda b: b.execute_script(focused) == "small-record", 10)
        again = wait_for(browser, lambda b: b.execute_script(field)["value"] == typed and b.execute_script(field)["kept_shown"], 5)
        check("en: Kitokey started again, the context typed before the start is still kept, and said", again, browser.execute_script(field))


def meeting_named(now: dt.datetime) -> None:
    """AL. A meeting named by its minutes (D48): the title in the row and the page, as text."""
    print("AL. a meeting takes the title its minutes give it — the date beneath it in the list, above it on the page; as text; made again, the new one (D48)")
    meetings = WORK / "named"
    def written(folder: Path, contract: str, title: str | None) -> None:
        named = f"title: {json.dumps(title)}\n" if title is not None else ""
        (folder / "minutes.md").write_text(
            f"---\ncontract: {contract}\nmade: 2026-10-28T15:00:00+01:00\nprovider: local\nmodel: m\ncontext: null\n{named}---\n\n"
            "## 1.\n\nLe budget.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    long = "Lancement du produit — calendrier, budget, recrutement, et tout ce qui reste à fixer"[:80]
    named, marked, old, without = [meetings / f"2026-10-2{i}_14-00" for i in (8, 7, 6, 5)]
    for i, folder in enumerate((named, marked, old, without)):
        session(folder, now - dt.timedelta(days=1 + i), 10, {"local.wav": 1000})
    written(named, "minutes/v2", long)
    written(marked, "minutes/v2", "<img src=x onerror=\"window.named_ran=1\">")
    written(old, "minutes/v1", None)
    rows_said = ("return [...document.querySelectorAll('#groups .session')].map(r => ["
                 "r.querySelector('.day').textContent, r.querySelector('.day').title, r.querySelector('.on')?.textContent ?? null])")
    with Window(meetings, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: len(b.find_elements(By.CSS_SELECTOR, "#groups .session")) == 4, 10)
        said = browser.execute_script(rows_said)
        check("the title in its row, whole in its tooltip, the date beneath it", said[0][0] == long and said[0][1] == long and bool(said[0][2]) and "·" in said[0][2], said)
        check("minutes made before, or none: the date as today, nothing beneath",
              all("·" in row[0] and row[2] is None for row in said[2:]), said)
        check("markup in a title is shown as text, never as markup",
              said[1][0] == "<img src=x onerror=\"window.named_ran=1\">"
              and not browser.execute_script("return document.querySelector('#groups img') || window.named_ran"), said[1])
        fits = browser.execute_script(
            "const d = document.querySelector('#groups .session .day'); const r = d.closest('.session').getBoundingClientRect();"
            "return [getComputedStyle(d).whiteSpace, getComputedStyle(d).textOverflow, d.getBoundingClientRect().height < 30, d.getBoundingClientRect().right <= r.right]")
        check("a long title on one line, cut by an ellipsis, inside its row", fits == ["nowrap", "ellipsis", True, True], fits)
        browser.execute_script("const f = document.getElementById('find'); f.value = 'recrutement'; f.dispatchEvent(new Event('input'))")
        found = wait_for(browser, lambda b: b.execute_script("return [...document.querySelectorAll('#groups li')].filter(l => !l.hidden).length") == 1, 5)
        check("the search finds a meeting by its title", found)
        browser.execute_script("const f = document.getElementById('find'); f.value = ''; f.dispatchEvent(new Event('input'))")
        head = "return [...document.querySelectorAll('#meeting .named > *')].map(e => [e.tagName, e.className, e.textContent])"
        # From the title's last letter to its length and weight, and their bottoms (D89, board 4): the
        # title's own text, not its box, which may be as wide as the date above it.
        beside_title = ("const w = document.createTreeWalker(document.querySelector('#meeting h1'), NodeFilter.SHOW_TEXT); let t = null; while (w.nextNode()) if (w.currentNode.textContent.trim()) t = w.currentNode;"
                        " const r = document.createRange(); r.selectNodeContents(t); const rs = [...r.getClientRects()], h = rs[rs.length - 1], l = document.querySelector('#meeting .lasts').getBoundingClientRect();"
                        " return [Math.round(l.left - h.right), Math.round(l.bottom - h.bottom)]")
        # The page left goes at once: nothing there until the page asked for is drawn.
        at_head = lambda b, line, part=2: (found[line][part] if len(found := b.execute_script(head)) > line else "")
        browser.execute_script(f"location.hash = '#/meeting/{named.name}'")
        page = wait_for(browser, lambda b: at_head(b, 1, 0) == "H1" and at_head(b, 1, 2) == long, 10)
        got = browser.execute_script(head)
        check("on its page: the date above, the title its heading", page and got[0][0] != "H1" and "·" in got[0][2], got)
        # D89, board 4: how long and how heavy beside the title, on its line — never on the date's above
        # it; read where the title holds on one line beside them, 1920 wide.
        was = browser.get_window_size()
        browser.set_window_size(1920, 1080)
        wait_for(browser, lambda b: b.execute_script("return innerWidth") == 1920, 5)
        beside = browser.execute_script(beside_title)
        check("on its page: its length and weight beside the title, on the title's line (D89)", 0 < beside[0] <= 24 and abs(beside[1]) < 6, beside)
        browser.set_window_size(was["width"], was["height"])
        shot(browser, "d48-named")
        browser.execute_script(f"location.hash = '#/meeting/{marked.name}'")
        as_text = wait_for(browser, lambda b: at_head(b, 1, 2).startswith("<img"), 10)
        written(marked, "minutes/v2", "W" * 80)
        browser.execute_script("location.hash = '#/'")
        browser.execute_script(f"location.hash = '#/meeting/{marked.name}'")
        wide = wait_for(browser, lambda b: at_head(b, 1, 2) == "W" * 80, 10)
        check("a long title with no space does not widen the page",
              wide and browser.execute_script("const e = document.scrollingElement; return e.scrollWidth <= e.clientWidth"),
              browser.execute_script("return [document.scrollingElement.scrollWidth, document.scrollingElement.clientWidth]"))
        written(marked, "minutes/v2", "<img src=x onerror=\"window.named_ran=1\">")
        check("markup in a title is text on the page too",
              as_text and not browser.execute_script("return document.querySelector('#meeting .named img') || window.named_ran"), browser.execute_script(head))
        browser.execute_script(f"location.hash = '#/meeting/{old.name}'")
        dated = wait_for(browser, lambda b: at_head(b, 0, 0) == "H1" and "·" in at_head(b, 0, 2), 10)
        check("minutes made before: the date is the heading, as today", dated, browser.execute_script(head))
        # Made again: the new title, read from the file as the list reads again.
        written(named, "minutes/v2", "Un nouveau nom")
        browser.execute_script(f"location.hash = '#/meeting/{named.name}'")
        again = wait_for(browser, lambda b: at_head(b, 1, 2) == "Un nouveau nom", 10)
        check("the minutes made again: the meeting takes the new title", again, browser.execute_script(head))
        retitled = wait_for(browser, lambda b: b.execute_script(rows_said)[0][0] == "Un nouveau nom", 10)
        check("its row too", retitled, browser.execute_script(rows_said)[0])
        # A title shorter than its date: its length and weight after its last letter all the same.
        written(named, "minutes/v2", "Point")
        browser.execute_script("location.hash = '#/'")
        browser.execute_script(f"location.hash = '#/meeting/{named.name}'")
        short = wait_for(browser, lambda b: at_head(b, 1, 2) == "Point", 10)
        beside = browser.execute_script(beside_title)
        check("a title shorter than its date: its length and weight just after it, on its line (D89)", short and 0 < beside[0] <= 24 and abs(beside[1]) < 6, beside)


def minutes_checked(now: dt.datetime) -> None:
    """AU. Minutes of `minutes/v3`: what the call cost on the maker's line alone (D51), and a line's
    time that opens its passage and plays its second (D75); Copy without the times."""
    print("AU. minutes/v3: the cost on the maker's line, nowhere else (D51); a time opens its passage and plays its second, keyboard too; none copied (D75)")
    meetings = WORK / "checking"
    played, freed, unplayable, bare, older, unpriced, secret, dear, early, hollow = (
        meetings / f"2026-10-2{n}_09-00" for n in range(10))
    head = '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":"fr","context":null}\n'
    # A long meeting: a line every two seconds, its passages far below the fold; one that starts within its second.
    said = "".join(
        json.dumps({"type": "segment", "speaker": "me", "track": "local", "voice": None, "start": n * 2 + (0.6 if n == 55 else 0.0),
                    "end": n * 2 + 1.5, "text": f"Ligne numéro {n}, inventée pour cet essai."}, ensure_ascii=False) + "\n"
        for n in range(60))
    transcript = head + said + '{"type":"complete","duration":120.0}\n'
    front = ("---\ncontract: minutes/{version}\nmade: 2026-10-13T12:21:00+02:00\nprovider: openrouter.ai\nmodel: anthropic/claude-sonnet\ncontext: null\n{more}---\n\n")
    body = ("## 1.\n\nLe budget est validé.\n\n"
            "## 2.\n\n- Mettre à jour le planning — Marc — aujourd’hui [01:40]\n- Couper un serveur — non précisé — non précisé\n- Relancer le fournisseur — Paul [01:50]\n\n"
            "## 3.\n\n- Un serveur est gardé. [01:10]\n- La salle reste à trouver [12:75]\n")
    priced = "cost: 0.01234\ncurrency: USD\n"
    for folder in (played, unplayable, freed, bare, older, unpriced, secret, dear, early, hollow):
        session(folder, now - dt.timedelta(days=2), 2, {})
    for folder in (played, unpriced, secret):
        with wave.open(str(folder / "local.wav"), "wb") as made:
            made.setnchannels(2)
            made.setsampwidth(3)
            made.setframerate(48000)
            made.writeframes(bytes(6 * 48000 * 120))
        (folder / "clock.jsonl").write_text('{"track":"local","sample":0,"at":0.0}\n')
        record = json.loads((folder / "session.json").read_text())
        record["tracks"] = {"local": {"device": "Mic"}}
        (folder / "session.json").write_text(json.dumps(record))
    # A track the window cannot play: there, and not playable.
    (unplayable / "local.wav").write_bytes(b"not a sound")
    for folder in (played, freed, unplayable, older, unpriced, secret):
        (folder / "transcript.jsonl").write_text(transcript)
    (secret / "confidential").write_bytes(b"")
    for folder in (played, freed, unplayable, bare, secret):
        (folder / "minutes.md").write_text(front.format(version="v3", more=priced) + body)
    (unpriced / "minutes.md").write_text(front.format(version="v3", more="cost: -1\ncurrency: USD\n") + body)
    (older / "minutes.md").write_text(front.format(version="v2", more=priced) + body)
    (dear / "minutes.md").write_text(front.format(version="v3", more="cost: 1e30\ncurrency: EUR\n") + body)
    # A time before the first line, that line already in view; nothing heard, and a time past the hour.
    (early / "transcript.jsonl").write_text(head + "".join(line for line in said.splitlines(True)[5:]) + '{"type":"complete","duration":120.0}\n')
    (early / "minutes.md").write_text(front.format(version="v3", more="") + "## 1.\n\nRien.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Un début [00:05]\n")
    (hollow / "transcript.jsonl").write_text(head + '{"type":"complete","duration":0.0}\n')
    (hollow / "minutes.md").write_text(front.format(version="v3", more="") + "## 1.\n\nRien.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Le long [65:00]\n")
    open_ = lambda b, folder: (b.execute_script(f"location.hash = '#/meeting/{folder.name}'"), wait_for(
        b, lambda c: c.execute_script("return document.getElementById('meeting').dataset.name") == folder.name
        and c.find_elements(By.CSS_SELECTOR, "#meeting .minutes .body h2"), 10))
    maker = "return document.querySelector('#meeting .minutes .maker').textContent"
    links = "return [...document.querySelectorAll('#meeting .minutes button.line-time')]"
    labels = ("return [...document.querySelectorAll('#meeting .minutes button.line-time')].map(l => [l.textContent, l.getAttribute('aria-label')])")
    row_text = "return [...document.querySelectorAll('#meeting .minutes table.actions tbody tr')].map(r => [...r.cells].map(c => c.textContent.trim()))"
    audio = "const a = document.querySelector('audio.track'); return a ? [a.currentTime, a.paused] : null"
    in_view = ("const l = [...document.querySelectorAll('#meeting .lines li')].find(l => l.textContent.includes(`numéro ${arguments[0]},`));"
               "const r = l.getBoundingClientRect(); return !l.hidden && r.height > 0 && r.top >= 0 && r.bottom <= innerHeight")
    marked = "return document.querySelector('#meeting .lines li.now')?.textContent ?? ''"
    # Where the page stands, whichever element of it scrolls.
    scrolled = "return [...document.querySelectorAll('*')].reduce((sum, e) => sum + e.scrollTop, 0)"
    away = "[...document.querySelectorAll('*')].forEach(e => e.scrollTop = 0)"
    stand_in = ("window.__copied = [];Object.defineProperty(navigator, 'clipboard', {configurable: true, value: {writeText: (text) =>"
                " (window.__copied.push(text), Promise.resolve())}})")
    overflows = ("const wide = [...document.querySelectorAll('#meeting .minutes, #meeting .minutes *')].filter(e => {"
                 "const r = e.getBoundingClientRect(); return !e.closest('th') && r.width > 0 && r.right > innerWidth + 0.5}).map(e => e.tagName + '.' + e.className);"
                 "const page = document.scrollingElement; return [page.scrollWidth - page.clientWidth, wide]")

    for language, short, cost, listen, passage, sections in (
        ("fr_FR.UTF-8", "fr", "0,012 $US", "Écouter depuis", "Voir le passage de", ("Points saillants", "Actions", "Points clés")),
        ("en_US.UTF-8", "en", "$0.012", "Listen from", "See the passage at", ("Highlights", "Actions", "Key points")),
    ):
        with Window(meetings, language, "reminder = false\n") as browser:
            browser.set_window_size(1100, 800)
            open_(browser, played)
            wait_for(browser, lambda b: b.find_elements(By.ID, "play"), 10)
            got = browser.execute_script(maker)
            check(f"{short}: the maker's line ends with the cost, in the window's language, its currency, two significant digits", got.endswith(f" · {cost}"), got)
            colours = browser.execute_script(
                "const m = getComputedStyle(document.querySelector('#meeting .minutes .maker')).color, p = document.createElement('p');"
                "p.style.color = 'var(--soft)'; document.body.append(p); const s = getComputedStyle(p).color; p.remove(); return [m, s]")
            check(f"{short}: in the soft colour", colours[0] == colours[1], colours)
            got = browser.execute_script(row_text)
            check(f"{short}: an action's row keeps its three cells, the time beside what, out of by when (D46)",
                  got[0] == ["Marc", "Mettre à jour le planning 01:40", "aujourd’hui"]
                  and got[1] == ["non précisé", "Couper un serveur", "non précisé"]
                  and got[2] == ["Relancer le fournisseur — Paul 01:50"], got)
            check(f"{short}: an item that does not split in three keeps its time in its place",
                  browser.execute_script("return document.querySelector('#meeting table.actions td.written button.line-time')?.textContent") == "01:50", got)
            got = browser.execute_script(labels)
            check(f"{short}: each time a link that says what it does; a time out of bounds stays the item's text",
                  got == [[at, f"{listen} {at}"] for at in ("01:40", "01:50", "01:10")]
                  and "[12:75]" in browser.find_element(By.CSS_SELECTOR, "#meeting .minutes .body").text
                  and "[01:" not in browser.find_element(By.CSS_SELECTOR, "#meeting .minutes .body").text, got)
            told = browser.execute_script("return [...document.querySelectorAll('#meeting .minutes button.line-time')].map(l => l.title === l.getAttribute('aria-label') && l.title !== '')")
            check(f"{short}: its words also said to a pointer resting on it", told == [True, True, True], told)
            quiet = browser.execute_script(
                "const l = document.querySelector('#meeting .minutes button.line-time'), s = getComputedStyle(l),"
                " b = getComputedStyle(document.querySelector('#meeting .minutes .body li')); return [parseFloat(s.fontSize) < parseFloat(b.fontSize), s.textDecorationLine, s.backgroundColor]")
            check(f"{short}: small, quiet, and a link by more than its colour", quiet[0] and quiet[1] == "underline" and quiet[2] == "rgba(0, 0, 0, 0)", quiet)
            check(f"{short}: the passages are below the fold before a click", not browser.execute_script(in_view, 50))
            browser.execute_script(links)[0].click()
            shown = wait_for(browser, lambda b: b.execute_script(in_view, 50), 3)
            playing = wait_for(browser, lambda b: (a := b.execute_script(audio)) and not a[1] and 100.0 <= a[0] < 102.0, 8)
            check(f"{short}: a click brings its passage into view and plays from that second", shown and playing, browser.execute_script(audio))
            check(f"{short}: the line being said is marked", "numéro 50," in browser.execute_script(marked) or "numéro 51," in browser.execute_script(marked), browser.execute_script(marked))
            # D26: left where the user puts it, while it plays on across lines.
            browser.execute_script(away)
            at = browser.execute_script(scrolled)
            time.sleep(2.5)
            check(f"{short}: the page does not follow what plays (D26)", browser.execute_script(scrolled) == at and not browser.execute_script(audio)[1],
                  (at, browser.execute_script(scrolled)))
            # By the keyboard alone: Enter, then Space, the focus staying on the link.
            third = browser.execute_script(links)[2]
            browser.execute_script("arguments[0].focus()", third)
            browser.switch_to.active_element.send_keys(Keys.ENTER)
            keyed = wait_for(browser, lambda b: b.execute_script(in_view, 35) and (a := b.execute_script(audio)) and 70.0 <= a[0] < 72.0 and not a[1], 8)
            check(f"{short}: Enter on a time does the same", keyed, browser.execute_script(audio))
            check(f"{short}: the focus stays on the link", browser.execute_script("return document.activeElement.textContent") == "01:10",
                  browser.execute_script("return document.activeElement.outerHTML"))
            outline = browser.execute_script("const a = document.activeElement; return [a.matches(':focus-visible'), getComputedStyle(a).outlineStyle]")
            check(f"{short}: reached by the keyboard, it shows its focus", outline == [True, "solid"], outline)
            # A failure said earlier is not left beside a sound that now plays.
            browser.execute_script("document.getElementById('not-played').textContent = 'dit plus tôt'")
            browser.execute_script(links)[1].send_keys(" ")
            spaced = wait_for(browser, lambda b: b.execute_script(in_view, 55) and (a := b.execute_script(audio)) and 110.0 <= a[0] < 112.0 and not a[1], 8)
            check(f"{short}: Space too, without pausing what plays; a line starting within that second is its passage", spaced, browser.execute_script(audio))
            check(f"{short}: what a failure said before is gone",
                  browser.execute_script("return document.getElementById('not-played').textContent") == "", browser.execute_script("return document.getElementById('not-played').textContent"))
            stop = browser.execute_script("return document.querySelector('#meeting .lines li[tabindex=\"0\"]').textContent")
            check(f"{short}: Tab goes on from its passage, the line starting at 110.6 s for a time of 110", "numéro 55," in stop, stop)
            # A search that hides the passage gives way to it.
            search = browser.find_element(By.ID, "search")
            browser.execute_script("arguments[0].scrollIntoView({block: 'nearest'})", search)
            search.send_keys("numéro 3,")
            wait_for(browser, lambda b: not b.execute_script(in_view, 50))
            browser.execute_script(links)[0].click()
            check(f"{short}: a passage a search hid is shown, the search given up",
                  wait_for(browser, lambda b: b.execute_script(in_view, 50), 3) and browser.find_element(By.ID, "search").get_attribute("value") == "",
                  browser.find_element(By.ID, "search").get_attribute("value"))
            browser.execute_script("document.querySelector('audio.track').pause()")
            # Copy: D45's body, the times out, every other bracket as written.
            browser.execute_script(stand_in)
            press(browser, "copy-minutes")
            wait_for(browser, lambda b: b.execute_script("return window.__copied.length") == 1, 3)
            expected = (f"## {sections[0]}\n\nLe budget est validé.\n\n## {sections[1]}\n\n- Mettre à jour le planning — Marc — aujourd’hui\n"
                        f"- Couper un serveur — non précisé — non précisé\n- Relancer le fournisseur — Paul\n\n## {sections[2]}\n\n- Un serveur est gardé.\n- La salle reste à trouver [12:75]\n")
            copied = browser.execute_script("return window.__copied")
            check(f"{short}: Copy leaves the times out, D45's body otherwise as it was", copied == [expected], copied)
            # Both widths, both themes: nothing overflows, nothing cut.
            for width, height in ((420, 800), (1100, 800)):
                browser.set_window_size(width, height)
                time.sleep(0.4)
                got = browser.execute_script(overflows)
                check(f"{short}: at {width} px nothing overflows", got[0] <= 0 and got[1] == [], got)
                for theme in ("light", "dark"):
                    browser.execute_script("document.querySelector('#meeting .minutes').scrollIntoView()")
                    shot_themed(browser, f"d75-{short}-{width}-{theme}", theme)

            # The audio freed: the transcript still answers; what is said never promises a sound.
            for folder, why, note in ((freed, "freed", True), (unplayable, "not playable", True)):
                open_(browser, folder)
                wait_for(browser, lambda b: b.find_elements(By.ID, "no-player") and b.find_elements(By.CSS_SELECTOR, "#meeting .lines li"), 10)
                got = browser.execute_script(labels)
                check(f"{short}: its audio {why}: a time brings its passage into view, and says that alone",
                      got == [[at, f"{passage} {at}"] for at in ("01:40", "01:50", "01:10")], got)
                browser.execute_script(away)
                browser.execute_script(links)[0].click()
                shown = wait_for(browser, lambda b: b.execute_script(in_view, 50), 3)
                check(f"{short}: its audio {why}: the passage shown and marked, nothing plays, the page's own sentence kept",
                      shown and "numéro 50," in browser.execute_script(marked) and browser.execute_script(audio) is None
                      and browser.find_element(By.ID, "no-player").is_displayed(), browser.execute_script(marked))
                browser.execute_script(links)[2].click()
                wait_for(browser, lambda b: b.execute_script(in_view, 35), 3)
                check(f"{short}: one passage marked at a time", browser.execute_script(
                    "return document.querySelectorAll('#meeting .lines li.now').length") == 1 and "numéro 35," in browser.execute_script(marked), browser.execute_script(marked))
                browser.execute_script(links)[1].click()
                check(f"{short}: its audio {why}: a time of 110 marks the line starting at 110.6 s, not the one before",
                      wait_for(browser, lambda b: "numéro 55," in b.execute_script(marked), 3), browser.execute_script(marked))
            shot_themed(browser, f"d75-{short}-freed", "dark")
            # Nothing to play, nothing to open: the time as plain text, no link that leads nowhere.
            open_(browser, bare)
            times = browser.execute_script("return [...document.querySelectorAll('#meeting .minutes .body time')].map(t => [t.textContent, t.closest('button') === null, t.tabIndex])")
            check(f"{short}: no audio, no transcript: each time is quiet text, no link", browser.execute_script(labels) == []
                  and times == [["01:40", True, -1], ["01:50", True, -1], ["01:10", True, -1]], times)
            got = browser.execute_script(row_text)
            check(f"{short}: and its row keeps its three cells", got[0] == ["Marc", "Mettre à jour le planning 01:40", "aujourd’hui"], got)
            # A transcript that holds no line: nothing to open, no link; the time as the page writes it.
            open_(browser, hollow)
            wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting .minutes .body time"), 5)
            times = browser.execute_script("return [...document.querySelectorAll('#meeting .minutes .body time')].map(t => t.textContent)")
            check(f"{short}: a transcript with no line: no link, and past the hour the time reads as the transcript writes it",
                  browser.execute_script(labels) == [] and times == ["01:05:00"], times)
            # Before the first line, that line in view: it is the passage, and the page does not move.
            open_(browser, early)
            wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting .lines li"), 10)
            # In view low in the pane, far from its centre: the link is out of view, so clicked by the page.
            browser.execute_script(
                "document.querySelector('#meeting .lines li').scrollIntoView({block: 'end'});"
                "const pane = [...document.querySelectorAll('*')].find(e => e.scrollTop > 0 && e.contains(document.getElementById('meeting'))); pane.scrollTop += 40")
            at = browser.execute_script(scrolled)
            check(f"{short}: the first line in view before a click", browser.execute_script(in_view, 5) and at > 0, at)
            browser.execute_script("arguments[0].click()", browser.execute_script(links)[0])
            first = wait_for(browser, lambda b: "numéro 5," in b.execute_script(marked), 3)
            time.sleep(0.5)
            check(f"{short}: a time before the first line opens that line; in view already, the page stays where it is",
                  first and browser.execute_script(scrolled) == at, (browser.execute_script(marked), at, browser.execute_script(scrolled)))
            # Older minutes: exactly as today.
            open_(browser, older)
            wait_for(browser, lambda b: b.find_elements(By.ID, "play") or b.find_elements(By.ID, "no-player"), 10)
            got = browser.execute_script(maker)
            check(f"{short}: minutes/v2 with the same keys: no cost, no time, their brackets their text",
                  not got.endswith(cost) and " · " in got and browser.execute_script(labels) == []
                  and not browser.find_elements(By.CSS_SELECTOR, "#meeting .minutes .body time")
                  and "[01:10]" in browser.find_element(By.CSS_SELECTOR, "#meeting .minutes .body").text, got)
            # A cost out of the contract's bounds: the line as it is without one, the times still links.
            open_(browser, unpriced)
            wait_for(browser, lambda b: b.find_elements(By.ID, "play"), 10)
            got = browser.execute_script(maker)
            check(f"{short}: a cost out of bounds: the line as it is without one", got.count(" · ") == 1, got)
            # Any number the contract allows stays inside the card, whatever its length.
            open_(browser, dear)
            browser.set_window_size(420, 800)
            time.sleep(0.4)
            got = browser.execute_script(overflows)
            check(f"{short}: a cost of thirty digits at 420 px: nothing overflows", got[0] <= 0 and got[1] == [], got)
            browser.set_window_size(1100, 800)
            # Said confidential: the same page; nothing here sends anything.
            open_(browser, secret)
            wait_for(browser, lambda b: b.find_elements(By.ID, "play"), 10)
            browser.execute_script(links)[0].click()
            plays = wait_for(browser, lambda b: (a := b.execute_script(audio)) and not a[1] and 100.0 <= a[0] < 102.0, 8)
            check(f"{short}: a meeting said confidential: its time plays here too, its lock still said",
                  plays and browser.execute_script("return document.getElementById('confidential-switch').checked") is True, browser.execute_script(audio))
            browser.execute_script("document.querySelector('audio.track').pause()")

    # Said confidential, services named in the settings: a time played and the minutes copied send nothing.
    counted = {"hear": 0, "write": 0}

    class Service(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_POST(self):
            counted[self.path.strip("/")] += 1
            self.rfile.read(int(self.headers.get("content-length") or 0))
            self.send_response(200)
            self.send_header("content-length", "0")
            self.end_headers()

        def log_message(self, *_):
            pass

    # An Ollama that holds a model: a step started here on this page's account would run, and be seen.
    class Ollama(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_GET(self):
            answer = json.dumps({"models": [{"name": "qwen3:8b"}]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(answer)))
            self.end_headers()
            self.wfile.write(answer)

        def log_message(self, *_):
            pass

    server, here = (http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) for handler in (Service, Ollama))
    for running in (server, here):
        threading.Thread(target=running.serve_forever, daemon=True).start()
    services = ('reminder = false\nengine = "service"\ntranscription_provider = "https://hears.example/v1"\ntranscription_model = "whisper-1"\n'
                'transcription_agreed = "hears.example"\nminutes_provider = "https://writes.example/v1"\nminutes_model = "gpt-4.1-mini"\nminutes_agreed = "writes.example"\n')
    hearing, writing = WORK / "hearing-args", WORK / "summary-args"
    hearing.unlink(missing_ok=True)
    writing.unlink(missing_ok=True)
    before = {name: (secret / name).read_bytes() for name in ("minutes.md", "transcript.jsonl")}
    try:
        window = Window(meetings, "fr_FR.UTF-8", services, ollama=f"http://127.0.0.1:{here.server_port}")
        window.env.update(STANDIN=f"http://127.0.0.1:{server.server_port}")
        with window as browser:
            browser.set_window_size(1100, 800)
            open_(browser, secret)
            wait_for(browser, lambda b: b.find_elements(By.ID, "play"), 10)
            browser.execute_script(links)[0].click()
            plays = wait_for(browser, lambda b: (a := b.execute_script(audio)) and not a[1] and 100.0 <= a[0] < 102.0, 8)
            browser.execute_script(stand_in)
            press(browser, "copy-minutes")
            copied = wait_for(browser, lambda b: b.execute_script("return window.__copied.length") == 1, 3)
            time.sleep(2)
            check("a meeting said confidential: its time plays, its minutes copy, and nothing is sent: no service asked, neither step started, its files as they were",
                  plays and copied and counted == {"hear": 0, "write": 0} and not hearing.exists() and not writing.exists()
                  and all((secret / name).read_bytes() == kept for name, kept in before.items()), (plays, copied, counted, hearing.exists(), writing.exists()))
    finally:
        server.shutdown()
        here.shutdown()

    # D51's fourth: neither the agreement nor the small window says a price.
    priced_words = ("return /0[,.]01|\\$|\\bUSD\\b|coût|\\bcost\\b|\\bprix\\b|\\bprice\\b/i.test(document.getElementById(arguments[0]).innerText)")
    online = 'reminder = false\nafter = "nothing"\nminutes_provider = "https://writes.example/v1"\nminutes_model = "gpt-4.1-mini"\n'
    with Window(meetings, "fr_FR.UTF-8", online) as browser:
        browser.execute_script(f"location.hash = '#/agree/{played.name}'")
        asked = wait_for(browser, lambda b: b.find_elements(By.ID, "agree-send"), 10)
        check("the agreement before a first sending shows no price", asked and not browser.execute_script(priced_words, "agree"),
              browser.find_element(By.ID, "agree").text)
    with Window(meetings, "fr_FR.UTF-8", online, small=True) as browser:
        check("the bar at rest shows no price",
              browser.find_elements(By.ID, "small-record") and not browser.execute_script(priced_words, "small"), browser.find_element(By.ID, "small").text)
        press(browser, "small-record")
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        check("nor while recording, no running count", not browser.execute_script(priced_words, "small"), browser.find_element(By.ID, "small").text)
        stop_from_small(browser)


def word_after_a_gap(now: dt.datetime) -> None:
    """AT. A track with a gap: a line clicked after it plays that line, by the clock's next point (D71)."""
    print("AT. after a gap in a track, a line clicked plays its own second; the track as it was (D71)")
    meetings = WORK / "gapped"
    played = meetings / "2026-10-19_10-00"
    session(played, now - dt.timedelta(hours=1), 0.2, {})
    record = json.loads((played / "session.json").read_text())
    record["tracks"] = {"local": {"device": "Mic", "changed": [{"at": 4.0, "device": None}, {"at": 7.0, "device": "Mic"}]}}
    (played / "session.json").write_text(json.dumps(record))
    # Stopped at 4 s; zeros for 2 s of the 3 the clock leaves; a note at 8 s of the clock, frame 336 000.
    with wave.open(str(played / "local.wav"), "wb") as made:
        made.setnchannels(2)
        made.setsampwidth(3)
        made.setframerate(48000)
        frames = bytearray()
        for n in range(480000):
            sample = int(0.3 * 8388607 * math.sin(2 * math.pi * 880 * n / 48000)) if 336000 <= n < 384000 else 0
            frames += struct.pack("<i", sample)[:3] * 2
        made.writeframes(bytes(frames))
    (played / "clock.jsonl").write_text(
        '{"track":"local","sample":0,"at":0.0}\n{"track":"local","silence":[192000,288000]}\n{"track":"local","sample":288000,"at":7.0}\n')
    (played / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":"fr","context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","start":1.0,"end":2.0,"text":"Avant le trou."}\n'
        '{"type":"segment","speaker":"me","track":"local","start":8.0,"end":9.0,"text":"Après le trou."}\n{"type":"complete","duration":11.0}\n')
    before = hashlib.sha256((played / "local.wav").read_bytes()).hexdigest()
    with Window(meetings, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.set_window_size(1100, 800)
        browser.execute_script(f"location.hash = '#/meeting/{played.name}'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, ".lines li") and b.find_elements(By.ID, "play"), 10)
        browser.find_element(By.ID, "play").click()
        wait_for(browser, lambda b: b.execute_script("const a = document.querySelector('audio.track'); return !!a && !a.paused"), 10)
        browser.find_elements(By.CSS_SELECTOR, ".lines li")[1].click()
        at = lambda b: b.execute_script("return document.querySelector('audio.track').currentTime")
        jumped = wait_for(browser, lambda b: 8.0 <= at(b) < 8.6 and b.execute_script("return document.querySelector('.lines li.now')?.dataset.start") == "8", 5)
        check("the line after the gap: played from its second, and marked", jumped, at(browser))
        url = browser.execute_script("return document.querySelector('audio.track').src")
        def loud(second: float) -> float:
            start = 44 + 2 * int(second * 48000)
            asked = urllib.request.Request(url, headers={"Range": f"bytes={start}-{start + 2 * 4800 - 1}"})
            with urllib.request.urlopen(asked, timeout=10) as answered:
                body = answered.read()
            values = struct.unpack(f"<{len(body) // 2}h", body)
            return max(abs(v) for v in values) / 32768
        check("what the player is given at that second is that line's sound", loud(8.05) > 0.2, loud(8.05))
        check("and within 0.2 s of it: silence just before, sound just after", loud(7.7) < 0.01 and loud(8.1) > 0.2, (loud(7.7), loud(8.1)))
        check("before the gap, the same place as ever: nothing at 2 s", loud(2.0) < 0.01, loud(2.0))
    check("the track is unchanged (ADR-0004)", hashlib.sha256((played / "local.wav").read_bytes()).hexdigest() == before)


def meeting_renamed(now: dt.datetime) -> None:
    """AS. A meeting renamed by hand (D70): the heading a field, the name everywhere at once, as text."""
    print("AS. a meeting's heading clicked, a name typed: the list, the page and Libérer de l'espace show it; Escape leaves it; emptied, the minutes' title or the date comes back (D70)")
    meetings = WORK / "renaming"
    with_minutes, bare = meetings / "2026-10-28_14-00", meetings / "2026-10-27_14-00"
    session(with_minutes, now - dt.timedelta(days=1), 10, {"local.wav": 1000})
    session(bare, now - dt.timedelta(days=40), 10, {"local.wav": 1000})
    def written(title: str) -> None:
        (with_minutes / "minutes.md").write_text(
            f"---\ncontract: minutes/v2\nmade: 2026-10-28T15:00:00+01:00\nprovider: local\nmodel: m\ncontext: null\ntitle: {json.dumps(title)}\n---\n\n"
            "## 1.\n\nLe budget.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    written("Du compte rendu")
    given = with_minutes / "title.txt"
    head = lambda b: b.execute_script("return document.querySelector('#meeting h1')?.textContent")
    row = lambda b, folder: b.execute_script(
        "const r = [...document.querySelectorAll('#groups .session')].find(r => r.getAttribute('href').endsWith(arguments[0])); return r.querySelector('.day').textContent", folder.name)
    # A save draws the page again a moment after the heading changes: pressed and typed through what is there then.
    def opened(b) -> None:
        press(b, "rename")
        wait_for(b, lambda c: c.execute_script("const f = document.getElementById('naming'); if (f && document.activeElement === f) { f.select(); return true; } return false"), 5)
    typed = lambda b, *keys: ActionChains(b).send_keys(*keys).perform()
    def name(b, text: str, key: str) -> None:
        opened(b)
        typed(b, (text or Keys.DELETE) + key)
    with Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n') as browser:
        browser.set_window_size(1100, 800)
        browser.execute_script(f"location.hash = '#/meeting/{with_minutes.name}'")
        wait_for(browser, lambda b: head(b) == "Du compte rendu", 10)
        name(browser, "Point budget Q4", Keys.ENTER)
        check("Enter: the page shows it at once", wait_for(browser, lambda b: head(b) == "Point budget Q4", 5), head(browser))
        check("the list too", wait_for(browser, lambda b: row(b, with_minutes) == "Point budget Q4", 5), row(browser, with_minutes))
        check("kept as one line in title.txt", given.read_text() == "Point budget Q4\n", given.read_text())
        check("the focus is back on the heading", browser.execute_script("return document.activeElement.id") == "rename")
        check("the folder keeps its name", with_minutes.is_dir() and (with_minutes / "session.json").is_file())
        name(browser, "Autre chose", Keys.ESCAPE)
        check("Escape: as it was", wait_for(browser, lambda b: head(b) == "Point budget Q4", 5) and given.read_text() == "Point budget Q4\n", head(browser))
        check("Escape: the focus is back on the heading", browser.execute_script("return document.activeElement.id") == "rename")
        # A reading of the page while typing, the window coming forward: what is typed stays.
        opened(browser)
        typed(browser, "Tapé en cours")
        browser.execute_async_script("const done = arguments[arguments.length - 1]; document.getElementById('meeting').dataset.at = '#/other'; refresh().then(done)")
        kept = browser.execute_script("const f = document.getElementById('naming'); return f && f.value === 'Tapé en cours' && document.activeElement === f")
        check("the page read again keeps what is typed, and the focus", kept)
        # The window losing the focus, the field keeping it: nothing is kept yet.
        browser.execute_script("document.getElementById('naming').dispatchEvent(new Event('blur'))")
        time.sleep(0.3)
        # Where the caret was, the page read again.
        browser.execute_script("document.getElementById('naming').setSelectionRange(2, 4)")
        browser.execute_async_script("const done = arguments[arguments.length - 1]; document.getElementById('meeting').dataset.at = '#/other'; refresh().then(done)")
        check("the page read again keeps the caret", browser.execute_script("const f = document.getElementById('naming'); return [f.selectionStart, f.selectionEnd]") == [2, 4])
        check("another window come forward keeps nothing yet", bool(browser.find_elements(By.ID, "naming")) and given.read_text() == "Point budget Q4\n")
        typed(browser, Keys.ESCAPE)
        # The same name, spaces around it: nothing written. Enter twice: saved once.
        stamp = given.stat().st_mtime_ns
        opened(browser)
        typed(browser, "  Point budget Q4  " + Keys.ENTER)
        wait_for(browser, lambda b: b.find_elements(By.ID, "rename"), 5)
        check("the same name with spaces round it writes nothing", given.stat().st_mtime_ns == stamp)
        browser.execute_script("const real = window.__TAURI__.core; window.__saves = 0; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => { if (asked === 'title_saved') window.__saves++; return real.invoke(asked, ...rest); } }; window.restore = () => { window.__TAURI__.core = real; };")
        opened(browser)
        browser.execute_script("const f = document.getElementById('naming'); f.value = 'Une fois'; for (let i = 0; i < 2; i++) f.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));")
        wait_for(browser, lambda b: head(b) == "Une fois", 5)
        check("Enter twice saves once", browser.execute_script("return window.__saves") == 1, browser.execute_script("return window.__saves"))
        browser.execute_script("window.restore()")
        name(browser, "Point budget Q4", Keys.ENTER)
        wait_for(browser, lambda b: head(b) == "Point budget Q4", 5)
        # A reading of the page while a save is on its way: the field is not opened again over it.
        browser.execute_script("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'title_saved' ? new Promise((d) => setTimeout(d, 800)).then(() => real.invoke(asked, ...rest)) : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        opened(browser)
        typed(browser, "Lent" + Keys.ENTER)
        time.sleep(0.2)
        browser.execute_script("document.getElementById('meeting').dataset.at = '#/other'; refresh()")
        wait_for(browser, lambda b: head(b) == "Lent", 5)
        check("a page read during a save: no field left open after it", not browser.find_elements(By.ID, "naming"))
        browser.execute_script("window.restore()")
        name(browser, "Point budget Q4", Keys.ENTER)
        wait_for(browser, lambda b: head(b) == "Point budget Q4", 5)
        # Made again: it stays. Emptied: the minutes' title.
        written("Refait")
        browser.execute_script("refresh()")
        time.sleep(0.5)
        check("the minutes made again: the name stays", head(browser) == "Point budget Q4", head(browser))
        check("the minutes made again leave the file alone", given.read_text() == "Point budget Q4\n")
        name(browser, "", Keys.ENTER)
        check("emptied: the minutes' title comes back", wait_for(browser, lambda b: head(b) == "Refait", 5), head(browser))
        check("emptied: no file left", not given.exists() and not (with_minutes / ".title.part").exists())
        # Unchanged: the minutes' title is not frozen into a name of the user's.
        opened(browser)
        typed(browser, Keys.ENTER)
        check("Enter on the minutes' title unchanged writes nothing", wait_for(browser, lambda b: b.find_elements(By.ID, "rename"), 5) and not given.exists())
        # Leaving the field keeps it.
        opened(browser)
        typed(browser, "En partant")
        browser.find_element(By.ID, "find").click()
        check("leaving the field keeps it", wait_for(browser, lambda b: head(b) == "En partant", 5), head(browser))
        # As text, a path no path.
        mark = "<img src=x onerror=\"window.renamed_ran=1\">"
        name(browser, mark, Keys.ENTER)
        shown = wait_for(browser, lambda b: head(b) == mark and row(b, with_minutes) == mark, 5)
        check("markup is shown as text, on the page and in the list", shown
              and not browser.execute_script("return document.querySelector('#meeting .named img') || document.querySelector('#groups img') || window.renamed_ran"), head(browser))
        browser.execute_script("location.hash = '#/free'")
        freed = wait_for(browser, lambda b: b.execute_script("return [...document.querySelectorAll('#free .freeable .name')].some(n => n.textContent === arguments[0])", mark), 10)
        check("markup is text in Libérer de l'espace too", freed and not browser.execute_script("return document.querySelector('#free img') || window.renamed_ran"))
        browser.execute_script(f"location.hash = '#/meeting/{with_minutes.name}'")
        wait_for(browser, lambda b: head(b) == mark and b.find_elements(By.ID, "rename"), 10)
        name(browser, "../../hors-du-dossier", Keys.ENTER)
        check("a path is a name: shown as text", wait_for(browser, lambda b: head(b) == "../../hors-du-dossier", 5), head(browser))
        check("a path: no folder moved, none made",
              sorted(p.name for p in meetings.iterdir()) == [bare.name, with_minutes.name] and given.read_text() == "../../hors-du-dossier\n", sorted(p.name for p in meetings.iterdir()))
        # No minutes, no name: the date; a name; emptied: the date.
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: "·" in (head(b) or ""), 10)
        dated = head(browser)
        name(browser, "W" * 90, Keys.ENTER)
        check("80 characters at most, the field says so by stopping", wait_for(browser, lambda b: head(b) == "W" * 80, 5), head(browser))
        # A heading that fits stays on one line: the button adds nothing to its width.
        name(browser, "Revue d'équipe", Keys.ENTER)
        wait_for(browser, lambda b: head(b) == "Revue d'équipe", 5)
        one = browser.execute_script("const b = document.getElementById('rename'), r = document.createRange(); r.selectNodeContents(b); return r.getClientRects().length")
        check("a short name stays on one line", one == 1, one)
        name(browser, "W" * 90, Keys.ENTER)
        wait_for(browser, lambda b: head(b) == "W" * 80, 5)
        check("the longest name does not widen the page", browser.execute_script("const e = document.scrollingElement; return e.scrollWidth <= e.clientWidth"))
        shot(browser, "d70-named")
        opened(browser)
        shot(browser, "d70-naming")
        typed(browser, "Revue d'équipe" + Keys.ENTER)
        wait_for(browser, lambda b: head(b) == "Revue d'équipe", 5)
        # Libérer de l'espace: the name, the date beside it.
        browser.execute_script("location.hash = '#/free'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#free .freeable .name"), 10)
        freed = browser.execute_script("return [...document.querySelectorAll('#free .freeable')].map(r => [r.querySelector('.name').textContent, r.querySelector('.length').textContent])")
        check("Libérer de l'espace names it, its date beside it", any(n == "Revue d'équipe" and "·" in l for n, l in freed), freed)
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == "Revue d'équipe", 10)
        name(browser, "", Keys.ENTER)
        check("emptied, no minutes: the date comes back", wait_for(browser, lambda b: head(b) == dated, 5), head(browser))
        # Refused: said, the field still open, what was typed kept.
        browser.execute_script("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'title_saved' ? Promise.reject('REFUS') : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        name(browser, "Refusé", Keys.ENTER)
        said = wait_for(browser, lambda b: b.find_elements(By.ID, "naming-said") and b.find_element(By.ID, "naming-said").is_displayed(), 5)
        check("a name refused is said, in French", said and browser.find_element(By.ID, "naming-said").get_attribute("textContent") == "Le nom n’a pas été enregistré\u00a0: REFUS")
        check("refused: the field stays, what was typed in it", browser.find_element(By.ID, "naming").get_attribute("value") == "Refusé")
        # A reading of the page keeps the sentence with the field, and leaves the focus where it is.
        browser.execute_async_script("const done = arguments[arguments.length - 1]; document.getElementById('meeting').dataset.at = '#/other'; refresh().then(done)")
        check("the page read again keeps the refusal", browser.find_element(By.ID, "naming-said").is_displayed() and browser.find_element(By.ID, "naming").get_attribute("value") == "Refusé")
        typed(browser, Keys.ESCAPE)
        # Refused, the user gone elsewhere: the page read again neither brings them back nor tries again.
        opened(browser)
        typed(browser, "Refusé ailleurs")
        browser.execute_script("window.__refused = 0; const was = window.__TAURI__.core; window.__TAURI__.core = { ...was, invoke: (asked, ...rest) => { if (asked === 'title_saved') window.__refused++; return was.invoke(asked, ...rest); } };")
        browser.find_element(By.ID, "find").click()
        wait_for(browser, lambda b: b.find_element(By.ID, "naming-said").is_displayed(), 5)
        for _ in range(2):
            browser.execute_async_script("const done = arguments[arguments.length - 1]; document.getElementById('meeting').dataset.at = '#/other'; refresh().then(done)")
        time.sleep(0.2)
        check("refused, gone elsewhere: the page read again leaves the focus there", browser.execute_script("return document.activeElement.id") == "find", browser.execute_script("return document.activeElement.id"))
        check("refused, gone elsewhere: nothing tried again without a gesture", browser.execute_script("return window.__refused") == 1, browser.execute_script("return window.__refused"))
        # The address left: each way out says it, where it stays.
        press(browser, "settings-link")
        told = wait_for(browser, lambda b: b.find_element(By.ID, "pane-said").is_displayed() and "REFUS" in b.find_element(By.ID, "pane-said").get_attribute("textContent"), 5)
        check("refused, then Réglages: said in the pane", told, browser.find_element(By.ID, "pane-said").get_attribute("textContent"))
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == dated, 10)
        # The answer coming after the page was left: said there too.
        browser.execute_script("const r = window.__TAURI__.core; window.__TAURI__.core = { ...r, invoke: (asked, ...rest) => asked === 'title_saved' ? new Promise((_, no) => setTimeout(() => no('LENT'), 500)) : r.invoke(asked, ...rest) };")
        opened(browser)
        typed(browser, "Refusé plus tard" + Keys.ENTER)
        browser.execute_script("location.hash = '#/settings'")
        told = wait_for(browser, lambda b: "LENT" in b.find_element(By.ID, "pane-said").get_attribute("textContent") and b.find_element(By.ID, "pane-said").is_displayed(), 5)
        check("refused after the page was left: said in the pane", told, browser.find_element(By.ID, "pane-said").get_attribute("textContent"))
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == dated, 10)
        browser.execute_script("window.restore(); const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'title_saved' ? Promise.reject('REFUS') : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        # Refused, then Escape: no sentence for the next address. Refused, then a slower try that saves: none either.
        opened(browser)
        typed(browser, "Refusé" + Keys.ENTER)
        wait_for(browser, lambda b: b.find_element(By.ID, "naming-said").is_displayed(), 5)
        typed(browser, Keys.ESCAPE)
        press(browser, "settings-link")
        wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 5)
        check("refused, then Escape: nothing said at the next address", not browser.find_element(By.ID, "pane-said").is_displayed())
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == dated, 10)
        opened(browser)
        typed(browser, "Refusé" + Keys.ENTER)
        wait_for(browser, lambda b: b.find_element(By.ID, "naming-said").is_displayed(), 5)
        browser.execute_script("window.restore(); const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'title_saved' ? new Promise((d) => setTimeout(d, 400)).then(() => real.invoke(asked, ...rest)) : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        browser.execute_script("document.getElementById('naming').blur(); location.hash = '#/settings'")
        wait_for(browser, lambda b: (bare / "title.txt").exists(), 5)
        time.sleep(0.3)
        check("refused, then saved by a slower try: no sentence says otherwise", not browser.find_element(By.ID, "pane-said").is_displayed(), browser.find_element(By.ID, "pane-said").get_attribute("textContent"))
        (bare / "title.txt").unlink()
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == dated, 10)
        browser.execute_script("window.restore(); const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'title_saved' ? Promise.reject('REFUS') : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        # Refused while leaving for another meeting: said in the pane, never to no one.
        opened(browser)
        typed(browser, "Refusé en partant")
        browser.execute_script(f"document.getElementById('naming').blur(); location.hash = '#/meeting/{with_minutes.name}'")
        told = wait_for(browser, lambda b: b.find_element(By.ID, "pane-said").is_displayed() and "REFUS" in b.find_element(By.ID, "pane-said").get_attribute("textContent"), 5)
        check("refused on the way to another meeting: said in the pane", told, browser.find_element(By.ID, "pane-said").get_attribute("textContent"))
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == dated, 10)
        # Refused, then the folder gone: the address is replaced, no hashchange, and it is said all the same.
        gone = meetings / "2026-10-20_14-00"
        session(gone, now - dt.timedelta(days=45), 10, {"local.wav": 1000})
        browser.execute_async_script("const done = arguments[arguments.length - 1]; refresh().then(done)")
        browser.execute_script(f"location.hash = '#/meeting/{gone.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "rename") and b.execute_script("return document.getElementById('meeting').dataset.name") == gone.name, 10)
        opened(browser)
        typed(browser, "Refusé" + Keys.ENTER)
        wait_for(browser, lambda b: b.find_element(By.ID, "naming-said").is_displayed(), 5)
        shutil.rmtree(gone)
        browser.execute_async_script("const done = arguments[arguments.length - 1]; refresh().then(done)")
        told = wait_for(browser, lambda b: b.find_element(By.ID, "pane-said").is_displayed() and "REFUS" in b.find_element(By.ID, "pane-said").get_attribute("textContent"), 5)
        check("refused, then its folder gone: said in the pane", told, browser.find_element(By.ID, "pane-said").get_attribute("textContent"))
        browser.execute_script(f"location.hash = '#/meeting/{bare.name}'")
        wait_for(browser, lambda b: head(b) == dated, 10)
        check("then the next address says nothing", not browser.find_element(By.ID, "pane-said").is_displayed())
        browser.execute_script("window.restore()")
        # Marks that show nothing, as a paste gives them: no name, and nothing to apologise for.
        name(browser, "Provisoire", Keys.ENTER)
        wait_for(browser, lambda b: head(b) == "Provisoire", 5)
        opened(browser)
        browser.execute_script("document.getElementById('naming').value = '\\u200b\\u2060'")
        typed(browser, Keys.ENTER)
        check("marks that show nothing: the name goes, the date back, nothing refused", wait_for(browser, lambda b: head(b) == dated, 5) and not (bare / "title.txt").exists() and not browser.find_elements(By.ID, "naming-said"), head(browser))
        opened(browser)
        typed(browser, "Refusé")
        typed(browser, Keys.ESCAPE)
        check("Escape: the sentence goes, the date as it was", not browser.find_elements(By.ID, "naming-said") and head(browser) == dated, head(browser))
        # The meeting being recorded can be named too.
        live = lambda b: "live" in (b.find_element(By.ID, "record").get_attribute("class") or "")
        recording = meetings / "2026-10-18_12-00"
        browser.find_element(By.ID, "record").click()
        wait_for(browser, lambda b: (recording / "session.json").exists() and live(b), 10)
        # D89: while it records, the header's name says it — the dot of its i red and 1.375 times larger, the cut following.
        dot = ("const d = document.getElementById('name-dot'), g = document.getElementById('name-gap');"
               " return [getComputedStyle(d).fill === getComputedStyle(document.getElementById('record')).backgroundColor,"
               " Math.abs(d.getAttribute('r') - d.dataset.rest * 1.375) <= 0.1, g.getAttribute('r') === g.dataset.live, d.getBoundingClientRect().width > 0]")
        check("recording: the name's dot red and 1.375 times larger, the cut following (D89)", browser.execute_script(dot) == [True, True, True, True], browser.execute_script(dot))
        browser.execute_script(f"location.hash = '#/meeting/{recording.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "rename") and b.execute_script("return document.getElementById('meeting').dataset.name") == recording.name, 10)
        name(browser, "En direct", Keys.ENTER)
        check("while recording: named at once", wait_for(browser, lambda b: head(b) == "En direct", 5), head(browser))
        check("while recording: kept in its folder", (recording / "title.txt").read_text() == "En direct\n")
        # While it records, Libérer de l'espace names it too, busy, its date beside.
        browser.execute_script("location.hash = '#/free'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#free .freeable.busy"), 10)
        busy = browser.execute_script("return [...document.querySelectorAll('#free .freeable.busy')].map(r => [r.querySelector('.name').textContent, r.querySelector('.length').textContent])")
        check("being recorded, Libérer de l'espace names it, its date beside", bool(busy) and busy[0][0] == "En direct" and "·" in busy[0][1], busy)
        browser.execute_script(f"location.hash = '#/meeting/{recording.name}'")
        wait_for(browser, lambda b: head(b) == "En direct", 10)
        # A press within half a second is the same press.
        time.sleep(0.6)
        browser.find_element(By.ID, "record").click()
        stopped = wait_for(browser, lambda b: "stopped" in (recording / "session.json").read_text() and not live(b), 10)
        check("stopped: the name stays", stopped and (recording / "title.txt").read_text() == "En direct\n" and head(browser) == "En direct", head(browser))
        rest = ("const d = document.getElementById('name-dot'), g = document.getElementById('name-gap');"
                " return [getComputedStyle(d).fill === getComputedStyle(document.querySelector('.brand svg')).color, d.getAttribute('r') === d.dataset.rest, g.getAttribute('r') === g.dataset.rest]")
        check("stopped: the name at rest again, its dot in the name's ink (D89)", browser.execute_script(rest) == [True, True, True], browser.execute_script(rest))


def actions_rows(now: dt.datetime) -> None:
    """AK. A meeting's actions as rows: Qui · Quoi · Pour quand (D46)."""
    print("AK. the minutes' actions as rows — who in its colour, by when in monospace, an item that does not split shown as written (D46)")
    meetings = WORK / "acting"
    folder = meetings / "2026-10-28_14-00"
    session(folder, now - dt.timedelta(days=1), 10, {"local.wav": 1000})
    (folder / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Bonjour."}\n{"type":"complete","duration":1.0}\n')
    (folder / "minutes.md").write_text(
        "---\ncontract: minutes/v1\nmade: 2026-10-28T15:00:00+01:00\nprovider: local\nmodel: qwen3:8b\ncontext: null\n---\n\n"
        "## 1.\n\nLe budget est validé.\n\n## 2.\n\n- Envoyer le devis — Inès — vendredi\n- Réserver la salle — non précisé — non précisé\n"
        "- Relancer le fournisseur — Paul\n- Rappeler **Jiří** — Marc — 2026-11-04\n"
        "- Valider le budget — Alexandre, Inès, Paul et le comité scientifique — avant la réunion du comité de pilotage de novembre\n\n## 3.\n\n- Rien\n")
    rows = ("return [...document.querySelectorAll('#meeting .minutes table.actions tbody tr')].map(r =>"
            " [...r.cells].map(c => [c.className, c.textContent]))")
    with Window(meetings, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.set_window_size(1100, 800)
        browser.execute_script(f"location.hash = '#/meeting/{folder.name}'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting .minutes table.actions"), 10)
        got = browser.execute_script(rows)
        check("each action one row of three cells, Qui · Quoi · Pour quand; one that does not split, as written, in its place",
              got == [[["who", "Inès"], ["what", "Envoyer le devis"], ["when", "vendredi"]],
                      [["who unsettled", "non précisé"], ["what", "Réserver la salle"], ["when unsettled", "non précisé"]],
                      [["written", "Relancer le fournisseur — Paul"]],
                      [["who", "Marc"], ["what", "Rappeler Jiří"], ["when", "2026-11-04"]],
                      [["who long", "Alexandre, Inès, Paul et le comité scientifique"], ["what", "Valider le budget"],
                       ["when long", "avant la réunion du comité de pilotage de novembre"]]], got)
        heads = browser.execute_script(
            "const th = [...document.querySelectorAll('#meeting table.actions thead th')];"
            "return [th.map(t => t.textContent), th.every(t => { const r = t.getBoundingClientRect(); return r.width <= 1 && r.height <= 1; })]")
        check("no visible headers: Qui, Quoi, Pour quand their names for assistive technology", heads == [["Qui", "Quoi", "Pour quand"], True], heads)
        looks = browser.execute_script(
            "const of = (v) => { const p = document.createElement('span'); p.style.color = `var(${v})`; document.querySelector('#meeting').append(p);"
            " const c = getComputedStyle(p).color; p.remove(); return c; };"
            "const s = (q) => getComputedStyle(document.querySelector(q));"
            "return [s('#meeting table.actions td.who:not(.unsettled)').color === of('--accent'), s('#meeting table.actions td.when').fontFamily,"
            " s('#meeting table.actions td.who.unsettled').color === of('--soft')]")
        check("who in its colour, by when in monospace, « non précisé » in the soft colour",
              looks[0] and "monospace" in looks[1] and looks[2], looks)
        # Long names and dates come back to the line: nothing overflows the card, at 1100 nor at 640 px.
        wide = "const b = document.querySelector('#meeting .minutes .body'); return [b.scrollWidth, b.clientWidth]"
        held = browser.execute_script(wide)
        browser.set_window_size(420, 800)
        narrow = wait_for(browser, lambda b: b.execute_script("return innerWidth") <= 420, 5) and browser.execute_script(wide)
        check("a long who or by when comes back to the line: the table never overflows the minutes, at 1100 nor 420 px",
              held[0] <= held[1] and narrow and narrow[0] <= narrow[1], (held, narrow))
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: b.execute_script("return innerWidth") > 1000, 5)
        time.sleep(0.5)
        browser.execute_script("document.querySelector('#meeting .minutes').scrollIntoView({block: 'start'})")
        time.sleep(0.3)
        shot(browser, "d46-actions-rows")


def meeting_told_once(now: dt.datetime) -> None:
    """AL. A meeting's notifications are fewer, and a click on one opens the meeting (D44)."""
    print("AL. a window seen: no stop told, the transcript not told before its minutes; their notification's click opens the meeting (D44)")
    meetings = WORK / "telling"
    meetings.mkdir()
    model = WORK / "telling-model.bin"
    model.write_bytes(b"\0" * 16)
    settings = (f'after = "minutes"\nreminder = false\nmodel = "{model}"\n'
                'minutes_provider = "http://127.0.0.1:9/v1"\nminutes_model = "mistral"\n')
    meeting = meetings / "2026-10-18_12-00"
    notified, answered = WORK / "notified-telling", WORK / "answered-telling"
    monitors = [subprocess.Popen(["dbus-monitor", "--session", rule], stdout=open(out, "w"), stderr=subprocess.DEVNULL)
                for rule, out in (("interface='org.freedesktop.Notifications',member='Notify'", notified),
                                  ("type='method_return',sender='org.freedesktop.Notifications'", answered))]
    live = lambda b: "live" in (b.find_element(By.ID, "record").get_attribute("class") or "")
    try:
        time.sleep(0.5)
        with Window(meetings, "fr_FR.UTF-8", settings) as browser:
            browser.find_element(By.ID, "record").click()
            wait_for(browser, lambda b: (meeting / "session.json").exists() and live(b), 10)
            # The button says it at the click now (D68); a press within half a second is the same press.
            time.sleep(0.6)
            browser.find_element(By.ID, "record").click()
            made = wait_for(browser, lambda b: "Compte rendu prêt" in notified.read_text(), 20)
            told = notified.read_text()
            check("a window seen: the stop is not told, nor the transcript before its minutes — their notification alone",
                  made and "Enregistrement arrêté" not in told and "Transcription prête" not in told, told[-600:])
            check("its words say the click opens it", "Cliquez pour l’ouvrir." in told, told[-400:])
            # Away from the meeting, then the notification clicked as the desktop clicks it.
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 5)
            number = re.findall(r"^ +uint32 (\d+)", answered.read_text(), re.M)[-1]
            subprocess.run(["gdbus", "emit", "--session", "--object-path", "/org/freedesktop/Notifications", "--signal",
                            "org.freedesktop.Notifications.ActionInvoked", f"uint32 {number}", "'default'"], capture_output=True)
            opened = wait_for(browser, lambda b: b.execute_script("return location.hash") == f"#/meeting/{meeting.name}"
                              and b.find_element(By.ID, "meeting").is_displayed(), 10)
            check("clicked: the full window opens on that meeting's page, never the file manager", opened,
                  browser.execute_script("return location.hash"))
            # Its transcript made again (Refaire…): ready, its click opens the meeting too.
            (meeting / "local.wav").write_bytes(b"\0" * 1000)
            mark = len(notified.read_text())
            browser.execute_script("window.__TAURI__.core.invoke('transcript_redo', {name: arguments[0], model: arguments[1]})",
                                   meeting.name, str(model))
            again = wait_for(browser, lambda b: "Transcription prête" in notified.read_text()[mark:], 15)
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 5)
            number = re.findall(r"^ +uint32 (\d+)", answered.read_text(), re.M)[-1]
            subprocess.run(["gdbus", "emit", "--session", "--object-path", "/org/freedesktop/Notifications", "--signal",
                            "org.freedesktop.Notifications.ActionInvoked", f"uint32 {number}", "'default'"], capture_output=True)
            reopened = wait_for(browser, lambda b: b.execute_script("return location.hash") == f"#/meeting/{meeting.name}", 10)
            check("a transcript made again says the click opens it, and the click does",
                  again and "Cliquez pour l’ouvrir." in notified.read_text()[mark:] and reopened, browser.execute_script("return location.hash"))
    finally:
        for monitor in monitors:
            monitor.terminate()
            monitor.wait(timeout=10)


def two_panes(now: dt.datetime) -> None:
    """AD. The window in two panes: the meetings in a sidebar, the one chosen beside them (D61)."""
    print("AD. the window in two panes: the sidebar, the meeting chosen beside it, as wide as the window (D61)")
    meetings = WORK / "panes"
    names = [f"2026-10-{day:02d}_10-00" for day in range(10, 24)]
    for index, name in enumerate(names):
        session(meetings / name, now - dt.timedelta(days=len(names) - index), 30, {"local.wav": 1000})
    newest = meetings / names[-1]
    lines = [
        {"type": "header", "contract": "transcript/v1", "engine": "whisper.cpp", "model": "ggml-large-v3-turbo-q8_0", "language": "fr", "context": None},
        *({"type": "segment", "speaker": "me", "track": "local", "voice": None, "start": 2.0 * i, "end": 2.0 * i + 1.5,
           "text": "Une phrase assez longue pour aller à la ligne dans une colonne de lecture, ni plus ni moins."} for i in range(1, 6)),
        {"type": "complete", "duration": 1800.0},
    ]
    (newest / "transcript.jsonl").write_text("".join(json.dumps(line, ensure_ascii=False) + "\n" for line in lines))
    (newest / "minutes.md").write_text(
        "---\ncontract: minutes/v1\nmade: 2026-10-23T11:00:00+02:00\nprovider: openrouter.ai\nmodel: claude-sonnet\ncontext: null\n---\n\n"
        "## 1.\n\n" + "Un compte rendu assez long pour que son texte aille à la ligne bien avant le bord de sa carte. " * 6 + "\n\n"
        "## 2.\n\n- Mettre à jour le planning — Marc — aujourd’hui\n\n## 3.\n\n- Un serveur est gardé pour la démonstration.\n"
    )
    box = lambda b, selector: b.execute_script(
        "const e = document.querySelector(arguments[0]); if (!e || !e.offsetParent && getComputedStyle(e).position !== 'fixed') return null;"
        "const r = e.getBoundingClientRect(); return [r.left, r.top, r.right, r.bottom];", selector)
    current = lambda b: b.execute_script("return [...document.querySelectorAll('#groups [aria-current=page]')].map(a => a.getAttribute('href'))")
    shown = lambda b: b.execute_script("const m = document.getElementById('meeting'); return m.hidden ? null : m.dataset.name")
    sideways = lambda b: b.execute_script("return document.documentElement.scrollWidth > window.innerWidth")

    def seen(browser, name: str) -> None:
        """A screenshot in the theme its name says."""
        time.sleep(0.3)
        shot_themed(browser, name, "light" if "light" in name.split("-") else "dark")

    with Window(meetings, "fr_FR.UTF-8") as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: shown(b) == names[-1], 10)
        side, pane = box(browser, "#sidebar"), box(browser, "#pane")
        check("840 px and more: the sidebar on the left, 280 px, the pane beside it",
              side is not None and pane is not None and abs(side[2] - side[0] - 280) <= 1 and side[2] <= pane[0] + 1, (side, pane))
        order = [box(browser, s) for s in ("#record", "#find", "#groups", "#weight", "#settings-link")]
        check("the sidebar holds, top to bottom: the recording, the search, the meetings, the space they take, Réglages",
              None not in order and all(order[i][1] < order[i + 1][1] for i in range(4))
              and all(side[0] <= o[0] and o[2] <= side[2] + 1 for o in order), order)
        check("« Faire de la place » an icon under the space they take (D89)", browser.find_element(By.ID, "free-link").is_displayed()
              and box(browser, "#free-link")[1] >= box(browser, "#weight")[1] - 1)
        check("the header's two tabs are gone", not browser.find_elements(By.ID, "meetings") and not browser.find_elements(By.CSS_SELECTOR, "header.bar"))
        check("the pane shows the last meeting, marked in the list", shown(browser) == names[-1]
              and current(browser) == [f"#/meeting/{names[-1]}"], (shown(browser), current(browser)))
        said = browser.execute_script("return [...document.querySelectorAll('#groups .session')].map(r => ["
                                      "r.querySelector('.day')?.textContent, r.querySelector('.length')?.textContent,"
                                      " [...r.querySelectorAll('.holds .chip')].filter(c => !c.hidden).length, [...r.querySelectorAll('.mark')].map(m => m.getAttribute('aria-label'))])")
        check("each meeting in the list says its title and date, its length and, by its marks, what it holds; no chip at rest (D89)",
              len(said) == len(names) and all(day and day not in names and " · " in day and any(c.isdigit() for c in day.split(" · ")[0])
                                               and length == "30 min" and chips == 0 for day, length, chips, _ in said)
              and [marks for *_, marks in said].count(["Compte rendu", "Transcription"]) == 1, said[:3])
        browser.execute_script("document.getElementById('list').scrollTop = 100000")
        scrolled = browser.execute_script("return document.getElementById('list').scrollTop")
        browser.find_elements(By.CSS_SELECTOR, "#groups .session")[-1].click()
        came = wait_for(browser, lambda b: shown(b) == names[0], 10)
        kept = browser.execute_script("return document.getElementById('list').scrollTop")
        check("a meeting clicked is shown in the pane, the list kept where it was", came and scrolled > 0 and abs(kept - scrolled) <= 1, (shown(browser), scrolled, kept))
        check("the meeting shown alone marked `aria-current`", current(browser) == [f"#/meeting/{names[0]}"], current(browser))
        browser.find_elements(By.CSS_SELECTOR, "#groups .session")[-1].send_keys(Keys.ARROW_UP)
        moved = wait_for(browser, lambda b: shown(b) == names[1], 10)
        focus = browser.execute_script("return document.activeElement.getAttribute('href')")
        check("↑ on the list shows the meeting above, the focus on its row", moved and focus == f"#/meeting/{names[1]}", (shown(browser), focus))
        browser.find_element(By.CSS_SELECTOR, f"#groups a[href='#/meeting/{names[1]}']").send_keys(Keys.ARROW_DOWN)
        back = wait_for(browser, lambda b: shown(b) == names[0], 10)
        check("↓ the one below", back and browser.execute_script("return document.activeElement.getAttribute('href')") == f"#/meeting/{names[0]}")
        press(browser, "settings-link")
        opened = wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 10)
        width = box(browser, "#settings")
        check("Réglages open in the pane, the sidebar kept, 720 px at most, marked in its place of the list",
              opened and browser.find_element(By.ID, "sidebar").is_displayed() and width and width[2] - width[0] <= 720.5
              and browser.find_element(By.ID, "settings-link").get_attribute("aria-current") == "page" and current(browser) == [],
              (width, current(browser)))
        press(browser, "free-link")
        freeing = wait_for(browser, lambda b: b.find_element(By.ID, "free").is_displayed(), 10)
        check("freeing space opens in the pane, the sidebar kept", freeing and browser.find_element(By.ID, "sidebar").is_displayed())
        browser.execute_script(f"location.hash = '#/meeting/{names[-1]}'")
        wait_for(browser, lambda b: shown(b) == names[-1] and b.find_elements(By.CSS_SELECTOR, ".transcript .lines li"), 10)
        minutes, transcript, drawn = box(browser, ".minutes"), box(browser, ".transcript"), box(browser, "#meeting .drawn")
        check("under 1000 px of pane: the minutes, then the transcript below", minutes and transcript and transcript[1] >= minutes[3], (minutes, transcript))
        browser.set_window_size(1920, 1080)
        wide = wait_for(browser, lambda b: (box(b, ".transcript") or [0, -1])[1] == (box(b, ".minutes") or [0, -2])[1], 5)
        minutes, transcript, drawn, pane = box(browser, ".minutes"), box(browser, ".transcript"), box(browser, "#meeting .drawn"), box(browser, "#pane")
        check("1000 px and more of pane: the minutes and the transcript side by side, together as wide as the audio",
              wide and abs(minutes[0] - drawn[0]) <= 1 and abs(transcript[2] - drawn[2]) <= 1, (minutes, transcript, drawn))
        width = browser.execute_script("return window.innerWidth")
        check("the pane as wide as the window leaves it, no width of its own", abs(pane[2] - width) <= 1 and abs(pane[0] - 280) <= 1, (pane, width))
        reading = browser.execute_script(
            "const t = document.querySelector('.minutes .body'); if (!t) return [0, 0, document.querySelector('.minutes').outerHTML.slice(0, 400)];"
            "return [t.getBoundingClientRect().width, document.querySelector('.minutes').getBoundingClientRect().width, getComputedStyle(t).maxWidth,"
            " getComputedStyle(document.querySelector('.transcript .lines')).maxWidth]")
        # 75ch, as WebKit computes it: in pixels, the width of 75 zeros of the text's font.
        zeros = browser.execute_script(
            "const s = document.createElement('span'); s.textContent = '0'.repeat(75); document.querySelector('.minutes .body').append(s);"
            "s.style.whiteSpace = 'nowrap'; const w = s.getBoundingClientRect().width; s.remove(); return w")
        check("their text 75 characters at most, inside a wider card",
              reading[2].endswith("px") and abs(float(reading[2][:-2]) - zeros) <= 1 and reading[0] <= zeros + 1 and reading[0] < reading[1] - 60
              and reading[3] == reading[2], (reading, zeros))
        # Seen as a person sees it: the list at its top, the pointer away from it.
        browser.execute_script("document.getElementById('list').scrollTop = 0")
        ActionChains(browser).move_to_element(browser.find_element(By.CSS_SELECTOR, "#meeting .drawn")).perform()
        seen(browser, "d61-1920-light")
        seen(browser, "d61-1920-dark")
        browser.set_window_size(420, 400)
        browser.execute_script("location.hash = '#/'")
        alone = wait_for(browser, lambda b: b.find_element(By.ID, "sidebar").is_displayed() and not b.find_element(By.ID, "pane").is_displayed(), 10)
        check("under 840 px: the list alone, nothing overflowing sideways", alone and not sideways(browser))
        seen(browser, "d61-420-list-light")
        seen(browser, "d61-420-list-dark")
        browser.find_elements(By.CSS_SELECTOR, "#groups .session")[0].click()
        page = wait_for(browser, lambda b: b.find_element(By.ID, "pane").is_displayed() and not b.find_element(By.ID, "sidebar").is_displayed(), 10)
        check("then the meeting alone, with a way back, nothing overflowing sideways",
              page and browser.find_element(By.CSS_SELECTOR, "#pane .back").is_displayed() and not sideways(browser))
        seen(browser, "d61-420-meeting-light")
        seen(browser, "d61-420-meeting-dark")
        browser.find_element(By.CSS_SELECTOR, "#pane .back").click()
        again = wait_for(browser, lambda b: b.find_element(By.ID, "sidebar").is_displayed() and not b.find_element(By.ID, "pane").is_displayed(), 10)
        check("the way back is the list again", again)
        browser.set_window_size(1100, 800)
        find = browser.find_element(By.ID, "find")
        find.send_keys("zzz")
        nothing = wait_for(browser, lambda b: not [r for r in b.find_elements(By.CSS_SELECTOR, "#groups .session") if r.is_displayed()]
                           and b.find_element(By.ID, "none-found").is_displayed(), 5)
        check("the search: a word no meeting holds, none shown, said", nothing, browser.find_element(By.ID, "none-found").text)
        find.send_keys(Keys.CONTROL, "a")
        find.send_keys(Keys.BACKSPACE)
        oldest = browser.find_elements(By.CSS_SELECTOR, "#groups .session .day")[-1].get_attribute("textContent")
        find.send_keys(oldest)
        one = wait_for(browser, lambda b: [r.get_attribute("href") for r in b.find_elements(By.CSS_SELECTOR, "#groups .session") if r.is_displayed()]
                       == [b.find_elements(By.CSS_SELECTOR, "#groups .session")[-1].get_attribute("href")], 5)
        check("a word one holds, that one alone", one and not browser.find_element(By.ID, "none-found").is_displayed())
        find.send_keys(Keys.CONTROL, "a")
        find.send_keys(Keys.BACKSPACE)
        browser.execute_script("document.activeElement.blur()")
        browser.find_element(By.TAG_NAME, "body").send_keys(Keys.TAB)
        inside = browser.execute_script("return document.getElementById('sidebar').contains(document.activeElement)")
        check("the keyboard: the sidebar before the pane", inside, browser.execute_script("return document.activeElement.outerHTML.slice(0, 80)"))

    # In English, gone through with the keyboard alone: the sidebar first, the focus always seen.
    with Window(meetings, "en_GB.UTF-8") as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: shown(b) == names[-1] and b.find_elements(By.CSS_SELECTOR, ".transcript .lines li"), 10)
        english = [browser.find_element(By.ID, "record-said").text] + [
            [browser.find_element(By.ID, i).get_attribute(a) for a in ("aria-label", "title")] for i in ("fold", "unfold", "free-link", "small-link", "settings-link")]
        browser.execute_script("document.activeElement.blur()")
        stops = []
        for _ in range(45):
            ActionChains(browser).send_keys(Keys.TAB).perform()
            # Seen: its outline or its shadow — or its `.choice`'s, for a radio — differs from the same
            # element's at rest, measured blurred; an outline at rest, or a shadow it always has, is no focus.
            stops.append(browser.execute_script(
                "const e = document.activeElement, n = e.closest('.choice') ?? e;"
                "const look = () => { const s = getComputedStyle(n); return [s.outlineStyle, s.outlineWidth, s.outlineColor, s.boxShadow].join('|'); };"
                "const focused = look(); e.blur(); const rest = look(); e.focus({focusVisible: true});"
                "const s = getComputedStyle(n), drawn = (s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) > 0) || s.boxShadow !== 'none';"
                "return [document.getElementById('sidebar').contains(e) ? 'side' : document.getElementById('pane').contains(e) ? 'pane' : 'other',"
                " drawn && focused !== rest && document.activeElement === e, e.id || e.className || e.tagName]"))
        places = [place for place, _, _ in stops]
        unseen = [name for _, seen_, name in stops if not seen_]
        # The pane's stops in one run: none of the sidebar's between them — past the last, the page starts again.
        last = len(places) - 1 - places[::-1].index("pane") if "pane" in places else 0
        check("in English, the keyboard reaches the sidebar first, then the pane's stops in one run", english == ["Start a recording", ["Fold the sidebar", "Fold the sidebar (Ctrl+B)"], ["Show the sidebar", "Show the sidebar (Ctrl+B)"],
                                                                                                         ["Free space", "Free space"], ["The recording bar", "The recording bar"], ["Settings", "Settings"]]
              and places[0] == "side" and "pane" in places and set(places[places.index("pane"):last + 1]) == {"pane"} and "other" not in places,
              (english, places))
        check("the focus is always seen", unseen == [], unseen)

    empty = WORK / "panes-none"
    empty.mkdir()
    with Window(empty, "en_US.UTF-8") as browser:
        said = wait_for(browser, lambda b: b.find_element(By.ID, "none").is_displayed(), 10)
        check("no meeting yet: the pane says what to do, and offers the recording", said and browser.execute_script(
            "return document.getElementById('pane').contains(document.getElementById('none'))")
              and browser.find_element(By.ID, "none-do").text == "Start a recording: its button, its shortcut, or the tray’s icon."
              and browser.find_element(By.ID, "empty-record").text == "Start a recording", browser.find_element(By.ID, "none").text)
        # D89: the shortcut, gone from under the sidebar's button, is said here, where nothing was recorded yet.
        keys = browser.execute_script("const k = document.getElementById('empty-keys'); return [!!k && k.offsetParent !== null, k && k.textContent, document.getElementById('record').title]")
        check("no meeting: the shortcut said beside what to do, and in the button's tooltip (D89)", keys[0] and keys[1] and keys[2] == f"Start a recording ({keys[1]})", keys)
        browser.set_window_size(420, 400)
        narrow = wait_for(browser, lambda b: b.find_element(By.ID, "sidebar").is_displayed() and b.find_element(By.ID, "none").is_displayed(), 5)
        check("under 840 px too: the sidebar, then what to do", narrow and not browser.execute_script(
            "return document.documentElement.scrollWidth > window.innerWidth"))


def sidebar_folded(now: dt.datetime) -> None:
    """AE. The sidebar folded by the user, and brought back; the window opened again as it was left (D61)."""
    print("AE. the sidebar folded by its button or Ctrl+B outside a text field, kept by the settings (D61)")
    meetings = WORK / "folding"
    for day in range(3):
        session(meetings / f"2026-10-2{day}_10-00", now - dt.timedelta(days=3 - day), 20, {"local.wav": 1000})
    shown = lambda b, id_: b.find_element(By.ID, id_).is_displayed()
    focused = lambda b: b.execute_script("return document.activeElement.id")
    ctrl_b = lambda b: ActionChains(b).key_down(Keys.CONTROL).send_keys("b").key_up(Keys.CONTROL).perform()
    window = Window(meetings, "fr_FR.UTF-8", "reminder = false\n")
    with window as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: shown(b, "meeting"), 10)
        fold = browser.find_element(By.ID, "fold")
        check("the sidebar's button to fold it, named with its shortcut", shown(browser, "fold") and not shown(browser, "unfold")
              and fold.get_attribute("aria-label") == "Replier la barre latérale" and fold.get_attribute("title") == "Replier la barre latérale (Ctrl+B)",
              (fold.get_attribute("aria-label"), fold.get_attribute("title")))
        press(browser, "fold")
        folded = wait_for(browser, lambda b: not shown(b, "sidebar"), 5)
        pane = browser.execute_script("const r = document.getElementById('pane').getBoundingClientRect(); return [r.left, r.right, window.innerWidth]")
        check("folded: the pane takes the whole width", folded and pane[0] == 0 and abs(pane[1] - pane[2]) <= 1, pane)
        check("a button brings it back, the focus on it", shown(browser, "unfold") and focused(browser) == "unfold", focused(browser))
        saved = wait_for(browser, lambda b: "folded = true" in window.file.read_text(), 5)
        check("the settings keep it folded", saved, window.file.read_text())
        press(browser, "unfold")
        back = wait_for(browser, lambda b: shown(b, "sidebar") and not shown(b, "unfold"), 5)
        check("brought back by its button, the focus on the one that folds it", back and focused(browser) == "fold", focused(browser))
        check("and the settings keep it open", wait_for(browser, lambda b: "folded = false" in window.file.read_text(), 5), window.file.read_text())
        browser.execute_script("document.activeElement.blur()")
        ctrl_b(browser)
        check("Ctrl+B outside a text field folds it", wait_for(browser, lambda b: not shown(b, "sidebar"), 5))
        ctrl_b(browser)
        check("Ctrl+B again brings it back", wait_for(browser, lambda b: shown(b, "sidebar"), 5))
        browser.find_element(By.ID, "find").click()
        ctrl_b(browser)
        time.sleep(0.5)
        check("Ctrl+B in a text field leaves it as it is", shown(browser, "sidebar"))
        browser.execute_script("document.activeElement.blur()")
        ctrl_b(browser)
        wait_for(browser, lambda b: "folded = true" in window.file.read_text(), 5)
        browser.set_window_size(420, 400)
        browser.execute_script("location.hash = '#/'")
        narrow = wait_for(browser, lambda b: shown(b, "sidebar") and not shown(b, "pane"), 5)
        check("under 840 px, folded or not, the list is shown alone, with no button to fold it",
              narrow and not shown(browser, "fold") and not shown(browser, "unfold"))
    again = Window(meetings, "fr_FR.UTF-8", "reminder = false\nfolded = true\n")
    with again as browser:
        browser.set_window_size(1100, 800)
        opened = wait_for(browser, lambda b: shown(b, "meeting") and not shown(b, "sidebar") and shown(b, "unfold"), 10)
        check("the window opens again as it was left: folded", opened)


def transcript_corrected(now: dt.datetime) -> None:
    """AB. A line of a transcript corrected by whoever listens to it (D62, ADR-0014)."""
    print("AB. a transcript's line corrected by whoever listens, the minutes offered again (D62)")
    keys = lambda browser, *sent: ActionChains(browser).send_keys(*sent).perform()
    focused = lambda b: b.execute_script("return document.activeElement.id || document.activeElement.className")

    def when(text) -> dt.datetime:
        """A time as the file says it, or long ago when it says none, or none that reads."""
        try:
            return dt.datetime.fromisoformat(text)
        except (TypeError, ValueError):
            return dt.datetime(1970, 1, 1, tzinfo=dt.timezone.utc)

    meetings = WORK / "correcting"
    heard = meetings / "2026-10-27_09-00"
    session(heard, now - dt.timedelta(days=1), 0.1, {})
    record = json.loads((heard / "session.json").read_text())
    record["tracks"] = {"local": {"device": "Mic"}}
    (heard / "session.json").write_text(json.dumps(record))
    with wave.open(str(heard / "local.wav"), "wb") as made:
        made.setnchannels(1)
        made.setsampwidth(2)
        made.setframerate(48000)
        made.writeframes(bytes(2 * 48000 * 3))
    (heard / "clock.jsonl").write_text('{"track":"local","sample":0,"at":0.0}\n')
    before = (
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","voice":null,"start":0.4,"end":1.2,"text":"Bonjour à tous, on commence."}\n'
        '{"type":"segment","speaker":"others","track":"remote","voice":null,"start":1.5,"end":2.5,"text":"Le budget de Jiri est validé."}\n'
        '{"type":"complete","duration":3.0}\n'
    )
    (heard / "transcript.jsonl").write_text(before)
    minutes = ("---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: local\nmodel: qwen3:8b\ncontext: null\n---\n\n"
               "## 1.\n\nLe budget est validé.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    (heard / "minutes.md").write_text(minutes)
    # Not complete: nothing to correct yet.
    unfinished = meetings / "2026-10-27_11-00"
    session(unfinished, now - dt.timedelta(days=1), 0.1, {"local.wav": 1000})
    (unfinished / "transcript.jsonl").write_text(before.replace('{"type":"complete","duration":3.0}\n', ""))
    model = WORK / "correcting-model.bin"
    model.write_bytes(b"\0" * 16)
    settings = f'reminder = false\nmodel = "{model}"\nminutes_provider = "http://127.0.0.1:9/v1"\nminutes_model = "mistral"\n'
    window = Window(meetings, "fr_FR.UTF-8", settings)
    pencils = "return [...document.querySelectorAll('#meeting .lines .edit')].filter(p => !p.hidden).length"
    with window as browser:
        browser.execute_script(f"location.hash = '#/meeting/{unfinished.name}'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting .lines li"), 10)
        refused = browser.execute_script(
            "return window.__TAURI__.core.invoke('transcript_corrected', {name: arguments[0], track: 'local', start: 0.4, text: 'x'}).then(() => 'kept', (why) => String(why))",
            unfinished.name)
        check("a transcript not complete offers no line to correct, and a correction asked is refused, the file as it was",
              browser.execute_script(pencils) == 0 and refused == "sa transcription n’est pas complète\u00a0: refaites-la d’abord." and (unfinished / "transcript.jsonl").read_text() == before.replace('{"type":"complete","duration":3.0}\n', ""),
              (browser.execute_script(pencils), refused))
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting .lines .edit"), 10)
        check("complete, each line has its pencil, named by the line's time", browser.execute_script(pencils) == 2
              and browser.find_elements(By.CSS_SELECTOR, "#meeting .lines .edit")[1].get_attribute("aria-label") == "Corriger la ligne de 00:01")
        check("before any correction, the minutes say nothing of one", not browser.find_elements(By.ID, "corrected-since"))
        # By the keyboard alone: Tab from the first line reaches its pencil; Enter opens the field.
        first = browser.find_element(By.CSS_SELECTOR, "#meeting .lines li")
        browser.execute_script("arguments[0].focus()", first)
        keys(browser, Keys.TAB)
        on_pencil = browser.execute_script("return document.activeElement.classList.contains('edit')")
        keys(browser, Keys.ENTER)
        field = wait_for(browser, lambda b: focused(b) == "correcting", 2)
        check("Tab from a line reaches its pencil, Enter opens the line as a field, focused, holding its text",
              on_pencil and field and browser.execute_script("return document.activeElement.value") == "Bonjour à tous, on commence.")
        # Escape: as it was.
        keys(browser, Keys.END, " encore", Keys.ESCAPE)
        back = wait_for(browser, lambda b: b.execute_script("return document.activeElement.classList.contains('edit')"), 2)
        check("Escape leaves the line as it was, the focus back on its pencil, the file untouched",
              back and (heard / "transcript.jsonl").read_text() == before
              and browser.find_element(By.CSS_SELECTOR, "#meeting .lines .text").text == "Bonjour à tous, on commence.")
        # Emptied: refused, said.
        keys(browser, Keys.ENTER)
        wait_for(browser, lambda b: focused(b) == "correcting", 2)
        browser.execute_script("document.activeElement.select()")
        keys(browser, Keys.DELETE, Keys.ENTER)
        said = wait_for(browser, lambda b: "ne reste pas vide" in b.find_element(By.ID, "correction-said").text, 2)
        check("a line left empty is refused, said, the field kept", said and focused(browser) == "correcting" and (heard / "transcript.jsonl").read_text() == before)
        keys(browser, "Bonjour à toutes, on commence — Γιώργος 🙂", Keys.ENTER)
        kept = wait_for(browser, lambda b: "toutes" in (heard / "transcript.jsonl").read_text(), 3)
        after = (heard / "transcript.jsonl").read_text().splitlines()
        changed = [i for i, (a, b) in enumerate(zip(before.splitlines(), after)) if a != b]
        line = json.loads(after[1]) if len(after) > 1 else {}
        check("Enter keeps it: the file differs by that line alone, its text as typed — any script —, edited when",
              kept and len(after) == 4 and changed == [1] and line.get("text") == "Bonjour à toutes, on commence — Γιώργος 🙂"
              and when(line.get("edited")) > now - dt.timedelta(minutes=30)
              and {k: v for k, v in line.items() if k not in ("text", "edited")} == {k: v for k, v in json.loads(before.splitlines()[1]).items() if k != "text"},
              after)
        # Against the contract's own schema, its formats checked: `edited` a date-time.
        import jsonschema
        schema = json.loads((Path(__file__).resolve().parents[3] / "contracts" / "transcript" / "v1" / "line.schema.json").read_text())
        validator = jsonschema.Draft202012Validator(schema, format_checker=jsonschema.Draft202012Validator.FORMAT_CHECKER)
        invalid = [str(error.message) for kept_line in after for error in validator.iter_errors(json.loads(kept_line))]
        check("the file corrected is valid against transcript/v1's schema, its `edited` a date-time", invalid == [], invalid)
        shown = browser.find_element(By.CSS_SELECTOR, "#meeting .lines li").text
        check("the page shows it, marked « corrigé », the focus back on its pencil",
              "Bonjour à toutes" in shown and "corrigé" in shown and browser.execute_script("return document.activeElement.classList.contains('edit')"), shown)
        # Left by the pointer — a click elsewhere —: what is typed is kept, as Enter keeps it.
        browser.find_elements(By.CSS_SELECTOR, "#meeting .lines .edit")[1].click()
        wait_for(browser, lambda b: focused(b) == "correcting", 2)
        keys(browser, Keys.END, " en novembre")
        browser.find_element(By.CSS_SELECTOR, "#meeting h1, #meeting .meeting-top").click()
        left = wait_for(browser, lambda b: "validé. en novembre" in (heard / "transcript.jsonl").read_text(), 3)
        check("leaving the field keeps what is typed", left, (heard / "transcript.jsonl").read_text().splitlines()[2:3])
        # Pasted with its line break and spaces around: one line, its ends trimmed.
        browser.find_elements(By.CSS_SELECTOR, "#meeting .lines .edit")[1].click()
        wait_for(browser, lambda b: focused(b) == "correcting", 2)
        browser.execute_script("const f = document.activeElement; f.select(); const d = new DataTransfer(); d.setData('text/plain', arguments[0]);"
                               "f.dispatchEvent(new ClipboardEvent('paste', {clipboardData: d, bubbles: true, cancelable: true}))", "  Le budget de Jiri\nest validé.  ")
        keys(browser, Keys.ENTER)
        pasted = wait_for(browser, lambda b: '"text":"Le budget de Jiri est validé."' in (heard / "transcript.jsonl").read_text().replace('": "', '":"'), 3)
        check("a line break pasted becomes a space, the spaces at its ends dropped", pasted, (heard / "transcript.jsonl").read_text().splitlines()[2:3])
        # Drawn again: the minutes, older than the correction, say so and offer to be made again.
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 5)
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        offered = wait_for(browser, lambda b: b.find_elements(By.ID, "corrected-since"), 10)
        check("the minutes written before it say the transcript was corrected since, and offer « Refaire avec les corrections »",
              offered and "corrigée après ce compte rendu" in browser.find_element(By.ID, "corrected-since").text
              and browser.find_element(By.ID, "redo-corrections").text == "Refaire avec les corrections")
        time.sleep(2)
        check("nothing is made again without that click", (heard / "minutes.md").read_text() == minutes)
        # Refaire… of a corrected transcript says its corrections will not be kept.
        press(browser, "redo-transcript")
        warned = wait_for(browser, lambda b: b.find_elements(By.ID, "redo-loses-corrections"), 5)
        check("Refaire… of a corrected transcript says, before it starts, that its corrections will not be kept",
              warned and "ne seront pas gardées" in browser.find_element(By.ID, "redo-loses-corrections").text)
        browser.execute_script(f"location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 5)
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-corrections"), 10)
        # While they are written, no line is corrected: they read the transcript before.
        (WORK / "slow-minutes").touch()
        try:
            press(browser, "redo-corrections")
            writing = wait_for(browser, lambda b: b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.minutes?.name === arguments[0])", heard.name), 5)
            asked = browser.execute_script(
                "return window.__TAURI__.core.invoke('transcript_corrected', {name: arguments[0], track: 'remote', start: 1.5, text: 'x'}).then(() => 'kept', (why) => String(why))",
                heard.name)
            hidden = wait_for(browser, lambda b: b.execute_script(pencils) == 0, 3)
            check("while the minutes are written, no line is corrected: refused, its pencils hidden",
                  writing and "en cours d’enregistrement ou de préparation" in asked and hidden and "Le budget de Jiri" in (heard / "transcript.jsonl").read_text(), (writing, asked))
        finally:
            (WORK / "slow-minutes").unlink()
        remade = wait_for(browser, lambda b: (heard / "minutes.md").read_text() != minutes, 10)
        check("« Refaire avec les corrections »: the minutes made again by whoever the settings name, from the corrected transcript",
              remade and "model: mistral" in (heard / "minutes.md").read_text(), (heard / "minutes.md").read_text()[:200])
        # Asked while the transcript is made again: refused — one writer at a time.
        (WORK / "slow-hearing").touch()
        try:
            # Refused while the minutes asked just before are still made: asked again until taken.
            wait_for(browser, lambda b: b.execute_script(
                "return window.__TAURI__.core.invoke('context_redo', {name: arguments[0]}).then(() => true, () => false)", heard.name), 10)
            # Made, or in line to be: what the shell says it is making.
            wait_for(browser, lambda b: b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.transcript?.name === arguments[0] || m.transcripts_waiting.includes(arguments[0]))",
                heard.name), 5)
            refused = browser.execute_script(
                "return window.__TAURI__.core.invoke('transcript_corrected', {name: arguments[0], track: 'local', start: 0.4, text: 'x'}).then(() => 'kept', (why) => String(why))",
                heard.name)
        finally:
            (WORK / "slow-hearing").unlink()
        check("a correction asked while the transcript is made again is refused", "en cours d’enregistrement ou de préparation" in refused, refused)
        # Its end waited for: the transcript set aside meanwhile, then the new one in its place.
        def remade() -> bool:
            try:
                return "Refait avec" in (heard / "transcript.jsonl").read_text()
            except FileNotFoundError:
                return False
        wait_for(browser, lambda b: remade(), 10)


def corrections_gated(now: dt.datetime) -> None:
    """AG. « Refaire avec les corrections » through Refaire…'s own gate (D62): a service not agreed to
    asks first, in the pane (D61); a meeting said confidential is made here (D60)."""
    print("AG. the minutes made again with the corrections: the agreement asked in the pane, a confidential meeting made here (D62, D61)")
    ollama = {"held": ["qwen3:8b"]}

    class Ollama(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_GET(self):
            body = json.dumps({"models": [{"name": name} for name in ollama["held"]]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Ollama)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_port}"
    meetings = WORK / "gated"
    minutes = ("---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: writes.example\nmodel: gpt-4.1-mini\ncontext: null\n---\n\n"
               "## 1.\n\nLe budget est validé.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    folders = []
    for name in ("2026-10-26_09-00", "2026-10-26_11-00"):
        folder = meetings / name
        session(folder, now - dt.timedelta(days=3), 0.1, {"local.wav": 1000})
        (folder / "transcript.jsonl").write_text(
            '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
            '{"type":"segment","speaker":"me","track":"local","voice":null,"start":0.4,"end":1.2,"text":"Le budget est validé.","edited":"2026-10-29T09:00:00+01:00"}\n'
            '{"type":"complete","duration":1.0}\n')
        (folder / "minutes.md").write_text(minutes)
        folders.append(folder)
    opened, secret = folders
    (secret / "confidential").write_bytes(b"")
    online = 'reminder = false\nminutes_provider = "https://writes.example/v1"\nminutes_model = "gpt-4.1-mini"\n'
    front = lambda folder, key: next((line.split(": ", 1)[1] for line in (folder / "minutes.md").read_text().splitlines()
                                      if line.startswith(f"{key}: ")), None)
    args = WORK / "summary-args"
    # The page of the meeting asked for, its own button: until `opened` answers, the page before
    # stays in the pane, with a button of the same name that acts on the meeting it shows.
    its_page = lambda name: lambda b: b.find_elements(By.ID, "redo-corrections") and b.execute_script("return document.getElementById('minutes-state')?.dataset.name") == name
    try:
        with Window(meetings, "fr_FR.UTF-8", online, ollama=base) as browser:
            browser.set_window_size(1100, 800)
            args.unlink(missing_ok=True)
            browser.execute_script(f"location.hash = '#/meeting/{opened.name}'")
            wait_for(browser, its_page(opened.name), 10)
            press(browser, "redo-corrections")
            asked = wait_for(browser, lambda b: b.execute_script("return location.hash") == f"#/agree/{opened.name}"
                             and b.find_elements(By.ID, "agree-send"), 10)
            time.sleep(2)
            check("a service not agreed to: « Refaire avec les corrections » asks first, as Refaire… does — the step not started, two seconds on",
                  asked and not args.exists() and (opened / "minutes.md").read_text() == minutes, browser.execute_script("return location.hash"))
            check("the agreement opens in the pane, the sidebar kept beside it (D61)", browser.execute_script(
                "return document.getElementById('pane').contains(document.getElementById('agree'))")
                  and browser.find_element(By.ID, "agree").is_displayed() and browser.find_element(By.ID, "sidebar").is_displayed())
            browser.execute_script(f"location.hash = '#/meeting/{secret.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-corrections"), 10)
            press(browser, "redo-corrections")
            # Made by Ollama on this computer, its model named in the minutes (the played step names no provider then).
            made = wait_for(browser, lambda b: front(secret, "model") == "qwen3:8b", 20)
            check("a meeting said confidential: made here, by Ollama, no agreement asked",
                  made and browser.execute_script("return location.hash") == f"#/meeting/{secret.name}"
                  and args.exists() and f"--provider {base}/v1" in args.read_text(), (front(secret, "model"), args.read_text() if args.exists() else ""))
    finally:
        server.shutdown()


def press_on_the_page_left(now: dt.datetime) -> None:
    """AS. Another page asked for: the page left is gone at once, nothing of it to press; a drawing
    that fails is said, with a way out and a way to try again; what is agreed to names its meeting."""
    print("AS. the page left goes at once: nothing to press, nothing sent; a failed drawing has a way out; the agreement names its meeting")
    meetings = WORK / "pressed"
    minutes = ("---\ncontract: minutes/v2\nmade: 2026-10-28T10:00:00+01:00\nprovider: writes.example\nmodel: m\ncontext: null\n{}---\n\n"
               "## 1.\n\nLe budget.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    left, asked, third = meetings / "2026-10-26_09-00", meetings / "2026-10-26_11-00", meetings / "2026-10-26_13-00"
    for folder, named in ((left, 'title: "Le budget inventé"\n'), (asked, ""), (third, "")):
        session(folder, now - dt.timedelta(days=3), 0.1, {"local.wav": 1000})
        (folder / "transcript.jsonl").write_text(
            '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
            '{"type":"segment","speaker":"me","track":"local","voice":null,"start":0.4,"end":1.2,"text":"Le budget.","edited":"2026-10-29T09:00:00+01:00"}\n'
            '{"type":"complete","duration":1.0}\n')
        (folder / "minutes.md").write_text(minutes.format(named))
    written = (left / "minutes.md").read_text()
    args = WORK / "summary-args"
    service = 'reminder = false\nminutes_provider = "https://writes.example/v1"\nminutes_model = "m"\n'
    # `opened` answers 800 ms late; `listed` refused by `refusing` calls from now, each `after` ms late.
    late = ("const real = window.__TAURI__.core; window.refusing = arguments[0] ?? 0; const after = arguments[1] ?? 0; window.__TAURI__.core = { ...real, invoke: async (asked, ...rest) => {"
            " if (asked === 'opened') await new Promise((r) => setTimeout(r, 800));"
            " if (asked === 'listed' && window.refusing > 0) { window.refusing -= 1; await new Promise((r) => setTimeout(r, after)); window.threw = true; throw Object.assign(new Error('REFUS'), { name: 'ScenarioAS' }); }"
            " return real.invoke(asked, ...rest); } }; window.restore = () => { window.__TAURI__.core = real; };")
    page_of = "return document.getElementById('meeting').hidden ? null : document.getElementById('minutes-state')?.dataset.name ?? null"
    said = "const p = document.getElementById('pane-said'); return p.hidden ? '' : p.textContent"
    not_shown = "Cette page n’a pas pu être affichée : réessayez."
    # Anything of the pages left, sampled until the page asked for is drawn.
    watching = ("const target = arguments[0]; window.seen = 0; window.addEventListener('hashchange', () => { window.watch = setInterval(() => { const m = document.getElementById('meeting');"
                " if (m.querySelector('#minutes-state')?.dataset.name === target) clearInterval(window.watch);"
                " else if (m.querySelector('button, a, input, textarea, select') || document.getElementById('agree').childElementCount) window.seen += 1; }, 10); }, { once: true });")
    def at_page(browser, folder):
        browser.execute_script(f"location.hash = '#/meeting/{folder.name}'")
        wait_for(browser, lambda b: b.execute_script(page_of) == folder.name, 10)
    def pressed(browser, element, key):
        # As a pointer does, at its place on the screen, or the key to what holds the focus.
        try:
            if key is None:
                element.click()
            else:
                ActionChains(browser).send_keys(key).perform()
        except WebDriverException:
            pass
    def nothing_sent():
        return not args.exists() and (left / "minutes.md").read_text() == written and not (left / "confidential").exists()
    with Window(meetings, "fr_FR.UTF-8", service + 'minutes_agreed = "writes.example"\n') as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: b.execute_script(page_of) is not None, 10)
        for kind, keys in (("redo-minutes", (None, Keys.ENTER, Keys.SPACE)), ("redo-corrections", (None, Keys.ENTER, Keys.SPACE)), ("confidential-switch", (None, Keys.SPACE))):
            for key in keys:
                at_page(browser, left)
                args.unlink(missing_ok=True)
                found = browser.find_element(By.ID, kind)
                browser.execute_script("arguments[0].focus()", found)
                browser.execute_script(late)
                browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
                pressed(browser, found, key)
                during = browser.execute_script("return [document.getElementById('meeting').childElementCount, !!document.getElementById('redo-panel')]")
                drawn = wait_for(browser, lambda b: b.execute_script(page_of) == asked.name, 5)
                browser.execute_script("window.restore()")
                check(f"{kind} ({'pointer' if key is None else 'key'}) while another meeting is asked for: nothing to press, nothing started, nothing sent",
                      drawn and during == [0, False] and nothing_sent(), (during, args.read_text() if args.exists() else ""))
        # Two addresses fast: nothing of either page left until the last is drawn, the first's refusal says nothing.
        at_page(browser, left)
        browser.execute_script(watching, third.name)
        browser.execute_script("window.threw = false")
        browser.execute_script(late, 1, 300)
        browser.execute_script(f"location.hash = '#/meeting/{asked.name}'; setTimeout(() => {{ location.hash = '#/meeting/{third.name}'; }}, 50);")
        wait_for(browser, lambda b: b.execute_script("return location.hash") == f"#/meeting/{third.name}", 2)
        wait_for(browser, lambda b: b.execute_script("return window.threw === true"), 2)
        quiet = browser.execute_script(said)
        drawn = wait_for(browser, lambda b: b.execute_script(page_of) == third.name, 5)
        browser.execute_script("window.restore()")
        check("two addresses fast: nothing of the page left, the first's refusal says nothing, the last is drawn",
              drawn and browser.execute_script("return window.seen") == 0 and quiet == "" and browser.execute_script(said) == "", (browser.execute_script("return window.seen"), quiet))
        # The same meeting at its own address: kept, with what is typed and the focus.
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.execute_script("return ['', '#/'].includes(location.hash)") and b.execute_script(page_of) is not None, 5)
        beside = browser.execute_script(page_of)
        context_opened(browser)
        browser.execute_script("window.__kept = document.getElementById('context-text'); window.__kept.focus()")
        browser.execute_script(f"location.hash = '#/meeting/{beside}'")
        same = wait_for(browser, lambda b: b.execute_script("return location.hash") == f"#/meeting/{beside}" and b.execute_script("return document.getElementById('meeting').dataset.at === location.hash"), 5)
        check("its own address: the page kept, its field and the focus with it",
              same and browser.execute_script("return window.__kept.isConnected && document.activeElement === window.__kept"), browser.execute_script("return document.activeElement.id"))
        # Away and straight back: the page left is drawn anew, never left empty as if kept.
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.execute_script("return ['', '#/'].includes(location.hash)") and b.execute_script(page_of) == beside, 5)
        other = left.name if beside != left.name else asked.name
        browser.execute_script(late)
        browser.execute_script(f"location.hash = '#/meeting/{other}'; setTimeout(() => {{ location.hash = '#/meeting/{beside}'; }}, 50);")
        wait_for(browser, lambda b: b.execute_script("return location.hash") == f"#/meeting/{beside}", 2)
        wait_for(browser, lambda b: b.execute_script(page_of) == beside, 5)
        browser.execute_script("window.restore()")
        check("away and straight back: the page is there", browser.execute_script(page_of) == beside, browser.execute_script("return document.getElementById('meeting').childElementCount"))
        # A page come after one left arrives; drawn again where it stands, it does not.
        at_page(browser, asked)
        moved = "return getComputedStyle(document.getElementById('meeting').firstElementChild).animationName"
        check("a page come after one left arrives, once", browser.execute_script(moved) == "page-arrives", browser.execute_script(moved))
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        check("drawn again where it stands: it does not", wait_for(browser, lambda b: b.execute_script(moved) == "none", 5), browser.execute_script(moved))
        browser.execute_script(f"location.hash = '#/meeting/{beside}'")
        wait_for(browser, lambda b: b.execute_script(page_of) == beside, 5)
        # A focus's drawing that fails on the page asked for: it stays; said.
        browser.execute_script("window.stopped = false; player = { name: arguments[0], stop() { window.stopped = true; } }", beside)
        browser.execute_script(late, 1, 0)
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        told = wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
        check("a drawing that fails on the page shown: said, the page stays, what plays plays on", told and browser.execute_script(page_of) == beside and browser.execute_script("return window.stopped") is False, browser.execute_script(page_of))
        browser.execute_script("player = null")
        browser.execute_script("window.restore()")
        # A throw in the drawing: nothing of the page left, said; tried again, drawn; logged.
        at_page(browser, left)
        browser.execute_script("window.logged = []; const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => { if (asked === 'page_failed') window.logged.push(rest[0]); return real.invoke(asked, ...rest); } }; window.restore = () => { window.__TAURI__.core = real; };"
                               "window.__drawn = showMeeting; showMeeting = () => { throw Object.assign(new Error('REFUS'), { name: 'ScenarioAS' }); };")
        browser.execute_script("player = { name: arguments[0], stop() { window.stopped = true; } }", asked.name)
        browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
        told = wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
        check("a throw in the drawing: said, nothing of the page left, what played stopped",
              told and browser.execute_script("return document.getElementById('meeting').querySelector('button, a, input, textarea, select') === null && window.stopped === true"), browser.execute_script("return [window.stopped, document.getElementById('meeting').textContent.slice(0, 80)]"))
        logged = browser.execute_script("return window.logged")
        check("a throw in the drawing: logged, with its kind", bool(logged) and logged[0]["kind"] == "ScenarioAS", logged)
        browser.execute_script("showMeeting = window.__drawn; window.restore()")
        again = browser.find_element(By.ID, "pane-again")
        check("tried again: offered, its words", again.is_displayed() and again.text == "Réessayer", again.text)
        again.click()
        check("tried again: drawn, the sentence and the button gone", wait_for(browser, lambda b: b.execute_script(page_of) == asked.name and b.execute_script(said) == "" and not b.find_element(By.ID, "pane-again").is_displayed(), 5))
        # The list refusing: a way out and a working retry, at each width.
        for size_, narrow in (((1100, 800), False), ((420, 400), True)):
            browser.set_window_size(*size_)
            wait_for(browser, lambda b: b.execute_script("return innerWidth < 840") == narrow, 5)
            at_page(browser, left)
            browser.execute_script(late, 99, 0)
            browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
            told = wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
            place = "const r = document.getElementById('pane-again').getBoundingClientRect(); return r.width > 0 && r.top >= 0 && r.bottom <= innerHeight"
            out = "#pane-back" if narrow else "#groups .session"
            check(f"`listed` refused at {size_[0]}: said, retry in sight, a way out, nothing of the page left",
                  told and browser.execute_script(place) and browser.find_element(By.CSS_SELECTOR, out).is_displayed() and browser.execute_script("return document.getElementById('meeting').childElementCount") == 0)
            # The list's own address, still refused: the sentence, retry in sight; the retry reads the list once it reads.
            browser.execute_script("location.hash = '#/'")
            told = wait_for(browser, lambda b: b.execute_script("return location.hash") == "#/" and b.execute_script(said) == not_shown, 5)
            check(f"`#/` with `listed` refused at {size_[0]}: said, retry in sight", told and browser.execute_script(place), browser.execute_script(said))
            browser.execute_script("window.refusing = 0; window.restore()")
            browser.find_element(By.ID, "pane-again").click()
            check(f"retried at {size_[0]}: the list and no sentence", wait_for(browser, lambda b: b.execute_script(said) == "" and b.find_elements(By.CSS_SELECTOR, "#groups .session")[0].is_displayed(), 5))
            if narrow:
                # On the list itself, shown: a drawing that fails there says so above it.
                browser.execute_script(late, 1, 0)
                browser.execute_script("window.dispatchEvent(new Event('focus'))")
                wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
                above = ("const y = (e) => e.getBoundingClientRect(); const p = y(document.getElementById('pane-said')), a = y(document.getElementById('pane-again')), r = y(document.querySelector('#groups .session'));"
                         " return [p.top >= 0, p.bottom <= a.top, a.bottom <= r.top, r.height > 0]")
                check("at 420, on the list: the sentence, its retry, then the list", browser.execute_script(above) == [True, True, True, True], browser.execute_script(above))
                browser.execute_script("window.restore()")
                browser.find_element(By.ID, "pane-again").click()
                wait_for(browser, lambda b: b.execute_script(said) == "", 5)
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: b.execute_script("return innerWidth >= 840"), 5)
        # That sentence no longer the last word: a meeting then gone keeps its own, its retry gone.
        at_page(browser, left)
        browser.execute_script(late, 1, 0)
        browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
        wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
        browser.execute_script("window.restore()")
        browser.execute_script("location.hash = '#/meeting/gone'")
        wait_for(browser, lambda b: b.execute_script("return location.hash") == "#/" and b.execute_script(page_of) is not None, 5)
        check("then a meeting gone: its own sentence, said and kept, no retry", browser.execute_script(said).startswith("Cette réunion n’a pas pu être ouverte") and not browser.find_element(By.ID, "pane-again").is_displayed(), browser.execute_script(said))
        # A failure's row and link: nothing marked for a page no longer shown.
        at_page(browser, left)
        browser.execute_script(late, 1, 0)
        browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
        wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
        browser.execute_script("window.restore()")
        check("a failure: no row left marked", browser.execute_script("return document.querySelectorAll('#groups .session[aria-current]').length") == 0)
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "sessions"), 5)
        browser.execute_script(late, 1, 0)
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
        browser.execute_script("window.restore()")
        check("a failure: Réglages no longer marked", browser.execute_script("return document.getElementById('settings-link').getAttribute('aria-current')") != "page")
        browser.find_element(By.ID, "pane-again").click()
        wait_for(browser, lambda b: b.execute_script(said) == "", 5)
        # An address that is none: the page goes, the drawing's failure says so.
        at_page(browser, left)
        browser.execute_script("location.hash = '#/meeting/%E0%A4%A'")
        told = wait_for(browser, lambda b: b.execute_script(said) == not_shown, 5)
        check("an address that does not decode: the page gone, said", told and browser.execute_script("return document.getElementById('meeting').childElementCount") == 0, browser.execute_script(said))
        # Settings: a link to where it is draws nothing, what is typed stays; left and back, nothing kept, as ever.
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "sessions"), 5)
        browser.execute_script("const f = document.getElementById('sessions'); f.value = '/typed'; f.dispatchEvent(new Event('input'))")
        browser.execute_script("const real = window.__TAURI__.core; window.drawn = 0; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => { if (asked === 'settings_shown') window.drawn += 1; return real.invoke(asked, ...rest); } }; window.restore = () => { window.__TAURI__.core = real; };")
        browser.find_element(By.ID, "settings-link").click()
        # Nothing to wait for if it holds: a drawing, had it come, comes within this.
        wait_for(browser, lambda b: b.execute_script("return window.drawn") > 0, 0.7)
        drawing_ = browser.execute_script("return [window.drawn, document.getElementById('sessions').value]")
        browser.execute_script("window.restore()")
        check("Réglages clicked again: nothing drawn, what is typed stays", drawing_ == [0, "/typed"], drawing_)
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.execute_script(page_of) is not None, 5)
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "sessions"), 5)
        check("left and back: what was typed and not saved is gone, as ever", browser.execute_script("return document.getElementById('sessions').value") != "/typed")
    # The agreement page: Envoyer is a send.
    for key in (None, Keys.ENTER, Keys.SPACE):
        window = Window(meetings, "fr_FR.UTF-8", service)
        with window as browser:
            browser.set_window_size(1100, 800)
            wait_for(browser, lambda b: b.execute_script(page_of) is not None, 10)
            browser.execute_script(f"location.hash = '#/agree/{left.name}'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "agree-send"), 10)
            args.unlink(missing_ok=True)
            found = browser.find_element(By.ID, "agree-send")
            browser.execute_script("arguments[0].focus()", found)
            browser.execute_script(late)
            browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
            pressed(browser, found, key)
            drawn = wait_for(browser, lambda b: b.execute_script(page_of) == asked.name, 5)
            browser.execute_script("window.restore()")
            check(f"Envoyer ({'pointer' if key is None else 'key'}) while another meeting is asked for: nothing sent, nothing agreed to",
                  drawn and nothing_sent() and "minutes_agreed" not in window.file.read_text(), window.file.read_text())
    # Pages that act on no meeting act all the same: a press on them while `listed` holds the way out.
    held = ("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: async (asked, ...rest) => {"
            " if (asked === 'listed') await new Promise((r) => setTimeout(r, 1000)); return real.invoke(asked, ...rest); } }; window.restore = () => { window.__TAURI__.core = real; };")
    for key in (None, Keys.ENTER):
        window = Window(meetings, "fr_FR.UTF-8", service)
        with window as browser:
            browser.set_window_size(1100, 800)
            wait_for(browser, lambda b: b.execute_script(page_of) is not None, 10)
            browser.execute_script("location.hash = '#/free'")
            wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#free .freeable input"), 10)
            browser.execute_script("document.querySelector('#free .freeable input').click()")
            wait_for(browser, lambda b: b.execute_script("return document.getElementById('free-do')?.disabled === false"), 5)
            files = sorted(meetings.glob("*/local.wav"))
            found = browser.find_element(By.ID, "free-do")
            browser.execute_script("arguments[0].focus()", found)
            browser.execute_script(held)
            browser.execute_script("location.hash = '#/'")
            pressed(browser, found, key)
            during = browser.execute_script("return document.getElementById('free').childElementCount")
            settled = wait_for(browser, lambda b: b.execute_script(page_of) is not None, 5)
            browser.execute_script("window.restore()")
            check(f"Libérer ({'pointer' if key is None else 'key'}) while another page is asked for: the file still there",
                  settled and during == 0 and all(file.exists() for file in files), (during, [file.parent.name for file in files if not file.exists()]))
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "reminder"), 10)
            before = window.file.read_text()
            found = browser.find_element(By.ID, "reminder")
            browser.execute_script("arguments[0].focus()", found)
            browser.execute_script(held)
            browser.execute_script("location.hash = '#/'")
            pressed(browser, found, Keys.SPACE if key else None)
            during = browser.execute_script("return document.getElementById('settings').childElementCount")
            settled = wait_for(browser, lambda b: b.execute_script(page_of) is not None, 5)
            browser.execute_script("window.restore()")
            check(f"a setting ({'pointer' if key is None else 'key'}) while another page is asked for: not changed",
                  settled and during == 0 and window.file.read_text() == before, during)
            # Coming back: each is drawn again.
            browser.execute_script("location.hash = '#/free'")
            free_back = wait_for(browser, lambda b: b.find_elements(By.ID, "free-do"), 5)
            browser.execute_script("location.hash = '#/settings'")
            check("left, then back: Libérer and Réglages are drawn again", free_back and wait_for(browser, lambda b: b.find_elements(By.ID, "reminder"), 5))
    # A name that needs encoding, at its own address as its row writes it: kept all the same.
    accented = WORK / "pressed-accented"
    named_so = "Réunion d'été (1) été"
    session(accented / named_so, now - dt.timedelta(days=1), 0.1, {"local.wav": 1000})
    with Window(accented, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: b.execute_script("return document.getElementById('meeting').dataset.name") == named_so and b.find_elements(By.ID, "context-text"), 10)
        written_so = browser.execute_script("return document.querySelector('#groups .session').getAttribute('href')")
        context_opened(browser)
        browser.execute_script("window.__kept = document.getElementById('context-text'); window.__kept.focus(); location.hash = arguments[0]", written_so)
        same = wait_for(browser, lambda b: b.execute_script("return document.getElementById('meeting').dataset.at === location.hash"), 5)
        check("a name that needs encoding, at its own address: the page kept, its field and the focus with it",
              same and written_so != f"#/meeting/{named_so}" and browser.execute_script("return window.__kept.isConnected && document.activeElement === window.__kept"), (written_so, browser.execute_script("return location.hash")))
    # What is agreed to names its meeting, as its page and the list do.
    for language, says, sentence in (("fr_FR.UTF-8", "Le texte de « {} » va quitter votre ordinateur", not_shown), ("en_US.UTF-8", "The text of “{}” is about to leave your computer", "This page could not be shown: try again.")):
        with Window(meetings, language, service) as browser:
            browser.set_window_size(1100, 800)
            for folder in (left, asked):
                at_page(browser, folder)
                named = browser.find_element(By.ID, "rename").text
                browser.execute_script(f"location.hash = '#/agree/{folder.name}'")
                wait_for(browser, lambda b: b.execute_script("return document.querySelector('#agree h1')?.textContent") == says.format(named), 10)
                title = browser.execute_script("return document.querySelector('#agree h1').textContent")
                check(f"the agreement names its meeting: {'its title' if folder == left else 'its date'} ({language})", title == says.format(named) and named != "", title)
            at_page(browser, left)
            browser.execute_script(late, 1, 0)
            browser.execute_script(f"location.hash = '#/meeting/{asked.name}'")
            check(f"a page not shown: said ({language})", wait_for(browser, lambda b: b.execute_script(said) == sentence, 5), browser.execute_script(said))
            browser.execute_script("window.restore()")
            # A folder the list does not hold: named by its folder, never by nothing.
            odd = meetings / "hors-liste"
            odd.mkdir(exist_ok=True)
            (odd / "session.json").write_text('{"contract":"another/v1"}')
            (odd / "transcript.jsonl").write_text((left / "transcript.jsonl").read_text())
            browser.execute_script("location.hash = '#/agree/hors-liste'")
            wait_for(browser, lambda b: b.execute_script("return document.querySelector('#agree h1')?.textContent") == says.format("hors-liste"), 10)
            title = browser.execute_script("return document.querySelector('#agree h1').textContent")
            check(f"a folder the list does not hold: named by its folder ({language})", title == says.format("hors-liste"), title)


def context_written(now: dt.datetime) -> None:
    """O. A meeting's context (D59): written during a recording, by the keyboard, heard by the
    transcript and the minutes; changed after, made again with it; the field closed meanwhile."""
    print("O. a meeting's context: written while recording, read by the transcript and the minutes, made again with it (D59)")
    keys = lambda browser, *sent: ActionChains(browser).send_keys(*sent).perform()
    focused = lambda b: b.execute_script("return document.activeElement.id")
    fits = lambda b: b.execute_script("const e = document.scrollingElement; return [e.scrollHeight, e.clientHeight];")
    count = lambda b, id_: b.execute_script("return document.getElementById(arguments[0]).textContent", f"{id_}-count")
    model = WORK / "context-model.bin"
    model.write_bytes(b"\0" * 16)
    settings = (f'after = "minutes"\nreminder = false\nmodel = "{model}"\n'
                'minutes_provider = "http://127.0.0.1:9/v1"\nminutes_model = "mistral"\n')

    def seen(browser, name: str) -> None:
        """A screenshot in the theme its name says."""
        time.sleep(0.3)
        shot_themed(browser, name, "light" if "light" in name.split("-") else "dark")

    def recording(name: str, language: str, dark: bool = False) -> tuple["Window", Path]:
        meetings = WORK / f"context-{name}"
        meetings.mkdir()
        window = Window(meetings, language, settings, small=True)
        if dark:
            window.env.update(GTK_THEME="Adwaita:dark")
        return window, meetings / "2026-10-18_12-00"

    def to_field(browser) -> bool:
        """By the keyboard alone: Tab to « Plus d’options », Enter, then Tab to the context's field (D90)."""
        for _ in range(6):
            if focused(browser) == "small-more":
                break
            keys(browser, Keys.TAB)
        if focused(browser) != "small-more":
            return False
        keys(browser, Keys.ENTER)
        for _ in range(6):
            if focused(browser) == "small-context":
                return True
            keys(browser, Keys.TAB)
        return focused(browser) == "small-context"

    said = "Budget du T4 avec Inès Durand, Jiří Novák et l’équipe achats."
    window, meeting = recording("fr", "fr_FR.UTF-8")
    with window as browser:
        # Started by the keyboard: the recording's gesture has the focus at rest — once the page has it.
        wait_for(browser, lambda b: focused(b) == "small-record", 10)
        keys(browser, Keys.ENTER)
        started = wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        more = browser.find_element(By.ID, "small-more")
        check("recording, the bar offers the context under « Plus d’options », closed", started and more.get_attribute("aria-label") == "Plus d’options"
              and more.get_attribute("aria-expanded") == "false" and not browser.find_element(By.ID, "small-context").is_displayed(), more.get_attribute("aria-label"))
        time.sleep(0.4)
        seen(browser, "d59-1-small-closed-light")
        check("closed, it holds in its window", fits(browser)[0] <= fits(browser)[1], fits(browser))
        reached = to_field(browser)
        opened = wait_for(browser, lambda b: focused(b) == "small-context", 2)
        area = browser.find_element(By.ID, "small-context")
        check("the options opened by Enter, the field reached by Tab, named by its label",
              reached and opened and area.get_attribute("aria-label") == "De quoi parle la réunion ? Qui y participe ?"
              and area.get_attribute("placeholder") == area.get_attribute("aria-label"), (reached, focused(browser), area.get_attribute("aria-label")))
        check("open, the note on the transcript under it, the bar growing to hold both", browser.find_element(By.ID, "small-note").is_displayed())
        check("its count is said where a screen reader hears it, before a word is typed",
              browser.find_element(By.ID, "small-context-count").get_attribute("role") == "status"
              and area.get_attribute("aria-describedby") == "small-context-count"
              and count(browser, "small-context") == "400 caractères au plus · 400 restants", count(browser, "small-context"))
        keys(browser, said)
        left = 400 - len(said)
        counted = wait_for(browser, lambda b: count(b, "small-context") == f"400 caractères au plus · {left} restants", 2)
        check("the count follows each character typed", counted, count(browser, "small-context"))
        kept = wait_for(browser, lambda b: (meeting / "context.txt").exists() and (meeting / "context.txt").read_text() == said, 3)
        check("what is typed is the meeting's context.txt, whole, as typed", kept, (meeting / "context.txt").read_text() if (meeting / "context.txt").exists() else None)
        check("nothing is left beside it", sorted(p.name for p in meeting.iterdir()) == ["context.txt", "session.json"], sorted(p.name for p in meeting.iterdir()))
        time.sleep(0.3)
        check("open and written, it holds in the bar — 440 wide, as tall as it shows", fits(browser)[0] <= fits(browser)[1]
              and browser.execute_script("return innerWidth") == 440, (fits(browser), browser.execute_script("return [innerWidth, innerHeight]")))
        seen(browser, "d59-2-small-open-light")
        # Another script: said in the count, the window still whole.
        keys(browser, " Γιώργος Παπαδόπουλος.")
        other = wait_for(browser, lambda b: "Dans cette écriture" in count(b, "small-context"), 2)
        time.sleep(0.3)
        check("a Greek name is said: Whisper may keep only its end, a service may refuse it", other
              and count(browser, "small-context").startswith(f"{left - len(' Γιώργος Παπαδόπουλος.')} restants"), count(browser, "small-context"))
        check("said, it still holds in its window", fits(browser)[0] <= fits(browser)[1], fits(browser))
        seen(browser, "d59-3-small-script-light")
        for _ in range(len(" Γιώργος Παπαδόπουλος.")):
            keys(browser, Keys.BACKSPACE)
        back = wait_for(browser, lambda b: (meeting / "context.txt").read_text() == said and "Dans cette écriture" not in count(b, "small-context"), 3)
        check("taken out, the context is as it was, the script no longer said", back, (meeting / "context.txt").read_text())
        keys(browser, Keys.ESCAPE)
        closed = wait_for(browser, lambda b: focused(b) == "small-more", 2)
        check("Escape closes the options, the focus back on « Plus d’options »",
              closed and browser.find_element(By.ID, "small-more").get_attribute("aria-expanded") == "false" and (meeting / "context.txt").read_text() == said, focused(browser))
        stop_from_small(browser)
        heard = wait_for(browser, lambda b: (meeting / "minutes.md").exists(), 15)
        header = json.loads((meeting / "transcript.jsonl").read_text().splitlines()[0]) if (meeting / "transcript.jsonl").exists() else {}
        check("the transcript made at the stop heard it", header.get("context") == said, header)
        asked = (WORK / "summary-context").read_text() if (WORK / "summary-context").exists() else None
        check("the minutes' request carried it", heard and asked is not None and json.loads(asked) == said, asked)

    # The meeting's page: the context shown, changed, made again with it — from its audio, which the
    # stand-in capture does not write: a track of its own, numbers only.
    (meeting / "local.wav").write_bytes(b"\0" * 1000)
    window = Window(meeting.parent, "fr_FR.UTF-8", settings)
    with window as browser:
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        shown = wait_for(browser, lambda b: b.find_elements(By.ID, "context-text"), 10)
        area = context_opened(browser)
        label = browser.execute_script("return document.querySelector('label[for=context-text]')?.textContent")
        check("a meeting's page shows its context, labelled", shown and area.get_attribute("value") == said
              and label == "De quoi parle la réunion ? Qui y participe ?", (area.get_attribute("value"), label))
        check("made with it, nothing is offered to make again", not browser.find_element(By.ID, "context-changed").is_displayed())
        more = " Siobhán O’Neill (juridique)."
        area.click()
        keys(browser, Keys.END, more)
        changed = wait_for(browser, lambda b: b.find_element(By.ID, "context-changed").is_displayed(), 3)
        check("changed after its transcript and minutes, Refaire avec ce contexte is offered, said why",
              changed and (meeting / "context.txt").read_text() == said + more
              and browser.find_element(By.CSS_SELECTOR, "#context-changed .says").text == "La transcription et le compte rendu ont été faits avec un autre contexte."
              and browser.find_element(By.ID, "context-redo").text == "Refaire avec ce contexte", (meeting / "context.txt").read_text())
        seen(browser, "d59-4-page-changed-light")
        # Made again, slowly: the field closed meanwhile, and said.
        (WORK / "slow-hearing").touch()
        try:
            browser.find_element(By.ID, "context-redo").send_keys(Keys.ENTER)
            making = wait_for(browser, lambda b: b.find_element(By.ID, "context-text").get_attribute("readonly") is not None, 5)
            sentence = browser.find_element(By.ID, "context-said").text
            check("while it is made again, the field is closed, and says so",
                  making and sentence == "Le contexte ne change pas pendant que la transcription ou le compte rendu se fait."
                  and not browser.find_element(By.ID, "context-changed").is_displayed(), sentence)
            seen(browser, "d59-5-page-closed-light")
            browser.find_element(By.ID, "context-text").send_keys(" X")
            check("closed, nothing typed is taken", (meeting / "context.txt").read_text() == said + more)
            # The shell itself refuses, not only the page.
            refused = browser.execute_async_script(
                "const done = arguments[arguments.length - 1], name = arguments[0];"
                "Promise.all([window.__TAURI__.core.invoke('context_saved', {name, text: 'X'}).then(() => 'saved', () => 'refused'),"
                "window.__TAURI__.core.invoke('context_redo', {name}).then(() => 'redone', () => 'refused')]).then(done)",
                meeting.name,
            )
            check("and the shell refuses a context written or made again meanwhile",
                  refused == ["refused", "refused"] and (meeting / "context.txt").read_text() == said + more, refused)
        finally:
            (WORK / "slow-hearing").unlink()
        remade = wait_for(browser, lambda b: json.loads((meeting / "minutes.md").read_text().split("context: ", 1)[1].split("\n", 1)[0]) == said + more, 20)
        header = json.loads((meeting / "transcript.jsonl").read_text().splitlines()[0])
        check("made again: the transcript, then the minutes, with the context as it is now",
              remade and header.get("context") == said + more and json.loads((WORK / "summary-context").read_text()) == said + more, header)
        check("the transcript set aside is not left beside the new one", not list(meeting.glob("transcript-before-*")), sorted(p.name for p in meeting.iterdir()))
        reopened = wait_for(browser, lambda b: b.find_element(By.ID, "context-text").get_attribute("readonly") is None
                            and not b.find_element(By.ID, "context-changed").is_displayed(), 10)
        check("made, the field opens again, nothing left to make again", reopened)
        # Made again and failing: the transcript and the minutes there put back, never one of another context.
        before = (meeting / "transcript.jsonl").read_text(), (meeting / "minutes.md").read_text()
        browser.find_element(By.ID, "context-text").send_keys(Keys.END, " Et Lena.")
        wait_for(browser, lambda b: b.find_element(By.ID, "context-changed").is_displayed(), 3)
        (WORK / "fail-hearing").touch()
        try:
            press(browser, "context-redo")
            failed = wait_for(browser, lambda b: b.find_elements(By.ID, "transcript-again"), 10)
        finally:
            (WORK / "fail-hearing").unlink()
        check("failing, the transcript and the minutes made with the earlier context are put back",
              failed and ((meeting / "transcript.jsonl").read_text(), (meeting / "minutes.md").read_text()) == before
              and not list(meeting.glob("transcript-before-*")), sorted(p.name for p in meeting.iterdir()))
        check("and Refaire avec ce contexte is offered again", wait_for(browser, lambda b: b.find_element(By.ID, "context-changed").is_displayed(), 5))
        # Pasted past the most: cut to it, as the shell counts — in characters. As WebKit pastes: a
        # beforeinput that may refuse it, then the text in and an input.
        browser.execute_script(
            "const a = document.getElementById('context-text'), data = '🙂'.repeat(10) + 'é'.repeat(440);"
            "const asked = new InputEvent('beforeinput', {inputType: 'insertFromPaste', data, cancelable: true});"
            "if (a.dispatchEvent(asked)) { a.value = data; a.dispatchEvent(new InputEvent('input', {inputType: 'insertFromPaste', data})) }"
        )
        cut = wait_for(browser, lambda b: len((meeting / "context.txt").read_text()) == 400, 3)
        check("a paste past 400 characters is cut to 400, in the field and the file",
              cut and len(browser.find_element(By.ID, "context-text").get_attribute("value")) == 400
              and count(browser, "context-text") == "400 caractères au plus · 0 restant", count(browser, "context-text"))
        full_ = (meeting / "context.txt").read_text()
        browser.find_element(By.ID, "context-text").send_keys(Keys.END, "x")
        browser.find_element(By.ID, "context-text").send_keys(Keys.CONTROL, Keys.HOME)
        browser.find_element(By.ID, "context-text").send_keys("x")
        time.sleep(0.5)
        check("at 400, a character typed is not taken, at the end or at the start — nothing of the rest pushed out",
              (meeting / "context.txt").read_text() == full_ == browser.find_element(By.ID, "context-text").get_attribute("value")
              and len(full_) == 400 and full_.startswith("🙂"), len(full_))

    # A transcript left unfinished: the field closed, said so.
    unfinished = WORK / "context-unfinished"
    session(unfinished / "half", now - dt.timedelta(hours=2), 3, {"local.wav": 1000})
    (unfinished / "half" / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Bonjour."}\n'
    )
    with Window(unfinished, "fr_FR.UTF-8", settings) as browser:
        browser.execute_script("location.hash = '#/meeting/half'")
        closed = wait_for(browser, lambda b: b.find_elements(By.ID, "context-text") and b.find_element(By.ID, "context-text").get_attribute("readonly") is not None, 10)
        # Read as written: WebDriver's text makes a no-break space a plain one.
        sentence = browser.execute_script("return document.getElementById('context-said').textContent") if closed else ""
        check("a transcript left unfinished closes the field, said with its way out",
              sentence == "La transcription n’est pas terminée : terminez-la (Réessayer) avant de changer le contexte.", sentence)

    # In English, in the dark theme: the small window and a meeting's page.
    window, meeting = recording("en", "en_GB.UTF-8", dark=True)
    with window as browser:
        wait_for(browser, lambda b: focused(b) == "small-record", 10)
        keys(browser, Keys.ENTER)
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        more = browser.find_element(By.ID, "small-more").get_attribute("aria-label")
        check("in English, the context is offered under « More options »", more == "More options", more)
        time.sleep(0.4)
        seen(browser, "d59-1-small-closed-dark")
        to_field(browser)
        wait_for(browser, lambda b: focused(b) == "small-context", 2)
        english = "Q4 budget with Inès Durand and Jiří Novák."
        keys(browser, english)
        counted = wait_for(browser, lambda b: count(b, "small-context") == f"400 characters at most · {400 - len(english)} left", 2)
        kept = wait_for(browser, lambda b: (meeting / "context.txt").exists() and (meeting / "context.txt").read_text() == english, 3)
        time.sleep(0.3)
        check("in English, counted, kept, and holding in its window", counted and kept and fits(browser)[0] <= fits(browser)[1],
              (count(browser, "small-context"), fits(browser)))
        seen(browser, "d59-2-small-open-dark")
        keys(browser, " 王小明")
        wait_for(browser, lambda b: "In this script" in count(b, "small-context"), 2)
        time.sleep(0.3)
        check("a Chinese name is said, in English, the window whole", "In this script" in count(browser, "small-context")
              and fits(browser)[0] <= fits(browser)[1], (count(browser, "small-context"), fits(browser)))
        seen(browser, "d59-3-small-script-dark")
        stop_from_small(browser)
        wait_for(browser, lambda b: (meeting / "minutes.md").exists(), 15)
    (meeting / "local.wav").write_bytes(b"\0" * 1000)
    window = Window(meeting.parent, "en_GB.UTF-8", settings)
    window.env.update(GTK_THEME="Adwaita:dark")
    with window as browser:
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "context-text"), 10)
        context_opened(browser).send_keys(Keys.END, " And Lena.")
        changed = wait_for(browser, lambda b: b.find_element(By.ID, "context-changed").is_displayed(), 3)
        check("in English, a meeting's page offers Redo with this context", changed
              and browser.find_element(By.ID, "context-redo").text == "Redo with this context", browser.find_element(By.ID, "context-redo").text)
        seen(browser, "d59-4-page-changed-dark")


def context_gaps(now: dt.datetime) -> None:
    """AH. What D59's criteria said and O did not run: the full window's page of the meeting being
    recorded, a Cyrillic name and an emoji typed, nothing of it leaving in private mode, « Refaire
    avec ce contexte » by the engine the settings name, the minutes set aside put back when they
    fail, nothing overflowing sideways at 380 × 400."""
    print("AH. a meeting's context: on the page being recorded, kept here in private mode, made again by the settings' engine (D59)")
    keys = lambda browser, *sent: ActionChains(browser).send_keys(*sent).perform()
    model = WORK / "context-gaps-model.bin"
    model.write_bytes(b"\0" * 16)
    here = f'after = "minutes"\nreminder = false\nmodel = "{model}"\nminutes_provider = "http://127.0.0.1:9/v1"\nminutes_model = "mistral"\n'
    hearing, writing = WORK / "hearing-args", WORK / "summary-args"
    meetings = WORK / "context-gaps"
    meetings.mkdir()
    meeting = meetings / "2026-10-18_12-00"
    typed = "Иван Петров и Ольга 🙂"
    hearing.unlink(missing_ok=True)
    writing.unlink(missing_ok=True)
    with Window(meetings, "fr_FR.UTF-8", here) as browser:
        live = lambda b: "live" in (b.find_element(By.ID, "record").get_attribute("class") or "")
        browser.find_element(By.ID, "record").click()
        wait_for(browser, lambda b: (meeting / "session.json").exists() and live(b), 10)
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        open_ = wait_for(browser, lambda b: b.find_elements(By.ID, "context-text")
                         and b.find_element(By.ID, "context-text").get_attribute("readonly") is None, 10)
        context_opened(browser).click()
        keys(browser, typed)
        kept = wait_for(browser, lambda b: (meeting / "context.txt").exists() and (meeting / "context.txt").read_text() == typed, 3)
        check("the full window's page of the meeting being recorded offers its context: a Cyrillic name and an emoji kept whole",
              open_ and kept, (meeting / "context.txt").read_text() if (meeting / "context.txt").exists() else None)
        # The button says it at the click now (D68); a press within half a second is the same press.
        time.sleep(0.6)
        browser.find_element(By.ID, "record").click()
        stopped = wait_for(browser, lambda b: "stopped" in (meeting / "session.json").read_text() and not live(b), 10)
        made = wait_for(browser, lambda b: (meeting / "minutes.md").exists() and writing.exists(), 20)
        header = json.loads((meeting / "transcript.jsonl").read_text().splitlines()[0]) if (meeting / "transcript.jsonl").exists() else {}
        check("in private mode the context goes to Whisper and Ollama on this computer alone",
              made and header.get("context") == typed and "--provider" not in hearing.read_text()
              and "--provider http://127.0.0.1:9/v1" in writing.read_text(),
              (stopped, sorted(p.name for p in meeting.iterdir()), hearing.read_text() if hearing.exists() else "", writing.read_text() if writing.exists() else ""))
        if not made:
            return
    # A service the settings name hears from now on: « Refaire avec ce contexte » by it, the minutes failing.
    (meeting / "local.wav").write_bytes(b"\0" * 1000)
    service = here + 'engine = "service"\ntranscription_provider = "http://127.0.0.1:9/v1"\ntranscription_model = "hears"\n'
    with Window(meetings, "fr_FR.UTF-8", service) as browser:
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "context-text"), 10)
        minutes = (meeting / "minutes.md").read_text()
        context_opened(browser).send_keys(Keys.END, " Et Lena.")
        wait_for(browser, lambda b: b.find_element(By.ID, "context-changed").is_displayed(), 3)
        hearing.unlink(missing_ok=True)
        writing.unlink(missing_ok=True)
        (WORK / "fail-minutes").touch()
        try:
            press(browser, "context-redo")
            def context_heard() -> object:
                """The context the transcript's header names — none while it is set aside and made again."""
                try:
                    return json.loads((meeting / "transcript.jsonl").read_text().splitlines()[0]).get("context")
                except (FileNotFoundError, IndexError, ValueError):
                    return None

            heard = wait_for(browser, lambda b: hearing.exists() and context_heard() == typed + " Et Lena.", 15)
            # Asked of the step, which fails: then nothing is being written, and nothing left set aside.
            failed = wait_for(browser, lambda b: writing.exists() and b.execute_script(
                "return window.__TAURI__.core.invoke('makings').then(m => m.minutes === null && m.minutes_waiting.length === 0)") and not list(meeting.glob("minutes-before-*")), 15)
        finally:
            (WORK / "fail-minutes").unlink()
        check("« Refaire avec ce contexte » hears by the engine the settings name: the service",
              heard and "--provider http://127.0.0.1:9/v1" in hearing.read_text() and "--model hears" in hearing.read_text(), hearing.read_text() if hearing.exists() else "")
        check("its minutes failing, the ones set aside are put back", failed and (meeting / "minutes.md").read_text() == minutes,
              sorted(p.name for p in meeting.iterdir()))
    # The small window at 380 × 400, the field open and full of a long name: nothing sideways.
    small = WORK / "context-gaps-small"
    small.mkdir()
    with Window(small, "fr_FR.UTF-8", here, small=True) as browser:
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-record"), 10)
        browser.find_element(By.ID, "small-record").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        browser.find_element(By.ID, "small-more").click()
        wait_for(browser, lambda b: b.find_element(By.ID, "small-context").is_displayed(), 2)
        browser.find_element(By.ID, "small-context").send_keys("Константинопольская-Переяславская " * 3)
        time.sleep(0.5)
        wide = browser.execute_script("const e = document.scrollingElement; return [e.scrollWidth, e.clientWidth, innerWidth]")
        check("440 wide, the field open and a long name typed, nothing overflows sideways", wide[0] <= wide[1] and wide[2] == 440, wide)
        stop_from_small(browser)


def confidential_kept(now: dt.datetime) -> None:
    """AA. A meeting said confidential (D60): with a service named in the settings to hear and to
    write, nothing is asked of it — a stand-in service counts its requests —; the meeting is made
    here, by Whisper and Ollama; turned on while a service makes it, after it wrote, or while its
    agreement is asked, made again here and what had left said; turned off, the settings choose
    again."""
    print("AA. a meeting said confidential: nothing sent to the services the settings name, made here; turned off, the settings choose again (D60)")
    counted = {"hear": 0, "write": 0}
    ollama = {"held": ["mistral:7b", "qwen3:8b"], "silent": False}

    # The services the settings name, stood in: each request the steps send them is counted.
    class Service(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_POST(self):
            counted[self.path.strip("/")] += 1
            self.rfile.read(int(self.headers.get("content-length") or 0))
            self.send_response(200)
            self.send_header("content-length", "0")
            self.end_headers()

        def log_message(self, *_):
            pass

    # Ollama on this computer: its models; silent when the scenario says so; a pull that is whole.
    class Ollama(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"

        def do_GET(self):
            body = json.dumps({"models": [{"name": name} for name in ollama["held"]]}).encode()
            self.send_response(500 if ollama["silent"] else 200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self):
            self.rfile.read(int(self.headers.get("content-length") or 0))
            ollama["held"].append("qwen3:8b")
            self.send_response(200)
            self.send_header("content-type", "application/x-ndjson")
            self.end_headers()
            self.wfile.write(b'{"status":"success"}\n')

        def log_message(self, *_):
            pass

    servers = [http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) for handler in (Service, Ollama)]
    for server in servers:
        threading.Thread(target=server.serve_forever, daemon=True).start()
    standin, base = (f"http://127.0.0.1:{server.server_port}" for server in servers)
    model = WORK / "confidential-model.bin"
    model.write_bytes(b"\0" * 16)
    named = (f'after = "minutes"\nreminder = false\nmodel = "{model}"\n'
             'engine = "service"\ntranscription_provider = "https://hears.example/v1"\ntranscription_model = "whisper-1"\n'
             'transcription_agreed = "hears.example"\nminutes_provider = "https://writes.example/v1"\nminutes_model = "gpt-4.1-mini"\n')
    agreed = named + 'minutes_agreed = "writes.example"\n'
    fits = lambda b: b.execute_script("const e = document.scrollingElement; return [e.scrollHeight, e.clientHeight];")
    front = lambda folder, key: next((line.split(": ", 1)[1] for line in (folder / "minutes.md").read_text().splitlines()
                                      if line.startswith(f"{key}: ")), None) if (folder / "minutes.md").exists() else None
    engine = lambda folder: json.loads((folder / "transcript.jsonl").read_text().splitlines()[0]).get("engine") \
        if (folder / "transcript.jsonl").exists() and (folder / "transcript.jsonl").read_text().strip() else None
    names = lambda folder: sorted(p.name for p in folder.iterdir())
    switch = lambda b, id_: b.find_element(By.ID, id_)
    focused = lambda b: b.execute_script("return document.activeElement.id")
    left = "return document.getElementById('confidential-left')?.hidden ? '' : (document.getElementById('confidential-left')?.innerText || '')"

    def seen(browser, name: str) -> None:
        """A screenshot in the theme its name says."""
        time.sleep(0.3)
        shot_themed(browser, name, "light" if "light" in name.split("-") else "dark")

    def window(folder: Path, language: str, more: str, small: bool = False, dark: bool = False) -> "Window":
        made = Window(folder, language, more, ollama=base, small=small)
        made.env.update(STANDIN=standin)
        if dark:
            made.env.update(GTK_THEME="Adwaita:dark")
        return made

    # While recording, in the small window: said confidential by the keyboard, the file written.
    meetings = WORK / "confidential"
    meetings.mkdir()
    meeting = meetings / "2026-10-18_12-00"
    with window(meetings, "fr_FR.UTF-8", agreed, small=True) as browser:
        # Started by the keyboard, once the page has the focus.
        wait_for(browser, lambda b: focused(b) == "small-record", 10)
        ActionChains(browser).send_keys(Keys.ENTER).perform()
        started = wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        press(browser, "small-more")
        # The options drawn before they are read: the bar grows to hold them after the press.
        wait_for(browser, lambda b: b.find_element(By.CSS_SELECTOR, "#small .confidential").is_displayed(), 5)
        row = browser.find_element(By.CSS_SELECTOR, "#small .confidential")
        check("recording, the bar's options offer « Réunion confidentielle — rien ne quitte cet ordinateur », off",
              started and "Réunion confidentielle" in row.text and "Rien ne quitte cet ordinateur" in row.text
              and switch(browser, "small-confidential").get_attribute("role") == "switch"
              and not switch(browser, "small-confidential").is_selected(), row.text)
        check("the lock before its words is drawn, never read aloud",
              browser.execute_script("return document.querySelector('#small .confidential svg.lock')?.getAttribute('aria-hidden')") == "true")
        time.sleep(0.4)
        seen(browser, "d60-1-small-off-light")
        wait_for(browser, lambda b: (meeting / "session.json").exists(), 3)
        switch(browser, "small-confidential").send_keys(Keys.SPACE)
        marked = wait_for(browser, lambda b: (meeting / "confidential").exists(), 3)
        check("turned on by the keyboard, the meeting's file confidential is written, empty, nothing beside it",
              marked and (meeting / "confidential").read_bytes() == b"" and names(meeting) == ["confidential", "session.json"], names(meeting))
        time.sleep(0.3)
        check("on, with its context, the bar holds whole — 440 wide, as tall as it shows", fits(browser)[0] <= fits(browser)[1]
              and browser.execute_script("return innerWidth") == 440, (fits(browser), browser.execute_script("return [innerWidth, innerHeight]")))
        seen(browser, "d60-2-small-on-light")
        stop_from_small(browser)
        made = wait_for(browser, lambda b: front(meeting, "provider") == "local", 20)
        check("stopped: the transcript made by Whisper with the settings' model, the minutes by Ollama's recommended model",
              made and engine(meeting) == "whisper.cpp" and front(meeting, "model") == "qwen3:8b"
              and f"--provider {base}/v1" in (WORK / "summary-args").read_text(), (engine(meeting), front(meeting, "model")))
        check("the services the settings name count no request", counted == {"hear": 0, "write": 0}, counted)

    # In the full window: the list's lock; turned off, the settings choose again; on, made again here.
    with window(meetings, "fr_FR.UTF-8", agreed) as browser:
        chips = browser.execute_script("return [...document.querySelectorAll('.session .mark.locked')].map(c => [c.getAttribute('aria-label'), c.title, !!c.querySelector('svg')])")
        check("the list shows the meeting with a lock, its word its name and tooltip (D89)", chips == [["Confidentielle", "Confidentielle", True]], chips)
        seen(browser, "d60-4-list-light")
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        # The last meeting, drawn beside the list already: its page may be drawn again under the driver.
        shown = wait_for(browser, lambda b: b.find_elements(By.ID, "confidential-switch"), 10)
        check("its page says it, the switch on, nothing said to have left",
              shown and read(browser, lambda b: switch(b, "confidential-switch").is_selected())
              and browser.execute_script(left) == "")
        press(browser, "confidential-switch")
        off = wait_for(browser, lambda b: not (meeting / "confidential").exists(), 3)
        check("turned off, the file is removed", off and "confidential" not in names(meeting), names(meeting))
        (meeting / "minutes.md").unlink()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script(f"location.hash = '#/meeting/{meeting.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes"), 10)
        press(browser, "make-minutes")
        sent = wait_for(browser, lambda b: front(meeting, "provider") == "writes.example", 10)
        check("turned off, the settings choose again: the service named writes them", sent and counted["write"] == 1, counted)
        press(browser, "confidential-switch")
        again = wait_for(browser, lambda b: front(meeting, "provider") == "local", 10)
        said = wait_for(browser, lambda b: b.execute_script(left) == "Déjà parti avant\u00a0: la transcription vers writes.example.", 5)
        check("turned on after a service wrote them: written again here, nothing more sent",
              again and front(meeting, "model") == "qwen3:8b" and counted["write"] == 1, (front(meeting, "provider"), counted))
        check("and the page says what had left, to which host", said, browser.execute_script(left))
        seen(browser, "d60-5-page-left-light")

    # Turned on while a service hears it: ended, heard here, the minutes after it here too.
    heard = WORK / "confidential-heard"
    session(heard / "live", now - dt.timedelta(hours=1), 3, {"local.wav": 1000})
    (WORK / "slow-hearing").touch()
    try:
        with window(heard, "fr_FR.UTF-8", agreed) as browser:
            browser.execute_script("location.hash = '#/meeting/live'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "make-transcript"), 10)
            press(browser, "make-transcript")
            asked = wait_for(browser, lambda b: counted["hear"] == 1, 10)
            press(browser, "confidential-switch")
            remade = wait_for(browser, lambda b: front(heard / "live", "provider") == "local", 25)
            check("turned on while a service hears it: ended, heard here by Whisper, the minutes written here",
                  asked and remade and engine(heard / "live") == "whisper.cpp" and counted == {"hear": 1, "write": 1}, (engine(heard / "live"), counted))
            said = wait_for(browser, lambda b: b.execute_script(left) == "Déjà parti avant\u00a0: le son vers hears.example.", 5)
            check("the page says the sound had left, to which host", said, browser.execute_script(left))
            check("its end is no failure said", not browser.find_elements(By.ID, "transcript-again"))
    finally:
        (WORK / "slow-hearing").unlink()

    # Its agreement asked: no longer, made here.
    asking = WORK / "confidential-asked"
    session(asking / "asked", now - dt.timedelta(hours=2), 3, {"local.wav": 1000, "transcript.jsonl": 0})
    (asking / "asked" / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Bonjour."}\n{"type":"complete","duration":1.0}\n'
    )
    with window(asking, "fr_FR.UTF-8", named) as browser:
        browser.execute_script("location.hash = '#/meeting/asked'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes"), 10)
        press(browser, "make-minutes")
        agreeing = wait_for(browser, lambda b: b.execute_script("return location.hash") == "#/agree/asked", 5)
        browser.execute_script("location.hash = '#/meeting/asked'")
        waits = wait_for(browser, lambda b: b.find_elements(By.ID, "minutes-agree"), 10)
        check("a service not agreed to: the agreement is asked, nothing sent", agreeing and waits and counted["write"] == 1, counted)
        press(browser, "confidential-switch")
        here = wait_for(browser, lambda b: front(asking / "asked", "provider") == "local", 10)
        check("turned on while it is asked: no longer asked, written here, nothing sent",
              here and not browser.find_elements(By.ID, "minutes-agree") and counted["write"] == 1, counted)
        browser.execute_script("location.hash = '#/agree/asked'")
        back = wait_for(browser, lambda b: b.execute_script("return location.hash") == "#/", 5)
        check("its agreement page is no longer shown", back, browser.execute_script("return location.hash"))

    # Ollama silent, then holding no model: the minutes wait, said, its model offered.
    waiting = WORK / "confidential-waiting"
    session(waiting / "wait", now - dt.timedelta(hours=3), 3, {"local.wav": 1000, "transcript.jsonl": 0})
    (waiting / "wait" / "transcript.jsonl").write_text((asking / "asked" / "transcript.jsonl").read_text())
    (waiting / "wait" / "confidential").touch()
    ollama.update(held=[], silent=True)
    state = "return document.getElementById('minutes-state')?.innerText || ''"
    with window(waiting, "fr_FR.UTF-8", agreed) as browser:
        browser.execute_script("location.hash = '#/meeting/wait'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes"), 10)
        press(browser, "make-minutes")
        silent = wait_for(browser, lambda b: "Ollama ne répond pas sur cet ordinateur" in b.execute_script(state), 10)
        check("Ollama silent: the minutes wait, said where to get it, nothing asked nor sent",
              silent and browser.find_elements(By.ID, "minutes-again") and counted["write"] == 1
              and not (waiting / "wait" / "minutes.md").exists(), browser.execute_script(state))
        ollama["silent"] = False
        press(browser, "minutes-again")
        empty = wait_for(browser, lambda b: "Ollama n’a aucun modèle sur cet ordinateur" in b.execute_script(state), 10)
        offered = browser.find_elements(By.ID, "ollama-fetch")
        check("holding no model: said, its recommended model offered", empty and offered and offered[0].text == "Télécharger qwen3:8b",
              browser.execute_script(state))
        seen(browser, "d60-6-ollama-empty-light")
        press(browser, "ollama-fetch")
        written = wait_for(browser, lambda b: front(waiting / "wait", "model") == "qwen3:8b", 15)
        check("fetched, the minutes waiting are written by it, here", written and front(waiting / "wait", "provider") == "local"
              and counted["write"] == 1, counted)

    # In English, in the dark theme.
    english = WORK / "confidential-en"
    english.mkdir()
    with window(english, "en_GB.UTF-8", agreed, small=True, dark=True) as browser:
        wait_for(browser, lambda b: focused(b) == "small-record", 10)
        ActionChains(browser).send_keys(Keys.ENTER).perform()
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop") and b.find_element(By.ID, "small-stop").is_displayed(), 10)
        wait_for(browser, lambda b: (english / "2026-10-18_12-00" / "session.json").exists(), 3)
        press(browser, "small-more")
        press(browser, "small-confidential")
        marked = wait_for(browser, lambda b: (english / "2026-10-18_12-00" / "confidential").exists(), 3)
        row = browser.find_element(By.CSS_SELECTOR, "#small .confidential").text
        time.sleep(0.3)
        check("in English, « Confidential meeting — nothing leaves this computer », on, the window whole",
              marked and "Confidential meeting" in row and "Nothing leaves this computer" in row and fits(browser)[0] <= fits(browser)[1], row)
        seen(browser, "d60-2-small-on-dark")
        stop_from_small(browser)
        wait_for(browser, lambda b: front(english / "2026-10-18_12-00", "provider") == "local", 20)
    with window(english, "en_GB.UTF-8", agreed, dark=True) as browser:
        chips = browser.execute_script("return [...document.querySelectorAll('.session .mark.locked')].map(c => c.getAttribute('aria-label'))")
        seen(browser, "d60-4-list-dark")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_12-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "confidential-switch"), 10)
        check("in English, the list and the page say it", chips == ["Confidential"] and read(browser, lambda b: switch(b, "confidential-switch").is_selected())
              and counted == {"hear": 1, "write": 1}, (chips, counted))
        seen(browser, "d60-5-page-dark")

    # Turned on while a service writes the minutes: that making ended, written again here.
    writing = WORK / "confidential-writing"
    session(writing / "sent", now - dt.timedelta(hours=3), 3, {"local.wav": 1000, "transcript.jsonl": 0})
    (writing / "sent" / "transcript.jsonl").write_text((asking / "asked" / "transcript.jsonl").read_text())
    (WORK / "slow-minutes").touch()
    try:
        with window(writing, "fr_FR.UTF-8", agreed) as browser:
            browser.execute_script("location.hash = '#/meeting/sent'")
            wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes"), 10)
            press(browser, "make-minutes")
            asked = wait_for(browser, lambda b: counted["write"] == 2, 10)
            press(browser, "confidential-switch")
            here = wait_for(browser, lambda b: front(writing / "sent", "provider") == "local", 25)
            check("turned on while a service writes the minutes: ended, written here, nothing more sent",
                  asked and here and counted == {"hear": 1, "write": 2}, (front(writing / "sent", "provider"), counted))
            said = wait_for(browser, lambda b: b.execute_script(left) == "Déjà parti avant\u00a0: la transcription vers writes.example.", 5)
            check("the page says the transcript had left, to which host", said, browser.execute_script(left))
            check("its end is no failure said", not browser.find_elements(By.ID, "minutes-again"))
    finally:
        (WORK / "slow-minutes").unlink()

    # A service named to hear, the default Whisper model not there: said confidential, the meeting
    # waits for the model, offered as Settings offer it, and nothing is sent.
    unmodelled = WORK / "confidential-no-model"
    session(unmodelled / "kept", now - dt.timedelta(hours=4), 3, {"local.wav": 1000})
    (unmodelled / "kept" / "confidential").touch()
    bare = window(unmodelled, "fr_FR.UTF-8", agreed.replace(f'model = "{model}"\n', ""))
    # Its own data folder: no default model fetched there by an earlier scenario.
    bare.env.update(XDG_DATA_HOME=str(WORK / "data-confidential"))
    with bare as browser:
        browser.execute_script("location.hash = '#/meeting/kept'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-transcript"), 10)
        press(browser, "make-transcript")
        waits = "return document.getElementById('transcript-state')?.innerText || ''"
        offered = wait_for(browser, lambda b: "En attente du modèle Whisper" in b.execute_script(waits)
                           and b.find_elements(By.CSS_SELECTOR, "#transcript-state #model-fetch"), 10)
        check("the Whisper model missing: said and offered as Settings offer it, the service named asked nothing",
              offered and counted == {"hear": 1, "write": 2}, (browser.execute_script(waits), counted, names(unmodelled / "kept")))
    for server in servers:
        server.shutdown()


def nothing_silent(now: dt.datetime) -> None:
    """AP. What the user does and fails says so; a folder that cannot be read is no empty list (D67)."""
    print("AP. a command that fails says it, in the user's language; a folder unread is said and tried again (D67)")
    meetings = WORK / "silence"
    session(meetings / "2026-10-20_10-00", now - dt.timedelta(days=3), 20, {"local.wav": 1000})
    shown = lambda b, id_: b.find_element(By.ID, id_).is_displayed()
    text = lambda b, id_: b.find_element(By.ID, id_).get_attribute("textContent")
    # `core` itself is frozen, the object holding it not: a copy with the command stood in.
    refuse = "const real = window.__TAURI__.core, refused = arguments[0]; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === refused ? Promise.reject('REFUS') : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };"
    with Window(meetings, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: shown(b, "meeting"), 10)
        # The real commands on a meeting gone: what the user reads is the shell's own sentence.
        for ask in ("minutes_again", "transcript_again"):
            said = browser.execute_async_script(
                "const done = arguments[arguments.length - 1], said = document.createElement('p'), again = document.createElement('button');"
                "said.dataset.name = 'gone'; askAgain({ ask: arguments[0] }, said, again).then(() => done(said.textContent));", ask)
            check(f"{ask} on a meeting gone says so, in French, never the command's raw text",
                  said.startswith("Impossible de recommencer") and "n’est plus dans le dossier" in said, said)
        browser.execute_script(refuse, "agreement_shown")
        browser.execute_script("location.hash = '#/agree/2026-10-20_10-00'")
        refused = wait_for(browser, lambda b: shown(b, "pane-said"), 5)
        check("an agreement refused: said in the pane, back at the list", refused
              and text(browser, "pane-said") == "Cette réunion ne peut pas être envoyée\u00a0: REFUS", text(browser, "pane-said"))
        browser.execute_script("window.restore(); location.hash = '#/settings'")
        check("another address: the sentence goes", wait_for(browser, lambda b: not shown(b, "pane-said"), 5))
        browser.execute_script("location.hash = '#/'")
        browser.execute_script(refuse, "settings_saved")
        press(browser, "fold")
        folded = wait_for(browser, lambda b: shown(b, "pane-said"), 5)
        check("the sidebar's fold not kept: said in the pane, the sidebar still as asked",
              folded and text(browser, "pane-said") == "La barre latérale reviendra comme avant au prochain démarrage\u00a0: REFUS"
              and not shown(browser, "sidebar"), text(browser, "pane-said"))
        shot(browser, "d67-pane-said")
        browser.execute_script("window.restore()")
        press(browser, "unfold")
        check("kept at last: the sentence goes", wait_for(browser, lambda b: not shown(b, "pane-said"), 5))
        browser.execute_script(refuse, "hearing_models")
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "model"), 10)
        # Settings drawn before, drawn anew: the list taken once that drawing is done.
        time.sleep(1)
        Select(browser.find_element(By.ID, "model")).select_by_value("service:0")
        refused = wait_for(browser, lambda b: shown(b, "transcription-models-refused"), 5)
        check("a service's models refused: said, and the model can still be typed",
              refused and text(browser, "transcription-models-refused") == "Les modèles de ce service n’ont pas pu être lus, vous pouvez en saisir un\u00a0: REFUS"
              and shown(browser, "transcription-model"), text(browser, "transcription-models-refused"))
        browser.execute_script("window.restore()")
        browser.execute_script("location.hash = '#/'")
        # The folder refused, then given back: said, and read again on the click.
        meetings.chmod(0)
        try:
            browser.execute_script("window.dispatchEvent(new Event('focus'))")
            said = wait_for(browser, lambda b: shown(b, "empty-again"), 10)
            check("the meetings' folder unread: said, no recording offered as if it were empty, a way to try again",
                  said and text(browser, "none") == f"Les réunions de {meetings} ne peuvent pas être lues."
                  and text(browser, "none-do").startswith("Vérifiez que le disque")
                  and text(browser, "empty-again") == "Réessayer" and not shown(browser, "empty-record"), text(browser, "none"))
            browser.set_window_size(420, 400)
            browser.execute_script("document.getElementById('empty').scrollIntoView()")
            for theme in ("light", "dark"):
                shot_themed(browser, f"d67-unread-{theme}", theme)
            press(browser, "empty-again")
            time.sleep(0.3)
            check("tried again, still refused: still said", shown(browser, "empty-again"))
        finally:
            meetings.chmod(0o755)
        browser.set_window_size(1100, 800)
        browser.execute_script("document.getElementById('empty-again').click()")
        check("tried again, given back: the meetings are there", wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#groups .session") and not shown(b, "empty"), 10))
    # A meeting's folder locked while it is recorded: the mark refused for real, the switch on says so.
    recorded = WORK / "silence-marked"
    recorded.mkdir()
    with Window(recorded, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True) as browser:
        press(browser, "small-record")
        wait_for(browser, lambda b: shown(b, "small-stop"), 10)
        press(browser, "small-more")
        made = wait_for(browser, lambda b: list(recorded.glob("*/session.json")) and shown(b, "small-confidential"), 15)
        folder = next(recorded.glob("*/session.json")).parent if made else recorded
        folder.chmod(0o555)
        click = "document.getElementById('small-confidential').click()"
        top = "return document.getElementById('small-stop').getBoundingClientRect().top"
        try:
            before = browser.execute_script(top)
            browser.execute_script(click)
            said = wait_for(browser, lambda b: shown(b, "small-said"), 8)
            check("said, Arrêter stays where it is", browser.execute_script(top) == before)
            check("confidential refused in the small window: said, its lock off, never promising what is not so",
                  said and "n’a pas pu être marquée confidentielle" in text(browser, "small-said")
                  and "on" not in browser.execute_script("return document.getElementById('small-confidential').closest('.confidential').className").split(),
                  text(browser, "small-said"))
        finally:
            folder.chmod(0o755)
        check("given back, it is kept at the next try and the sentence goes",
              wait_for(browser, lambda b: not shown(b, "small-said") and (folder / "confidential").exists(), 8), sorted(p.name for p in folder.glob("*")))
        browser.execute_script(click)
        wait_for(browser, lambda b: not (folder / "confidential").exists(), 5)
        folder.chmod(0o555)
        try:
            browser.execute_script(click)
            wait_for(browser, lambda b: shown(b, "small-said"), 8)
        finally:
            folder.chmod(0o755)
        # A press within half a second of the last is taken for the same one.
        time.sleep(0.6)
        press(browser, "small-stop")
        check("stopped before the next try: tried once more, and kept",
              wait_for(browser, lambda b: shown(b, "small-record") and (folder / "confidential").exists() and not shown(b, "small-said"), 10))
        # The clock stood in: the next recording is the same folder, its mark taken away.
        (folder / "confidential").unlink()
        time.sleep(2)
        press(browser, "small-record")
        wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 15)
        time.sleep(0.6)
        folder.chmod(0o555)
        try:
            browser.execute_script(click)
            wait_for(browser, lambda b: shown(b, "small-said"), 8)
            time.sleep(0.6)
            press(browser, "small-stop")
            at_rest = wait_for(browser, lambda b: shown(b, "small-record") and "n’est pas confidentielle" in text(b, "small-said"), 10)
            first = browser.execute_script("const s = document.getElementById('small-said'); return (s.parentElement.id === 'small' ? s : s.parentElement).previousElementSibling.className")
            check("stopped with the mark refused: said at rest as it is, right under the bar's row, and nothing marked",
                  at_rest and first == "bar-row" and not (folder / "confidential").exists(), (text(browser, "small-said"), first))
        finally:
            folder.chmod(0o755)
        time.sleep(2)
        press(browser, "small-record")
        check("the next recording starts clean", wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed() and not shown(b, "small-said"), 15))
        time.sleep(0.6)
        press(browser, "small-stop")
    for language, raw in (("fr_FR.UTF-8", False), ("en_US.UTF-8", True)):
        with Window(meetings, language, "reminder = false\n" + ("developer = true\n" if raw else ""), ) as browser:
            told = browser.execute_async_script(
                "const done = arguments[arguments.length - 1]; window.__TAURI__.core.invoke('minutes_again', { name: 'gone' }).then(() => done(''), (why) => done(String(why)));")
            check(f"the raw text only in developer mode ({'on' if raw else 'off'})", ("gone is no session" in told) == raw, told)
            if raw:
                # « More actions » with both items (D89): Enter opens it on the first, ↑ on the last, expanded while open.
                wait_for(browser, lambda b: b.find_elements(By.ID, "more-actions"), 10)
                opened_on = []
                for key in (Keys.ENTER, Keys.ARROW_UP):
                    browser.execute_script("document.getElementById('more-actions').focus()")
                    ActionChains(browser).send_keys(key).perform()
                    wait_for(browser, lambda b: b.find_element(By.ID, "more-menu").is_displayed(), 2)
                    opened_on.append(browser.execute_script("return [document.activeElement.id, document.getElementById('more-actions').getAttribute('aria-expanded')]"))
                    ActionChains(browser).send_keys(Keys.ESCAPE).perform()
                    wait_for(browser, lambda b: not b.find_element(By.ID, "more-menu").is_displayed(), 2)
                check("in developer mode, « More actions » opens on its first by Enter, on its last by ↑, expanded while open (D89)",
                      opened_on == [["rename-item", "true"], ["copy-diagnostic", "true"]], opened_on)
                browser.execute_script("document.getElementById('more-actions').focus()")
                ActionChains(browser).send_keys(Keys.ENTER).perform()
                wait_for(browser, lambda b: b.find_element(By.ID, "more-menu").is_displayed(), 2)
                ActionChains(browser).send_keys(Keys.TAB).perform()
                left_ = wait_for(browser, lambda b: not b.find_element(By.ID, "more-menu").is_displayed(), 2)
                check("Tab leaves « More actions » closed, not expanded (D89)",
                      left_ and browser.execute_script("return document.getElementById('more-actions').getAttribute('aria-expanded')") == "false")
                described = browser.execute_script("const b = document.getElementById('confidential-switch'); return document.getElementById(b.getAttribute('aria-describedby'))?.textContent")
                check("in English, the switch's description: « Nothing leaves this computer » (D89)", described == "Nothing leaves this computer", described)
    # What the page reads and cannot: said where the eye is, in the user's language; asked by no gesture
    # or not, never only a line in the log (D67).
    for language, not_opened, not_read, unread, gone in (
        ("fr_FR.UTF-8", "Cette réunion n’a pas pu être ouverte\u00a0: ", "Le contexte n’a pas pu être relu\u00a0: ", "Les réunions ne peuvent pas être lues", "n’est plus dans le dossier"),
        ("en_US.UTF-8", "This meeting could not be opened: ", "The context could not be read again: ", "The meetings cannot be read", "is no longer in the meetings"),
    ):
        context_file = meetings / "2026-10-20_10-00" / "context.txt"
        context_file.write_text("Budget 2026")
        with Window(meetings, language, "reminder = false\n") as browser:
            browser.set_window_size(1100, 800)
            wait_for(browser, lambda b: shown(b, "meeting") and b.find_elements(By.ID, "context"), 10)
            # A making stood in, then over, the context not read back: its text stays, the sentence comes.
            browser.execute_script(
                "const real = window.__TAURI__.core, name = arguments[0]; window.making = true; window.refusing = true;"
                "window.__TAURI__.core = { ...real, invoke: async (asked, ...rest) => {"
                " if (asked === 'context_shown' && window.refusing) throw 'REFUS';"
                " const answer = await real.invoke(asked, ...rest);"
                " return asked === 'makings' && window.making ? { ...answer, minutes: { name, model: 'm', provider: 'p' } } : answer; } };"
                "window.restore = () => { window.__TAURI__.core = real; };", "2026-10-20_10-00")
            wait_for(browser, lambda b: b.execute_script("return document.getElementById('context').dataset.making === 'true'"), 8)
            browser.execute_script("window.making = false")
            said = wait_for(browser, lambda b: text(b, "context-said") == not_read + "REFUS" and shown(b, "context-said"), 8)
            check(f"a context not read back after a making: said ({language})", said, text(browser, "context-said"))
            check("its text stays as it was", browser.execute_script("return document.getElementById('context-text').value") == "Budget 2026")
            # Read back after the next making: the sentence goes.
            def a_making(refusing: bool) -> None:
                browser.execute_script("window.refusing = arguments[0]; window.making = true", refusing)
                wait_for(browser, lambda b: b.execute_script("return document.getElementById('context').dataset.making === 'true'"), 8)
                browser.execute_script("window.making = false")
                wait_for(browser, lambda b: b.execute_script("return document.getElementById('context').dataset.making === 'false'"), 8)
            a_making(False)
            check("read back at the next making: the sentence goes", wait_for(browser, lambda b: not shown(b, "context-said"), 5), text(browser, "context-said"))
            a_making(True)
            wait_for(browser, lambda b: shown(b, "context-said"), 5)
            # Written since: it is no longer the last word, with no redraw to wipe it.
            browser.execute_script("window.__card = document.getElementById('context')")
            context_opened(browser).send_keys(" X")
            gone_said = wait_for(browser, lambda b: not shown(b, "context-said") and context_file.read_text() == "Budget 2026 X", 5)
            check("written after it could not be read: the sentence goes, on the same page", gone_said and browser.execute_script("return window.__card === document.getElementById('context')"), text(browser, "context-said"))
            browser.execute_script("window.restore()")
            browser.execute_script(refuse, "opened")
            browser.execute_script("location.hash = '#/meeting/2026-10-20_10-00'")
            said = wait_for(browser, lambda b: shown(b, "pane-said") and text(b, "pane-said") == not_opened + "REFUS", 5)
            check(f"a meeting that cannot be opened: said in the pane, back at the list ({language})",
                  said and browser.execute_script("return location.hash") == "#/" and not shown(browser, "meeting"), text(browser, "pane-said"))
            browser.execute_script("window.restore(); location.hash = '#/settings'")
            check("another address: the sentence goes", wait_for(browser, lambda b: not shown(b, "pane-said"), 5))
            # The real command, on a meeting gone: the shell's own sentence, not the stand-in's.
            browser.execute_script("location.hash = '#/meeting/gone'")
            said = wait_for(browser, lambda b: shown(b, "pane-said") and text(b, "pane-said").startswith(not_opened), 5)
            check(f"a meeting gone: said, with what the shell tells of it ({language})", said and gone in text(browser, "pane-said"), text(browser, "pane-said"))
            check("said at the list: never to go back to it", not any(word in text(browser, "pane-said") for word in ("revenez", "go back")), text(browser, "pane-said"))
            # A folder there that holds no session: the same sentence.
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: not shown(b, "pane-said"), 5)
            (meetings / "not-a-session").mkdir(exist_ok=True)
            (meetings / "not-a-session" / "session.json").write_text('{"contract":"another/v1"}')
            browser.execute_script("location.hash = '#/meeting/not-a-session'")
            said = wait_for(browser, lambda b: shown(b, "pane-said") and text(b, "pane-said").startswith(not_opened), 5)
            check(f"a folder that is no session: said, never to go back to the list ({language})",
                  said and gone in text(browser, "pane-said") and not any(word in text(browser, "pane-said") for word in ("revenez", "go back")), text(browser, "pane-said"))
            (meetings / "not-a-session" / "session.json").unlink()
            (meetings / "not-a-session").rmdir()
            # At the smallest size the pane is hidden on the list: the sentence is seen all the same, above it.
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: not shown(b, "pane-said"), 5)
            browser.set_window_size(420, 400)
            wait_for(browser, lambda b: b.execute_script("return innerWidth") < 840, 5)
            browser.execute_script(refuse, "opened")
            browser.execute_script("location.hash = '#/meeting/2026-10-20_10-00'")
            place = "const r = document.getElementById('pane-said').getBoundingClientRect(), l = document.querySelector('#groups .session').getBoundingClientRect(); return [r.width > innerWidth * 0.8, r.top >= 0, r.bottom <= innerHeight, r.bottom <= l.top, location.hash, getComputedStyle(document.querySelector('.pane > .back')).display]"
            seen = wait_for(browser, lambda b: text(b, "pane-said") == not_opened + "REFUS" and b.execute_script(place)[:3] == [True, True, True], 5)
            check(f"at 420 × 400, back at the list: the sentence is seen, above the list, no way back offered ({language})",
                  seen and browser.execute_script(place) == [True, True, True, True, "#/", "none"], browser.execute_script(place))
            browser.execute_script("window.restore(); location.hash = '#/settings'")
            check("at 420 × 400, the next address: the sentence goes", wait_for(browser, lambda b: not shown(b, "pane-said"), 5))
            browser.set_window_size(1100, 800)
            wait_for(browser, lambda b: b.execute_script("return innerWidth") >= 840, 5)
            # Refused late, the user gone to another address meanwhile: that address stays, nothing said.
            browser.execute_script("location.hash = '#/settings'")
            wait_for(browser, lambda b: not shown(b, "pane-said") and b.find_elements(By.ID, "model"), 5)
            browser.execute_script("const real = window.__TAURI__.core; window.late = 0; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'opened' ? new Promise((_, no) => setTimeout(() => { window.late += 1; no('REFUS'); }, 300)) : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; }; location.hash = '#/meeting/2026-10-20_10-00'; setTimeout(() => { location.hash = '#/settings'; }, 50);")
            wait_for(browser, lambda b: b.execute_script("return window.late") >= 1, 5)
            time.sleep(0.2)
            check(f"refused late, another address since: it stays, nothing said ({language})",
                  browser.execute_script("return location.hash") == "#/settings" and not shown(browser, "pane-said"), browser.execute_script("return location.hash"))
            browser.execute_script("window.restore()")
        with Window(recorded, language, 'after = "nothing"\nreminder = false\n', small=True) as browser:
            browser.execute_script(refuse, "context_shown")
            time.sleep(1.1)
            press(browser, "small-record")
            said = wait_for(browser, lambda b: shown(b, "small-said") and text(b, "small-said") == not_read + "REFUS", 10)
            check(f"a recording's context not read: said under the switch, Arrêter stays ({language})", said, text(browser, "small-said"))
            browser.execute_script("window.restore()")
            time.sleep(0.6)
            press(browser, "small-stop")
            wait_for(browser, lambda b: shown(b, "small-record"), 10)
            # Another meeting's context, read: the sentence of the last one goes.
            time.sleep(2)
            press(browser, "small-record")
            cleared = wait_for(browser, lambda b: shown(b, "small-stop") and not shown(b, "small-said"), 10)
            check(f"the next recording's context read: the sentence goes ({language})", cleared, text(browser, "small-said"))
            time.sleep(0.6)
            press(browser, "small-stop")
            wait_for(browser, lambda b: shown(b, "small-record"), 10)
    empty = WORK / "silence-none"
    empty.mkdir()
    with Window(empty, "fr_FR.UTF-8", "reminder = false\n") as browser:
        browser.set_window_size(420, 400)
        said = wait_for(browser, lambda b: shown(b, "none"), 10)
        # No meeting at all: the sentence alone comes first, the header and Démarrer still in sight.
        browser.execute_script("location.hash = '#/meeting/gone'")
        place = "const y = (id) => document.getElementById(id).getBoundingClientRect(); return [y('pane-said').top >= 0, y('pane-said').bottom <= y('record').top, y('record').bottom <= innerHeight, y('record').bottom <= y('empty-record').top]"
        seen = wait_for(browser, lambda b: shown(b, "pane-said") and text(b, "pane-said").startswith("Cette réunion n’a pas pu être ouverte"), 5)
        check("no meeting, at 420 × 400: the sentence, then the header and Démarrer in sight, what to do under them",
              seen and browser.execute_script(place) == [True, True, True, True], browser.execute_script(place))
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: not shown(b, "pane-said"), 5)
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: shown(b, "none"), 5)
        check("no meeting: what to do first, never only a path", said and text(browser, "none-do").startswith("Démarrez un enregistrement") and shown(browser, "empty-record"))
        browser.execute_script("document.getElementById('empty').scrollIntoView()")
        for theme in ("light", "dark"):
            shot_themed(browser, f"d67-none-{theme}", theme)


def gestures_answer(now: dt.datetime) -> None:
    """AQ. Each gesture answers at once, its command stood in to take two seconds (D68)."""
    print("AQ. Démarrer and Arrêter answer before their command, a second click asks nothing; Refaire… closes; a setting says Enregistré (D68)")
    meetings = WORK / "answering"
    written = meetings / "2026-10-28_09-00"
    session(written, now - dt.timedelta(days=1), 10, {"local.wav": 1000})
    (written / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","start":0.0,"end":1.0,"text":"Bonjour."}\n{"type":"complete","duration":1.0}\n')
    (written / "minutes.md").write_text(
        "---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: local\nmodel: qwen3:8b\ncontext: null\n---\n\n"
        "## 1.\n\nLe budget est validé.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    slow = ("const real = window.__TAURI__.core; window.__asked = 0; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'record'"
            " ? (window.__asked++, new Promise((done) => setTimeout(done, 2000)).then(() => real.invoke(asked, ...rest))) : real.invoke(asked, ...rest) };")
    shown = lambda b, id_: b.find_element(By.ID, id_).is_displayed()
    said = lambda b: b.find_element(By.ID, "record-said").get_attribute("textContent")
    running = lambda b: b.execute_async_script("const d = arguments[arguments.length - 1]; window.__TAURI__.core.invoke('recording_now').then((n) => d(n.recording))")
    with Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n') as browser:
        browser.set_window_size(1100, 800)
        browser.execute_script(f"location.hash = '#/meeting/{written.name}'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"), 10)
        # Refaire…: Annuler, then Escape, each giving the focus back, nothing changed.
        before = (written / "minutes.md").read_text()
        for way in ("cancel", "escape"):
            press(browser, "redo-minutes")
            wait_for(browser, lambda b: b.find_elements(By.ID, "redo-panel"), 5)
            if way == "cancel":
                press(browser, "redo-panel-cancel")
            else:
                browser.switch_to.active_element.send_keys(Keys.ESCAPE)
            closed = wait_for(browser, lambda b: not b.find_elements(By.ID, "redo-panel"), 3)
            check(f"Refaire… closed by {way}: the focus back on it, nothing changed", closed
                  and browser.execute_script("return document.activeElement.id") == "redo-minutes" and (written / "minutes.md").read_text() == before,
                  (closed, browser.execute_script("return document.activeElement.outerHTML.slice(0, 80)")))
        press(browser, "redo-minutes")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-panel"), 5)
        browser.execute_script("document.activeElement.blur()")
        ActionChains(browser).send_keys(Keys.ESCAPE).perform()
        check("Escape with the focus nowhere closes it too", wait_for(browser, lambda b: not b.find_elements(By.ID, "redo-panel"), 3))
        press(browser, "redo-transcript")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-panel"), 5)
        browser.switch_to.active_element.send_keys(Keys.ESCAPE)
        check("the transcript's too", wait_for(browser, lambda b: not b.find_elements(By.ID, "redo-transcript-panel"), 3)
              and browser.execute_script("return document.activeElement.id") == "redo-transcript")
        # The setting's word, a moment.
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "reminder"), 10)
        time.sleep(0.5)
        # How long it is said is timed by the page itself, from the sentence put there to the sentence
        # gone: a WebDriver round trip on a loaded machine never counts against it (#634).
        browser.execute_script("""
            window.said = {};
            window.saidWatch = new MutationObserver(() => {
              const sentence = document.getElementById('saved-said');
              if (sentence && !window.said.node) window.said = { node: sentence, shown: performance.now() };
              if (window.said.node && !window.said.node.isConnected && window.said.gone === undefined) window.said.gone = performance.now();
            });
            window.saidWatch.observe(document.body, { childList: true, subtree: true });""")
        browser.execute_script("document.getElementById('reminder').click()")
        here = lambda b: b.find_elements(By.ID, "saved-said")
        told = wait_for(browser, here, 3)
        check("a setting written says Enregistré, under it", told and here(browser)[0].text == "Enregistré"
              and here(browser)[0].get_attribute("role") == "status"
              and browser.execute_script("const s = document.getElementById('saved-said'), r = document.getElementById('reminder').closest('label, .row, .field'); return r.nextElementSibling === s"))
        wait_for(browser, lambda b: b.execute_script("return window.said.gone !== undefined"), 4)
        said_for = browser.execute_script("window.saidWatch.disconnect(); return window.said.gone === undefined ? null : window.said.gone - window.said.shown")
        check("still said after a second", said_for is not None and said_for >= 1000, said_for)
        check("gone at a second and a half", said_for is not None and said_for <= 2500, said_for)
        refuse = "const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'settings_saved' ? Promise.reject('REFUS') : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };"
        browser.execute_script(refuse)
        browser.execute_script("document.getElementById('reminder').click()")
        refused = wait_for(browser, lambda b: b.find_element(By.ID, "not-saved").is_displayed(), 5)
        check("a write refused: said, no Enregistré", refused and not here(browser), browser.find_element(By.ID, "not-saved").text)
        browser.execute_script("window.restore()")
        # A radio with no id, far down at the smallest size: said under it, in view.
        time.sleep(0.5)
        browser.set_window_size(420, 400)
        browser.execute_script("document.querySelector('input[name=after][value=nothing]').scrollIntoView(); document.querySelectorAll('input[name=after]')[1].click()")
        wait_for(browser, here, 3)
        check("said under a radio with no id, in view", here(browser) and browser.execute_script(
            "const r = document.getElementById('saved-said').getBoundingClientRect(); return r.top >= 0 && r.bottom <= innerHeight && document.getElementById('saved-said').previousElementSibling.querySelector('input[name=after]') !== null"))
        wait_for(browser, lambda b: not here(b), 3)
        # A button in no row: said beside it, the page not sent to its top.
        browser.execute_script("const p = document.getElementById('minutes'); p.scrollIntoView({block: 'center'}); p.value = 'custom'; p.dispatchEvent(new Event('change', {bubbles: true}))")
        wait_for(browser, lambda b: b.find_elements(By.ID, "minutes-address"), 5)
        browser.find_element(By.ID, "minutes-address").send_keys("http://127.0.0.1:9/v1")
        browser.find_element(By.ID, "minutes-model").send_keys("mistral")
        browser.execute_script("document.getElementById('minutes-save').click()")
        wait_for(browser, here, 5)
        check("the minutes' Enregistrer: said beside its button, in view", here(browser) and browser.execute_script(
            "const s = document.getElementById('saved-said'), r = s.getBoundingClientRect(); return s.previousElementSibling?.id === 'minutes-save' && r.top >= 0 && r.top + r.height / 2 <= innerHeight"),
              browser.execute_script("const s = document.getElementById('saved-said'); return [s?.previousElementSibling?.id, s?.getBoundingClientRect().top]"))
        browser.set_window_size(1100, 800)
        browser.execute_script("location.hash = '#/'")
        time.sleep(0.5)
        # The real command, a double click: the second is the same press, and the button says so at once.
        browser.execute_script("const r = document.getElementById('record'); r.click(); setTimeout(() => r.click(), 200);")
        time.sleep(1)
        check("a double click: recording, and the button says it, not the stop that was not made",
              running(browser) and said(browser) == "Arrêter l’enregistrement", said(browser))
        time.sleep(0.6)
        browser.execute_script("document.getElementById('record').click()")
        wait_for(browser, lambda b: not running(b), 10)
        time.sleep(0.6)
        # A press that does nothing, as a start that failed: the button comes back as soon as it is answered.
        browser.execute_script("const real = window.__TAURI__.core; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'record' ? Promise.resolve() : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        browser.execute_script("document.getElementById('record').click()")
        check("a start that did nothing: the button comes back", wait_for(browser, lambda b: said(b) == "Démarrer un enregistrement", 1), said(browser))
        browser.execute_script("window.restore()")
        # Its command two seconds long: the header answers first.
        browser.execute_script(slow)
        browser.execute_script("const r = document.getElementById('record'); r.click(); r.click();")
        check("Démarrer: the button says its new state before the command answers, a second click asks nothing",
              said(browser) == "Arrêter l’enregistrement" and browser.execute_script("return window.__asked") == 1, said(browser))
        check("the recording did start, once", wait_for(browser, running, 8))
        time.sleep(1.5)
        browser.execute_script("const r = document.getElementById('record'); r.click(); r.click();")
        check("Arrêter: the same, the button waiting", said(browser) == "Arrêt…" and not browser.find_element(By.ID, "record").is_enabled()
              and browser.execute_script("return window.__asked") == 2, said(browser))
        check("the recording did stop, once", wait_for(browser, lambda b: not running(b), 15))
        check("ended, Démarrer again", wait_for(browser, lambda b: said(b) == "Démarrer un enregistrement" and b.find_element(By.ID, "record").is_enabled(), 5), said(browser))
    with Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True) as browser:
        browser.execute_script(slow)
        browser.execute_script("const r = document.getElementById('small-record'); r.click(); r.click();")
        start = browser.find_element(By.ID, "small-record")
        check("the bar's ●: at once, disabled, named « Démarrage… »", start.get_attribute("aria-label") == "Démarrage…" and not start.is_enabled()
              and browser.execute_script("return window.__asked") == 1, start.get_attribute("aria-label"))
        wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
        time.sleep(1)
        browser.execute_script("const r = document.getElementById('small-stop'); r.click(); r.click();")
        stop = browser.find_element(By.ID, "small-stop")
        check("its ■: the same", stop.get_attribute("aria-label") == "Arrêt…" and not stop.is_enabled() and browser.execute_script("return window.__asked") == 2, (stop.get_attribute("aria-label"), stop.is_enabled(), browser.execute_script("return window.__asked")))
        back = wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed() and b.find_element(By.ID, "small-record").is_enabled(), 15)
        check("at rest again, its ● back", back and browser.find_element(By.ID, "small-record").get_attribute("aria-label") == "Démarrer un enregistrement")
    with Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True) as browser:
        # A start the shell lets go later: true when first read, false after, no recording seen.
        browser.execute_script("const real = window.__TAURI__.core; let read = 0; window.__TAURI__.core = { ...real, invoke: (asked, ...rest) => asked === 'record' ? Promise.resolve() : asked === 'header' ? Promise.resolve({ recording: read++ === 0, stopping: false, shortcut: null }) : real.invoke(asked, ...rest) }; window.restore = () => { window.__TAURI__.core = real; };")
        browser.execute_script("document.getElementById('small-record').click()")
        late = wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_enabled() and b.find_element(By.ID, "small-record").get_attribute("aria-label") == "Démarrer un enregistrement", 3)
        check("a start refused late: its ● comes back", late, browser.find_element(By.ID, "small-record").get_attribute("aria-label"))
        browser.execute_script("window.restore()")
        time.sleep(0.6)
        # Arrêter within the double press: it was not made, so the button is not stuck.
        browser.execute_script("document.getElementById('small-record').click()")
        wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
        browser.execute_script("document.getElementById('small-stop').click()")
        again = wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_enabled() and b.find_element(By.ID, "small-stop").get_attribute("aria-label") == "Arrêter l’enregistrement", 3)
        if again:
            check("a stop taken for the same press: Arrêter comes back", True)
            time.sleep(0.6)
            press(browser, "small-stop")
        else:
            check("a stop past the double press: stopped", wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10))


def window_looks(now: dt.datetime) -> None:
    """AR. The window looks finished: Petite fenêtre an icon with its name, one primary action, little motion (D69)."""
    print("AR. one primary action, La barre d’enregistrement an icon named for assistive technology; 200 ms at most between states (D69)")
    meetings = WORK / "looking"
    made = meetings / "2026-10-28_09-00"
    session(made, now - dt.timedelta(days=1), 10, {"local.wav": 1000})
    with Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n') as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: b.find_element(By.ID, "meeting").is_displayed(), 10)
        small = browser.find_element(By.ID, "small-link")
        check("La barre d’enregistrement: an icon, named for assistive technology and to the pointer (D89)", small.text == "" and small.get_attribute("aria-label") == "La barre d’enregistrement"
              and small.get_attribute("title") == "La barre d’enregistrement" and small.find_elements(By.CSS_SELECTOR, "svg"), small.text)
        # D89: each secondary action of the sidebar an icon button, its tooltip and its name; every icon Lucide's, 16 or 20 px.
        secondary = browser.execute_script(
            "return ['fold', 'free-link', 'small-link', 'settings-link'].map(id => document.getElementById(id)).map(e => [e.id, e.textContent.trim() === '' && e.classList.contains('icon-button'),"
            " (e.getAttribute('aria-label') || '') !== '' && (e.title || '').startsWith(e.getAttribute('aria-label')), e.querySelectorAll('svg.glyph').length === 1]).filter(([, a, b, c]) => !(a && b && c))")
        check("each secondary action of the sidebar: an icon button, its tooltip and its name (D89)", secondary == [], secondary)
        icons = browser.execute_script(
            "return [...document.querySelectorAll('#frame svg')].filter(v => v.getBoundingClientRect().width > 0 && !v.closest('.brand') && !v.matches('.relief'))"
            ".map(v => [v.getAttribute('class'), v.getBoundingClientRect().width, v.getBoundingClientRect().height]).filter(([c, w, h]) => !(c || '').split(' ').includes('glyph') || ![16, 20].includes(w) || w !== h)")
        check("every icon shown is a Lucide glyph of 16 or 20 px (D89)", icons == [], icons)
        keys = browser.execute_script("return [document.getElementById('header-keys'), document.getElementById('record').title]")
        check("no shortcut line under the recording's button: the shortcut is in its tooltip (D89)", keys[0] is None and keys[1].startswith("Démarrer un enregistrement"), keys)
        # Drawn, and seen: stroked in the text's colour, the size of the text; a rule of another name once hid it.
        drawn = browser.execute_script("const v = document.querySelector('#small-link svg'), c = getComputedStyle(v), p = getComputedStyle(v.parentElement); return [c.stroke === p.color, c.fill, parseFloat(c.width) === parseFloat(p.fontSize), parseFloat(c.height) === parseFloat(p.fontSize)]")
        check("the icon is stroked, the size of its text", drawn == [True, "none", True, True], drawn)
        check("the header shows one primary action", browser.execute_script(PRIMARIES) == ["record"], browser.execute_script(PRIMARIES))
        # D89: the accepted name, alone in the text's ink as its board draws it — no mark, no word written beside it (the decider,
        # 2026-11-09); controls of 32 px; fields bordered with --edge.
        brand = browser.execute_script(
            "const b = document.querySelector('.brand'), s = b.querySelector('svg'), r = document.getElementById('record'), d = document.getElementById('name-dot');"
            " return [b.querySelectorAll('svg').length, b.textContent.trim(), s.id, s.getAttribute('role'), s.getAttribute('aria-label'),"
            " getComputedStyle(s).color === getComputedStyle(document.body).color, Math.round(s.getBoundingClientRect().height),"
            " getComputedStyle(d).fill === getComputedStyle(s).color, d.getAttribute('r') === d.dataset.rest, getComputedStyle(s.querySelector('g')).fill === getComputedStyle(s).color]")
        check("the header shows the name alone, named Kitokey, in the ink, its dot at rest (D89)", brand == [1, "", "name", "img", "Kitokey", True, 18, True, True, True], brand)
        # A meeting's setting is one control, its list inside it: the setting is measured (board 4).
        tall = browser.execute_script(
            "return [...document.querySelectorAll('.primary, .open, .icon-button, select, .search, .field input, .setting')]"
            ".filter(e => e.offsetParent && !e.closest('#small') && !e.matches('.setting select')).map(e => [e.id || e.className, e.getBoundingClientRect().height]).filter(([, h]) => h !== 32)")
        check("every control shown is 32 px high (D89)", tall == [], tall)
        edged = browser.execute_script(
            "const t = document.createElement('i'); t.style.color = 'var(--edge)'; document.body.append(t); const edge = getComputedStyle(t).color; t.remove();"
            " const all = [...document.querySelectorAll('.open, select, .search, .field input, .setting')].filter(e => e.offsetParent && !e.closest('#small') && !e.matches('.setting select'));"
            " return [all.length > 0, all.filter(e => getComputedStyle(e).borderTopColor !== edge).map(e => e.id || e.className)]")
        check("fields and secondary buttons are bordered with --edge (D89)", edged == [True, []], edged)
        filled = browser.execute_script(
            "const t = document.createElement('i'); t.style.background = 'var(--surface)'; document.body.append(t); const surface = getComputedStyle(t).backgroundColor; t.remove();"
            " const all = [...document.querySelectorAll('select, .search, .setting')].filter(e => e.offsetParent && !e.closest('#small') && !e.matches('.setting select'));"
            " return [all.length > 0, all.filter(e => getComputedStyle(e).backgroundColor !== surface).map(e => e.id || e.className)]")
        check("fields are drawn on the surface, only edged, as board 2 (D89)", filled == [True, []], filled)
        # The trial over (D83): buying holds it in the recording's place, then gives it back.
        swapped = [browser.execute_script(f"paintRefused({state}); return arguments[0]", None) or browser.execute_script(PRIMARIES) for state in ("true", "false")]
        check("the trial over, one primary still: « Acheter Kitokey », then the recording's again", swapped == [["buy"], ["record"]], swapped)
        for theme in ("light", "dark"):
            shot_themed(browser, f"d69-full-{theme}", theme)
        # A chip turning: the making stood in, then the minutes there.
        browser.execute_script("const real = window.__TAURI__.core, name = arguments[0]; window.__TAURI__.core = { ...real, invoke: async (asked, ...rest) => { const answer = await real.invoke(asked, ...rest); return asked === 'makings' ? { ...answer, minutes: { name, model: 'm', provider: 'p' } } : answer; } }; window.restore = () => { window.__TAURI__.core = real; };", made.name)
        wait_for(browser, lambda b: b.execute_script("return [...document.querySelectorAll('.chip.making')].some(c => !c.hidden)"), 8)
        (made / "minutes.md").write_text(
            "---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: local\nmodel: m\ncontext: null\n---\n\n"
            "## 1.\n\nLe budget.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
        browser.execute_script("window.restore(); window.dispatchEvent(new Event('focus'))")
        turned = wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, ".turned"), 5)
        duration = browser.execute_script("const c = document.querySelector('.turned'); return c ? parseFloat(getComputedStyle(c).animationDuration) : null")
        check("a chip turned: one small move, 200 ms at most", turned and duration is not None and 0 < duration <= 0.2, duration)
        once = browser.execute_script("const c = getComputedStyle(document.querySelector('.turned')); return [c.animationIterationCount, c.animationDelay]")
        check("once, with no wait", once == ["1", "0s"], once)
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        time.sleep(0.8)
        check("once: drawn again, it stands still", not browser.find_elements(By.CSS_SELECTOR, ".turned"))
        # Made again, the minutes there: nothing has turned, however often it is drawn.
        browser.execute_script("const real = window.__TAURI__.core, name = arguments[0]; window.__TAURI__.core = { ...real, invoke: async (asked, ...rest) => { const answer = await real.invoke(asked, ...rest); return asked === 'makings' ? { ...answer, minutes: { name, model: 'm', provider: 'p' } } : answer; } }; window.restore = () => { window.__TAURI__.core = real; };", made.name)
        wait_for(browser, lambda b: b.execute_script("return [...document.querySelectorAll('.chip.making')].some(c => !c.hidden)"), 8)
        seen = False
        for _ in range(3):
            browser.execute_script("window.dispatchEvent(new Event('focus'))")
            time.sleep(0.5)
            seen = seen or bool(browser.find_elements(By.CSS_SELECTOR, ".turned"))
        browser.execute_script("window.restore()")
        check("minutes made again: no chip turned", not seen)
    with Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True) as browser:
        moves = "return [...document.querySelectorAll('.small-rest, .small-live')].map(e => parseFloat(getComputedStyle(e).animationDuration))"
        lengths = browser.execute_script(moves)
        check("between the small window's states: 200 ms at most", lengths and all(0 < length <= 0.2 for length in lengths), lengths)
        # The page's own policy refuses what it may not load: a mark drawn from a data: address was blank (#597).
        browser.execute_script("window.__refused = []; document.addEventListener('securitypolicyviolation', (e) => window.__refused.push(e.violatedDirective + ' ' + e.blockedURI))")
        press(browser, "small-record")
        wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
        drawn = wait_for(browser, lambda b: b.execute_script("return [...document.querySelectorAll('#small .meter svg')].filter(v => v.getBoundingClientRect().width === 16).length") == 2, 8)
        check("each side's meter has its icon drawn, 16 px", drawn)
        shot(browser, "d90-recording")
        check("nothing refused by the page's policy", browser.execute_script("return window.__refused.length") == 0, browser.execute_script("return window.__refused"))
        time.sleep(0.6)
        press(browser, "small-stop")
        full = browser.find_element(By.ID, "small-full")
        check("its way to the full window: an icon, named", full.text == "" and full.find_elements(By.CSS_SELECTOR, "svg") and full.get_attribute("aria-label"), full.text)
        check("no less motion asked: the page's query says so", not browser.execute_script(LESS_MOTION))
    # Less motion asked, in a running window: rest to recording to rest, and a chip turning, stand still.
    with asking_less_motion(Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True)) as browser:
        check("less motion asked: the page's query says so", browser.execute_script(LESS_MOTION))
        browser.execute_script(WATCH_MOTION)
        check("at rest, nothing moves", browser.execute_script(MOVING) == [], browser.execute_script(MOVING))
        press(browser, "small-record")
        wait_for(browser, lambda b: b.find_element(By.ID, "small-stop").is_displayed(), 10)
        time.sleep(0.6)
        check("the small window recording: nothing moved, nothing can", browser.execute_script(MOVING) == [], browser.execute_script(MOVING))
        press(browser, "small-stop")
        wait_for(browser, lambda b: b.find_element(By.ID, "small-record").is_displayed(), 10)
        check("back at rest: nothing moved, nothing can", browser.execute_script(MOVING) == [], browser.execute_script(MOVING))
    (made / "minutes.md").unlink()
    with asking_less_motion(Window(meetings, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n')) as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: b.find_element(By.ID, "meeting").is_displayed(), 10)
        check("less motion asked: the page's query says so", browser.execute_script(LESS_MOTION))
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "sessions"), 5)
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting.arriving > *"), 5)
        arrives = browser.execute_script("return getComputedStyle(document.querySelector('#meeting.arriving > *')).animationName")
        check("a page come after one left, less motion asked: it does not fade", arrives == "none", arrives)
        browser.execute_script(WATCH_MOTION)
        browser.execute_script("const real = window.__TAURI__.core, name = arguments[0]; window.__TAURI__.core = { ...real, invoke: async (asked, ...rest) => { const answer = await real.invoke(asked, ...rest); return asked === 'makings' ? { ...answer, minutes: { name, model: 'm', provider: 'p' } } : answer; } }; window.restore = () => { window.__TAURI__.core = real; };", made.name)
        wait_for(browser, lambda b: b.execute_script("return [...document.querySelectorAll('.chip.making')].some(c => !c.hidden)"), 8)
        (made / "minutes.md").write_text(
            "---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: local\nmodel: m\ncontext: null\n---\n\n"
            "## 1.\n\nLe budget.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
        browser.execute_script("window.restore(); window.dispatchEvent(new Event('focus'))")
        turned = wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, ".turned"), 5)
        said = browser.execute_script("const c = document.querySelector('.turned'); return c ? getComputedStyle(c).animationName : null")
        check("a chip turned: it does not move", turned and said == "none", said)
        check("the full window: nothing moved, nothing can", browser.execute_script(MOVING) == [], browser.execute_script(MOVING))


def pages_seen(now: dt.datetime) -> None:
    """AX. The window's pages in both themes, at 1920 and 420 CSS px, for the eye against D89's boards."""
    print("AX. the list, a meeting's page and Settings in the light and the dark theme the system asks, at 1920 and 420 CSS px (D89)")
    meetings = WORK / "pages"
    titles = ["Point produit de la semaine", "Revue du budget T4", "Entretien — poste de développeur",
              "Atelier sur la feuille de route", "Comité de pilotage", "Rétrospective de l’équipe"]
    names = [f"2026-10-{day:02d}_10-00" for day in range(18, 18 + len(titles))]
    for index, (name, title) in enumerate(zip(names, titles)):
        folder = meetings / name
        session(folder, now - dt.timedelta(days=2 * (len(titles) - index)), 25 + 5 * index, {"local.wav": 1000})
        (folder / "minutes.md").write_text(
            f"---\ncontract: minutes/v2\nmade: 2026-10-23T11:00:00+02:00\nprovider: local\nmodel: m\ncontext: null\ntitle: {json.dumps(title)}\n---\n\n"
            "## 1.\n\nLe calendrier est validé ; la démonstration est avancée d’une semaine.\n\n"
            "## 2.\n\n- Mettre à jour le planning — Marc — vendredi\n- Relancer le fournisseur — Paul — non précisé\n\n"
            "## 3.\n\n- Un serveur est gardé pour la démonstration.\n")
    lines = [
        {"type": "header", "contract": "transcript/v1", "engine": "whisper.cpp", "model": "ggml-large-v3-turbo-q8_0", "language": "fr", "context": None},
        *({"type": "segment", "speaker": "me", "track": "local", "voice": None, "start": 6.0 * i, "end": 6.0 * i + 5,
           "text": "Une phrase de la réunion, inventée pour l’essai, assez longue pour aller à la ligne."} for i in range(8)),
        {"type": "complete", "duration": 1500.0},
    ]
    (meetings / names[-1] / "transcript.jsonl").write_text("".join(json.dumps(line, ensure_ascii=False) + "\n" for line in lines))
    for theme in ("light", "dark"):
        window = Window(meetings, "fr_FR.UTF-8", "reminder = false\n")
        # The theme asked of GTK, as a desktop asks it: WebKitGTK's prefers-color-scheme follows it.
        window.env.update(GTK_THEME="Adwaita:dark" if theme == "dark" else "Adwaita")
        with window as browser:
            dark = browser.execute_script("return matchMedia('(prefers-color-scheme: dark)').matches")
            check(f"{theme}: the page's theme is the one the system asks", dark == (theme == "dark"), dark)
            for width, height in ((1920, 1080), (420, 900)):
                browser.set_window_size(width, height)
                if not wait_for(browser, lambda b: b.execute_script("return innerWidth") == width, 3):
                    # The window's frame took its share of what was asked: asked again, by what was missing.
                    browser.set_window_size(2 * width - browser.execute_script("return innerWidth"), height)
                    wait_for(browser, lambda b: b.execute_script("return innerWidth") == width, 5)
                got = browser.execute_script("return innerWidth")
                check(f"{theme}: the page {width} CSS px wide", got == width, got)
                for page, address in (("list", "#/"), ("meeting", f"#/meeting/{names[-1]}"), ("settings", "#/settings")):
                    browser.execute_script("location.hash = arguments[0]", address)
                    shown = {"list": "#sidebar", "meeting": "#meeting", "settings": "#settings"}[page]
                    there = wait_for(browser, lambda b: b.find_element(By.CSS_SELECTOR, shown).is_displayed(), 10)
                    if page == "meeting":
                        there = there and wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, ".transcript .lines li"), 10)
                    check(f"{theme}, {width} px: {page} shown", bool(there))
                    browser.execute_script("document.getElementById('list').scrollTop = 0; scrollTo(0, 0)")
                    time.sleep(0.4)
                    shot_themed(browser, f"d89-{page}-{width}-{theme}", theme)


def trial_over(now: dt.datetime) -> None:
    """AV. The trial over on a stand-in licence (PDR-0005, D83): no recording from the window, the
    small one or the tray; why and « Acheter Kitokey » said in the full window; every meeting still read, played, corrected and
    copied; bought while open, the next one records; a licence not read records, said once."""
    print("AV. the trial over: nothing recorded from the window, the small one or the tray, said in the full window with « Acheter Kitokey »; every meeting"
          " still read, played, corrected, copied; bought while open, the next records; not read, it records (D83)")
    keys = lambda browser, *sent: ActionChains(browser).send_keys(*sent).perform()
    logs = WORK / "state" / "kitokey" / "logs"
    logged = lambda message: sum(f.read_text().count(f'"message":"{message}"') for f in logs.glob("shell.*.jsonl"))
    asked = WORK / "recorder-args"
    licence = WORK / "licence"
    # As the page holds it: the driver's text turns a space that does not break into a plain one.
    text = lambda browser, element_id: browser.execute_script("return document.getElementById(arguments[0]).textContent", element_id)
    over_fr = "L’essai est terminé\u202f; vos réunions restent à vous, lisibles, jouables et exportables."
    over_en = "Your trial is over; your meetings stay yours, readable, playable and exportable."
    # Begun 30 days ago, ten meetings made here since: over.
    meetings = WORK / "trial-over"
    day = now - dt.timedelta(days=30)
    names = [(day + dt.timedelta(days=index + 1)).strftime("%Y-%m-%d_10-00") for index in range(10)]
    for index, name in enumerate(names):
        session(meetings / name, day + dt.timedelta(days=index + 1, hours=1), 10, {})
    (meetings / ".kitokey-trial").write_text(f"{int(day.timestamp())}\n")
    made = "".join(f"{name}\n" for name in names)
    (meetings / ".kitokey-trial-meetings").write_text(made)
    # The last one heard, its minutes written: read, played, corrected, copied.
    heard = meetings / names[-1]
    record = json.loads((heard / "session.json").read_text())
    record["tracks"] = {"local": {"device": "Mic"}}
    (heard / "session.json").write_text(json.dumps(record))
    with wave.open(str(heard / "local.wav"), "wb") as written:
        written.setnchannels(1)
        written.setsampwidth(2)
        written.setframerate(48000)
        written.writeframes(bytes(2 * 48000 * 3))
    (heard / "clock.jsonl").write_text('{"track":"local","sample":0,"at":0.0}\n')
    (heard / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"me","track":"local","voice":null,"start":0.4,"end":1.2,"text":"Bonjour à tous."}\n'
        '{"type":"complete","duration":3.0}\n')
    (heard / "minutes.md").write_text(
        "---\ncontract: minutes/v1\nmade: 2026-10-28T10:00:00+01:00\nprovider: local\nmodel: qwen3:8b\ncontext: null\n---\n\n"
        "## 1.\n\nLe budget est validé.\n\n## 2.\n\n- Rien\n\n## 3.\n\n- Rien\n")
    licence.write_text("trial")
    refusals = logged("shell.recording_refused")
    window = Window(meetings, "fr_FR.UTF-8", 'reminder = false\nafter = "nothing"\n')
    window.env["KITOKEY_LICENCE"] = str(licence)
    with window as browser:
        asked.unlink(missing_ok=True)
        press(browser, "record")
        said = wait_for(browser, lambda b: b.find_element(By.ID, "refused").is_displayed(), 5)
        time.sleep(1)
        check("from the window: nothing recorded, the button at rest, what stays said under it, as information, with « Acheter Kitokey » its primary action",
              said and not asked.exists() and "live" not in browser.find_element(By.ID, "record").get_attribute("class")
              and text(browser, "refused") == over_fr and browser.find_element(By.ID, "refused").get_attribute("role") == "status"
              and browser.find_element(By.ID, "buy").text == "Acheter Kitokey" and len([f for f in meetings.iterdir() if f.is_dir()]) == 10
              # Information, not a failure: the window's neutral note, never the red of an error; buying, its primary action.
              and browser.find_element(By.ID, "refused").get_attribute("class") == "note"
              and browser.execute_script(PRIMARIES) == ["buy"] and "open" in browser.find_element(By.ID, "record").get_attribute("class"),
              (text(browser, "refused"), asked.exists()))
        shot(browser, "d83-trial-over-fr")
        time.sleep(0.6)
        check("from the tray: nothing recorded either, the refusal logged each time",
              tray_click("Démarrer") and wait_for(browser, lambda b: logged("shell.recording_refused") == refusals + 2, 5) and not asked.exists(),
              logged("shell.recording_refused") - refusals)
        # Every meeting still read, played, corrected and copied (PDR-0001).
        browser.execute_script(f"location.hash = '#/meeting/{heard.name}'")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#meeting .lines .edit") and b.find_elements(By.ID, "play")
                 and b.find_elements(By.ID, "copy-minutes"), 10)
        check("a meeting is still read: its transcript and its minutes",
              "Bonjour à tous." in browser.find_element(By.CSS_SELECTOR, "#meeting .lines").text
              and "Le budget est validé." in browser.find_element(By.CSS_SELECTOR, "#meeting .minutes").text)
        browser.find_element(By.ID, "play").click()
        check("and played", wait_for(browser, lambda b: b.execute_script("const a = document.querySelector('audio.track'); return !!a && !a.paused"), 10))
        browser.find_element(By.ID, "play").click()
        browser.find_element(By.CSS_SELECTOR, "#meeting .lines .edit").click()
        wait_for(browser, lambda b: b.execute_script("return document.activeElement.id") == "correcting", 2)
        keys(browser, Keys.END, " Et à toutes.", Keys.ENTER)
        check("and corrected", wait_for(browser, lambda b: "Et à toutes." in (heard / "transcript.jsonl").read_text(), 3))
        browser.execute_script("window.__copied = []; Object.defineProperty(navigator, 'clipboard', {configurable: true,"
                               " value: {writeText: (text) => (window.__copied.push(text), Promise.resolve())}})")
        press(browser, "copy-minutes")
        check("and its minutes copied", wait_for(browser, lambda b: "Le budget est validé." in "".join(b.execute_script("return window.__copied")), 3))
        purchases = logged("shell.purchase_asked")
        # Clicked from the page's own script: whether the button is disabled is read before the
        # command can answer.
        pending = browser.execute_script("const buy = document.getElementById('buy'); buy.click(); return buy.disabled")
        check("« Acheter Kitokey » is disabled while the purchase is asked: one purchase at a time", pending is True, pending)
        check("« Acheter Kitokey » asks the purchase — the stand-in's, said in the log —; not bought, the refusal stays",
              wait_for(browser, lambda b: logged("shell.purchase_asked") == purchases + 1, 5) and browser.find_element(By.ID, "refused").is_displayed())
        check("the purchase answered, « Acheter Kitokey » is enabled again",
              wait_for(browser, lambda b: not b.find_element(By.ID, "buy").get_property("disabled"), 5))
        # From the small window: it has no room to say why; the full window comes in its place and says it.
        main = browser.current_window_handle
        press(browser, "small-link")
        wait_for(browser, lambda b: len(b.window_handles) == 2, 5)
        browser.switch_to.window([h for h in browser.window_handles if h != main][0])
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-record"))
        time.sleep(0.6)
        try:
            browser.find_element(By.ID, "small-record").click()
        except NoSuchWindowException:
            pass  # the small window goes with the click
        replaced = wait_for(browser, lambda b: len(b.window_handles) == 1, 5)
        browser.switch_to.window(browser.window_handles[0])
        said = wait_for(browser, lambda b: b.find_element(By.ID, "refused").is_displayed(), 5)
        check("from the small window: nothing recorded, the full window in its place says why, with « Acheter Kitokey »",
              replaced and said and not asked.exists() and text(browser, "refused") == over_fr, (browser.window_handles, asked.exists()))
        # Bought in the Store's dialog — the stand-in's turn —: « Acheter Kitokey » reads the licence
        # again once the purchase is answered, and the refusal goes, nothing recorded.
        licence.write_text("bought")
        press(browser, "buy")
        check("« Acheter Kitokey », the purchase answered: the licence read again, the refusal gone, nothing recorded",
              wait_for(browser, lambda b: not b.find_element(By.ID, "refused").is_displayed(), 5)
              and not browser.find_element(By.ID, "buy").is_displayed() and not asked.exists())
        # Bought while Kitokey is open: the next recording records, no restart.
        time.sleep(0.6)
        press(browser, "record")
        recorded = wait_for(browser, lambda b: "live" in b.find_element(By.ID, "record").get_attribute("class") and asked.exists(), 5)
        check("bought while open: the next recording records, nothing said of the trial any more",
              recorded and not browser.find_element(By.ID, "refused").is_displayed() and not browser.find_element(By.ID, "buy").is_displayed()
              and browser.execute_script(PRIMARIES) == ["record"], browser.execute_script(PRIMARIES))
        time.sleep(1)
        press(browser, "record")
        wait_for(browser, lambda b: "live" not in b.find_element(By.ID, "record").get_attribute("class"), 5)
        check("bought, a meeting is not counted for the trial", (meetings / ".kitokey-trial-meetings").read_text() == made)
    licence.write_text("trial")
    window = Window(meetings, "en_GB.UTF-8", 'reminder = false\nafter = "nothing"\n')
    window.env["KITOKEY_LICENCE"] = str(licence)
    with window as browser:
        asked.unlink(missing_ok=True)
        press(browser, "record")
        said = wait_for(browser, lambda b: b.find_element(By.ID, "refused").is_displayed(), 5)
        check("in English: why, and « Buy Kitokey »", said and not asked.exists() and text(browser, "refused") == over_en
              and browser.find_element(By.ID, "buy").text == "Buy Kitokey", text(browser, "refused"))
        shot(browser, "d83-trial-over-en")
        # A licence that cannot be read: it records, each time, said once in the log.
        licence.write_text("unread")
        unread = logged("shell.licence_unread")
        for _ in range(2):
            asked.unlink(missing_ok=True)
            time.sleep(0.6)
            press(browser, "record")
            recorded = wait_for(browser, lambda b: "live" in b.find_element(By.ID, "record").get_attribute("class") and asked.exists(), 5)
            time.sleep(1)
            press(browser, "record")
            wait_for(browser, lambda b: "live" not in b.find_element(By.ID, "record").get_attribute("class"), 5)
            check("a licence that cannot be read records", recorded)
        time.sleep(1)
        check("and is said once in the log", logged("shell.licence_unread") == unread + 1, logged("shell.licence_unread") - unread)
    # A trial just begun: its day kept at the start, each recording counted as made here.
    begun = WORK / "trial-begun"
    begun.mkdir()
    licence.write_text("trial")
    window = Window(begun, "fr_FR.UTF-8", 'reminder = false\nafter = "nothing"\n')
    window.env["KITOKEY_LICENCE"] = str(licence)
    with window as browser:
        check("the first start on a trial licence keeps its day", wait_for(browser, lambda b: (begun / ".kitokey-trial").is_file(), 5))
        press(browser, "record")
        wait_for(browser, lambda b: "live" in b.find_element(By.ID, "record").get_attribute("class"), 5)
        counted = wait_for(browser, lambda b: (begun / ".kitokey-trial-meetings").is_file()
                           and (begun / ".kitokey-trial-meetings").read_text() == "2026-10-18_12-00\n", 5)
        check("a trial not over records, the meeting counted as made here", counted and not browser.find_element(By.ID, "refused").is_displayed())
        # The limit reached while it records: never cut, and its stop is no start to judge.
        shutil.copy(meetings / ".kitokey-trial", begun / ".kitokey-trial")
        for name in names:
            shutil.copytree(meetings / name, begun / name)
        with open(begun / ".kitokey-trial-meetings", "a") as kept:
            kept.write(made)
        time.sleep(2)
        check("a recording in progress when the limit is reached is not cut", "live" in browser.find_element(By.ID, "record").get_attribute("class")
              and '"stopped"' not in (begun / "2026-10-18_12-00" / "session.json").read_text())
        press(browser, "record")
        stopped = wait_for(browser, lambda b: "live" not in b.find_element(By.ID, "record").get_attribute("class")
                           and '"stopped"' in (begun / "2026-10-18_12-00" / "session.json").read_text(), 5)
        check("and it stops as asked", stopped and not browser.find_element(By.ID, "refused").is_displayed())


def trial_left(now: dt.datetime) -> None:
    """AW. Settings on a trial licence say what is left of it, in days and in meetings (PDR-0005, D83):
    the 14 days and the 10 meetings whole at its first start, each in its number, the days alone once 10
    meetings are made, the meetings alone once the days have passed; nothing once it is over, nor on a
    licence bought, not active, not read or not from the Store."""
    print("AW. Settings on a trial licence say what is left of it, in days and in meetings; nothing when over, bought or outside the Store (D83)")
    licence = WORK / "licence-left"
    meetings = WORK / "trial-left"
    meetings.mkdir()
    # Ten meetings in the folder, an hour ago: those named in the list of meetings made here count.
    names = [f"left-{index}" for index in range(10)]
    for name in names:
        session(meetings / name, now - dt.timedelta(hours=1), 10, {})

    def kept(past: dt.timedelta, count: int) -> None:
        """The trial begun `past` ago, the first `count` meetings made here."""
        (meetings / ".kitokey-trial").write_text(f"{int((now - past).timestamp())}\n")
        (meetings / ".kitokey-trial-meetings").write_text("".join(f"{name}\n" for name in names[:count]))

    def line(browser) -> list[str] | None:
        """Settings drawn anew, as the window does when it comes back: the trial's line, its name and what it says."""
        browser.execute_script("document.getElementById('settings').replaceChildren(); window.dispatchEvent(new Event('focus'))")
        wait_for(browser, lambda b: b.find_elements(By.ID, "sessions"), 5)
        # As the page holds it: the driver's text turns a space that does not break into a plain one.
        return browser.execute_script("const t = document.getElementById('trial-left');"
                                      " return t ? [t.querySelector('.name').textContent, t.querySelector('.says').textContent] : null")

    both_fr = " ; l’essai prend fin une fois les deux épuisés."
    both_en = "; the trial ends once both are used up."
    licence.write_text("trial")
    window = Window(meetings, "fr_FR.UTF-8", 'reminder = false\nafter = "nothing"\n')
    window.env["KITOKEY_LICENCE"] = str(licence)
    with window as browser:
        browser.set_window_size(1100, 800)
        wait_for(browser, lambda b: (meetings / ".kitokey-trial").is_file(), 5)
        browser.execute_script("location.hash = '#/settings'")
        said = line(browser)
        first = browser.execute_script("return document.querySelector('#settings > section')?.id")
        # One line in a card as the model's state is, never a card stretched below it.
        tall = browser.execute_script("return document.getElementById('trial-left').getBoundingClientRect().height")
        check("at its first start: the 14 days whole and the 10 meetings, the first thing Settings say, on one card no taller than its words",
              said == ["Essai gratuit", f"Il reste 14 jours et 10 réunions{both_fr}"] and first == "trial-left" and tall < 140, (said, first, tall))
        shot(browser, "d83-trial-left-fr")
        kept(dt.timedelta(days=13, hours=1), 9)
        said = line(browser)
        check("a day begun and one meeting left: each in the singular", said == ["Essai gratuit", f"Il reste 1 jour et 1 réunion{both_fr}"], said)
        kept(dt.timedelta(days=13, hours=1), 10)
        said = line(browser)
        check("ten meetings made, days left: the days alone", said == ["Essai gratuit", "Il reste 1 jour."], said)
        kept(dt.timedelta(days=30), 3)
        said = line(browser)
        check("the days past, meetings left: the meetings alone", said == ["Essai gratuit", "Il reste 7 réunions."], said)
        kept(dt.timedelta(days=30), 10)
        said = line(browser)
        check("over: nothing said of what is left", said is None, said)
        kept(dt.timedelta(days=3, hours=1), 3)
        said = line(browser)
        check("days and meetings left, in the plural", said == ["Essai gratuit", f"Il reste 11 jours et 7 réunions{both_fr}"], said)
        for word in ["bought", "inactive", "none", "unread"]:
            licence.write_text(word)
            said = line(browser)
            check(f"a licence {word}: nothing said of a trial", said is None, said)
        licence.write_text("trial")
        said = line(browser)
        check("a trial licence again: said again", said == ["Essai gratuit", f"Il reste 11 jours et 7 réunions{both_fr}"], said)
    window = Window(meetings, "en_GB.UTF-8", 'reminder = false\nafter = "nothing"\n')
    window.env["KITOKEY_LICENCE"] = str(licence)
    with window as browser:
        browser.set_window_size(1100, 800)
        browser.execute_script("location.hash = '#/settings'")
        said = line(browser)
        check("in English: days and meetings left", said == ["Free trial", f"11 days and 7 meetings left{both_en}"], said)
        shot(browser, "d83-trial-left-en")
        kept(dt.timedelta(days=13, hours=1), 9)
        said = line(browser)
        check("in English, in the singular", said == ["Free trial", f"1 day and 1 meeting left{both_en}"], said)
        kept(dt.timedelta(days=3, hours=1), 10)
        said = line(browser)
        check("in English, the days alone", said == ["Free trial", "11 days left."], said)
        kept(dt.timedelta(days=30), 9)
        said = line(browser)
        check("in English, the meetings alone", said == ["Free trial", "1 meeting left."], said)
        licence.write_text("none")
        said = line(browser)
        check("in English, outside the Store: nothing said of a trial", said is None, said)


def main() -> int:
    now = dt.datetime.now().astimezone()
    sessions = WORK / "sessions"
    # Sessions this week, last week, earlier this month when there is room for one, two months
    # ago; one whose start is written with an offset of hours alone; one whose start is no instant.
    this, last, months = now - dt.timedelta(seconds=30), now - dt.timedelta(days=7), now - dt.timedelta(days=61)
    earlier = (week_of(now) - dt.timedelta(days=8)).replace(hour=12)
    hours_only = (now - dt.timedelta(days=3)).astimezone(dt.timezone(dt.timedelta(hours=2)))
    session(
        sessions / "a", last, 42, {"me.wav": 100_000_000, "others.wav": 100_000_000, "transcript.jsonl": 7000, "minutes.md": 500}, "v1"
    )
    session(sessions / "b", this, None, {"local.wav": 2000})
    session(sessions / "c", months, 35, {"transcript.jsonl": 3000})
    session(sessions / "e", earlier, 20, {"transcript.jsonl": 1000})
    session(sessions / "f", hours_only, 10, {"local.wav": 999_999})
    record = json.loads((sessions / "f" / "session.json").read_text())
    for key in ("started", "stopped"):
        record[key] = record[key].replace("+02:00", "+02")
    (sessions / "f" / "session.json").write_text(json.dumps(record))
    (sessions / "g").mkdir(parents=True)
    (sessions / "g" / "session.json").write_text(json.dumps({"contract": "session-audio/v2", "started": "yesterday", "tracks": {}}))
    (sessions / "not-a-session").mkdir()
    expected = sorted(
        [
            (this, "Interrompu"),
            (last, "42 min|Compte rendu|Transcription"),
            (months, "35 min|Transcription|Audio supprimé"),
            (earlier, "20 min|Transcription|Audio supprimé"),
            (hours_only, "10 min"),
        ],
        key=lambda pair: pair[0],
        reverse=True,
    )
    expected = [(group(day.astimezone(), now), holds) for day, holds in expected]
    expected.append(("Date inconnue", "Interrompu|Audio supprimé"))

    print("H. the window, in French: the meetings by week, what each holds and weighs")
    with Window(sessions, "fr_FR.UTF-8") as browser:
        check("the sidebar is named in French", browser.find_element(By.ID, "sidebar").get_attribute("aria-label") == "Réunions")
        lang = browser.execute_script("return document.documentElement.lang")
        check("the page says it is in French", lang == "fr", lang)
        sections = browser.find_element(By.TAG_NAME, "nav").get_attribute("aria-label")
        check("its sections are named in French for a screen reader", sections == "Rubriques", sections)
        listed = rows(browser)
        shot(browser, "1-sessions")
        granted = browser.find_element(By.ID, "record").get_attribute("title")
        check("the shortcut is said as a person reads it, in the recording's tooltip, never in GTK's notation (#312, D89)",
              granted.startswith("Démarrer un enregistrement (") and "<" not in granted and "Press" not in granted, granted)
        # The header's gesture, as the board has it — never pressed here: no recorder is played.
        said = browser.find_element(By.ID, "record").text
        check("the header offers to start a recording", said == "Démarrer un enregistrement", said)
        # Compact rows (D89, boards 2 and 4): no relief; what each holds an icon, its word its name and tooltip.
        drawn = browser.execute_script(
            "return [document.querySelectorAll('#groups .relief').length, [...document.querySelectorAll('#groups .mark')].map(m =>"
            " [m.getAttribute('role'), m.title === m.getAttribute('aria-label'), m.textContent, m.querySelectorAll('svg.glyph').length])]")
        check("the list's rows: no relief, each mark an icon named, its tooltip its name (D89)", drawn[0] == 0 and drawn[1]
              and all(mark == ["img", True, "", 1] for mark in drawn[1]), drawn)
        # A meeting with its audio and no transcript offers one — never asked here: no engine is played.
        browser.execute_script("location.hash = '#/meeting/f'")
        offered = wait_for(browser, lambda b: b.find_elements(By.ID, "make-transcript"), 5)
        check("a meeting without its transcript offers it on its page", offered)
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 5)
        check(
            "every session, newest first, each under its week; the one with no readable day last",
            [(heading, holds) for heading, _, holds in listed] == expected,
            listed,
        )
        check("a session with no readable day is named by its folder", listed[-1][1] == "g", listed[-1])
        total = browser.find_element(By.ID, "weight").text
        check("what they weigh, on one line, then the bar, no legend (D89, board 4)", total == "201 Mo · 6 réunions", total)
        bar = browser.find_element(By.ID, "bar")
        check("the picture of the weights is described, to the pointer too", "L’audio occupe 201 Mo" in (bar.get_attribute("aria-label") or "")
              and bar.get_attribute("title") == bar.get_attribute("aria-label"), bar.get_attribute("title"))
        # The theme is the system's: the page's ground is that of the theme the system shows.
        dark = browser.execute_script("return matchMedia('(prefers-color-scheme: dark)').matches")
        ground = browser.execute_script("return getComputedStyle(document.body).backgroundColor")
        check(f"the ground of the system's {'dark' if dark else 'light'} theme", ground == GROUND["dark" if dark else "light"], ground)
        # Every action from the keyboard: the first stop of Tab is the section's link.
        browser.find_element(By.TAG_NAME, "body").send_keys(Keys.TAB)
        focused = browser.execute_script("return document.activeElement.id")
        check("Tab reaches the window's first action, in the sidebar: the button that folds it (D61)", focused == "fold", focused)
        # A session recorded meanwhile is shown when the window comes forward again.
        session(sessions / "d", now - dt.timedelta(minutes=5), 3, {"local.wav": 1000, "remote.wav": 1000})
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        came = wait_for(browser, lambda b: len(rows(b)) == 7)
        check("a new session is listed when the window comes forward", came, rows(browser))

    print("J. one meeting: its minutes beside its transcript, searched")
    meetings = WORK / "meetings"
    session(meetings / "m", now - dt.timedelta(hours=3), 16, {"local.wav": 4000, "remote.wav": 4000})
    lines = [
        {"type": "header", "contract": "transcript/v1", "engine": "whisper.cpp", "model": "ggml-large-v3-turbo-q8_0", "language": "fr", "context": None},
        {"type": "segment", "speaker": "others", "track": "remote", "voice": None, "start": 8.0, "end": 11.5, "text": "Bonjour Claire, on peut commencer."},
        {"type": "segment", "speaker": "me", "track": "local", "voice": "1", "start": 1.0, "end": 7.2, "text": "Bonjour à tous, merci d’être là."},
        {"type": "segment", "speaker": "me", "track": "local", "voice": "2", "start": 15.0, "end": 19.0, "text": "La version de test sera prête le 12 novembre."},
        {"type": "complete", "duration": 960.0},
    ]
    (meetings / "m" / "transcript.jsonl").write_text("".join(json.dumps(line, ensure_ascii=False) + "\n" for line in lines))
    (meetings / "m" / "minutes.md").write_text(
        "---\ncontract: minutes/v1\nmade: 2026-10-13T12:21:00+02:00\nprovider: openrouter.ai\nmodel: claude-sonnet\ncontext: null\n---\n\n"
        # Numbered, as a small local model writes them (#261): the page names them.
        "## 1.\n\nL’ouverture est fixée au lundi 16 novembre.\n\n"
        "## 2.\n\n- Mettre à jour le planning — Marc — aujourd’hui\n- Couper deux serveurs de test — Julien — avant vendredi\n\n"
        "## 3.\n\n- Un serveur est gardé pour la démonstration du 20.\n"
    )
    session(meetings / "n", now - dt.timedelta(days=1), 5, {"local.wav": 100})
    (meetings / "n" / "minutes.md").write_text("Not minutes.\n")
    (meetings / "n" / "transcript.jsonl").write_text(
        json.dumps(lines[0]) + "\n" + json.dumps({"type": "segment", "speaker": "me", "track": "local", "voice": None, "start": 1.0, "end": 2.0, "text": "Une autre réunion."}) + "\n"
    )
    with Window(meetings, "fr_FR.UTF-8") as browser:
        browser.find_element(By.TAG_NAME, "body").send_keys(Keys.TAB, Keys.TAB)
        focused = browser.execute_script("return document.activeElement.id")
        check("Tab reaches the sidebar's gesture after the button that folds it (D61)", focused == "record", focused)
        browser.find_element(By.ID, "record").send_keys(Keys.TAB)
        focused = browser.execute_script("return document.activeElement.id")
        check("then the search of the meetings", focused == "find", focused)
        browser.find_element(By.ID, "find").send_keys(Keys.TAB)
        focused = browser.execute_script("return document.activeElement.className")
        check("then the first meeting", "session" in focused, focused)
        browser.execute_script("[...document.querySelectorAll('#groups .session')].at(-1).focus()")
        browser.switch_to.active_element.send_keys(Keys.TAB)
        focused = browser.execute_script("return document.activeElement.id")
        check("after the meetings, freeing space, beside the weights", focused == "free-link", focused)
        # The whole of the sidebar's order, exactly (D89): no stop added, moved or skipped.
        browser.execute_script("document.getElementById('fold').focus()")
        order = ["fold"]
        for _ in range(16):
            ActionChains(browser).send_keys(Keys.TAB).perform()
            stop = browser.execute_script("const e = document.activeElement; return document.getElementById('sidebar').contains(e) ? (e.classList.contains('session') ? 'meeting' : e.id) : null")
            if stop is None:
                break
            order.append(stop)
        meetings_listed = len(browser.find_elements(By.CSS_SELECTOR, "#groups .session"))
        check("Tab through the sidebar: fold, record, find, the meetings, freeing space, the recording bar, Réglages — exactly",
              order == ["fold", "record", "find", *["meeting"] * meetings_listed, "free-link", "small-link", "settings-link"], order)
        drawn = browser.execute_script("return [...document.querySelectorAll('#groups .session')].map(s => [...s.querySelectorAll('.mark')].map(m => m.getAttribute('aria-label')))")
        check("a meeting with its minutes and transcript marks both, at its row's end (D89)", ["Compte rendu", "Transcription"] in drawn, drawn)
        shot(browser, "1-sessions-drawn")
        browser.find_elements(By.CSS_SELECTOR, ".session")[0].click()
        shown = wait_for(browser, lambda b: b.find_element(By.ID, "meeting").is_displayed() and b.find_elements(By.CSS_SELECTOR, ".lines li"))
        check("a click on a meeting opens it, marked in the list (D61)", shown
              and browser.find_elements(By.CSS_SELECTOR, ".session")[0].get_attribute("aria-current") == "page")
        heard_here = [p.text for p in browser.find_elements(By.CSS_SELECTOR, "#meeting .made .maker")]
        check("a meeting heard here says so, by the model's name", heard_here == ["Transcrit par Whisper large v3 turbo, sur cet ordinateur"], heard_here)
        shot(browser, "2-session")
        minutes = browser.find_element(By.CSS_SELECTOR, ".minutes")
        titles = [h.text for h in minutes.find_elements(By.CSS_SELECTOR, ".body h2")]
        check("the minutes' three sections, named by the page whatever the model titled them", titles == ["Points saillants", "Actions", "Points clés"], titles)
        items = [li.text for li in minutes.find_elements(By.CSS_SELECTOR, ".body li")]
        actions = browser.execute_script("return [...document.querySelectorAll('#meeting .minutes table.actions tbody tr')].map(r => [...r.cells].map(c => c.textContent))")
        check(
            "the minutes as written: each action a row, Qui · Quoi · Pour quand (D46), each key point",
            actions == [["Marc", "Mettre à jour le planning", "aujourd’hui"], ["Julien", "Couper deux serveurs de test", "avant vendredi"]]
            and items == ["Un serveur est gardé pour la démonstration du 20."],
            (actions, items),
        )
        drawn = browser.execute_script(
            "const s = document.querySelector('#meeting .relief'); return [s.getAttribute('aria-label'),"
            " ...['local', 'remote'].map(t => s.querySelector('.' + t).getAttribute('points').split(' ').map(p => +p.split(',')[1]))]"
        )
        label, here, remote = drawn
        # Speech in the first sixtieth of the meeting, on each side: here rises above the middle, remote falls below.
        check(
            "who spoke is drawn over the meeting, here above and remote below",
            label.startswith("Qui a parlé") and len(here) == len(remote) == 60 and here[0] < 42 and remote[0] > 46 and here[-1] == 42,
            drawn,
        )
        maker = minutes.find_element(By.CSS_SELECTOR, ".maker").text
        check(
            "who wrote the minutes, and where, as the board says it — the model as people say it, the day without its year",
            maker.startswith("Écrit par Claude Sonnet, via openrouter.ai · ") and "2026" not in maker and "," not in maker.split(" · ")[-1] and " à " in maker.split(" · ")[-1],
            maker,
        )
        named = browser.execute_script(
            "return ['anthropic/claude-sonnet-4-5', 'mistral:7b-instruct-q4_K_M', 'llama3.1:8b-fp16', 'org/model/', 'gpt-4o-mini',"
            " 'claude-sonnet-4-5-20250929', 'openai/gpt-4o-2024-08-06', 'llama3.1:latest', 'model-q4_K_M.gguf', 'q4_K_M', 'qwen3:14b-q8_0'].map(modelSaid)"
        )
        check(
            "a model as people say it: a version with its point, no date, tag, file or quantisation, a name even after a last slash",
            named == ["Claude Sonnet 4.5", "Mistral 7b Instruct", "Llama3.1 8b", "Model", "GPT 4o Mini", "Claude Sonnet 4.5", "GPT 4o", "Llama3.1", "Model", "q4_K_M", "Qwen3 14b"],
            named,
        )
        said = browser.execute_script(
            "return [...document.querySelectorAll('.lines li')].map(li => [li.querySelector('.at'), li.querySelector('.who'), li.querySelector('.text')].map(c => c.textContent).join(' | '))"
        )
        check(
            "the transcript in the meeting's order, each line with its side and voice",
            said
            == [
                "00:01 | Ici · voix 1 | Bonjour à tous, merci d’être là.",
                "00:08 | À distance | Bonjour Claire, on peut commencer.",
                "00:15 | Ici · voix 2 | La version de test sera prête le 12 novembre.",
            ],
            said,
        )
        search = browser.find_element(By.ID, "search")
        search.send_keys("ETRE")
        visible = browser.execute_script(
            "return [...document.querySelectorAll('.lines li')].filter(li => !li.hidden).map(li => li.querySelector('.text').textContent)"
        )
        check("a search finds what was said, whatever its accents and case", visible == ["Bonjour à tous, merci d’être là."], visible)
        search.clear()
        search.send_keys("zzz")
        none = [p.text for p in browser.find_elements(By.CSS_SELECTOR, ".transcript .none") if p.is_displayed()]
        check("a search that finds nothing says so", none == ["Aucun passage ne correspond."], none)
        search.clear()
        search.send_keys("voix")
        none = [p.text for p in browser.find_elements(By.CSS_SELECTOR, ".transcript .none") if p.is_displayed()]
        check("a search looks at what was said, not at who or when", none == ["Aucun passage ne correspond."], none)
        check("the folder can be opened from the meeting", browser.find_element(By.ID, "open-folder").get_attribute("aria-label") == "Ouvrir le dossier")
        # D89: the meeting's header — its title at 18 px, its actions on its right; each card's actions icons, each with its tooltip and its name.
        head = browser.execute_script(
            "const h = document.querySelector('#meeting h1'), o = document.getElementById('open-folder');"
            " return [parseFloat(getComputedStyle(h).fontSize), Math.abs(h.getBoundingClientRect().top - o.getBoundingClientRect().top) < 24 && o.getBoundingClientRect().left > h.getBoundingClientRect().right]")
        check("the meeting's title is 18 px, « Ouvrir le dossier » on the same line, at its right (D89)", head == [18, True], head)
        bar = browser.execute_script(
            "const b = document.querySelector('#meeting .meeting-head'), h = document.querySelector('#meeting h1'), l = document.querySelector('#meeting .lasts'), s = getComputedStyle(b);"
            " const t = document.createElement('i'); t.style.color = 'var(--line)'; document.body.append(t); const line = getComputedStyle(t).color; t.remove();"
            " return [b.getBoundingClientRect().height, s.borderBottomColor === line && s.borderBottomWidth === '1px', [...l.children].map(c => c.textContent),"
            " l.getBoundingClientRect().left > h.getBoundingClientRect().right && Math.abs(l.getBoundingClientRect().bottom - h.getBoundingClientRect().bottom) < 6,"
            " [...b.querySelectorAll('.chip')].filter(c => !c.hidden).map(c => c.textContent)]")
        check("the header: a bar of 56 px over its divider, its length and weight beside the title, no chip at rest (D89, board 4)",
              bar == [56, True, ["16 min", "audio 8 ko"], True, []], bar)
        beside = browser.execute_script(
            "const f = document.querySelector('.transcript .top .finding'), r = document.getElementById('redo-transcript'), a = f.getBoundingClientRect(), b = r.getBoundingClientRect();"
            " return [r.parentElement === f.parentElement, Math.abs((a.top + a.bottom) - (b.top + b.bottom)) < 4 && b.left >= a.right]")
        check("the transcript's Refaire… beside its search, at its right (D89, board 4)", beside == [True, True], beside)
        actions = browser.execute_script(
            "return ['open-folder', 'copy-minutes', 'redo-minutes', 'redo-transcript'].map(id => document.getElementById(id)).map(e => [e.id, e.textContent.trim(), e.getAttribute('aria-label'), e.title,"
            " e.classList.contains('icon-button') && e.querySelectorAll('svg.glyph').length === 1])")
        searched = browser.execute_script("return !!document.getElementById('search').previousElementSibling?.matches('svg.glyph')")
        check("the transcript's search holds its icon, as the sidebar's (D89)", searched)
        check("each secondary action of the meeting: an icon button, its tooltip and its name (D89)", actions == [
            ["open-folder", "", "Ouvrir le dossier", "Ouvrir le dossier", True], ["copy-minutes", "", "Copier le compte rendu", "Copier le compte rendu", True],
            ["redo-minutes", "", "Refaire…", "Refaire…", True], ["redo-transcript", "", "Refaire…", "Refaire…", True]], actions)
        # D89, board 4: the meeting's settings in one row of 32 px controls; the context opened from it.
        row = browser.execute_script(
            "const r = document.querySelector('#meeting .settings-row'); if (!r) return null;"
            " return [...r.children].map(e => [e.id || e.querySelector('select, input')?.id, Math.round(e.getBoundingClientRect().top), Math.round(e.getBoundingClientRect().height)])")
        tops = {top for _, top, _ in row or []}
        check("the meeting's settings: context, spoken language, Confidentielle, on one row of 32 px (D89)",
              row is not None and [id_ for id_, _, _ in row] == ["context-toggle", "meeting-language", "confidential-switch"]
              and len(tops) == 1 and {height for _, _, height in row} == {32}, row)
        # « Confidentielle »: a sliding switch, the checkbox drawn otherwise, what it means read with it (D89, board 4).
        drawn = browser.execute_script(
            "const b = document.getElementById('confidential-switch'), s = getComputedStyle(b), d = document.getElementById(b.getAttribute('aria-describedby'));"
            " return [b.type, b.getAttribute('role'), s.webkitAppearance || s.appearance, Math.round(b.getBoundingClientRect().width), Math.round(b.getBoundingClientRect().height),"
            " b.labels[0].textContent, d?.textContent, !!d && !b.labels[0].contains(d)]")
        check("« Confidentielle »: a switch drawn sliding, 32 × 20, named by its word, « Rien ne quitte cet ordinateur » its description (D89)",
              drawn == ["checkbox", "switch", "none", 32, 20, "Confidentielle", "Rien ne quitte cet ordinateur", True], drawn)
        listed = browser.execute_script(
            "const p = document.getElementById('meeting-language'), s = getComputedStyle(p);"
            " return [s.webkitAppearance || s.appearance, !!p.nextElementSibling?.matches('svg.glyph.chevron')]")
        check("the spoken language: its value and a chevron, the system's box not drawn (D89, board 4)", listed == ["none", True], listed)
        toggle = browser.find_element(By.ID, "context-toggle")
        closed = toggle.get_attribute("aria-expanded") == "false" and not browser.find_element(By.ID, "context-text").is_displayed()
        toggle.click()
        opened = wait_for(browser, lambda b: b.find_element(By.ID, "context-text").is_displayed(), 2)
        check("the context closed in its row, opened by it, the focus in its field (D89)",
              closed and opened and toggle.get_attribute("aria-expanded") == "true" and browser.execute_script("return document.activeElement.id") == "context-text",
              (closed, toggle.get_attribute("aria-expanded")))
        # « Plus d’actions »: a menu button — Enter opens it on its first item, Escape closes it, the focus back on it.
        more = browser.find_element(By.ID, "more-actions")
        labelled = [more.get_attribute(a) for a in ("aria-label", "title", "aria-haspopup", "aria-expanded")]
        browser.execute_script("arguments[0].focus()", more)
        ActionChains(browser).send_keys(Keys.ENTER).perform()
        menu = wait_for(browser, lambda b: b.find_element(By.ID, "more-menu").is_displayed(), 2)
        expanded = more.get_attribute("aria-expanded")
        items = browser.execute_script("return [...document.querySelectorAll('#more-menu [role=menuitem]')].map(e => e.textContent)")
        first = browser.execute_script("return [document.activeElement.getAttribute('role'), document.activeElement.textContent, document.getElementById('more-menu').getAttribute('role')]")
        ActionChains(browser).send_keys(Keys.ESCAPE).perform()
        back = wait_for(browser, lambda b: not b.find_element(By.ID, "more-menu").is_displayed(), 2)
        check("« Plus d’actions »: a menu of the rarer actions, opened by Enter on its first, Escape closes it, the focus back on its button (D89)",
              labelled == ["Plus d’actions", "Plus d’actions", "menu", "false"] and menu and items == ["Renommer cette réunion"]
              and first == ["menuitem", "Renommer cette réunion", "menu"] and back and browser.execute_script("return document.activeElement.id") == "more-actions"
              and more.get_attribute("aria-expanded") == "false" and expanded == "true", (labelled, items, first, expanded))
        more.click()
        wait_for(browser, lambda b: b.find_element(By.ID, "more-menu").is_displayed(), 2)
        browser.find_element(By.CSS_SELECTOR, "#more-menu [role=menuitem]").click()
        naming = wait_for(browser, lambda b: b.execute_script("return document.activeElement.id") == "naming", 2)
        ActionChains(browser).send_keys(Keys.ESCAPE).perform()
        check("its « Renommer cette réunion » opens the name's field, Escape leaves it as it was (D89)",
              naming and wait_for(browser, lambda b: b.find_elements(By.ID, "rename"), 2) and not browser.find_element(By.ID, "more-menu").is_displayed())
        browser.find_elements(By.CSS_SELECTOR, ".session")[1].click()
        wait_for(browser, lambda b: b.find_element(By.ID, "meeting").is_displayed() and b.find_elements(By.CSS_SELECTOR, ".minutes .none"))
        kept = browser.execute_script("return document.getElementById('search') && document.getElementById('search').value")
        check("another meeting opens without the last one's search", not kept, kept)
        notes = [p.text for p in browser.find_elements(By.CSS_SELECTOR, "#meeting .none") if p.is_displayed()]
        check("minutes that cannot be read are said so", notes == ["Ce fichier ne peut pas être lu par cette version de Kitokey."], notes)
        chips = browser.find_element(By.CSS_SELECTOR, "#meeting .meeting-head").text
        check("minutes that cannot be read claim nothing in the header: their card says them (D89)", "Compte rendu" not in chips, chips)
        being = [p.text for p in browser.find_elements(By.CSS_SELECTOR, ".transcript .note")]
        check("a transcript without its end says it is being written", being == ["En cours d’écriture — ce qui est là est vrai quand même."], being)

        browser.execute_script("location.hash = '#/meeting/' + encodeURIComponent('gone')")
        listed = wait_for(browser, lambda b: b.execute_script("return location.hash") == "#/")
        drawn = wait_for(browser, lambda b: b.execute_script("return document.getElementById('meeting').dataset.name") == "m", 5)
        check("a meeting that is not there goes back to the list, the last meeting beside it (D61)", listed and drawn,
              browser.execute_script("return document.getElementById('meeting').dataset.name"))

    print("K. the settings: each choice saved at once, the user's file kept")
    models = WORK / "data" / "kitokey" / "models"
    models.mkdir(parents=True, exist_ok=True)
    for name in ("ggml-large-v3-turbo-q8_0.bin", "ggml-small.bin"):
        (models / name).write_bytes(b"x" * 1000)
    chosen = WORK / "chosen"
    chosen.mkdir()
    window = Window(chosen, "fr_FR.UTF-8", "# mine\n")
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed() and b.find_elements(By.ID, "model"))
        check("the settings open from the sidebar's foot, an icon named Réglages", browser.find_element(By.ID, "settings-link").get_attribute("aria-label") == "Réglages")
        shot(browser, "4-settings")
        browser.execute_script("window.scrollTo(0, document.body.scrollHeight)")
        shot(browser, "4-settings-end")
        browser.execute_script("window.scrollTo(0, 0)")
        checked = browser.execute_script("return document.querySelector('input[name=after]:checked').value")
        check("out of the box, the transcript follows", checked == "transcript", checked)
        names = [o.text.split(" — ")[0] for o in browser.find_elements(By.CSS_SELECTOR, "#model option")]
        check("the models on this computer are offered, by the names people say", names == ["Whisper large v3 turbo", "Whisper small", "OpenRouter", "OpenAI", "Un service que je choisis"], names)
        ready = browser.find_element(By.ID, "model-state").text
        check("the transcription model's state is said at the page's end", "Prêt" in ready and "sans réseau" in ready, ready)
        # Each change is saved, then the page is drawn again: the next acts on the new one.
        # The page drawn again under the driver — the window coming back to the front draws it anew,
        # the desktop being in use —: the gesture made again on the new one.
        def changed(act):
            for _ in range(5):
                before = browser.find_element(By.ID, "reminder")
                try:
                    act()
                    break
                except StaleElementReferenceException:
                    time.sleep(0.2)
            WebDriverWait(browser, 5).until(staleness_of(before))

        # Back to Settings from another page: its last drawing stays in the hidden page until the
        # new one replaces it — waited for, so that nothing is clicked or read on the old one.
        def to_settings():
            old = browser.find_elements(By.ID, "reminder")
            browser.execute_script("location.hash = '#/settings'")
            if old:
                WebDriverWait(browser, 5).until(staleness_of(old[0]))
            wait_for(browser, lambda b: b.find_elements(By.ID, "developer"))

        # A refusal is said on the page drawn after the save: waited for, never read too soon.
        def not_saved():
            wait_for(browser, lambda b: b.find_elements(By.ID, "not-saved") and b.find_element(By.ID, "not-saved").text, 5)
            return browser.find_element(By.ID, "not-saved").text

        changed(lambda: browser.find_element(By.CSS_SELECTOR, "input[name=after][value=nothing]").click())
        changed(lambda: Select(browser.find_element(By.ID, "keep")).select_by_value("30"))
        changed(lambda: Select(browser.find_element(By.ID, "model")).select_by_visible_text(
            next(o.text for o in browser.find_elements(By.CSS_SELECTOR, "#model option") if o.text.startswith("Whisper small"))
        ))
        # The user changes the file by hand while Kitokey runs: the next save keeps it.
        window.file.write_text(window.file.read_text().replace("keep_audio = 30", "keep_audio = 90 # by hand"))
        changed(lambda: browser.find_element(By.ID, "reminder").click())
        changed(lambda: browser.find_element(By.ID, "developer").click())
        # When an application takes the microphone (D30, board 4): offer to record by default;
        # nothing chosen here, then the offer again, each written.
        offer_on = read(browser, lambda b: Select(b.find_element(By.ID, "offer")).first_selected_option.get_attribute("value")) == "offer"
        changed(lambda: Select(browser.find_element(By.ID, "offer")).select_by_value("nothing"))
        offer_off = "offer = false" in window.file.read_text()
        changed(lambda: Select(browser.find_element(By.ID, "offer")).select_by_value("offer"))
        offer_again = "offer = true" in window.file.read_text()
        changed(lambda: Select(browser.find_element(By.ID, "offer")).select_by_value("nothing"))
        # Started with the user's session (D32): on from the first start, the desktop's own entry;
        # off, then turned off by the desktop's tools, then on again, each read from the entry.
        entry = window.file.parent.parent / "autostart" / "com.napkinstack.Kitokey.desktop"
        switch = lambda: browser.find_element(By.ID, "autostart")
        start_on = read(browser, lambda b: switch().is_selected()) and entry.is_file() and "Exec=kitokey\n" in entry.read_text()
        changed(lambda: switch().click())
        start_off = not read(browser, lambda b: switch().is_selected()) and not entry.exists() and "autostart = false" in window.file.read_text()
        entry.write_text("[Desktop Entry]\nType=Application\nExec=kitokey\nX-GNOME-Autostart-enabled=false\n")
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 10)
        to_settings()
        tools_off = not read(browser, lambda b: switch().is_selected())
        changed(lambda: switch().click())
        start_again = read(browser, lambda b: switch().is_selected()) and "TryExec=kitokey\n" in entry.read_text() and "autostart = true" in window.file.read_text()
        check("the session starts Kitokey from its first start; turned off here or by the desktop's tools, on again",
              start_on and start_off and tools_off and start_again, (start_on, start_off, tools_off, start_again))
        text = window.file.read_text()
        check(
            "each choice is in the settings file, the user's lines kept, a change by hand too",
            'after = "nothing"' in text
            and "reminder = false" in text
            and "developer = true" in text
            and offer_on and offer_off and offer_again and "offer = false" in text
            and "keep_audio = 90 # by hand" in text
            and f'model = "{models / "ggml-small.bin"}"' in text
            and text.startswith(f'sessions = "{chosen}"\n')
            and "# mine" in text,
            text,
        )
        # On, the developer mode offers the logs' level and their folder (D29).
        check("the developer mode offers the logs' level and their folder",
              browser.find_elements(By.ID, "log-level") and browser.find_elements(By.ID, "open-logs"))
        changed(lambda: Select(browser.find_element(By.ID, "log-level")).select_by_value("debug"))
        check("the logs' level chosen is in the file", 'log = "debug"' in window.file.read_text(), window.file.read_text())
        changed(lambda: Select(browser.find_element(By.ID, "log-level")).select_by_value("info"))
        check("the usual level chosen again is in the file", 'log = "info"' in window.file.read_text(), window.file.read_text())
        changed(lambda: browser.find_element(By.ID, "developer").click())
        check("the developer mode turned off again is said so in the file, and hides its rows",
              "developer = false" in window.file.read_text() and not browser.find_elements(By.ID, "log-level"), window.file.read_text())
        failed = browser.find_element(By.ID, "not-saved").is_displayed()
        check("nothing said failed", not failed)
        # Any number of days, when the list has not the one wanted.
        choose(browser, "keep", "other")
        typed = browser.find_element(By.ID, "keep-other")
        changed(lambda: (typed.send_keys("45"), typed.send_keys(Keys.ENTER)))
        kept = window.file.read_text()
        offered = read(browser, lambda b: [o.text for o in b.find_elements(By.CSS_SELECTOR, "#keep option")])
        check("any number of days can be typed, and is offered after", "keep_audio = 45" in kept and "45 jours" in offered, (kept, offered))
        choose(browser, "keep", "other")
        typed_days = browser.find_element(By.ID, "keep-other")
        typed_days.send_keys("0", Keys.ENTER)
        refused = wait_for(browser, lambda b: b.find_element(By.ID, "not-saved").is_displayed(), 5)
        check(
            "a number that is no number of days is said, nothing saved",
            refused and "0 n’est pas un nombre de jours" in browser.find_element(By.ID, "not-saved").text and "keep_audio = 45" in window.file.read_text(),
            browser.find_element(By.ID, "not-saved").text,
        )
        # The shortcut: the keys the desktop granted, or that it granted none; its keyboard
        # settings offered where they can be opened — never opened here.
        keys = browser.find_element(By.ID, "shortcut").text
        offered = bool(browser.find_elements(By.ID, "shortcut-settings"))
        gnome = "gnome" in os.environ.get("XDG_CURRENT_DESKTOP", "").lower().split(":")
        seen = "none granted" if keys == "Aucun raccourci défini" else "keys granted"
        check(
            f"the shortcut is shown as the desktop granted it ({seen} here), and where it changes",
            keys != "" and offered == (gnome and shutil.which("gnome-control-center") is not None),
            f"{keys!r}, settings offered: {offered}",
        )
        # The sessions' folder: typed, made when it is not there, and what the list reads.
        elsewhere = WORK / "elsewhere" / "Kitokey"
        spoken = browser.find_element(By.ID, "sessions-shown").get_attribute("textContent")
        # Outside the home folder, from the root: / tmp / … / chosen.
        check("the sessions' folder is shown as people say it", spoken == "/ " + " / ".join(chosen.parts[1:]), spoken)

        def folder_field():
            # A click on the folder as said makes it a path to type.
            browser.find_element(By.ID, "sessions-shown").click()
            return browser.find_element(By.ID, "sessions")

        typed = lambda field, text: (browser.execute_script("arguments[0].select()", field), field.send_keys(text, Keys.ENTER))  # noqa: E731
        folder = folder_field()
        check("a click makes it the path, to type", folder.get_attribute("value") == str(chosen), folder.get_attribute("value"))
        changed(lambda: typed(folder, str(elsewhere)))
        text = window.file.read_text()
        check("a folder typed is made, and in the settings file", elsewhere.is_dir() and f'sessions = "{elsewhere}"' in text, text)
        folder = folder_field()
        changed(lambda: typed(folder, "Kitokey"))
        said = not_saved()
        check("a folder that is not a whole path is said, and nothing changes", said.startswith("Le réglage n’a pas été enregistré") and window.file.read_text() == text, said)
        browser.execute_script("location.hash = '#/'")
        listed = wait_for(browser, lambda b: str(elsewhere) in b.find_element(By.ID, "none").text, 5)
        check("the list reads the folder chosen", listed, browser.find_element(By.ID, "none").text)
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "reminder"))
        # This computer's Ollama saved, and not answering (here, nothing listens on its port): said
        # so, never "it has no such model" from a list that could not be read.
        kept = window.file.read_text()
        window.file.write_text(kept + 'minutes_provider = "http://127.0.0.1:9/v1"\nminutes_model = "mistral:7b"\n')
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/settings'")
        silent = wait_for(browser, lambda b: b.find_elements(By.ID, "minutes-silent"), 5)
        said = read(browser, lambda b: " ".join(p.text for p in b.find_elements(By.CSS_SELECTOR, "#minutes-where, #minutes-silent, #minutes-lacking")))
        check("this computer's Ollama not answering: said so, here, never a model said missing",
              silent and "Sur cet ordinateur" in said and "ne répond pas" in said and not browser.find_elements(By.ID, "minutes-lacking"), said)
        window.file.write_text(kept)
        browser.execute_script("location.hash = '#/'")
        to_settings()
        # A file broken by hand is never written over: the change is said not saved.
        # Developer mode off, the sentences alone, read whole; on, the raw text under each, without
        # the file's path (D47).
        held = window.file.read_text()
        lines = [line for line in held.splitlines() if not line.startswith("keep_audio")] + ["keep_audio = 0"]
        broken = "\n".join(lines) + "\n"
        refused = "Le réglage n’a pas été enregistré\u00a0: le fichier des réglages ne peut pas être lu\u00a0: corrigez-le, puis réessayez.".replace("\xa0", " ")  # WebDriver's `.text` reads U+00A0 as a space
        locked_said = "Le fichier des réglages ne peut pas être lu\u00a0: Kitokey ne l’écrit pas tant qu’il n’est pas corrigé.".replace("\xa0", " ")  # WebDriver's `.text` reads U+00A0 as a space
        unreadable = lambda: browser.find_element(By.ID, "settings-unreadable").text  # noqa: E731
        window.file.write_text(broken)
        changed(lambda: browser.find_element(By.ID, "reminder").click())
        said = not_saved()
        check("a file that cannot be read is not written over", said == refused and window.file.read_text() == broken, said)
        check("developer mode off, a settings file that cannot be read is said in a sentence, no raw text", unreadable() == locked_said, unreadable())
        window.file.write_text(held)
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        wait_for(browser, lambda b: not b.find_element(By.ID, "reminder").get_attribute("disabled"), 5)
        changed(lambda: browser.find_element(By.ID, "developer").click())
        check("developer mode on again, to read a failure", "developer = true" in window.file.read_text(), window.file.read_text())
        held = window.file.read_text()
        lines = [line for line in held.splitlines() if not line.startswith("keep_audio")] + ["keep_audio = 0"]
        window.file.write_text("\n".join(lines) + "\n")
        raw = f'settings.toml, line {len(lines)}: keep_audio is "always" or a number of days'
        changed(lambda: browser.find_element(By.ID, "reminder").click())
        said = not_saved()
        check("developer mode on, the raw text is under the sentence of the command that failed, without its path", said == f"{refused}\n{raw}", said)
        check("developer mode on, the raw text is under the settings file's sentence too", unreadable() == f"{locked_said}\n{raw}", unreadable())
    # Who writes the minutes: an Ollama made here, an online service, an address of the user's. A
    # key goes to the scenario's own keyring, which lives as long as the program.
    class Tags(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            body = json.dumps({"models": [{"name": "qwen3:14b"}, {"name": "mistral:7b"}]}).encode()
            self.send_response(200)
            self.send_header("content-length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_):
            pass

    tags = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Tags)
    threading.Thread(target=tags.serve_forever, daemon=True).start()
    ollama = f"http://127.0.0.1:{tags.server_address[1]}"
    writers = WORK / "writers"
    writers.mkdir()
    window = Window(writers, "fr_FR.UTF-8", ollama=ollama)
    with window as browser:
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "minutes"))
        offered = browser.execute_script("return [...document.querySelectorAll('#minutes option')].map(o => o.textContent)")
        check(
            "who can write the minutes: Ollama's models, the online services, one's own",
            offered == ["Pas encore choisi", "mistral:7b", "qwen3:14b", "OpenRouter", "OpenAI", "Mistral", "Personnalisé…"],
            offered,
        )
        # Going through the list never saves: what is chosen is saved on the word (D25).
        before = browser.find_element(By.ID, "minutes")
        Select(before).select_by_visible_text("qwen3:14b")
        Select(before).select_by_visible_text("mistral:7b")
        time.sleep(1)
        unsaid = browser.find_element(By.ID, "minutes-unsaved")
        shot(browser, "8-minutes-unsaved")
        check(
            "a model chosen in the list is not saved by itself; who still writes them is said",
            "minutes_" not in window.file.read_text() and unsaid.is_displayed() and unsaid.text == "Pas encore enregistré — aucun compte rendu n’est fait tant que ce n’est pas le cas.",
            (window.file.read_text(), unsaid.text),
        )
        browser.find_element(By.ID, "minutes-save").click()
        WebDriverWait(browser, 5).until(staleness_of(before))
        local = wait_for(browser, lambda b: "mistral:7b" in window.file.read_text(), 5)
        check(
            "a model of Ollama is saved on the word, nothing left unsaid",
            local and f'minutes_provider = "{ollama}/v1"' in window.file.read_text() and not browser.find_element(By.ID, "minutes-unsaved").is_displayed(),
            window.file.read_text(),
        )
        # This computer's Ollama with a model it does not have: said as here — nothing leaves —,
        # and the model it lacks named; never "online, the transcript's text is sent".
        saved = window.file.read_text()
        window.file.write_text(saved.replace('minutes_model = "mistral:7b"', 'minutes_model = "absent:latest"'))
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/settings'")
        lacking = wait_for(browser, lambda b: b.find_elements(By.ID, "minutes-lacking"), 5)
        said = " ".join(p.text for p in browser.find_elements(By.CSS_SELECTOR, "#minutes-where, #minutes-lacking"))
        check(
            "this computer's Ollama with a model it lacks: said here, nothing leaves, the model named",
            lacking and "Sur cet ordinateur" in said and "En ligne" not in said and "absent:latest" in said,
            said,
        )
        window.file.write_text(saved)
        # The page drawn anew from the file just written: its last drawing waited out, so that the
        # old Tester, hidden, is never the one pressed (as #302 found of the agreement page).
        browser.execute_script("location.hash = '#/'")
        to_settings()
        wait_for(browser, lambda b: b.find_elements(By.ID, "minutes-test"), 5)
        # Tester asks kitokey-summary about the choice saved: no key for Ollama here.
        press(browser, "minutes-test")
        answered = wait_for(browser, lambda b: b.find_element(By.ID, "minutes-tested").text == "Ça marche : le modèle répond.", 10)
        check(
            "Tester says the model answers, asking kitokey-summary with the choice saved",
            answered
            and (WORK / "summary-args").read_text().split() == ["--test", "--provider", f"{ollama}/v1", "--model", "mistral:7b"]
            and (WORK / "summary-key").read_text() == "none",
            browser.find_element(By.ID, "minutes-tested").text,
        )
        Select(browser.find_element(By.ID, "minutes")).select_by_visible_text("OpenRouter")
        key = browser.find_element(By.ID, "minutes-key")
        check("a key is typed where it is not shown, and never filled in", key.get_attribute("type") == "password" and key.get_attribute("value") == "")
        browser.find_element(By.ID, "minutes-model").send_keys("anthropic/claude-sonnet")
        key.send_keys("sk-scenario")
        before = browser.find_element(By.ID, "minutes")
        browser.find_element(By.ID, "minutes-save").click()
        # Saved, the page is drawn again: what follows acts on the new one.
        WebDriverWait(browser, 5).until(staleness_of(before))
        online = wait_for(browser, lambda b: "openrouter.ai" in window.file.read_text(), 5)
        text = window.file.read_text()
        check(
            "an online service, with its address and the model named",
            online and 'minutes_provider = "https://openrouter.ai/api/v1"' in text and 'minutes_model = "anthropic/claude-sonnet"' in text,
            text,
        )
        kept = browser.find_element(By.ID, "minutes-key")
        browser.execute_script("arguments[0].scrollIntoView({block: 'center'})", kept)
        shot(browser, "4-settings-online")
        check(
            "the key is kept, said so, and never shown back nor written in the settings",
            kept.get_attribute("value") == "" and kept.get_attribute("placeholder").startswith("Une clé est gardée") and "sk-scenario" not in text,
            kept.get_attribute("placeholder"),
        )
        # Each cause its own sentence (D72).
        code = WORK / "test-code"
        try:
            for cause, sentence in TEST_CAUSES:
                free(browser, "minutes-test", f"the minutes, {cause}")
                code.write_text(cause)
                tester_click(browser, "minutes-test", f"the minutes, {cause}")
                held = wait_for(browser, lambda b: b.find_element(By.ID, "minutes-tested").get_attribute("textContent") == sentence, 10)
                check(f"Tester of the minutes, {cause}: its own sentence, what to do, no raw text", held, browser.find_element(By.ID, "minutes-tested").get_attribute("textContent"))
                # A second test in flight once the sentence is read: the next case waits for it (#799).
                tester_click(browser, "minutes-test", f"the minutes, {cause} again")
            # In developer mode, the sentence and the program's last line, its name off.
            changed(lambda: browser.find_element(By.ID, "developer").click())
            free(browser, "minutes-test", "the minutes in developer mode")
            tester_click(browser, "minutes-test", "the minutes in developer mode")
            raw = wait_for(browser, lambda b: "raw words of the provider" in b.find_element(By.ID, "minutes-tested").get_attribute("textContent"), 10)
            told = browser.find_element(By.ID, "minutes-tested").get_attribute("textContent")
            check("Tester in developer mode: the cause, then the raw line alone", raw and told.startswith(TEST_CAUSES[-1][1]) and "kitokey-summary:" not in told and "earlier line" not in told, told)
            changed(lambda: browser.find_element(By.ID, "developer").click())
            text = window.file.read_text()
        finally:
            code.unlink(missing_ok=True)
        free(browser, "minutes-test", "the minutes refused")
        tester_click(browser, "minutes-test", "the minutes refused")
        refused = wait_for(browser, lambda b: b.find_element(By.ID, "minutes-tested").text.startswith("Le modèle ne répond pas"), 10)
        said = browser.find_element(By.ID, "minutes-tested").text
        check(
            "a model that does not answer is said in one sentence; the key goes by the environment, never the arguments",
            refused
            and said.endswith("vérifiez son adresse, son nom et sa clé, puis réessayez.")
            and "openrouter.ai" not in said
            and (WORK / "summary-key").read_text() == "sk-scenario"
            and "sk-scenario" not in (WORK / "summary-args").read_text(),
            said,
        )
        Select(browser.find_element(By.ID, "minutes")).select_by_visible_text("Personnalisé…")
        browser.find_element(By.ID, "minutes-address").send_keys("not an address")
        browser.find_element(By.ID, "minutes-model").send_keys("m")
        before = browser.find_element(By.ID, "minutes")
        browser.find_element(By.ID, "minutes-save").click()
        # Refused, the page is drawn again too, the refusal said on the new one.
        WebDriverWait(browser, 5).until(staleness_of(before))
        said = wait_for(browser, lambda b: b.find_element(By.ID, "not-saved").is_displayed(), 5)
        check("an address that is none is said, and nothing changes", said and window.file.read_text() == text, browser.find_element(By.ID, "not-saved").text)
    broken = WORK / "broken"
    broken.mkdir()
    # No window named: a file that cannot be read opens the full one, where it is said (D35).
    window = Window(broken, "fr_FR.UTF-8", "keep_audio = 0\n", small=True)
    with window as browser:
        check("a settings file that cannot be read opens the full window, not the small one", not browser.find_elements(By.ID, "small-record") and browser.find_element(By.ID, "settings-link").is_displayed())
        browser.find_element(By.ID, "settings-link").click()
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#settings .warn"))
        warned = browser.find_element(By.CSS_SELECTOR, "#settings .warn").text
        locked = browser.execute_script("return [...document.querySelectorAll('#settings input, #settings select')].every(e => e.disabled)")
        check("a settings file that cannot be read is said so, and nothing can be changed", warned == "Le fichier des réglages ne peut pas être lu : Kitokey ne l’écrit pas tant qu’il n’est pas corrigé." and locked, warned)
        check("it is left as it was", window.file.read_text() == f'sessions = "{broken}"\nwelcomed = true\nkeep_audio = 0\n')
        # The shell itself refuses, not only the page.
        refused = browser.execute_async_script(
            "const done = arguments[0]; window.__TAURI__.core.invoke('settings_saved', {change: {reminder: false}})"
            ".then(() => done('saved'), why => done('refused: ' + why))"
        )
        check("the shell refuses a change while the file cannot be read", refused.startswith("refused"), refused)
        # Corrected by hand, the page opens again when the window comes forward.
        window.file.write_text(f'sessions = "{broken}"\nkeep_audio = 7\n')
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        open_again = wait_for(browser, lambda b: not b.find_element(By.ID, "reminder").get_attribute("disabled"), 5)
        check("a file corrected by hand opens the page again", open_again and not browser.find_elements(By.CSS_SELECTOR, "#settings .warn:not([hidden])")[1:],
              browser.execute_script("return [document.getElementById('reminder')?.disabled, [...document.querySelectorAll('#settings .warn:not([hidden])')].map(e => e.textContent), [...document.querySelectorAll('#settings:not([hidden]) .fields input')].filter(i => i.value !== i.defaultValue).map(i => i.id)]"))

    print("N. before a meeting first leaves for a host: what leaves, to whom; sent only on the word")
    agreeing = WORK / "agreeing"
    asked = agreeing / "2026-10-18_11-00"
    asked.mkdir(parents=True)
    (asked / "session.json").write_text('{"contract":"session-audio/v2","started":"2026-10-18T11:00:00+02:00","stopped":"2026-10-18T11:20:00+02:00","tracks":{}}')
    (asked / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":null,"context":null}\n'
        '{"type":"segment","speaker":"others","track":"remote","start":0.0,"end":2.0,"text":"Le budget est validé."}\n'
        '{"type":"segment","speaker":"me","track":"local","start":2.0,"end":4.0,"text":"Je préviens Alexandre."}\n'
        '{"type":"complete","duration":1200.0}\n'
    )
    (asked / "context.txt").write_text("Réunion budget, trois personnes.\n")
    # A second meeting, its minutes asked while the first one's are written (D25).
    other = agreeing / "2026-10-18_14-00"
    other.mkdir()
    for name in ("session.json", "transcript.jsonl"):
        (other / name).write_text((asked / name).read_text().replace("T11:", "T14:"))
    (WORK / "summary-args").unlink(missing_ok=True)
    online = 'minutes_provider = "https://openrouter.ai/api/v1"\nminutes_model = "a-model"\n'
    window = Window(agreeing, "fr_FR.UTF-8", online, ollama=ollama)
    with window as browser:
        # To the agreement page: its last drawing stays in the hidden page until the new one
        # replaces it — waited for, so that nothing is clicked on the old one (as #302 found).
        def to_agree(ready):
            # Read while the page replaces it — after `#/` — the drawing cannot be held: read
            # again, as `press` does (#309).
            for _ in range(10):
                try:
                    old = browser.find_elements(By.CSS_SELECTOR, "#agree > *")
                    break
                except StaleElementReferenceException:
                    time.sleep(0.1)
            else:
                old = browser.find_elements(By.CSS_SELECTOR, "#agree > *")
            browser.execute_script("location.hash = '#/agree/2026-10-18_11-00'")
            if old:
                WebDriverWait(browser, 5).until(staleness_of(old[0]))
            wait_for(browser, lambda b: b.find_elements(By.ID, ready))

        to_agree("agree-send")
        shot(browser, "agreement")
        page = browser.find_element(By.ID, "agree").text
        check(
            "what leaves is said — the transcript, its length and words, the context —, what stays, to whom",
            "Le texte de «" in page and "» va quitter votre ordinateur" in page
            and "20 minutes, environ 7 mots" in page and "Environ 4 mots" in page
            and "Reste ici : l’audio" in page and "openrouter.ai" in page and "a-model" in page,
            page,
        )
        check("nothing has left before the word", not (WORK / "summary-args").exists())
        check("not asking again is the user's tick, unticked", not browser.find_element(By.ID, "agree-remember").is_selected())
        # The provider changed in the file while the page asks: the word was not for that one.
        window.file.write_text(window.file.read_text().replace("https://openrouter.ai/api/v1", "https://api.mistral.ai/v1"))
        press(browser, "agree-send")
        refused = wait_for(browser, lambda b: any(said.is_displayed() for said in b.find_elements(By.ID, "not-saved")), 5)
        check(
            "the word is for the host shown: chosen otherwise meanwhile, nothing is sent",
            refused and not (WORK / "summary-args").exists() and "minutes_agreed" not in window.file.read_text(),
            [said.text for said in browser.find_elements(By.ID, "not-saved")],
        )
        window.file.write_text(window.file.read_text().replace("https://api.mistral.ai/v1", "https://openrouter.ai/api/v1"))
        browser.execute_script("location.hash = '#/'")
        to_agree("agree-send")
        press(browser, "agree-send")
        sent = wait_for(browser, lambda b: (asked / "minutes.md").exists(), 10)
        check(
            "on the word, this meeting is sent — once: the host is not kept",
            sent and "minutes_agreed" not in window.file.read_text()
            and (WORK / "summary-args").read_text().split() == [str(asked), "--provider", "https://openrouter.ai/api/v1", "--model", "a-model"],
            window.file.read_text(),
        )
        check("then the meeting is shown", wait_for(browser, lambda b: b.execute_script("return location.hash") == "#/meeting/2026-10-18_11-00", 5))
        # Ticked: this meeting and the next ones go to that service without asking.
        (asked / "minutes.md").unlink()
        to_agree("agree-remember")
        press(browser, "agree-remember")
        press(browser, "agree-send")
        kept = wait_for(browser, lambda b: (asked / "minutes.md").exists(), 10)
        check("ticked, the host is kept for the next ones", kept and 'minutes_agreed = "openrouter.ai"' in window.file.read_text(), window.file.read_text())
        # On this computer instead: nothing goes to the host, nothing is agreed.
        (asked / "minutes.md").unlink()
        (WORK / "summary-args").unlink()
        window.file.write_text(window.file.read_text().replace('minutes_agreed = "openrouter.ai"\n', ""))
        to_agree("agree-local-open")
        browser.find_element(By.ID, "agree-local-open").click()
        Select(browser.find_element(By.ID, "agree-local")).select_by_visible_text("qwen3:14b")
        browser.find_element(By.ID, "agree-local-write").click()
        here = wait_for(browser, lambda b: (asked / "minutes.md").exists(), 10)
        text = window.file.read_text()
        check(
            "or written on this computer, chosen for the next ones too, nothing agreed",
            here and f'minutes_provider = "{ollama}/v1"' in text and 'minutes_model = "qwen3:14b"' in text
            and "minutes_agreed" not in text
            and (WORK / "summary-args").read_text().split()[1:] == ["--provider", f"{ollama}/v1", "--model", "qwen3:14b"],
            text,
        )
        # A meeting without its minutes offers them on its page, made on the word — once the ones
        # before are ended: while they are made, it is not offered.
        idle = lambda b: b.execute_async_script("window.__TAURI__.core.invoke('makings').then(m => arguments[arguments.length - 1](m.minutes === null))")
        wait_for(browser, idle, 10)
        (asked / "minutes.md").unlink()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        # Offered once what was being made for it is read as ended.
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes") and b.find_element(By.ID, "make-minutes").is_displayed())
        heard = browser.find_element(By.CSS_SELECTOR, "#meeting .made .maker").text
        check("the meeting's page says where its transcript was made, on its card (D89)", heard.endswith("sur cet ordinateur"), heard)
        (WORK / "slow-minutes").touch()
        browser.find_element(By.ID, "make-minutes").click()
        state = lambda b: b.find_element(By.ID, "minutes-state").text if b.find_elements(By.ID, "minutes-state") else ""
        shown = wait_for(browser, lambda b: "Rédaction du compte rendu" in state(b), 3)
        said = state(browser)
        check("while they are written, the page says so — what, by which model, for how long", shown and "Qwen3 14b" in said and "0:0" in said, said)
        # Asked meanwhile for another meeting: it waits its turn, said on its page and in the list.
        browser.execute_script("location.hash = '#/meeting/2026-10-18_14-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "make-minutes"))
        browser.find_element(By.ID, "make-minutes").click()
        waits = wait_for(browser, lambda b: "En file d’attente · 1 avant" in state(b), 3)
        check("one asked while another is written waits its turn, said on its page", waits, state(browser))
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: len([c for c in b.find_elements(By.CSS_SELECTOR, "#groups .chip.making") if c.is_displayed()]) == 2, 3)
        rowed = sorted(c.text for c in browser.find_elements(By.CSS_SELECTOR, "#groups .chip.making") if c.is_displayed())
        check("the list says it too, on each meeting's row", rowed == ["Compte rendu…", "En attente"], rowed)
        # While its minutes are written, the meeting is busy: shown, never offered to be freed.
        browser.execute_script("location.hash = '#/free'")
        held = wait_for(browser, lambda b: [r.text for r in b.find_elements(By.CSS_SELECTOR, "#free .freeable.busy")], 3)
        busy = [r.text for r in browser.find_elements(By.CSS_SELECTOR, "#free .freeable.busy")]
        check("while its minutes are written, the meeting is busy on the page that frees space", held and "compte rendu" in " ".join(busy), busy)
        # Elsewhere when they are written — the settings, being changed —: that page is left alone.
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "minutes"), 5)
        browser.execute_script("window.__kept = document.getElementById('settings').firstElementChild")
        made = wait_for(browser, lambda b: (asked / "minutes.md").exists(), 10)
        check("what a meeting lacks is offered, and made on the word", made)
        time.sleep(2)
        check("written while another page is shown, that page is not drawn again under the user",
              browser.execute_script("return document.getElementById('settings').firstElementChild === window.__kept && location.hash === '#/settings'"))
        # The one waiting, now being written, shown: written, its page shows them by itself.
        browser.execute_script("location.hash = '#/meeting/2026-10-18_14-00'")
        turn = wait_for(browser, lambda b: (other / "minutes.md").exists(), 10)
        (WORK / "slow-minutes").unlink()
        check("then the one waiting is made in its turn, never dropped", turn)
        drawn = wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes") and not b.find_element(By.ID, "minutes-state").is_displayed(), 5)
        check("written, the page shows them by itself, the state gone", drawn,
              (state(browser), bool(browser.find_elements(By.ID, "redo-minutes")), browser.execute_async_script("window.__TAURI__.core.invoke('makings').then(arguments[arguments.length - 1])")))
        # The line never waits on an agreement: set aside, said on its page, the next one made.
        invoke = lambda command, args: browser.execute_script("window.__TAURI__.core.invoke(arguments[0], arguments[1])", command, args)
        makings = lambda: browser.execute_async_script("window.__TAURI__.core.invoke('makings').then(arguments[arguments.length - 1])")
        settled = window.file.read_text()
        unchosen = "\n".join(l for l in settled.splitlines() if not l.startswith(("minutes_provider", "minutes_model", "minutes_agreed"))) + "\n"
        (WORK / "slow-minutes").touch()
        invoke("minutes_redo", {"name": "2026-10-18_11-00", "provider": None, "model": None})
        wait_for(browser, lambda b: (makings()["minutes"] or {}).get("name") == "2026-10-18_11-00", 3)
        invoke("minutes_asked", {"name": "2026-10-18_11-00"})
        time.sleep(1)
        check("asked again for the one being written, without a Refaire…: not made a second time", makings()["minutes_waiting"] == [], makings())
        window.file.write_text(unchosen + 'minutes_provider = "https://openrouter.ai/api/v1"\nminutes_model = "a-model"\n')
        invoke("minutes_redo", {"name": "2026-10-18_11-00", "provider": None, "model": None})
        line = wait_for(browser, lambda b: makings()["minutes_waiting"] == ["2026-10-18_11-00"], 3)
        check("a Refaire… with the settings' model while being written: it waits its turn, never dropped", line, makings())
        invoke("minutes_redo", {"name": "2026-10-18_14-00", "provider": f"{ollama}/v1", "model": "mistral:7b"})
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        (WORK / "slow-minutes").unlink()
        passed = wait_for(browser, lambda b: makings()["minutes"] is None and not makings()["minutes_waiting"], 15)
        aside_ = wait_for(browser, lambda b: "En attente de votre accord pour l’envoyer à openrouter.ai" in state(b) and b.find_elements(By.ID, "minutes-agree"), 5)
        check(
            "one needing the user's agreement is set aside, said on its page with the way to give it; the next one made",
            passed and aside_ and (WORK / "summary-args").read_text().split()[-1] == "mistral:7b"
            and browser.execute_script("return location.hash") == "#/meeting/2026-10-18_11-00",
            (makings(), state(browser), (WORK / "summary-args").read_text()),
        )
        browser.execute_script("location.hash = '#/'")
        chip_ = wait_for(browser, lambda b: "Accord requis" in [c.text for c in b.find_elements(By.CSS_SELECTOR, "#groups .chip.making") if c.is_displayed()], 3)
        check("the list says it, on the meeting's row", chip_)
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "minutes-agree"), 5)
        press(browser, "minutes-agree")
        wait_for(browser, lambda b: b.find_elements(By.ID, "agree-not"))
        press(browser, "agree-not")
        kept_ = wait_for(browser, lambda b: makings()["awaiting"] == {"2026-10-18_11-00": "openrouter.ai"}, 3)
        check("not now: still waiting for the word, nothing sent", kept_ and (WORK / "summary-args").read_text().split()[-1] == "mistral:7b", makings())
        # The settings changed to this computer meanwhile: looked at again, made, no word needed.
        invoke("minutes_saved", {"provider": f"{ollama}/v1", "model": "qwen3:14b", "key": None})
        again_ = wait_for(browser, lambda b: (WORK / "summary-args").read_text().split()[-1] == "qwen3:14b" and not makings()["awaiting"] and makings()["minutes"] is None, 10)
        check("set aside, then the settings changed: looked at again and made", again_, (makings(), (WORK / "summary-args").read_text()))
        # Taken from the line with no one chosen any more: said on its page, no window over the user.
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        (WORK / "slow-minutes").touch()
        invoke("minutes_redo", {"name": "2026-10-18_14-00", "provider": f"{ollama}/v1", "model": "mistral:7b"})
        wait_for(browser, lambda b: makings()["minutes"] is not None, 3)
        window.file.write_text(unchosen)
        invoke("minutes_asked", {"name": "2026-10-18_11-00"})
        wait_for(browser, lambda b: makings()["minutes_waiting"] == ["2026-10-18_11-00"], 3)
        (WORK / "slow-minutes").unlink()
        unsaid = wait_for(browser, lambda b: "2026-10-18_11-00" in makings()["failed"], 15)
        time.sleep(1)
        check("taken from the line with no one chosen: said on its page, no settings window opened over the user",
              unsaid and browser.execute_script("return location.hash") == "#/meeting/2026-10-18_11-00", (makings(), browser.execute_script("return location.hash")))
        window.file.write_text(settled)
        # The trash emptied of these, as the user would: the next check counts its own.
        for made_before in (WORK / "data" / "Trash" / "files").glob("minutes-before-*.md"):
            made_before.unlink()
            (WORK / "data" / "Trash" / "info" / f"{made_before.name}.trashinfo").unlink(missing_ok=True)
        # Refaire…: another model in its « Avancé », kept for this meeting (D91); the minutes there go to the trash once replaced.
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"))
        press(browser, "redo-minutes")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-do"))
        browser.execute_script("document.getElementById('redo-panel').scrollIntoView({block: 'center'})")
        shot(browser, "6-redo")
        offered = redo_offered(browser, "redo-panel", "minutes")
        check(
            "Refaire offers, in its « Avancé », Settings' model first, then the models on this computer",
            offered == ["Réglages\u00a0: Qwen3 14b", "Mistral 7b — sur cet ordinateur"],
            offered,
        )
        before = (asked / "minutes.md").read_text()
        redo_pick(browser, "redo-panel", "minutes", 1)
        browser.find_element(By.ID, "redo-do").click()
        def aside(folder):
            return sorted(folder.glob("minutes-before-*.md")) if folder.exists() else []

        trash_files = WORK / "data" / "Trash" / "files"
        redone = wait_for(browser, lambda b: aside(trash_files), 10)
        check(
            "made again with the model chosen, kept for this meeting, Settings unchanged; the minutes there in the trash",
            redone
            and json.loads((asked / "models.json").read_text()).get("minutes", {}).get("model") == "mistral:7b"
            and (WORK / "summary-args").read_text().split()[-1] == "mistral:7b"
            and aside(trash_files)[0].read_text() == before
            and 'minutes_model = "qwen3:14b"' in window.file.read_text()
            and not aside(asked),
            ((WORK / "summary-args").read_text(), aside(trash_files), aside(trash_files) and aside(trash_files)[0].read_text()[:60], before[:60], aside(asked)),
        )
        # Made again and failing — on this computer, then online through the agreement: the minutes
        # there stay, and the notice says a redo, not a first making.
        (WORK / "fail-minutes").touch()
        heard = WORK / "notified-redo"
        monitor = subprocess.Popen(
            ["dbus-monitor", "--session", "interface='org.freedesktop.Notifications',member='Notify'"],
            stdout=open(heard, "w"), stderr=subprocess.DEVNULL,
        )
        time.sleep(0.5)
        before = (asked / "minutes.md").read_text()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"))
        press(browser, "redo-minutes")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-do"))
        redo_pick(browser, "redo-panel", "minutes", 1)
        browser.find_element(By.ID, "redo-do").click()
        told = wait_for(browser, lambda b: "pas été refait" in heard.read_text(), 10)
        check("minutes made again that fail: the ones there stay, and the notice says a redo", told and (asked / "minutes.md").read_text() == before, heard.read_text()[-400:])
        failed = wait_for(browser, lambda b: "n’a pas été fait" in state(b) and b.find_elements(By.ID, "minutes-again"), 5)
        check("the failure said on the meeting's page too, in plain words, with a way to try again — never what the program said",
              failed and "not minutes" not in state(browser) and "not minutes" not in heard.read_text(), (state(browser), heard.read_text()[-400:]))
        # Its cause named by the program, its gesture said, on the page and in the notice (D29).
        check("the cause the program named is said with its gesture, on the page and in the notice",
              "écrit de compte rendu" in state(browser) and "écrit de compte rendu" in heard.read_text(),
              (state(browser), heard.read_text()[-400:]))
        # The developer mode shows what the program said, under the sentence; never in a notice (D29).
        to_settings()
        press(browser, "developer")
        wait_for(browser, lambda b: "developer = true" in window.file.read_text(), 5)
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        detailed = wait_for(browser, lambda b: "the provider's answer is not minutes" in state(b) and "n’a pas été fait" in state(b), 5)
        check("in developer mode, what the program said is under the sentence", detailed, state(browser))
        # And the meeting's diagnostic: its makings' lines, every program's — nothing of a path of the run.
        offered = wait_for(browser, lambda b: b.find_elements(By.ID, "copy-diagnostic"), 5)
        lines = browser.execute_async_script(
            "window.__TAURI__.core.invoke('diagnostic', {name: '2026-10-18_11-00'}).then(arguments[arguments.length - 1])"
        )
        check("in developer mode, the meeting offers its diagnostic: its failed minutes, no path of the run",
              offered and '"shell.minutes_failed"' in lines and str(WORK) not in lines, lines[-600:])
        check("and each track's level, after the logs: one line a track, numbers only",
              lines.count('"type":"levels"') == 2 and lines.rstrip().endswith("}") and '"track":"remote"' in lines.splitlines()[-1], lines[-400:])
        to_settings()
        press(browser, "developer")
        wait_for(browser, lambda b: "developer = false" in window.file.read_text(), 5)
        settled = window.file.read_text()
        window.file.write_text(
            "\n".join(line for line in settled.splitlines() if not line.startswith(("minutes_provider", "minutes_model", "minutes_agreed")))
            + '\nminutes_provider = "https://openrouter.ai/api/v1"\nminutes_model = "a-model"\n'
        )
        # This time a failure no code names: the earlier cause is not said again (D29).
        (WORK / "fail-minutes").unlink()
        (WORK / "fail-plainly").touch()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"))
        press(browser, "redo-minutes")
        redo_pick(browser, "redo-panel", "minutes", 0)
        time.sleep(1)
        since = len(heard.read_text())
        browser.find_element(By.ID, "redo-do").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "agree-send"))
        press(browser, "agree-send")
        # Only what is said after this gesture: a notice may show its title more than once.
        told = wait_for(browser, lambda b: "pas été refait" in heard.read_text()[since:], 10)
        said = heard.read_text()[since:]
        monitor.terminate()
        (WORK / "fail-plainly").unlink()
        check("asked through the agreement, still a redo: its failure said as one", told and "pas été fait" not in said and (asked / "minutes.md").read_text() == before, said[-400:])
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        plain = wait_for(browser, lambda b: "n’a pas été fait" in state(b), 5)
        check("a later failure that names no cause does not show the earlier one's",
              plain and "écrit de compte rendu" not in state(browser) and "écrit de compte rendu" not in said, (state(browser), said[-300:]))
        time.sleep(3)
        left = sorted(p.name for p in asked.glob("minutes*"))
        check("nothing else is made after, no copy left beside", (asked / "minutes.md").read_text() == before and left == ["minutes.md"], (left, (asked / "minutes.md").read_text()[:80], (WORK / "summary-args").read_text()))
        window.file.write_text(settled)
        # A trash that refuses: the minutes replaced stay beside the new ones, never deleted.
        trash = WORK / "data" / "Trash"
        trash.rename(WORK / "data" / "Trash-aside")
        trash.write_text("")
        before = (asked / "minutes.md").read_text()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"))
        press(browser, "redo-minutes")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-do"))
        browser.find_element(By.ID, "redo-do").click()
        # The new minutes written — they name their model —, then a moment for what follows them.
        written = wait_for(browser, lambda b: "model: qwen3:14b" in (asked / "minutes.md").read_text(), 10)
        time.sleep(2)
        kept = [path.read_text() for path in aside(asked)]
        check("a trash that refuses keeps the minutes replaced beside the new ones", written and kept == [before], kept)
        # Made again while the trash still refuses: the copy kept is never written over.
        second = (asked / "minutes.md").read_text()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes"))
        press(browser, "redo-minutes")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-do"))
        redo_pick(browser, "redo-panel", "minutes", 1)
        browser.find_element(By.ID, "redo-do").click()
        written = wait_for(browser, lambda b: "model: mistral:7b" in (asked / "minutes.md").read_text(), 10)
        time.sleep(2)
        kept = sorted(path.read_text() for path in aside(asked))
        check("made again, the copies kept are all there, none written over", written and kept == sorted([before, second]), kept)
        for path in aside(asked):
            path.unlink()
        # Minutes that cannot be set aside are not made over: nothing runs, they stay as they are.
        (asked / "minutes.md").chmod(0o000)
        (WORK / "summary-args").unlink()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-minutes") or b.find_elements(By.ID, "make-minutes"))
        browser.execute_script("window.__TAURI__.core.invoke('minutes_redo', {name: '2026-10-18_11-00', provider: null, model: null})")
        time.sleep(3)
        (asked / "minutes.md").chmod(0o644)
        check(
            "minutes that cannot be set aside are not made over",
            not (WORK / "summary-args").exists() and "model: mistral:7b" in (asked / "minutes.md").read_text() and not aside(asked),
            (WORK / "summary-args").exists(),
        )
        trash.unlink()
        (WORK / "data" / "Trash-aside").rename(trash)
        # Refaire… on the transcript: another model of this computer; the transcript there in the trash.
        heard = (asked / "transcript.jsonl").read_text()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"))
        press(browser, "redo-transcript")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-do"))
        offered = redo_offered(browser, "redo-transcript-panel", "transcription")
        check("Refaire offers, in its « Avancé », Settings' model first, then the Whisper models of this computer",
              offered == ["Réglages\u00a0: Whisper large v3 turbo", "Whisper small"], offered)
        redo_pick(browser, "redo-transcript-panel", "transcription", 1)
        browser.find_element(By.ID, "redo-transcript-do").click()
        redone = wait_for(browser, lambda b: sorted(trash_files.glob("transcript-before-*.jsonl")), 10)
        trashed = sorted(trash_files.glob("transcript-before-*.jsonl"))
        check(
            "the transcript made again with the model picked; the one there in the trash",
            redone
            and "Refait avec ggml-small." in (asked / "transcript.jsonl").read_text()
            and [path.read_text() for path in trashed] == [heard]
            and not sorted(asked.glob("transcript-before-*")),
            (asked / "transcript.jsonl").read_text(),
        )
        # A new transcript that fails: the one there comes back, nothing left aside.
        heard = (asked / "transcript.jsonl").read_text()
        (WORK / "fail-hearing").write_text("")
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"))
        press(browser, "redo-transcript")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-do"))
        browser.find_element(By.ID, "redo-transcript-do").click()
        time.sleep(3)
        (WORK / "fail-hearing").unlink()
        check(
            "a new transcript that fails gives the one there back, nothing left aside",
            (asked / "transcript.jsonl").read_text() == heard and not sorted(asked.glob("transcript-before-*")),
            (asked / "transcript.jsonl").read_text(),
        )
        heard_state = lambda b: b.find_element(By.ID, "transcript-state").text if b.find_elements(By.ID, "transcript-state") else ""
        unmade = wait_for(browser, lambda b: "La transcription n’a pas été faite" in heard_state(b) and b.find_elements(By.ID, "transcript-again"), 5)
        shot(browser, "9-transcript-failed")
        check("what is not known of who wrote the minutes is left unsaid (#312)",
              "— · —" not in browser.find_element(By.ID, "meeting").text and "via —" not in browser.find_element(By.ID, "meeting").text,
              [p.text for p in browser.find_elements(By.CSS_SELECTOR, "#meeting .maker")])
        check("a transcript not made says so on its card in plain words, with a way to try again", unmade and "engine stopped" not in heard_state(browser), heard_state(browser))
        # Tried again as it was asked: made again with the meeting's own model, kept by the Refaire…, not resumed.
        # Made again — the one there set aside, then in the trash —, never resumed from it.
        trashed_before = len(sorted(trash_files.glob("transcript-before-*.jsonl")))
        press(browser, "transcript-again")
        tried = wait_for(browser, lambda b: len(sorted(trash_files.glob("transcript-before-*.jsonl"))) == trashed_before + 1, 10)
        check("Try again after a Refaire… makes it again with the meeting's own model it kept", tried and "Refait avec ggml-small" in (asked / "transcript.jsonl").read_text(), (asked / "transcript.jsonl").read_text() if (asked / "transcript.jsonl").exists() else "")
        back_ = wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"), 5)
        check("Refaire… is offered again once asked, the panel gone", back_)
        # Being made: said on its card, by which model, for how long; a Refaire… meanwhile waits its
        # turn — the one being made too —, then is made, and the card shows it by itself.
        (WORK / "slow-hearing").touch()
        browser.execute_script("location.hash = '#/'")
        browser.execute_script("location.hash = '#/meeting/2026-10-18_11-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript"))
        press(browser, "redo-transcript")
        wait_for(browser, lambda b: b.find_elements(By.ID, "redo-transcript-do"))
        paths = [browser.execute_script("return window.__TAURI__.core.invoke('settings_shown').then(s => s.model)")]
        redo_pick(browser, "redo-transcript-panel", "transcription", 1)
        browser.find_element(By.ID, "redo-transcript-do").click()
        making_ = wait_for(browser, lambda b: "Transcription en cours" in heard_state(b), 3)
        shot(browser, "7-transcript-making")
        check("while it is made, its card says so — what, by which model, for how long",
              making_ and "Whisper small · 0:0" in heard_state(browser) and not browser.find_elements(By.ID, "transcript-again"), heard_state(browser))
        browser.execute_script("window.__TAURI__.core.invoke('transcript_redo', {name: '2026-10-18_11-00', model: arguments[0]})", paths[0])
        lined = lambda b: b.execute_async_script("window.__TAURI__.core.invoke('makings').then(m => arguments[arguments.length - 1](m.transcripts_waiting))")
        queued = wait_for(browser, lambda b: lined(b) == ["2026-10-18_11-00"], 3)
        check("a Refaire… while one is made waits its turn, never refused", queued, lined(browser))
        (WORK / "slow-hearing").unlink()
        said_ = lambda: (asked / "transcript.jsonl").read_text() if (asked / "transcript.jsonl").exists() else ""
        turned = wait_for(browser, lambda b: "Refait avec ggml-large" in said_() and "Refait avec ggml-large" in b.find_element(By.ID, "meeting").text and not b.find_element(By.ID, "transcript-state").is_displayed(), 15)
        check("then made in its turn, and the card shows it by itself", turned, (said_(), heard_state(browser)))
    tags.shutdown()

    print("P. freeing space: the audio alone or the whole meeting, the heaviest first, to the trash")
    roomy = WORK / "roomy"
    now = dt.datetime.now().astimezone()
    session(roomy / "light", now - dt.timedelta(days=40), 10, {"local.wav": 1_000_000, "transcript.jsonl": 100})
    session(roomy / "heavy", now - dt.timedelta(days=2), 60, {"local.wav": 5_000_000, "derived/local.wav": 1_000_000, "transcript.jsonl": 100})
    session(roomy / "old", now - dt.timedelta(days=60), 30, {"local.wav": 3_000_000, "minutes.md": 50})
    with Window(roomy, "fr_FR.UTF-8") as browser:
        offered = browser.find_element(By.ID, "free-link").get_attribute("aria-label")
        check("the weights offer to free space", offered == "Faire de la place", offered)
        browser.find_element(By.ID, "free-link").click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "free-do"))
        shot(browser, "3-free")
        weighs = [w.text for w in browser.find_elements(By.CSS_SELECTOR, "#free .weighs")]
        check("the heaviest first, by their audio", weighs == ["6 Mo", "3 Mo", "1 Mo"], weighs)
        check("nothing is freed before a choice", not browser.find_element(By.ID, "free-do").is_enabled())
        browser.find_element(By.ID, "free-older").click()
        check("the page drawn again, the focus stays where the keyboard left it", browser.execute_script("return document.activeElement.id") == "free-older")
        browser.find_element(By.CSS_SELECTOR, "#free .freeable input[value=heavy]").click()
        browser.find_element(By.CSS_SELECTOR, "#free .freeable input[value=heavy]").click()
        check("on a meeting's box too", browser.execute_script("return document.activeElement.value") == "heavy", browser.execute_script("return document.activeElement.outerHTML"))
        ticked = browser.execute_script("return [...document.querySelectorAll('#free .freeable input:checked')].map(i => i.value)")
        check("those older than 30 days are ticked at once", sorted(ticked) == ["light", "old"], ticked)
        said = browser.find_element(By.CSS_SELECTOR, "#free .free-total").text
        check("what will be freed is said before anything is done", "2 réunions" in said and "corbeille" in said, said)
        browser.find_element(By.ID, "free-do").click()
        gone = wait_for(browser, lambda b: not (roomy / "old" / "local.wav").exists() and not (roomy / "light" / "local.wav").exists(), 10)
        trash = WORK / "data" / "Trash" / "files"
        check(
            "their audio alone goes to the trash; transcript and minutes stay",
            gone and (roomy / "light" / "transcript.jsonl").exists() and (roomy / "old" / "minutes.md").exists()
            and (roomy / "heavy" / "local.wav").exists() and len(list(trash.glob("local.wav*"))) >= 2,
            sorted(p.name for p in trash.glob("*")),
        )
        # The whole meeting: its folder, to the trash.
        # The page is drawn again after each gesture: each is found afresh.
        time.sleep(1)
        browser.execute_script("document.querySelector('#free input[name=free-what][value=true]').click()")
        wait_for(browser, lambda b: b.find_elements(By.CSS_SELECTOR, "#free .freeable input[value=heavy]"))
        browser.execute_script("document.querySelector('#free .freeable input[value=heavy]').click()")
        button = browser.find_element(By.ID, "free-do")
        check("the whole meeting is said as such", button.text == "Mettre ces réunions à la corbeille", button.text)
        button.click()
        whole = wait_for(browser, lambda b: not (roomy / "heavy").exists(), 10)
        check("the whole meeting's folder goes to the trash", whole and (trash / "heavy").is_dir(), sorted(p.name for p in trash.glob("*")))
        browser.execute_script("location.hash = '#/'")
        listed = wait_for(browser, lambda b: len(b.find_elements(By.CSS_SELECTOR, "#groups .session")) == 2, 5)
        check("a meeting whose audio was freed is still listed; the whole one is not", listed)

    print("Q. the recording bar (D35, D90): opened first, at rest, 440 × 44; recording in it; back to rest at the stop; one window at a time")
    taping = WORK / "taping"
    taping.mkdir()
    with (window := Window(taping, "fr_FR.UTF-8", 'after = "nothing"\nreminder = false\n', small=True)) as browser:
        small = browser.current_window_handle
        start = browser.find_element(By.ID, "small-record")
        check("opened from the menu the first time, the bar, at rest, alone", start.get_attribute("aria-label") == "Démarrer un enregistrement" and browser.window_handles == [small], (start.get_attribute("aria-label"), browser.window_handles))
        check("the recording's gesture has the focus", browser.execute_script("return document.activeElement.id") == "small-record")
        # The bar as the decider's boards draw it (D90): one row, its buttons icons alone, 28 px, each
        # named by its words — its tooltip too, the shortcut beside ● when the desktop granted one.
        row = ("const shown = (q) => [...document.querySelectorAll(q)].filter(b => b.getClientRects().length);"
               "return {size: [innerWidth, innerHeight], row: document.querySelector('#small .bar-row').getBoundingClientRect().height,"
               " mark: shown('#small .handle svg.mark').length, buttons: shown('#small .bar-row button').map(b => [b.id, b.getBoundingClientRect().height, b.getBoundingClientRect().width, b.textContent.trim(), b.getAttribute('aria-label'), b.title]),"
               " gone: shown('#small .last, #small-keys, #small .none').length, scroll: [document.scrollingElement.scrollHeight, document.scrollingElement.clientHeight]}")
        rest = wait_for(browser, lambda b: b.execute_script(row)["size"] == [440, 44], 5)
        got = browser.execute_script(row)
        check("at rest the bar is 440 × 44: one row, the mark as its handle, nothing to scroll", rest and got["row"] == 44 and got["mark"] == 1 and got["scroll"][0] <= got["scroll"][1], got)
        check("●, « Plus d’options », the whole window and « Masquer la barre », each 28 px, an icon alone named by its words, its tooltip the same",
              [b[0] for b in got["buttons"]] == ["small-record", "small-more", "small-full", "small-hide"] and all(b[1] == b[2] == 28 and b[3] == "" and b[4] and b[5].startswith(b[4]) for b in got["buttons"])
              and got["buttons"][2][4] == "Toute la fenêtre", got["buttons"])
        keys = browser.execute_script("return window.__TAURI__.core.invoke('header')")["shortcut"]
        check("●'s tooltip: « Démarrer un enregistrement », the shortcut beside when granted", got["buttons"][0][5] == (f"Démarrer un enregistrement ({keys})" if keys else "Démarrer un enregistrement"), [got["buttons"][0][5], keys])
        check("the last meeting and the shortcut's line are no longer in it (D90)", got["gone"] == 0, got)
        reached = []
        for _ in range(5):
            browser.switch_to.active_element.send_keys(Keys.TAB)
            reached.append(browser.execute_script("return document.activeElement.id"))
        check("Tab reaches each of its controls — the context's field too (D91) —, then comes back round",
              reached == ["small-more", "small-full", "small-hide", "small-next-context", "small-record"], reached)
        shot(browser, "6-small-rest")
        start.click()
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop"))
        time.sleep(1.5)
        check("Démarrer un enregistrement records in the same window", browser.window_handles == [small], browser.window_handles)
        live = ("const s = document.getElementById('small-stop'); const d = document.querySelector('#small .handle .dot'); const t = document.querySelector('#small .bar-row .time');"
                "return {stop: [s.getBoundingClientRect().height, s.getBoundingClientRect().width, s.textContent.trim(), s.getAttribute('aria-label'), s.closest('.bar-row') !== null],"
                " dot: [getComputedStyle(d).fill === getComputedStyle(s).backgroundColor, d.getAttribute('r')], time: [t.getClientRects().length, getComputedStyle(t).fontVariantNumeric], record: document.getElementById('small-record').getClientRects().length,"
                " scroll: [document.scrollingElement.scrollHeight, document.scrollingElement.clientHeight], width: innerWidth}")
        got = browser.execute_script(live)
        check("recording: ■ in the row, 28 px, an icon named « Arrêter l’enregistrement »; ● gone", got["stop"] == [28, 28, "", "Arrêter l’enregistrement", True] and got["record"] == 0, got)
        check("the handle's dot red and larger; the time in the row, in tabular figures", got["dot"] == [True, "2.2"] and got["time"] == [1, "tabular-nums"], got)
        check("as at rest, 440 × 44: the row alone, its meters in it (D90)", wait_for(browser, lambda b: b.execute_script("return [innerWidth, innerHeight]") == [440, 44], 3), browser.execute_script("return [innerWidth, innerHeight]"))
        shot(browser, "6-small")
        fits = browser.execute_script("const e = document.scrollingElement; return [e.scrollHeight, e.clientHeight];")
        check("it holds in its window, nothing to scroll", fits[0] <= fits[1], fits)
        moving = {browser.find_element(By.ID, "level-me").get_attribute("style") for _ in range(6) if time.sleep(0.15) is None}
        clock = browser.find_element(By.CSS_SELECTOR, "#small .time").text
        check("it shows the time recorded and the levels, moving as sound comes", clock.startswith("00:00:0") and clock != "00:00:00" and len(moving) > 1, (clock, moving))
        check("the meeting's level too", "50%" in browser.find_element(By.ID, "level-others").get_attribute("style"))
        # Each side's colour as board 5 draws it: here in the accent, the meeting in green.
        colours = browser.execute_script(
            "const as = (v) => { const t = document.createElement('i'); t.style.color = `var(${v})`; document.body.append(t); const c = getComputedStyle(t).color; t.remove(); return c; };"
            " return [getComputedStyle(document.getElementById('level-me')).backgroundColor === as('--accent'), getComputedStyle(document.getElementById('level-others')).backgroundColor === as('--ok')]")
        check("your level in the accent, the meeting's in green (board 5)", colours == [True, True], colours)
        # What each side is recorded from, named under its level, and followed (D33).
        # Read from the page: WebKitWebDriver leaves this line out of an element's text, though drawn (shot 6-small-devices).
        named = lambda b: tuple(b.execute_script("return ['device-me', 'device-others'].map(id => document.getElementById(id).textContent)"))
        check("it names the microphone and the output recorded", named(browser) == ("Mic", "Speaker"), named(browser))
        (taping / "move").touch()
        moved = wait_for(browser, lambda b: named(b) == ("AirPods Pro", "Speaker"), 1.5)
        (taping / "lose").touch()
        lost = wait_for(browser, lambda b: named(b) == ("AirPods Pro", ""), 1.5)
        check("a device changed is named within a second; a side with none names none", moved and lost, named(browser))
        shot(browser, "6-small-devices")
        check("nothing follows the stop here: the small window does not say the transcript writes itself", not browser.find_element(By.ID, "small-note").is_displayed())
        browser.find_element(By.ID, "small-stop").click()
        rest = wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 5)
        stopped = '"stopped"' in (taping / "2026-10-18_12-00" / "session.json").read_text()
        check("■ stops the recording, and the bar comes back to rest — it does not close", rest and stopped and browser.window_handles == [small], browser.window_handles)
        back = wait_for(browser, lambda b: b.execute_script(row)["size"] == [440, 44], 5)
        check("at rest again, 440 × 44, the handle's dot back", back and browser.execute_script("return document.querySelector('#small .handle .dot').getAttribute('r')") == "1.6", browser.execute_script(row))
        shot(browser, "6-small-after")
        try:
            browser.find_element(By.ID, "small-full").click()
        except NoSuchWindowException:
            pass  # the small window goes with the click, sometimes before the driver has answered it
        full = wait_for(browser, lambda b: len(b.window_handles) == 1 and b.window_handles != [small], 5)
        check("Toute la fenêtre replaces the small one: one window", full, browser.window_handles)
        browser.switch_to.window(browser.window_handles[0])
        wait_for(browser, lambda b: b.execute_script("return (document.getElementById('settings-link').getAttribute('aria-label') || '') !== ''"))
        check("the full window left is the one kept in the settings", 'window = "full"' in window.file.read_text(), window.file.read_text())
        back = browser.find_element(By.ID, "small-link")
        check("at rest too, the full window offers the small one", back.is_displayed() and back.get_attribute("aria-label") == "La barre d’enregistrement", back.get_attribute("aria-label"))
        main = browser.current_window_handle
        back.click()
        smaller = wait_for(browser, lambda b: len(b.window_handles) == 2, 5)
        browser.switch_to.window([h for h in browser.window_handles if h != main][0])
        shown = wait_for(browser, lambda b: b.find_elements(By.ID, "small-record"), 5)
        browser.switch_to.window(main)
        hidden = browser.execute_script("return document.visibilityState")
        check("Petite fenêtre replaces the full one, hidden behind it", smaller and shown and hidden == "hidden", (browser.window_handles, hidden))
        check("the small window left is the one kept in the settings", 'window = "small"' in window.file.read_text(), window.file.read_text())
    taping_en = WORK / "taping-en"
    taping_en.mkdir()
    with Window(taping_en, "en_GB.UTF-8", "reminder = false\n") as browser:
        main = browser.current_window_handle
        browser.find_element(By.ID, "record").click()
        wait_for(browser, lambda b: b.find_element(By.ID, "record").get_attribute("class").find("live") >= 0, 5)
        check("a recording started from the full window opens no other window", browser.window_handles == [main], browser.window_handles)
        browser.find_element(By.ID, "small-link").click()
        wait_for(browser, lambda b: len(b.window_handles) == 2, 5)
        browser.switch_to.window([h for h in browser.window_handles if h != main][0])
        wait_for(browser, lambda b: b.find_elements(By.ID, "small-stop"))
        time.sleep(1)
        fits = browser.execute_script("const e = document.scrollingElement; return [e.scrollHeight, e.clientHeight];")
        said = browser.execute_script("return document.querySelector('#small .meter .track').getAttribute('aria-label')")
        check("in English too, it holds in its window", said == "Your microphone" and fits[0] <= fits[1], (said, fits))
        starts = browser.execute_script("return [...document.querySelectorAll('#small .track')].map(t => t.getBoundingClientRect().left)")
        check("the two meters start at the same place", len(starts) == 2 and starts[0] == starts[1], starts)
        press(browser, "small-more")
        note = browser.find_element(By.ID, "small-note")
        check("the transcript follows the stop: the bar's options say so", wait_for(browser, lambda b: note.is_displayed(), 3) and "transcript" in note.text, note.text)
        browser.find_element(By.ID, "small-stop").click()
        rest = wait_for(browser, lambda b: b.find_elements(By.ID, "small-record") and b.find_element(By.ID, "small-record").is_displayed(), 5)
        check("opened from the full window during a recording, it too comes back to rest, in English", rest and browser.find_element(By.ID, "small-record").get_attribute("aria-label") == "Start a recording")
        back = wait_for(browser, lambda b: b.execute_script("return [innerWidth, innerHeight]") == [440, 44], 5)
        check("at rest, in English, 440 × 44 again", back, browser.execute_script("return [innerWidth, innerHeight]"))

    print("I. the window, in English; and a folder with no session")
    empty = WORK / "empty"
    empty.mkdir()
    with Window(empty, "en_GB.UTF-8") as browser:
        said = browser.find_element(By.ID, "none").text
        check("an empty folder is said, with its path", said == f"No meeting in {empty} yet.", said)
        check("the sidebar is named in English", browser.find_element(By.ID, "sidebar").get_attribute("aria-label") == "Meetings")
        lang = browser.execute_script("return document.documentElement.lang")
        check("the page says it is in English", lang == "en", lang)
        sections = browser.find_element(By.TAG_NAME, "nav").get_attribute("aria-label")
        check("its sections are named in English for a screen reader", sections == "Sections", sections)
        check("no weight is shown for nothing", not browser.find_element(By.ID, "weight").is_displayed())
        # One meeting is one, not "1 meetings".
        session(empty / "one", now - dt.timedelta(hours=2), 10, {"local.wav": 3000})
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        wait_for(browser, lambda b: b.find_element(By.ID, "weight").is_displayed())
        one = browser.find_element(By.ID, "taken").text
        check("one meeting is said in the singular", one == "1 meeting", one)
        # A failure of the page's own, thrown and rejected: logged by the shell (declared in the
        # manifest), its kind and where — never its sentence.
        browser.execute_script(
            "class ScenarioI extends Error { name = 'ScenarioI' }"
            " setTimeout(() => { throw new ScenarioI('Réunion avec Paul-Scenario') });"
            " setTimeout(() => Promise.reject(new ScenarioI('Réunion avec Paul-Scenario')))")
        logs = WORK / "state" / "kitokey" / "logs"
        logged = lambda: "".join(p.read_text() for p in logs.glob("shell.*.jsonl")).count('"shell.page_failed"')
        wait_for(browser, lambda b: logged() >= 2, 10)
        text = "".join(p.read_text() for p in logs.glob("shell.*.jsonl"))
        check("the page's failures reach the log, never their sentence", logged() >= 2 and "Paul-Scenario" not in text)
        # What the notification plugin's own script asks at each page's start is answered: refused,
        # it failed every page, a rejection nobody catches.
        asked = browser.execute_async_script(
            "const done = arguments[arguments.length - 1];"
            " window.__TAURI_INTERNALS__.invoke('plugin:notification|is_permission_granted')"
            ".then(() => done('answered'), (why) => done('refused: ' + why))")
        check("the notification plugin's question at a page's start is answered", asked == "answered", asked)

    print("R. a meeting listened to, its words following (D26) — played to the run's own output, nothing reaches the speakers")
    kept = remembered()
    listening = WORK / "listening"
    played = listening / "2026-10-19_10-00"
    session(played, now - dt.timedelta(hours=1), 0.1, {})
    record = json.loads((played / "session.json").read_text())
    record["tracks"] = {"local": {"device": "Mic"}, "remote": {"device": "Speaker"}}
    (played / "session.json").write_text(json.dumps(record))
    for name, hertz in (("local", 440), ("remote", 880)):
        with wave.open(str(played / f"{name}.wav"), "wb") as made:
            made.setnchannels(2)
            made.setsampwidth(3)
            made.setframerate(48000)
            frames = bytearray()
            for n in range(48000 * 6):
                sample = int(0.2 * 8388607 * math.sin(2 * math.pi * hertz * n / 48000))
                frames += struct.pack("<i", sample)[:3] * 2
            made.writeframes(bytes(frames))
    # The remote track's frame 0 is a second and a half into the session: it ends after the local one.
    (played / "clock.jsonl").write_text('{"track":"local","sample":0,"at":0.0}\n{"track":"remote","sample":0,"at":1.5}\n')
    heard = [(0.4, "local", "Bonjour à tous."), (2.5, "remote", "Bonjour, je vous entends."), (4.2, "local", "Commençons.")]
    (played / "transcript.jsonl").write_text(
        '{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"m","language":"fr","context":null}\n'
        + "".join(json.dumps({"type": "segment", "speaker": "me" if track == "local" else "others", "track": track, "voice": None,
                              "start": start, "end": start + 1.5, "text": text}, ensure_ascii=False) + "\n" for start, track, text in heard)
        + '{"type":"complete","duration":6.0}\n'
    )
    freed = listening / "2026-10-19_09-00"
    session(freed, now - dt.timedelta(hours=2), 0.1, {})
    shutil.copy(played / "transcript.jsonl", freed / "transcript.jsonl")
    # Audio there, and no track this version plays: not said freed.
    unplayable = listening / "2026-10-19_08-00"
    session(unplayable, now - dt.timedelta(hours=3), 0.1, {"local.wav": 4000})
    shutil.copy(played / "transcript.jsonl", unplayable / "transcript.jsonl")
    # A meeting of two minutes, one track of silence: more than a player reads ahead of itself.
    long = listening / "2026-10-19_06-00"
    session(long, now - dt.timedelta(hours=5), 2, {})
    record = json.loads((long / "session.json").read_text())
    record["tracks"] = {"local": {"device": "Mic"}}
    (long / "session.json").write_text(json.dumps(record))
    with wave.open(str(long / "local.wav"), "wb") as made:
        made.setnchannels(1)
        made.setsampwidth(2)
        made.setframerate(48000)
        made.writeframes(bytes(2 * 48000 * 120))
    (long / "clock.jsonl").write_text('{"track":"local","sample":0,"at":0.0}\n')
    shutil.copy(played / "transcript.jsonl", long / "transcript.jsonl")
    # Tracks a WAV header names, in a form no player reads: listed, and refused by the window.
    unreadable = listening / "2026-10-19_07-00"
    session(unreadable, now - dt.timedelta(hours=4), 0.1, {})
    record = json.loads((unreadable / "session.json").read_text())
    record["tracks"] = {"local": {"device": "Mic"}}
    (unreadable / "session.json").write_text(json.dumps(record))
    noise = bytes(range(256)) * 16
    fmt = struct.pack("<HHIIHH", 0x9999, 2, 48000, 48000 * 6, 6, 24)
    (unreadable / "local.wav").write_bytes(b"RIFF" + struct.pack("<I", 4 + 24 + 8 + len(noise)) + b"WAVE"
                                           + b"fmt " + struct.pack("<I", len(fmt)) + fmt + b"data" + struct.pack("<I", len(noise)) + noise)
    (unreadable / "clock.jsonl").write_text('{"track":"local","sample":0,"at":0.0}\n')
    shutil.copy(played / "transcript.jsonl", unreadable / "transcript.jsonl")
    with (window := Window(listening, "fr_FR.UTF-8")) as browser:
        browser.execute_script("location.hash = '#/meeting/2026-10-19_10-00'")
        ready = wait_for(browser, lambda b: b.find_elements(By.ID, "play") and len(b.execute_script("return document.querySelectorAll('audio.track')")) == 1, 10)
        check("a meeting with its audio has a player: its tracks as one sound (ADR-0009)", ready)
        # The session's time is the one sound's, while it plays.
        at = lambda b: b.execute_script("const a = document.querySelector('audio.track'); return a && !a.paused ? a.currentTime : 0")
        playing = lambda b: b.execute_script("const a = document.querySelector('audio.track'); return !!a && !a.paused")
        # As long as the last track ends: the remote one, 1.5 s in, for 6 s.
        lasts = wait_for(browser, lambda b: b.execute_script("return document.querySelector('audio.track')?.duration || 0") > 0, 10)
        length = browser.execute_script("return document.querySelector('audio.track').duration || 0")
        check("the sound lasts to the end of the last track", lasts and abs(length - 7.5) < 0.05, length)
        # The lines below the window's edge, the page at its top: playing moves nothing (the decider, 2026-10-24).
        browser.execute_script("document.querySelector('#meeting .columns').style.marginTop = '2000px'; window.scrollTo(0, 0)")
        press(browser, "play")
        moving = wait_for(browser, lambda b: at(b) > 1.9, 10)
        check("Écouter plays the meeting", moving and playing(browser))
        marked = wait_for(browser, lambda b: b.execute_script("return document.querySelector('.lines li.now')?.dataset.start") == "0.4", 5)
        check("the line being said is marked", marked, browser.execute_script("return document.querySelector('.lines li.now')?.textContent"))
        still = browser.execute_script("return [window.scrollY, document.querySelector('.lines li.now').getBoundingClientRect().top > window.innerHeight]")
        check("the page stays where the user left it: the line marked is not brought into view", still == [0, True], still)
        browser.execute_script("document.querySelector('#meeting .columns').style.marginTop = ''")
        shot(browser, "10-playing")
        lasting = browser.find_element(By.CSS_SELECTOR, "#meeting .length").text
        check("a meeting under a minute says its seconds (#312)", lasting == "6 s", lasting)
        browser.find_element(By.ID, "search").send_keys(" ")
        check("a space typed in a field is the field's", not browser.execute_script("return document.querySelector('audio.track').paused"))
        browser.find_elements(By.CSS_SELECTOR, ".lines li")[2].click()
        jumped = wait_for(browser, lambda b: abs(at(b) - 4.2) < 0.6
                          and b.execute_script("return document.querySelector('.lines li.now')?.dataset.start") == "4.2", 5)
        check("a click on a line plays from it, and marks it", jumped, at(browser))
        width = browser.execute_script("return document.querySelector('.drawing').getBoundingClientRect().width")
        ActionChains(browser).move_to_element_with_offset(browser.find_element(By.CSS_SELECTOR, ".drawing"), int(-width / 2 + width * 0.4), 0).click().perform()
        sought = wait_for(browser, lambda b: abs(at(b) - 2.4) < 0.6, 5)
        check("a click on who spoke moves the playback there", sought, at(browser))
        # Space for a meeting playing, its seek over — not one still under way (#710's verifier):
        # the seek settled and the sound flowing again, then the pause it gives waited for.
        settled = wait_for(browser, lambda b: b.execute_script(
            "const a = document.querySelector('audio.track'); return !a.seeking && !a.paused && a.readyState >= 3"), 5)
        browser.execute_script("window.__paused = false; document.querySelector('audio.track')"
                               ".addEventListener('pause', () => (window.__paused = true), {once: true})")
        browser.find_element(By.TAG_NAME, "body").send_keys(Keys.SPACE)
        paused = wait_for(browser, lambda b: b.execute_script(
            "return window.__paused && [...document.querySelectorAll('audio.track')].every(a => a.paused)"
            " && document.getElementById('play').textContent === 'Écouter'"), 5)
        check("the space bar pauses", settled and paused, (settled, browser.find_element(By.ID, "play").text))
        # Pause clicked with the mouse, held as a hand holds it: playing, the button's words are
        # read ten times a second, and a press on words written again is no click in WebKitGTK.
        browser.find_element(By.TAG_NAME, "body").send_keys(Keys.SPACE)
        wait_for(browser, playing, 5)
        ActionChains(browser).move_to_element(browser.find_element(By.ID, "play")).click_and_hold().pause(0.3).release().perform()
        paused = wait_for(browser, lambda b: not playing(b), 3)
        check("Pause clicked with the mouse, held 0.3 s, pauses", paused and browser.find_element(By.ID, "play").text == "Écouter",
              browser.find_element(By.ID, "play").text)
        # By the keyboard alone: the drawing is a slider, a line plays from it on Enter.
        drawing = browser.find_element(By.CSS_SELECTOR, ".drawing")
        slider = lambda b: b.find_element(By.CSS_SELECTOR, ".drawing").get_attribute("aria-valuenow")
        drawing.send_keys(Keys.HOME)
        drawing.send_keys(Keys.ARROW_RIGHT)
        moved = wait_for(browser, lambda b: slider(b) == "5", 5)
        check("the arrows move the playback on the drawing, a slider", moved and drawing.get_attribute("role") == "slider"
              and drawing.get_attribute("tabindex") == "0", slider(browser))
        drawing.send_keys(Keys.ARROW_LEFT)
        back = wait_for(browser, lambda b: slider(b) == "0", 5)
        # To the end of the sound — 7.5 s, past the session's 6 — a tenth before it.
        maximum = lambda b: float(b.find_element(By.CSS_SELECTOR, ".drawing").get_attribute("aria-valuemax"))
        drawing.send_keys(Keys.END)
        ended_at = wait_for(browser, lambda b: abs(float(slider(b)) - 7.4) < 0.05, 5)
        check("back, and to the end of the sound", back and ended_at and maximum(browser) == 7.5, (slider(browser), maximum(browser)))
        drawing.send_keys(Keys.PAGE_DOWN)
        down = wait_for(browser, lambda b: slider(b) == "0", 5)
        drawing.send_keys(Keys.ARROW_UP)
        up = wait_for(browser, lambda b: slider(b) == "5", 5)
        drawing.send_keys(Keys.ARROW_DOWN)
        check("up and down, page up and page down, as a slider's keys", down and up and wait_for(browser, lambda b: slider(b) == "0", 5), slider(browser))
        stops = lambda b: b.execute_script("return [...document.querySelectorAll('.lines li')].map(li => li.tabIndex)")
        # The line clicked last is the one in Tab's order.
        check("one line in the order of Tab, the one clicked last", stops(browser) == [-1, -1, 0], stops(browser))
        first = browser.find_elements(By.CSS_SELECTOR, ".lines li")[0]
        first.send_keys(Keys.ARROW_DOWN)
        line = browser.find_elements(By.CSS_SELECTOR, ".lines li")[1]
        focused = browser.execute_script("return document.activeElement === arguments[0]", line)
        check("the arrow moves the line in the order of Tab", stops(browser) == [-1, 0, -1], stops(browser))
        line.send_keys(Keys.ENTER)
        entered = wait_for(browser, lambda b: abs(at(b) - 2.5) < 0.6, 5)
        check("down to a line, Enter plays from it", focused and entered and line.get_attribute("role") == "button", at(browser))
        # A search that hides that line: the first line shown takes its place in Tab's order.
        browser.find_element(By.ID, "search").send_keys("Commençons")
        sought = wait_for(browser, lambda b: stops(b) == [-1, -1, 0], 5)
        check("a search hiding the line in Tab's order gives the first shown its place", sought, stops(browser))
        browser.find_element(By.ID, "search").send_keys(Keys.CONTROL, "a")
        browser.find_element(By.ID, "search").send_keys(Keys.DELETE)
        # Space held down: the first press alone counts.
        browser.execute_script("document.body.dispatchEvent(new KeyboardEvent('keydown', {key: ' ', repeat: true, bubbles: true}))")
        check("Space held does not toggle again", not browser.execute_script("return document.querySelector('audio.track').paused"))
        # The page drawn again while it plays: the same player goes on.
        drawn_before = browser.find_element(By.CSS_SELECTOR, ".drawing")
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        try:
            WebDriverWait(browser, 5).until(staleness_of(drawn_before))
        except TimeoutException:
            check("the page is drawn again", False)
        count = browser.execute_script("return document.querySelectorAll('audio.track').length")
        check("drawn again, it plays on", count == 1 and playing(browser) and at(browser) > 2.5, (count, at(browser)))
        # From the start, through the end of the shorter track, to the end of the last.
        press(browser, "play")
        wait_for(browser, lambda b: browser.find_element(By.ID, "play").text == "Écouter", 5)
        browser.find_element(By.CSS_SELECTOR, ".drawing").send_keys(Keys.HOME)
        press(browser, "play")
        past = wait_for(browser, lambda b: at(b) > 6.3, 12)
        check("from the start, past the end of the shorter track, it plays on", past and playing(browser), at(browser))
        ended = wait_for(browser, lambda b: browser.find_element(By.ID, "play").text == "Écouter", 5)
        check("at the end of the last, the meeting stops", ended and not playing(browser))
        press(browser, "play")
        anew = wait_for(browser, lambda b: 0 < at(b) < 1.4, 5)
        check("played again, it starts over", anew, at(browser))
        # Paused at once, the sound not yet loaded: no failure is said.
        browser.execute_script("location.hash = '#/settings'")
        wait_for(browser, lambda b: b.find_element(By.ID, "settings").is_displayed(), 10)
        browser.execute_script("location.hash = '#/meeting/2026-10-19_10-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "play") and b.find_element(By.ID, "meeting").is_displayed()
                 and b.execute_script("return document.getElementById('meeting').dataset.name") == "2026-10-19_10-00", 10)
        browser.execute_script("document.getElementById('play').click(); document.getElementById('play').click()")
        time.sleep(1)
        check("a play paused at once is no failure", not browser.find_element(By.ID, "not-played").text and not playing(browser))
        # Its audio freed while it plays: it stops, and says so.
        for name in ("local", "remote"):
            (played / f"{name}.wav").rename(listening / f"aside-{name}.wav")
        browser.execute_script("window.dispatchEvent(new Event('focus'))")
        freed = wait_for(browser, lambda b: not b.execute_script("return document.querySelectorAll('audio.track').length")
                         and b.find_elements(By.ID, "no-player"), 10)
        check("its audio freed while it plays, it stops", freed)
        for name in ("local", "remote"):
            (listening / f"aside-{name}.wav").rename(played / f"{name}.wav")
        browser.execute_script("location.hash = '#/'")
        wait_for(browser, lambda b: b.find_element(By.ID, "list").is_displayed(), 10)
        browser.execute_script("location.hash = '#/meeting/2026-10-19_10-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "play") and b.find_element(By.ID, "meeting").is_displayed()
                 and b.execute_script("return document.getElementById('meeting').dataset.name") == "2026-10-19_10-00", 10)
        press(browser, "play")
        wait_for(browser, lambda b: at(b) > 0.3, 5)
        heard = played_to(window)
        check("its sound goes to the run's own output, never the speakers", bool(QUIET) and heard == {QUIET}, heard)
        # The list beside it, the meeting still shown, plays on (D61); the list alone, under 840 px, stops it.
        browser.set_window_size(420, 400)
        wait_for(browser, lambda b: b.execute_script("return matchMedia('(max-width: 839px)').matches"), 5)
        browser.execute_script("location.hash = '#/'")
        stopped = wait_for(browser, lambda b: not b.execute_script("return document.querySelectorAll('audio.track').length"), 10)
        check("the list shown alone, nothing plays", stopped)
        browser.set_window_size(1100, 800)
        # Its sound refused by the server — its track gone since the page was drawn, then a part of
        # it asked for that the player had not read: said, never silence.
        browser.execute_script("location.hash = '#/meeting/2026-10-19_06-00'")
        wait_for(browser, lambda b: b.find_elements(By.ID, "play") and b.execute_script("return document.querySelector('audio.track')?.duration || 0") > 0, 10)
        (long / "local.wav").rename(listening / "aside-long.wav")
        browser.find_element(By.CSS_SELECTOR, ".drawing").send_keys(Keys.END)
        press(browser, "play")
        said = wait_for(browser, lambda b: b.find_elements(By.ID, "not-played") and b.find_element(By.ID, "not-played").text, 10)
        check("a sound that cannot be played is said", said and "Lecture impossible ici" in browser.find_element(By.ID, "not-played").text)
        (listening / "aside-long.wav").rename(long / "local.wav")
        # A track in a form no player reads: no sound, and it is said not playable here.
        browser.execute_script("location.hash = '#/meeting/2026-10-19_07-00'")
        unread = wait_for(browser, lambda b: b.find_elements(By.ID, "no-player") and "ne peut pas lire cet audio" in b.find_element(By.ID, "no-player").text, 10)
        check("a track no player reads is said not playable here", unread)
        # The page before stays drawn until the next one is: its sentence is waited for, read in
        # the page at once — an element found then read may be the page before's, drawn again.
        no_player = "return document.getElementById('no-player')?.innerText || ''"
        browser.execute_script("location.hash = '#/meeting/2026-10-19_09-00'")
        none = wait_for(browser, lambda b: "supprimé pour faire de la place" in b.execute_script(no_player), 10)
        check("a meeting whose audio was freed has no player, and says why",
              none and not browser.find_elements(By.ID, "play")
              and not browser.execute_script("return document.querySelectorAll('audio.track').length"),
              browser.execute_script(no_player))
        browser.execute_script("location.hash = '#/meeting/2026-10-19_08-00'")
        there = wait_for(browser, lambda b: "ne peut pas lire cet audio" in b.execute_script(no_player), 10)
        check("a meeting whose audio is there but not playable is not said freed",
              there and "supprimé pour faire de la place" not in browser.execute_script(no_player), browser.execute_script(no_player))
    # None remembered before the run, as on a desktop where Kitokey never played: none of the user's to keep.
    check("the volume the system remembers for the user's Kitokey is left as it was", kept is None or same_volume(remembered(), kept), [remembered(), kept])
    model_offered(now)
    models_listed(now)
    list_refreshed(now)
    speech_offered(now)
    service_named(now)
    ollama_pulled(now)
    first_run(now)
    context_written(now)
    confidential_kept(now)
    meeting_language(now)
    transcript_corrected(now)
    corrections_gated(now)
    press_on_the_page_left(now)
    context_gaps(now)
    two_panes(now)
    sidebar_folded(now)
    app_language(now)
    confidential_every_way(now)
    meeting_told_once(now)
    minutes_copied(now)
    nothing_silent(now)
    gestures_answer(now)
    window_looks(now)
    pages_seen(now)
    actions_rows(now)
    announced(now)
    heard_checked(now)
    microphone_chosen(now)
    sound_checked(now)
    bar_window(now)
    bar_opened(now)
    context_before_start(now)
    own_models(now)
    models_advanced(now)
    advanced_latest_answer(now)
    meeting_named(now)
    word_after_a_gap(now)
    minutes_checked(now)
    meeting_renamed(now)
    trial_over(now)
    trial_left(now)
    return 1 if FAILED else 0


if __name__ == "__main__":
    # A scenario that breaks off says where, on the output the evidence keeps: its error stream
    # is not kept, and a run that failed once must say why.
    try:
        sys.exit(main())
    except Exception:
        traceback.print_exc(file=sys.stdout)
        sys.exit(1)
