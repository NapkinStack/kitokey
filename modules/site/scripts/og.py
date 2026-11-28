"""The picture a link to the site shows (Open Graph, 1200 x 630, one per language): the mark, the name, the
hero's promise (PDR-0009) in the titles' face, the address. Drawn once and kept in public/og/, as the marks
are; drawn again when the promise changes, from modules/site: `python3 scripts/og.py` (Pillow, which reads
public/fonts/bricolage-latin.woff2 and its instances, and ImageMagick's `magick` for public/kitokey.svg)."""

import io
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
    """Broken between words where the next would pass `width`: the picture is never cut."""
    out: list[str] = []
    for word in said.split(" "):
        if out and font.getlength(f"{out[-1]} {word}") <= width:
            out[-1] = f"{out[-1]} {word}"
        else:
            out.append(word)
    return out


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
    said = [(line, INK) for line in lines(first, title, 1024)] + [(line, SOFT) for line in lines(second, title, 1024)]
    for i, (line, fill) in enumerate(said):
        draw.text((88, 284 + i * 80), line, font=title, fill=fill, anchor="ls")
    draw.text((88, 574), "kitokey.com", font=face(30, b"SemiBold"), fill=ACCENT, anchor="ls")
    draw.rectangle((0, 618, 1200, 630), fill=ACCENT)
    picture.save(ROOT / "public" / "og" / f"og-{lang}.png", optimize=True)
