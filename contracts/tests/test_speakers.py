"""The names a person gives a meeting's speakers (#1205): session-audio/v3's `speakers.json`, written by the
shell, and summarising/v1 reading it, so that the minutes say who does what by name. A meeting that holds none
reads as before."""

import json
from pathlib import Path

CONTRACTS = Path(__file__).parent.parent
SESSION = (CONTRACTS / "session-audio/v3/README.md").read_text(encoding="utf-8")
SUMMARISING = (CONTRACTS / "summarising/v1/README.md").read_text(encoding="utf-8")


def flat(text):
    return " ".join(text.split())


def test_the_folder_may_hold_the_names_the_user_gave_its_speakers():
    schema = json.loads((CONTRACTS / "session-audio/v3/speakers.schema.json").read_text(encoding="utf-8"))
    assert set(schema["properties"]) == {"me", "others", "voices"}
    row = next(line for line in SESSION.splitlines() if line.startswith("| `speakers.json` |"))
    assert "The shell, on the user's word; optional" in row
    assert "the shell and `summary` read it" in row
    page = flat(SESSION)
    assert "a name absent, the speaker keeps the shell's word for it" in page
    assert "its voice's name, else its speaker's" in page
    assert "a name the schema refuses reads as absent, and the rest of the file still reads" in page


def test_summary_reads_the_names_and_says_each_segment_by_its_speakers_name():
    page = flat(SUMMARISING)
    assert "its `context.txt` and its `speakers.json` (`session-audio/v3`), when there are" in page
    assert "a segment is said by its speaker's name" in page
    assert "A meeting without `speakers.json` is read as before" in page
