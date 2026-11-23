"""How often each check of the shell's e2e fails where nothing changed (D104, criterion 2).

    flake_rate.py <a folder of attests' e2e> [--since <commit> [--repo <a clone>]]

The folder holds one file per attest, `<stamp>-pr<n>-<commit>.txt`, its e2e's text as the attest
printed it. A check's FAIL counts when the same check passed on another run of the same commit:
nothing changed between them but the run. Only a commit run more than once can show that, so a
check's rate is those FAILs over its runs on such commits — a commit attested once tells nothing
either way, and is said as the coverage, never counted as a pass. A check that fails on every run
of its commit is a defect, not a flake. Said too: the FAILs on commits run once, which nobody can
class, and the share of attests made red by flakes alone — what a red costs the team.

`--since` keeps the attests whose commit has that one among its ancestors (D104's window: from its
last fix), read in a clone (`--repo`, by default here); a commit the clone does not know is said,
not counted. A name holding a run's own temporary path is compared without it. An attest with no
check at all is said apart, by its file; none read at all fails.

Said apart, never as a flake of its own: `logs_read`, which fails whenever another check of its
shard made the logs say an error (it follows that FAIL), and the shards' own lines.
"""

from __future__ import annotations

import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

LINE = re.compile(r"^\s+(ok|FAIL)\s+(.+?)\s*$")
# A run's own folder in a check's name: /…/run-<pid>/tmp/tmp.<x>, never the same twice.
RUN_PATH = re.compile(r"run-\d+/tmp/tmp\.\w+")
DERIVED = ("logs_read", "shard ")
BOUND = 0.02


def checks(file: Path) -> list[tuple[str, str]]:
    """An attest's checks, each its state and its name, a run's own path taken out."""
    found = (LINE.match(line) for line in file.read_text(errors="replace").splitlines())
    return [(state, RUN_PATH.sub("run-*/tmp/tmp.*", name)) for state, name in (m.groups() for m in found if m)]


def count(read: dict[str, list[list[tuple[str, str]]]]) -> dict[str, tuple[int, int, int]]:
    """Per check: its FAILs where it passed on another run of the commit, its runs on commits run more
    than once, and all its runs. `read` maps each commit to its attests' checks."""
    states: dict[str, dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))
    for commit, attests in read.items():
        for attest in attests:
            for state, name in attest:
                states[name][commit].append(state)
    counted = {}
    for name, by in states.items():
        again = [said for said in by.values() if len(said) > 1]
        counted[name] = (sum(said.count("FAIL") for said in again if "ok" in said), sum(map(len, again)), sum(map(len, by.values())))
    return counted


def after(commit: str, since: str, repo: Path) -> bool | None:
    """Whether `since` is among `commit`'s ancestors (itself included); None when the clone does not know it."""
    known = subprocess.run(["git", "-C", str(repo), "cat-file", "-e", f"{commit}^{{commit}}"], capture_output=True)
    if known.returncode != 0:
        return None
    return subprocess.run(["git", "-C", str(repo), "merge-base", "--is-ancestor", since, commit], capture_output=True).returncode == 0


def main() -> int:
    args = sys.argv[1:]
    option = lambda name: args[args.index(name) + 1] if name in args else None
    folder = Path(args[0])
    since, repo = option("--since"), Path(option("--repo") or ".")
    read: dict[str, list[list[tuple[str, str]]]] = defaultdict(list)
    empty, unknown, files = [], [], sorted(folder.glob("2*-pr*-*.txt"))
    for file in files:
        commit = file.stem.rsplit("-", 1)[-1]
        if since:
            kept = after(commit, since, repo)
            if kept is None:
                unknown.append(file.name)
            if not kept:
                continue
        said = checks(file)
        if not said:
            empty.append(file.name)
            continue
        read[commit].append(said)
    attests = sum(map(len, read.values()))
    again = sum(len(runs) for runs in read.values() if len(runs) > 1)
    print(f"attests read: {attests}{' since ' + since if since else ''}, of {len(read)} commits;"
          f" run more than once: {sum(1 for runs in read.values() if len(runs) > 1)} commits, {again} attests")
    for name in empty:
        print(f"  no check at all, set apart: {name}")
    for name in unknown:
        print(f"  its commit unknown in {repo}, not counted: {name}")
    if not attests:
        print("nothing to count")
        return 2
    counted = count(read)
    # Each FAIL classed: a flake where the commit also passed that check, a defect where it never did,
    # unclassed on a commit run once. An attest red by flakes alone: its FAILs, derived ones apart, all flakes.
    passed = {(commit, name) for commit, runs in read.items() for run in runs for state, name in run if state == "ok"}
    unclassed, red, by_flakes = 0, 0, 0
    for commit, runs in read.items():
        for run in runs:
            fails = [name for state, name in run if state == "FAIL" and not name.startswith(DERIVED)]
            if len(runs) == 1:
                unclassed += len(fails)
            elif fails:
                red += 1
                by_flakes += all((commit, name) in passed for name in fails)
    print(f"FAILs on commits run once, unclassed: {unclassed}")
    print(f"attests red, on commits run more than once: {red}; by flakes alone: {by_flakes} of {again} ({by_flakes / again if again else 0:.1%})")
    flaky = sorted(((fails / again_runs, fails, again_runs, runs, name) for name, (fails, again_runs, runs) in counted.items() if fails),
                   reverse=True)
    print(f"checks: {len(counted)}; rate: FAILs where the commit also passed, over the runs of commits run more than once")
    for rate, fails, again_runs, runs, name in flaky:
        apart = " (said apart: it follows another FAIL)" if name.startswith(DERIVED) else ""
        print(f"{rate:6.1%}  {fails}/{again_runs} (of {runs} runs)  {name}{apart}")
    over = [name for rate, *_, name in flaky if rate >= BOUND and not name.startswith(DERIVED)]
    print(f"checks at or over {BOUND:.0%}, derived ones apart: {len(over)}")
    return 1 if over else 0


if __name__ == "__main__":
    sys.exit(main())
