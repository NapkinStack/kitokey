"""Kitokey's speed, measured on its release build (D105): the start to the window drawn, the memory
at rest, and a meeting of an hour's page opened — each the median of its runs, written with the
commit and the machine.

    speed.py <the release binary> <a work folder> <runs> <the result, JSON>

Run in a session of its own (`KITOKEY_SPEED=<runs> scripts/headless.sh`, through `scripts/speed.sh`),
alone on the machine: what runs beside it is measured with it. Each run is a cold start as Android
counts one — no Kitokey running, the system's caches as they are — started by tauri-driver as the
e2e starts it (`scripts/e2e_window.py`, `Window`). The driver's own time is outside every interval:
- the start: from the process's own start (`/proc/<pid>/stat`) to its page's first contentful paint
  (Paint Timing), the time to initial display, as Android and Apple's 400 ms to first frame count it;
- the memory: the proportional set size of Kitokey and every process under it (its web view's),
  10 s after the window is drawn, nothing pressed;
- the page: in the page itself, from the gesture (the address set) to its transcript's last line
  drawn and one frame after, as RAIL's 100 ms response counts it.
"""

from __future__ import annotations

import json
import os
import statistics
import subprocess
import sys
import time
from pathlib import Path

# A meeting of an hour, made up: no real meeting is ever measured. Lines of 6 s, two tracks.
HOUR, EVERY = 3600, 6
AT_REST = 10


def started_at(stat: str, booted: float, ticks: int) -> float:
    """A process's start, in seconds since the epoch, from its `/proc/<pid>/stat`: its 22nd field,
    in clock ticks since the boot — read after the name's last `)`, which may hold spaces."""
    return booted + int(stat.rsplit(")", 1)[1].split()[19]) / ticks


def booted(proc_stat: str) -> float:
    """The boot, in seconds since the epoch: `/proc/stat`'s `btime`."""
    return float(next(line.split()[1] for line in proc_stat.splitlines() if line.startswith("btime ")))


def pss_kib(rollup: str) -> int:
    """A process's proportional set size, in KiB, from its `/proc/<pid>/smaps_rollup`."""
    return int(next(line.split()[1] for line in rollup.splitlines() if line.startswith("Pss:")))


def under(pid: int) -> list[int]:
    """A process and every process under it, read in /proc: those still there."""
    found, ahead = [], [pid]
    while ahead:
        one = ahead.pop()
        found.append(one)
        for task in Path(f"/proc/{one}/task").glob("*"):
            try:
                ahead += [int(child) for child in (task / "children").read_text().split()]
            except OSError:
                pass
    return found


def summary(runs: dict[str, list[float]]) -> dict[str, float]:
    """Each measure's median over its runs."""
    return {name: statistics.median(values) for name, values in runs.items()}


def hour(meeting: Path) -> int:
    """A meeting of an hour, its transcript complete: how many lines it has."""
    lines = [{"type": "header", "contract": "transcript/v1", "engine": "whisper.cpp", "model": "m", "language": "fr", "context": None}]
    for n, start in enumerate(range(0, HOUR, EVERY)):
        local = n % 2 == 0
        lines.append({"type": "segment", "speaker": "me" if local else "others", "track": "local" if local else "remote",
                      "start": float(start), "end": float(start + EVERY - 1),
                      "text": f"Phrase {n} de la réunion, inventée pour la mesure, de la longueur d'une phrase dite."})
    lines.append({"type": "complete", "duration": float(HOUR)})
    (meeting / "transcript.jsonl").write_text("".join(json.dumps(line, ensure_ascii=False) + "\n" for line in lines))
    return len(lines) - 2


