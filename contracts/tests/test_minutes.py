"""minutes/v5 (D111): the page's own example holds the rules its page states, and each file the page
says to refuse is refused. A Markdown contract has no schema for the comparator (ADR-0005); these
rules are what both sides' tests read: summary writing v5, the shell reading it."""

import json
import re
from pathlib import Path

import pytest

PAGE = Path(__file__).parent.parent / "minutes" / "v5" / "README.md"
NAME = re.compile(r"[a-z0-9_]{1,40}")


def front_matter(text):
    """The front matter's keys, each value read as the page says: in double quotes or square
    brackets as JSON, any other as written."""
    lines = text.split("\n")
    assert lines[0] == "---", "no front matter"
    end = lines.index("---", 1)
    keys = {}
    for line in lines[1:end]:
        key, sep, value = line.partition(": ")
        assert sep and re.fullmatch(r"[a-z_]+", key), f"not key: value: {line!r}"
        value = value.strip()
        keys[key] = json.loads(value) if value[:1] in '"[' else value
    return keys, "\n".join(lines[end + 1 :])


def refusal(text):
    """Why a v5 file is refused, or None when it holds."""
    try:
        keys, body = front_matter(text)
    except (AssertionError, ValueError) as why:
        return str(why)
    if keys.get("contract") != "minutes/v5":
        return "not minutes/v5"
    structure, sections = keys.get("structure"), keys.get("sections")
    if not isinstance(structure, str) or not NAME.fullmatch(structure):
        return "structure missing or out of bounds"
    if not isinstance(sections, list) or not 1 <= len(sections) <= 6:
        return "sections missing or not one to six"
    ids = []
    for section in sections:
        if not isinstance(section, dict) or not isinstance(section.get("id"), str):
            return "a section without its id"
        about = section.get("about")
        if (
            not NAME.fullmatch(section["id"])
            or not isinstance(about, str)
            or not about
            or "\n" in about
            or len(about) > 200
        ):
            return "a section out of bounds"
        ids.append(section["id"])
    if len(set(ids)) != len(ids):
        return "an id twice"
    if "actions" in keys and keys["actions"] not in ids:
        return "actions names no id"
    titles = [line for line in body.split("\n") if line.startswith("## ")]
    if len(titles) != len(sections):
        return f"{len(titles)} sections in the body, {len(sections)} in the front matter"
    return None


def example():
    page = PAGE.read_text()
    valid = page[page.index("Valid:") :]
    return valid[valid.index("```markdown\n") + len("```markdown\n") : valid.index("\n```", valid.index("```markdown"))]


def test_the_pages_example_holds_its_rules():
    assert refusal(example()) is None


def swapped(text, old, new):
    assert old in text, old
    return text.replace(old, new, 1)


@pytest.mark.parametrize(
    "change",
    [
        lambda t: swapped(t, "contract: minutes/v5", "contract: minutes/v6"),
        lambda t: swapped(t, "structure: project_review\n", ""),
        lambda t: swapped(t, "structure: project_review", "structure: Project review"),
        lambda t: t[: t.index("sections: ")] + t[t.index("\n", t.index("sections: ")) + 1 :],
        lambda t: swapped(t, '"id": "risks"', '"id": "status"'),
        lambda t: swapped(t, '"id": "risks"', '"id": "Risks"'),
        lambda t: swapped(t, "actions: actions", "actions: next_steps"),
        lambda t: swapped(t, "## Décisions\n", ""),
        lambda t: swapped(t, "## Actions", "## Actions\n\nRien.\n\n## Suite"),
        lambda t: swapped(t, "made: ", "made "),
    ],
    ids=[
        "another contract",
        "no structure",
        "structure out of bounds",
        "no sections",
        "an id twice",
        "an id out of bounds",
        "actions naming no id",
        "a section fewer",
        "a section more",
        "not key: value",
    ],
)
def test_what_the_page_refuses_is_refused(change):
    assert refusal(change(example())) is not None


def test_the_six_structures_name_their_sections_and_where_actions_are():
    page = PAGE.read_text()
    table = page[page.index("The six structures this version names") :].split("\n\n")[1]
    rows = table.split("\n")[2:]
    named = {}
    for row in rows:
        cells = [cell.strip() for cell in row.strip("|").split("|")]
        ids = re.findall(r"`([a-z0-9_]+)`", cells[1].split(" — ")[0])
        actions = cells[2].strip("`")
        assert len(ids) == len(set(ids)) and 1 <= len(ids) <= 6 and all(NAME.fullmatch(i) for i in ids), row
        assert actions in ids, row
        named[cells[0].strip("`")] = ids
    assert set(named) == {"general", "one_to_one", "stand_up", "client_call", "interview", "project_review"}
    assert named["general"] == ["summary", "actions", "key_points"]
