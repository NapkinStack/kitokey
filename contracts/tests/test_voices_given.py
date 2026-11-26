"""A voice given by a person (#1244, S1 of #1230, option A — the decider, 2026-11-25): once a transcript is
complete, the shell may set one segment's `voice` on the user's word and say when in `given`; a voice so given
outlives a transcript made again (D113); and no voice embedding is ever written or kept (GDPR art. 9, EDPB
Guidelines 3/2019 §80–82). transcript/v1 and session-audio/v3 change in place, the comparator's way."""

import json
from pathlib import Path

from jsonschema import Draft202012Validator

CONTRACTS = Path(__file__).parent.parent
TRANSCRIPT = (CONTRACTS / "transcript/v1/README.md").read_text(encoding="utf-8")
SESSION = (CONTRACTS / "session-audio/v3/README.md").read_text(encoding="utf-8")
LINE = json.loads((CONTRACTS / "transcript/v1/line.schema.json").read_text(encoding="utf-8"))
READS = Draft202012Validator(LINE)
SEGMENT = {"type": "segment", "speaker": "others", "track": "remote", "start": 6.0, "end": 7.0, "text": "x"}


def flat(text):
    return " ".join(text.split())


def bullet(page, starts):
    """The bullet of a page that starts with `starts`, flattened, up to the next bullet or heading."""
    rest = flat(page.split(starts, 1)[1])
    return rest.split(" - **", 1)[0].split(" ## ", 1)[0]


def test_a_segment_may_say_its_voice_was_given_and_when():
    segment = next(kind for kind in LINE["oneOf"] if kind["properties"]["type"].get("const") == "segment")
    assert "given" in segment["properties"] and "given" not in segment["required"]
    assert segment.get("dependentRequired", {}).get("given") == ["voice"]
    assert READS.is_valid({**SEGMENT, "voice": "2", "given": "2026-11-26T10:02:00+01:00"})
    assert READS.is_valid({**SEGMENT, "voice": None, "given": "2026-11-26T10:02:00+01:00"})
    assert READS.is_valid({**SEGMENT, "voice": "2"}), "a voice an engine told, given by no one, reads as before"
    for wrong in ("", None, 1763023330, "2026-11-26T10:02:00+01:00\n", "a\rb"):
        assert not READS.is_valid({**SEGMENT, "voice": "2", "given": wrong}), wrong
    assert not READS.is_valid({**SEGMENT, "given": "2026-11-26T10:02:00+01:00"}), "given with no voice"


def test_who_spoke_says_a_voice_is_estimated_or_given():
    said = bullet(TRANSCRIPT, "- **Who spoke is two questions**")
    assert "estimated by the producer, or given by a person" in said
    assert "`given` says when a person gave it" in said


def test_the_shell_may_give_one_segment_its_voice_once_complete():
    said = bullet(TRANSCRIPT, "- **A voice given, once complete**")
    assert "After its `complete` line, and never while it is made again" in said
    assert "the shell may set one segment's `voice` on the user's word and set its `given` to when" in said
    assert "The file replaced whole" in said
    assert "nothing else changed" in said
    # The correction's own bullet still says « nothing else » for the text: it now names the voice beside it.
    correction = bullet(TRANSCRIPT, "- **A correction, once complete**")
    assert "a voice given, below" in correction


def test_a_given_voice_outlives_a_transcript_made_again():
    said = bullet(TRANSCRIPT, "- **A given voice outlives a transcript made again**")
    assert "known by its track and its stretch of the clock, from `start` to `end`" in said
    assert (
        "a new segment of the same track whose middle falls within a given segment's stretch takes its `voice` and `given`"
        in said
    )
    assert "The shell carries them over, not the producer" in said
    assert "a given segment that no new segment falls within is said to the user, never dropped silently" in said


def test_no_voice_embedding_is_ever_written_or_kept():
    never = flat(TRANSCRIPT.split("## What it never does", 1)[1].split("## ", 1)[0])
    assert "It never writes or keeps a voice's embedding" in never
    assert "never compares voices across meetings" in never
    assert "GDPR art. 9" in never and "EDPB Guidelines 3/2019" in never


def test_a_voice_the_shell_names_may_be_told_or_given():
    row = next(line for line in SESSION.splitlines() if line.startswith("| `voices`, optional |"))
    assert "No producer tells voices apart yet" not in row
    assert "one `transcript/v1` tells apart or one a person gave there (its `given`)" in flat(row)
