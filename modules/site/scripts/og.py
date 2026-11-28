"""The picture a link to the site shows (Open Graph, 1200 x 630, one per language): the mark, the name, the
hero's promise (PDR-0009) in the titles' face, its second line in the accent as the hero's (.hero h1 .soft), the
address. Drawn once and kept in public/og/, as the marks
are; drawn again when the promise changes, from modules/site: `python3 scripts/og.py` (Pillow, which reads
public/fonts/bricolage-latin.woff2 and its instances, and ImageMagick's `magick` for public/kitokey.svg)."""

import io
from itertools import combinations
import json
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FACE = ROOT / "public" / "fonts" / "bricolage-latin.woff2"
INK, SOFT, ACCENT, GROUND = "#15171C", "#4C5464", "#2253CF", "#F2F4F7"


def face(size: int, instance: bytes) -> ImageFont.FreeTypeFont:
    font = ImageFont.truetype(str(FACE), size)
    font.set_variation_by_name(instance)
    return font


def lines(said: str, font: ImageFont.FreeTypeFont, width: int) -> list[str]:
    """As few lines as fit `width` (the picture is never cut), balanced as the hero's `text-wrap: balance`: of the
    breaks giving that many lines, the one whose longest line is shortest, so no word stands alone."""
    words = said.split(" ")
    greedy: list[str] = []
    for word in words:
        if greedy and font.getlength(f"{greedy[-1]} {word}") <= width:
            greedy[-1] = f"{greedy[-1]} {word}"
        else:
            greedy.append(word)
    count = len(greedy)
    best, longest = greedy, max(font.getlength(line) for line in greedy)
    for cuts in combinations(range(1, len(words)), count - 1):
        bounds = (0, *cuts, len(words))
        split = [" ".join(words[a:b]) for a, b in zip(bounds, bounds[1:])]
        widest = max(font.getlength(line) for line in split)
        if widest <= width and widest < longest:
            best, longest = split, widest
    return best


hero = json.loads(subprocess.run(
    ["node", "--input-type=module", "-e",
     "import { landing } from './src/lib/landing.mjs'; console.log(JSON.stringify(landing.hero))"],
    cwd=ROOT, check=True, capture_output=True, text=True).stdout)
mark = Image.open(io.BytesIO(subprocess.run(
    ["magick", "-background", "none", "-density", "600", str(ROOT / "public" / "kitokey.svg"), "-resize", "80x80", "png:-"],
    check=True, capture_output=True).stdout)).convert("RGBA")
(ROOT / "public" / "og").mkdir(exist_ok=True)
for lang in ("en", "fr"):
    first, second = hero[lang][:2]
    picture = Image.new("RGB", (1200, 630), GROUND)
    draw = ImageDraw.Draw(picture)
    picture.paste(mark, (88, 80), mark)
    draw.text((188, 120), "Kitokey", font=face(46, b"Bold"), fill=INK, anchor="lm")
    title = face(66, b"ExtraBold")
    said = [(line, INK) for line in lines(first, title, 1024)] + [(line, ACCENT) for line in lines(second, title, 1024)]
    for i, (line, fill) in enumerate(said):
        draw.text((88, 284 + i * 80), line, font=title, fill=fill, anchor="ls")
    draw.text((88, 574), "kitokey.com", font=face(30, b"SemiBold"), fill=SOFT, anchor="ls")
    draw.rectangle((0, 618, 1200, 630), fill=ACCENT)
    picture.save(ROOT / "public" / "og" / f"og-{lang}.png", optimize=True)
