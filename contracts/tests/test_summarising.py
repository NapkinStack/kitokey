"""summarising/v1's `--format` (D129): a format a person made, its sections and their `about` in the person's
own words, written as minutes/v5; the six structures and no structure as before. What a schema cannot say
is the page's: the ids each once, `actions` one of them, the two options never together."""

from pathlib import Path

PAGE = (Path(__file__).parent.parent / "summarising" / "v1" / "README.md").read_text(encoding="utf-8")
FLAT = " ".join(PAGE.split())


def test_a_format_a_person_made_is_read_from_a_file_and_written_as_minutes_v5():
    bullet = FLAT.split("**Minutes in a format a person made** (D129)", 1)[1].split(" - **", 1)[0]
    assert "`--format <file>`" in bullet
    assert "`format.schema.json`" in bullet
    assert "writes `minutes/v5`" in bullet
    assert "each `about` as the person wrote them, in their language" in bullet


def test_a_format_it_cannot_read_is_a_wrong_argument_and_nothing_is_written():
    bullet = FLAT.split("**Minutes in a format a person made** (D129)", 1)[1].split(" - **", 1)[0]
    for wrong in (
        "with `--structure`",
        "a file it cannot read",
        "not the schema's",
        "an `id` twice",
        "`actions` that names no section",
    ):
        assert wrong in bullet, wrong
    assert "is a wrong argument: nothing is written" in bullet


def test_the_six_structures_and_no_structure_are_as_before():
    assert "Without `--structure`, it writes `minutes/v4` as before" in FLAT
    assert "A name it does not know is a wrong argument" in FLAT
    assert "The six structures keep their names: a format never takes one" in FLAT
