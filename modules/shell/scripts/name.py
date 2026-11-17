#!/usr/bin/env python3
"""The name in the window's header, « kitokey » outlined (D89, accepted 2026-11-08:
docs/project/drawings/d89-d90-name-kitokey): Montserrat 700, kerned, letter-spaced -0.02 em, its
i drawn as the mark's stem — the stem rises to the record dot, as wide as the stroke, its top at
the k's height, cut from it by an empty gap of 0.41 times the stroke; the « t » untouched.

Montserrat is under the SIL Open Font License 1.1 (Copyright 2024 The Montserrat.Git Project
Authors, https://openfontlicense.org): its outlines are drawn into the page, no font is shipped.

`name.py` writes the drawing into ui/index.html, between its two markers; `name.py --check` draws
it again and fails when the page holds another. Needs fontTools and the font (Fedora:
julietaula-montserrat-fonts); KITOKEY_NAME_FONT names another copy of Montserrat-Bold.otf.
"""
import os
import pathlib
import sys

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

FONT = os.environ.get("KITOKEY_NAME_FONT", "/usr/share/fonts/julietaula-montserrat-fonts/Montserrat-Bold.otf")
PAGE = pathlib.Path(__file__).resolve().parent.parent / "ui" / "index.html"
BEGIN, END = "<!-- The name: drawn by scripts/name.py, never by hand. -->", "<!-- The name's end. -->"
SIZE = 100  # the drawing's unit: a hundredth of the em
HEADER = 18  # the font size, in CSS px, the header shows it at: 0.75 of the mark's 24 px, the board's lock-up
TRACKING = -0.02  # em, after each letter, as the board's letter-spacing
GAP = 0.41  # of the stroke, the empty ring around the dot
LIVE = 1.375  # the dot while recording, as the mark's


def number(value: float) -> str:
    text = f"{round(value, 1):.1f}".rstrip("0").rstrip(".")
    return "0" if text == "-0" else text


def kerning(font: TTFont, left: str, right: str) -> int:
    """The pair's adjustment by GPOS's kern feature, pair positioning formats 1 and 2."""
    gpos = font["GPOS"].table
    indices = {i for record in gpos.FeatureList.FeatureRecord if record.FeatureTag == "kern" for i in record.Feature.LookupListIndex}
    for index in sorted(indices):
        lookup = gpos.LookupList.Lookup[index]
        for sub in lookup.SubTable:
            sub = sub.ExtSubTable if lookup.LookupType == 9 else sub  # noqa: PLW2901 — an extension's subtable read as the one it wraps, on purpose
            if sub.LookupType != 2 or left not in sub.Coverage.glyphs:
                continue
            if sub.Format == 1:
                pairs = sub.PairSet[sub.Coverage.glyphs.index(left)].PairValueRecord
                found = next((pair.Value1 for pair in pairs if pair.SecondGlyph == right), None)
            else:
                first, second = sub.ClassDef1.classDefs.get(left, 0), sub.ClassDef2.classDefs.get(right, 0)
                found = sub.Class1Record[first].Class2Record[second].Value1
            if found is not None:
                return getattr(found, "XAdvance", 0) or 0
    return 0


def draw() -> str:
    font = TTFont(FONT)
    glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
    scale = SIZE / font["head"].unitsPerEm
    names = [cmap[ord(c)] for c in "kıtokey"]

    def box(name: str) -> tuple:
        pen = BoundsPen(glyphs)
        glyphs[name].draw(pen)
        return pen.bounds

    path, x, left, right = SVGPathPen(glyphs, ntos=number), 0.0, None, 0.0
    for i, name in enumerate(names):
        x_min, _, x_max, _ = box(name)
        left = x * scale + x_min * scale if left is None else left
        right = max(right, x * scale + x_max * scale)
        if name == "dotlessi":
            stem_left, stem_right, i_top = x * scale + x_min * scale, x * scale + x_max * scale, box(name)[3] * scale
        glyphs[name].draw(TransformPen(path, (scale, 0, 0, -scale, x * scale, 0)))
        following = names[i + 1] if i + 1 < len(names) else None
        x += glyphs[name].width + (kerning(font, name, following) if following else 0) + TRACKING * font["head"].unitsPerEm
    k_top, descent = box("k")[3] * scale, min(box(n)[1] for n in names) * scale
    stem = stem_right - stem_left
    r, cx = stem / 2, (stem_left + stem_right) / 2
    cy = -k_top + r  # the dot's top at the k's height, the baseline at 0
    top = cy - r * LIVE  # room for the dot while recording
    width, height = right - left, -descent - top
    size = HEADER / SIZE
    v = number
    return (
        f'<svg id="name" role="img" aria-label="Kitokey" width="{v(width * size)}" height="{v(height * size)}"'
        f' viewBox="{v(left)} {v(top)} {v(width)} {v(height)}">\n'
        f"<!-- {font['name'].getDebugName(4)}, {font['name'].getDebugName(5)}: SIL Open Font License 1.1. -->\n"
        f'<mask id="name-cut" maskUnits="userSpaceOnUse" x="{v(left)}" y="{v(top)}" width="{v(width)}" height="{v(height)}">'
        f'<rect x="{v(left)}" y="{v(top)}" width="{v(width)}" height="{v(height)}" fill="#fff"></rect>'
        f'<circle id="name-gap" cx="{v(cx)}" cy="{v(cy)}" r="{v(r + GAP * stem)}" data-rest="{v(r + GAP * stem)}"'
        f' data-live="{v(r * LIVE + GAP * stem)}" fill="#000"></circle></mask>\n'
        f'<g mask="url(#name-cut)" fill="currentColor"><path d="{path.getCommands()}"></path>'
        f'<rect x="{v(stem_left)}" y="{v(cy)}" width="{v(stem)}" height="{v(-i_top - cy + 1)}"></rect></g>\n'
        f'<circle id="name-dot" cx="{v(cx)}" cy="{v(cy)}" r="{v(r)}" data-rest="{v(r)}" data-live="{v(r * LIVE)}"></circle>\n'
        "</svg>"
    )


def main() -> int:
    page = PAGE.read_text()
    start, end = page.index(BEGIN) + len(BEGIN), page.index(END)
    drawn = page[:start] + "\n" + draw() + "\n" + page[end:]
    if sys.argv[1:] == ["--check"]:
        if drawn != page:
            print("ui/index.html holds another name than scripts/name.py draws: run scripts/name.py")
            return 1
        return 0
    PAGE.write_text(drawn)
    return 0


if __name__ == "__main__":
    sys.exit(main())
