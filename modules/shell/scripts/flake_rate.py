"""How often each check of the shell's e2e fails where nothing changed (D104, criterion 2).

    flake_rate.py <a folder of attests' e2e> [--since <stamp>]

The folder holds one file per attest, `<stamp>-pr<n>-<commit>.txt`, its e2e's text as the attest
printed it. A check's FAIL counts when the same check passed on another run of the same commit:
nothing changed between them but the run. A check that fails on every run of its commit is a defect,
not a flake, and is not counted. Its rate is those FAILs over all its runs. `--since` keeps the
attests from a stamp on: D104's window starts at its last fix.

Said apart, never as a flake of its own: `logs_read`, which fails whenever another check of its shard
made the logs say an error (it follows that FAIL), and the shards' own lines, which sum their checks.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

LINE = re.compile(r"^\s+(ok|FAIL)\s+(.+?)\s*$")
DERIVED = ("logs_read", "shard ")
BOUND = 0.02


def count(files: list[Path]) -> dict[str, tuple[int, int]]:
    """Each check's FAILs where it passed on another run of its commit, and its runs."""
    runs: dict[str, int] = defaultdict(int)
    states: dict[str, dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))
    for file in files:
        commit = file.stem.rsplit("-", 1)[-1]
        for line in file.read_text(errors="replace").splitlines():
            found = LINE.match(line)
            if found:
                state, check = found.groups()
                runs[check] += 1
                states[check][commit].append(state)
    return {check: (sum(said.count("FAIL") for said in states[check].values() if "ok" in said), runs[check]) for check in runs}


def attests(folder: Path, since: str = "") -> list[Path]:
    """The attests' files, from a stamp on."""
    return sorted(file for file in folder.glob("2*-pr*-*.txt") if file.name >= since)


def main() -> int:
    args = sys.argv[1:]
    since = args[args.index("--since") + 1] if "--since" in args else ""
    folder = Path(next(arg for i, arg in enumerate(args) if arg != "--since" and (i == 0 or args[i - 1] != "--since")))
    files = attests(folder, since)
    counted = count(files)
    flaky = sorted(((fails / n, fails, n, check) for check, (fails, n) in counted.items() if fails), reverse=True)
    print(f"attests read: {len(files)}{' since ' + since if since else ''}; checks: {len(counted)}")
    for rate, fails, n, check in flaky:
        apart = " (said apart: it follows another FAIL)" if check.startswith(DERIVED) else ""
        print(f"{rate:6.1%}  {fails}/{n}  {check}{apart}")
    over = [check for rate, _, _, check in flaky if rate >= BOUND and not check.startswith(DERIVED)]
    print(f"checks at or over {BOUND:.0%}, derived ones apart: {len(over)}")
    return 1 if over else 0


if __name__ == "__main__":
    sys.exit(main())
