"""The window's scenarios cut in shards, and the shards' text put back in one (#862).

    e2e_shards.py lists N e2e_window.py e2e_times.txt   N lines, each a shard's names (KITOKEY_E2E_ONLY)
    e2e_shards.py merge e2e_window.py SHARD...           one text, the scenarios in main()'s order

The lists are balanced by each scenario's measured time, the longest first into the lightest shard,
as Bazel's shard_count and pytest-xdist's load groups do; a scenario not measured counts as the median.
"""
import pathlib
import re
import statistics
import sys

# Beside it, as e2e_window.py finds it, run with -I as the scripts are.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import e2e_only  # noqa: E402

TIMED = re.compile(r"^    \(([a-z_]+): [0-9.]+ s\)$")


def lists(shards: int, source: str, times: str) -> list[list[str]]:
    """The scenarios main() calls, in `shards` lists of about the same time, each in main()'s order."""
    called = e2e_only.scenarios(source)
    measured = {}
    for line in times.splitlines():
        if line.strip() and not line.startswith("#"):
            name, seconds = line.split()
            measured[name] = float(seconds)
    median = statistics.median(measured.values()) if measured else 1.0
    load = [0.0] * shards
    given: list[list[str]] = [[] for _ in range(shards)]
    for name in sorted(called, key=lambda name: (-measured.get(name, median), called.index(name))):
        lightest = load.index(min(load))
        given[lightest].append(name)
        load[lightest] += measured.get(name, median)
    return [[name for name in called if name in shard] for shard in given if shard]


def blocks(text: str) -> tuple[list[str], dict[str, list[str]]]:
    """A shard's text: its lines outside any scenario, and each scenario's own, by its name — every
    line after the window's « partial run: … » or the scenario before it, up to the line that says
    its time (scripts/e2e_window.py's `timed`)."""
    lines = text.splitlines()
    starts = [number for number, line in enumerate(lines) if line.startswith("partial run: ")]
    begin = starts[-1] + 1 if starts else 0
    outside = lines[:begin]
    found: dict[str, list[str]] = {}
    pending: list[str] = []
    for line in lines[begin:]:
        pending.append(line)
        timed = TIMED.match(line)
        if timed:
            found[timed.group(1)] = pending
            pending = []
    # After the last scenario: the logs read, the shard's verdict; or a scenario cut short, unnamed.
    outside.extend(pending)
    return outside, found


def merge(source: str, shards: list[str]) -> str:
    """The tray's shard as it ran, then main()'s own scenarios and every one it calls, in its order,
    whichever shard played them, then what each window shard said outside its scenarios."""
    tray, *windows = shards
    lines = tray.splitlines()
    rest: list[tuple[int, list[str]]] = []
    every: dict[str, list[str]] = {}
    for number, text in enumerate(windows, start=1):
        outside, found = blocks(text)
        every.update(found)
        rest.append((number, outside))
    # main()'s own scenarios first, as main() plays them, then each it calls.
    for name in ["main", *e2e_only.scenarios(source)]:
        lines.extend(every.get(name, [f"{name}: NOT RUN — no shard said it"]))
    for number, outside in rest:
        lines.append(f"shard {number}:")
        lines.extend(f"  {line}" for line in outside)
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    if sys.argv[1] == "lists":
        source = pathlib.Path(sys.argv[3]).read_text()
        for shard in lists(int(sys.argv[2]), source, pathlib.Path(sys.argv[4]).read_text()):
            print(",".join(shard))
    elif sys.argv[1] == "merge":
        source = pathlib.Path(sys.argv[2]).read_text()
        print(merge(source, [pathlib.Path(path).read_text() for path in sys.argv[3:]]), end="")
    else:
        raise SystemExit(__doc__)