def machine() -> dict[str, str]:
    """What the numbers were measured on."""
    # The CPU and the kernel read from /proc and uname, not Python's `platform`, whose name is a module's
    # here: the boundary check reads `import platform` as one (nstack 0.33.0).
    cpu = next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines()
                if line.startswith("model name")), "unknown")
    memory = next(line.split()[1] for line in Path("/proc/meminfo").read_text().splitlines() if line.startswith("MemTotal:"))
    webkit = subprocess.run(["rpm", "-q", "--qf", "%{VERSION}", "webkit2gtk4.1"], capture_output=True, text=True).stdout.strip()
    return {"cpu": cpu, "cores": str(os.cpu_count()), "memory_kib": memory, "kernel": os.uname().release, "webkitgtk": webkit or "unknown"}


# The page's first contentful paint, as an epoch in ms; none, null.
PAINTED = ("const p = performance.getEntriesByName('first-contentful-paint')[0];"
           " return p ? performance.timeOrigin + p.startTime : null")
# The gesture, then the transcript's last line drawn and one frame after: the time in the page. Lines
# drawn before are marked, so that only this opening's count.
OPENED = ("const [hash, count, done] = arguments, lines = '#meeting .transcript ol.lines > li';"
          " document.querySelectorAll(lines).forEach((li) => { li.dataset.before = 'yes'; });"
          " const began = performance.now(); location.hash = hash;"
          " const drawn = () => document.querySelectorAll(lines + ':not([data-before])').length >= count;"
          " const wait = () => drawn() ? requestAnimationFrame(() => done(performance.now() - began)) : setTimeout(wait, 1);"
          " wait();")


def main() -> int:
    binary, work, runs, result = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3]), Path(sys.argv[4])
    sys.argv = [sys.argv[0], str(binary), str(work)]
    sys.path.insert(0, str(Path(__file__).parent))
    import e2e_window as e2e  # noqa: E402 — it reads the binary and its work folder from sys.argv

    meetings = work / "speed"
    meeting = meetings / "2026-11-20_10-00"
    e2e.session(meeting, e2e.dt.datetime.now().astimezone() - e2e.dt.timedelta(hours=2), 60, {"local.wav": 1000})
    count = hour(meeting)
    ticks, boot = os.sysconf("SC_CLK_TCK"), booted(Path("/proc/stat").read_text())
    measured: dict[str, list[float]] = {"start_ms": [], "memory_kib": [], "page_ms": []}
    for run in range(runs):
        with e2e.Window(meetings, "fr_FR.UTF-8", "reminder = false\n") as browser:
            kitokey = e2e.instances()
            painted = browser.execute_script(PAINTED)
            if len(kitokey) != 1 or painted is None:
                print(f"run {run + 1}: not measured — Kitokey's processes {kitokey}, first contentful paint {painted}", flush=True)
                return 1
            start = started_at(Path(f"/proc/{kitokey[0]}/stat").read_text(), boot, ticks)
            measured["start_ms"].append(painted - start * 1000)
            time.sleep(AT_REST)
            memory = 0
            for pid in under(kitokey[0]):
                try:
                    memory += pss_kib(Path(f"/proc/{pid}/smaps_rollup").read_text())
                except OSError:
                    pass
            measured["memory_kib"].append(memory)
            # From Settings, so that the address changes and the page is drawn for it.
            browser.execute_script("location.hash = '#/settings'")
            e2e.wait_for(browser, lambda b: b.find_element(e2e.By.ID, "reminder").is_displayed(), 10)
            browser.set_script_timeout(30)
            measured["page_ms"].append(browser.execute_async_script(OPENED, f"#/meeting/{meeting.name}", count))
            print(f"run {run + 1}: start {measured['start_ms'][-1]:.0f} ms, memory {memory} KiB,"
                  f" page {measured['page_ms'][-1]:.0f} ms ({count} lines)", flush=True)
    commit = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=Path(__file__).parent).stdout.strip()
    said = {"commit": commit, "measured": time.strftime("%Y-%m-%dT%H:%M:%S%z"), "machine": machine(), "runs": runs,
            "lines": count, "median": summary(measured), "each": measured}
    result.write_text(json.dumps(said, indent=1) + "\n")
    print(json.dumps(said["median"]), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
