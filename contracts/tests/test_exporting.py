"""exporting/v1's page: what a PDF's fonts are, and what a person does when none has a meeting's letters."""

from pathlib import Path

PAGE = (Path(__file__).parent.parent / "exporting/v1/README.md").read_text(encoding="utf-8")


def test_a_pdf_is_written_in_static_faces_alone_a_letter_only_a_variable_face_has_is_pdf_no_font():
    """The decider, 2026-11-25: refuse rather than write a variable face's default instance (Thin)."""
    flat = " ".join(PAGE.split())
    assert "**The fonts of a PDF** are the system's static faces." in flat
    assert "is a letter the program does not have: `pdf.no_font`, nothing written" in flat
    assert "never a variable face's default instance" in flat
    assert "at its default weight" not in flat, "a variable face is never used"
    row = next(line for line in PAGE.splitlines() if line.startswith("| `pdf.no_font` |"))
    assert "No static font of the system has the meeting's letters" in row
    assert "on Fedora, for Chinese, Japanese and Korean: `sudo dnf install google-noto-sans-cjk-fonts`" in row
