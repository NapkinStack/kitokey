"""Writes Kitokey's design system out of its code (D120, criterion 5): `design-system/`, from `ui/tokens.css` and
`ui/components.css`, which stay the one source. Run from `modules/shell`:

    python3 scripts/design-export.py

It writes:
- `design-system/tokens.css`, `ui/tokens.css` as it is;
- `design-system/tokens.json`, the tokens in the W3C Design Tokens format (DTCG), the dark theme in each colour's
  `$extensions["com.kitokey"].dark`;
- `design-system/project/`, what Claude Design's « Design System » reads as it is (`editing: code`): its index
  `design-system.json`, `tokens.json` in lists (each token's CSS variable `--<name>`, the names components.css
  reads), `README.md` (the brand book), `fonts/`, and per component its `README.md` and `preview.html`, with
  `components/bundle.css` = `ui/components.css`.

Made again after any change of tokens.css or components.css: `tests/design.rs` fails until it is. The commit
named is the one the export is made from (`HEAD`), its date the date written: the same code, the same files.
"""

import json
import os
import re
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
UI = HERE / "ui"
OUT = HERE / "design-system"
PROJECT = OUT / "project"
FONT = "bricolage-latin-600-700.woff2"

# What each role says (PDR-0008), for the page and the W3C file.
USAGE = {
    "color-canvas": "The window's ground.",
    "color-surface": "A card, a panel, a field: what sits on the canvas.",
    "color-sunken": "A well in a surface: a search field, a track.",
    "color-raised": "A dialog, a menu: what floats above.",
    "color-hover": "A secondary control under the pointer.",
    "color-pressed": "A secondary control pressed.",
    "color-tint": "The accent's ground: a chosen card, an information banner.",
    "color-divider": "A line that divides, and only that.",
    "color-edge": "A control's edge, 3:1 on each ground.",
    "color-text": "The words.",
    "color-text-soft": "Secondary words: a hint, a date, a detail.",
    "color-text-faint": "Tertiary words, on any ground at 4.5:1.",
    "color-accent": "Links, focus, the primary action; « Vous ».",
    "color-accent-hover": "The primary action under the pointer.",
    "color-on-accent": "Words on the accent.",
    "color-success": "Ready, sent, saved: success alone, never a voice.",
    "color-success-ground": "A success chip's ground, opaque: its words keep 4.5:1 on any page.",
    "color-warning": "A warning's icon and words.",
    "color-warning-ground": "A warning's ground.",
    "color-record": "A recording: its live dot. Red says a recording or a deletion, nothing else.",
    "color-destructive": "A deletion's button, always confirmed, « Annuler » focused.",
    "color-on-destructive": "Words on a deletion's button.",
    "color-private": "« Confidentielle », with the lock: its pill, its banner. Nowhere else.",
    "color-private-ground": "The ground of « Confidentielle »'s banner.",
    "color-voice-you": "« Vous »: the microphone, in the waveform, the meters, the transcript.",
    "color-voice-others": "« Les autres »: the call's sound; teal in light, golden amber in dark.",
    "color-scrim": "Behind a modal dialog.",
}


def git(*args: str) -> str:
    return subprocess.run(["git", *args], cwd=HERE, capture_output=True, text=True, check=True).stdout.strip()


def rules(css: str) -> list[tuple[str, str]]:
    """Each rule's selector and body, comments aside; a rule inside `@media` keeps the query in its selector."""
    css = re.sub(r"/\*.*?\*/", "", css, flags=re.S)
    found = []
    for rule in css.split("}"):
        if "{" in rule:
            selector, body = rule.rsplit("{", 1)
            found.append((" ".join(selector.split()), body))
    return found


def declarations(body: str) -> dict[str, str]:
    return {name.strip(): value.strip() for name, value in (one.split(":", 1) for one in body.split(";") if ":" in one)}


def themes(css: str) -> tuple[dict[str, str], dict[str, str]]:
    """The custom properties, light (`:root`) and dark (Settings' dark laid over them), without their `--`."""
    light, dark = {}, {}
    for selector, body in rules(css):
        into = light if selector == ":root" else dark if selector == ':root[data-theme="dark"]' else None
        if into is not None:
            into.update({name[2:]: value for name, value in declarations(body).items() if name.startswith("--")})
    return light, {**light, **dark}


def hex_of(value: str) -> str:
    """`#RRGGBB` as it is; `rgb(r g b / a)` as `#RRGGBBAA`."""
    value = value.strip()
    if value.startswith("#"):
        return value.upper()
    found = re.fullmatch(r"rgb\(\s*(\d+)\s+(\d+)\s+(\d+)\s*/\s*([\d.]+)\s*\)", value)
    if not found:
        raise ValueError(f"no colour the export can write: {value}")
    r, g, b, a = int(found[1]), int(found[2]), int(found[3]), float(found[4])
    return f"#{r:02X}{g:02X}{b:02X}{round(a * 255):02X}"


def alias_or_hex(value: str) -> str:
    """A role's value for the page: `{name}` for `var(--name)`, else its colour."""
    found = re.fullmatch(r"var\(--([A-Za-z0-9_.-]+)\)", value.strip())
    return "{" + found[1] + "}" if found else hex_of(value)


def shadow_of(value: str) -> str:
    """A box-shadow with its colours as `#RRGGBBAA`."""
    return re.sub(r"rgb\([^)]*\)", lambda found: hex_of(found[0]), value)


def shadow_dtcg(value: str) -> list[dict]:
    """A box-shadow as the W3C format writes one: a layer each, offsets, blur, spread, colour, inset."""
    layers = []
    for layer in re.split(r",(?![^(]*\))", value):
        colour = re.search(r"rgb\([^)]*\)", layer)[0]
        words = layer.replace(colour, " ").split()
        inset = "inset" in words
        lengths = [word for word in words if word != "inset"]
        lengths += ["0"] * (4 - len(lengths))
        layers.append({"color": hex_of(colour), "offsetX": lengths[0], "offsetY": lengths[1],
                       "blur": lengths[2], "spread": lengths[3], "inset": inset})
    return layers


def expand(value: str, light: dict[str, str]) -> str:
    """A value with the `var(--…)` it reads written out: the page takes no var()."""
    return re.sub(r"var\(--([A-Za-z0-9_.-]+)\)", lambda found: expand(light[found[1]], light), value)


def tokens(css: str) -> dict:
    light, dark = themes(css)
    colours, shadows, families = [], [], {}
    scales = {"spacing": [], "radius": [], "fontSize": [], "duration": [], "easing": []}
    prefix = {"sp-": "spacing", "r-": "radius", "fs-": "fontSize", "d-": "duration", "ease-": "easing"}
    for name, value in light.items():
        if name == "color-scheme":
            continue
        if value.startswith("#"):
            colours.append({"name": name, "value": {"light": value.upper(), "dark": value.upper()},
                            "usage": "A primitive of the palette: read through a role, never by a component."})
        elif name.startswith("color-"):
            colours.append({"name": name, "value": {"light": alias_or_hex(value), "dark": alias_or_hex(dark[name])},
                            "usage": USAGE[name]})
        elif re.fullmatch(r"e\d", name):
            shadows.append({"name": name, "value": {"light": shadow_of(value), "dark": shadow_of(dark[name])},
                            "usage": "An elevation: e1 a card, e2 a toast, e3 a dialog."})
        elif name.startswith("font-"):
            families[name[len("font-"):]] = expand(value, light)
        elif family := next((family for start, family in prefix.items() if name.startswith(start)), None):
            scales[family].append({"name": name, "value": value})
        # Else one of the window's older names (--ground, --ink…): no component reads them, the export leaves them out.
    return {"colours": colours, "shadows": shadows, "families": families, "scales": scales, "light": light, "dark": dark}


def project_tokens(read: dict, ref: str) -> dict:
    return {
        "name": "Kitokey",
        "version": 1,
        "meta": {"source": "github", "repo": "NapkinStack/kitokey", "ref": ref,
                 "paths": {"tokens": ["modules/shell/ui/tokens.css"], "components": ["modules/shell/ui/components.css"]}},
        "color": {"themes": [{"id": "light", "name": "Clair"}, {"id": "dark", "name": "Sombre"}], "tokens": read["colours"]},
        "shadow": {"tokens": read["shadows"]},
        "spacing": {"tokens": read["scales"]["spacing"]},
        "radius": {"tokens": read["scales"]["radius"]},
        "fontSize": {"tokens": read["scales"]["fontSize"]},
        "duration": {"tokens": read["scales"]["duration"]},
        "easing": {"tokens": read["scales"]["easing"]},
        "type": {"fonts": [{"family": "Bricolage Grotesque", "file": f"fonts/{FONT}", "weight": "600 700"}],
                 "families": read["families"], "groups": []},
    }


def dtcg(read: dict) -> dict:
    """The W3C Design Tokens format: a group per kind, aliases `{color.<name>}`."""
    def colour(value: str) -> str:
        return "{color." + value[1:] if value.startswith("{") else value
    colours = {}
    for token in read["colours"]:
        light, dark = (colour(token["value"][theme]) for theme in ("light", "dark"))
        colours[token["name"]] = {"$type": "color", "$value": light, "$description": token["usage"]}
        if dark != light:
            colours[token["name"]]["$extensions"] = {"com.kitokey": {"dark": dark}}
    def dimension(family: str) -> dict:
        return {token["name"]: {"$type": "dimension", "$value": token["value"]} for token in read["scales"][family]}
    shadows = {}
    for token in read["shadows"]:
        name = token["name"]
        shadows[name] = {"$type": "shadow", "$value": shadow_dtcg(read["light"][name]),
                         "$extensions": {"com.kitokey": {"dark": shadow_dtcg(read["dark"][name])}}}
    return {
        "color": colours,
        "shadow": shadows,
        "spacing": dimension("spacing"),
        "radius": dimension("radius"),
        "fontSize": dimension("fontSize"),
        "duration": {token["name"]: {"$type": "duration", "$value": token["value"]} for token in read["scales"]["duration"]},
        "easing": {token["name"]: {"$type": "cubicBezier",
                                   "$value": [float(n) for n in re.findall(r"[\d.]+", token["value"])]}
                   for token in read["scales"]["easing"]},
        "fontFamily": {key: {"$type": "fontFamily", "$value": [face.strip().strip('"') for face in value.split(",")]}
                       for key, value in read["families"].items()},
    }


BOOK = """# Kitokey — design system

Exported from the code by `modules/shell/scripts/design-export.py`, from commit `{sha}` ({date}). The code is the
source: `modules/shell/ui/tokens.css` and `ui/components.css`. A design explored here comes back as a pull request on
those two files, never as an edit of this export.

## Principles

- **A colour is a role, and says one thing** (PDR-0008). « Vous » is the accent's blue (`color-voice-you`); « Les
  autres » teal in the light theme, golden amber in the dark one (`color-voice-others`). Red says a recording — its
  live dot, `color-record` — or a deletion — its button, `color-destructive`, always confirmed with « Annuler »
  focused —, and nothing else. Violet with the lock says « Confidentielle » (`color-private`), and nothing else.
  Green says success alone (`color-success`), never a voice.
- **A component reads roles and scales**, never a primitive (`gray-*`, `slate-*`, `blue-*`…) nor a value.
- **Words reach 4.5:1** on what they are written on, an edge, a voice's trace and the focus ring 3:1 (WCAG 2.2),
  in both themes; `tests/look.rs` holds each pair.
- **A control is 32 px high** (`sp-8`), on the 4 px grid (`sp-1` to `sp-8`).
- **Titles in Bricolage Grotesque** (`font-display`, weights 600 to 700, SIL OFL 1.1); every other word in the
  system's face (`font-text`).
- **The platform's own controls first**: a native radio in its label, the platform's modal `<dialog>`.

## The themes

`Clair` and `Sombre`: each role names its value in both. The window follows the system's theme unless Settings
choose one.

## Left out of this export

The window's older names (`--ground`, `--ink`, `--soft`…), which no component reads: they go as each screen comes
over to the components (cycle 12, D121 to D125).
"""

# Lucide's icons the previews draw (ISC, `ui/LUCIDE-LICENSE`), as the page draws them: no file fetched.
def svg(*shapes: str) -> str:
    return ('<svg viewBox="0 0 24 24" aria-hidden="true">' + "".join(
        shape if shape.startswith("<") else f'<path d="{shape}"/>' for shape in shapes) + "</svg>")


INFO = svg('<circle cx="12" cy="12" r="10"/>', "M12 16v-4", "M12 8h.01")
LOCK = svg('<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>', "M7 11V7a5 5 0 0 1 10 0v4")
ALERT = svg("m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3", "M12 9v4", "M12 17h.01")
CHECK = svg("M20 6 9 17l-5-5")
DRIVE = svg("M10 16h.01", "M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z", "M21.946 12.013H2.054", "M6 16h.01")
GLOBE = svg('<circle cx="12" cy="12" r="10"/>', "M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20", "M2 12h20")
FILE = svg("M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z", "M14 2v5a1 1 0 0 0 1 1h5", "M10 9H8", "M16 13H8", "M16 17H8")

COMPONENTS = {
    "Button": ("Buttons", 150, """A button does one thing, said in its words. Primary for the one action a view is for; secondary on
the surface; ghost, words alone; icon, a square whose name is in `aria-label` and its tooltip; destructive for a
deletion, red (`color-destructive`), always confirmed in a dialog whose « Annuler » is focused. 32 px high.
""", """<div style="display:flex;flex-wrap:wrap;gap:12px;padding:16px;background:var(--color-canvas)">
  <button type="button" class="button primary">Démarrer un enregistrement</button>
  <button type="button" class="button secondary">Ouvrir le dossier</button>
  <button type="button" class="button ghost">Annuler</button>
  <button type="button" class="button destructive">Mettre à la corbeille</button>
  <button type="button" class="button primary" disabled>Démarrage…</button>
</div>"""),
    "Banner": ("Feedback", 210, """A banner says what holds for the whole page, with its icon: information (`color-tint`), « Confidentielle »
(`color-private-ground`, the lock), a warning (`color-warning-ground`). Never red. It always carries its tone class.
""", """<div style="display:grid;gap:12px;padding:16px;background:var(--color-canvas)">
  <div class="banner info">""" + INFO + """<p>Vous pourrez la restaurer depuis la corbeille de votre système.</p></div>
  <div class="banner private">""" + LOCK + """<p>Rien ne quitte cet ordinateur</p></div>
  <div class="banner warning">""" + ALERT + """<p>Rédigé par une IA : relisez-le avant de le partager.</p></div>
</div>"""),
    "Dialog": ("Overlays", 250, """A dialog asks one question: the platform's modal `<dialog>`, its title in the titles' face, the safe
answer first and focused, Escape answering no, the focus back where it was. A deletion's answer is destructive.
""", """<div style="padding:16px;background:var(--color-canvas)">
  <div class="dialog" role="group" aria-label="Mettre cette réunion à la corbeille ?">
    <h2>Mettre cette réunion à la corbeille ?</h2>
    <p>Vous pourrez la restaurer depuis la corbeille de votre système.</p>
    <div class="actions"><button type="button" class="button secondary">Annuler</button><button type="button" class="button destructive">Mettre à la corbeille</button></div>
  </div>
</div>"""),
    "EngineCard": ("Choices", 330, """An engine card offers an engine for a job: a native radio in its label, its name, where it runs, its
state (ready, to download, to install, a key missing), its details folded. A group per job, in a fieldset whose
legend names the job. A state that asks something stays selectable.
""", """<div style="padding:16px;background:var(--color-canvas)">
  <fieldset class="engine-cards"><legend>Moteur de rédaction</legend>
    <div class="engine-card" data-state="ready" data-where="here"><label><input type="radio" name="w" checked><span class="name">Auto</span><span class="default chip accent">Par défaut</span><span class="where">Aujourd’hui Qwen3 8b</span><span class="state chip success">Prêt</span></label><details><summary>Détails</summary><p class="detail">qwen3:8b</p></details></div>
    <div class="engine-card" data-state="download" data-where="here"><label><input type="radio" name="w"><span class="name">Sur cet ordinateur</span><span class="where">Sur cet ordinateur</span><span class="state chip neutral">À télécharger · 5,2 Go</span></label><details><summary>Détails</summary><p class="detail">qwen3:8b</p></details></div>
    <div class="engine-card" data-state="key" data-where="online"><label><input type="radio" name="w"><span class="name">Service en ligne</span><span class="where">En ligne</span><span class="state chip warning">Clé manquante</span></label><details><summary>Détails</summary></details></div>
  </fieldset>
</div>"""),
    "Chip": ("Feedback", 90, """A chip says a short fact on an item, in one of five tones: neutral; accent, « Par défaut »; success, ready
or sent (`color-success-ground`, opaque); private, « Confidentielle » with the lock; warning, what is missing. Always
with its tone class. 24 px high.
""", """<div style="display:flex;flex-wrap:wrap;gap:12px;padding:16px;background:var(--color-canvas)">
  <span class="chip neutral">Transcription</span>
  <span class="chip accent">Par défaut</span>
  <span class="chip success">""" + CHECK + """Prêt</span>
  <span class="chip private">""" + LOCK + """Confidentielle</span>
  <span class="chip warning">""" + ALERT + """Clé manquante</span>
</div>"""),
    "OptionCard": ("Choices", 230, """An option card is one choice among a few: a native radio in its label, its icon on a tile, its title and
a sentence. Chosen, it is edged in the accent on the tint. The radios give the group and the arrow keys.
""", """<div style="padding:16px;background:var(--color-canvas)">
  <div class="option-cards" role="radiogroup" aria-label="Où il tourne">
    <label class="option-card"><input type="radio" name="o" checked><span class="tile">""" + DRIVE + """</span><span><span class="title">Sur cet ordinateur</span><span class="says">Privé et gratuit. Un peu plus lent.</span></span></label>
    <label class="option-card"><input type="radio" name="o"><span class="tile">""" + GLOBE + """</span><span><span class="title">En ligne</span><span class="says">Plus rapide. Seul le texte part, jamais le son.</span></span></label>
  </div>
</div>"""),
    "Progress": ("Feedback", 130, """A progress says what is being made and how long is left, with its cancel. A bar per step, each a
progress bar (`role=progressbar`) named and at its value.
""", """<div style="padding:16px;background:var(--color-canvas)">
  <div class="progress">
    <div class="task">""" + FILE + """<b>Rédaction du compte rendu</b><span class="left">environ 3 min</span><button type="button" class="button ghost">Annuler</button></div>
    <div class="bars"><div class="bar" role="progressbar" aria-label="Transcription" aria-valuemin="0" aria-valuemax="100" aria-valuenow="100"><i style="width:100%"></i></div><div class="bar" role="progressbar" aria-label="Rédaction du compte rendu" aria-valuemin="0" aria-valuemax="100" aria-valuenow="38"><i style="width:38%"></i></div></div>
  </div>
</div>"""),
    "Toast": ("Feedback", 90, """A toast says an action's outcome once, at the window's foot, and is gone after 5 s (`role=status`): the
text's colour on the surface's, so that it stands out in both themes. It pops in, unless the system asks for less
motion.
""", """<div style="padding:16px;background:var(--color-canvas)">
  <div class="toast" role="status" style="position:static">""" + CHECK + """<span>Copié</span></div>
</div>"""),
    "Tabs": ("Navigation", 140, """Tabs split a page into its parts, one shown at a time (WAI-ARIA's Tabs, automatic activation): drawn as
segments — the list a sunken band, each tab its words, the chosen one raised on the surface with e1. The chosen tab
is the only stop of Tab; ← → Home End move and choose; its panel comes next in Tab's order. Settings' five parts use
them (D123).
""", """<div style="padding:16px;background:var(--color-canvas)">
  <div class="tabs" role="tablist" aria-label="Réglages">
    <button type="button" class="tab" role="tab" aria-selected="true" tabindex="0">Enregistrement</button>
    <button type="button" class="tab" role="tab" aria-selected="false" tabindex="-1">Écoute et rédaction</button>
    <button type="button" class="tab" role="tab" aria-selected="false" tabindex="-1">Partage</button>
  </div>
  <div class="tab-panel" role="tabpanel"><p style="margin:0;color:var(--color-text-soft)">Testez votre micro et le son de la réunion avant d’enregistrer.</p></div>
</div>"""),
}

COVER = """<!-- @dsCard group="Kitokey" height=220 -->
<div style="display:grid;gap:8px;place-content:center;height:220px;background:var(--color-canvas);color:var(--color-text);font-family:var(--font-text)">
  <div style="font:700 32px var(--font-display)">Kitokey</div>
  <div style="color:var(--color-text-soft)">Design system — exported from the code</div>
  <div style="display:flex;gap:8px">
    <span style="width:24px;height:24px;border-radius:999px;background:var(--color-voice-you)"></span>
    <span style="width:24px;height:24px;border-radius:999px;background:var(--color-voice-others)"></span>
    <span style="width:24px;height:24px;border-radius:999px;background:var(--color-private)"></span>
    <span style="width:24px;height:24px;border-radius:999px;background:var(--color-record)"></span>
  </div>
</div>
"""


def change(sha: str, date: str) -> dict:
    """Who made the export: in CI, the run and who started it; on a workstation, the commit alone and nobody — a
    published file names no person."""
    note = "Exported from ui/tokens.css and ui/components.css by scripts/design-export.py."
    if os.environ.get("GITHUB_ACTIONS") == "true":
        return {"by": os.environ["GITHUB_ACTOR"], "at": date, "via": f"CI · NapkinStack/kitokey@{sha}", "note": note}
    return {"at": date, "via": f"local · {sha}", "note": note}


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="\n")


def main() -> None:
    css = (UI / "tokens.css").read_text(encoding="utf-8")
    sha = git("rev-parse", "HEAD")
    date = git("show", "-s", "--format=%cI", "HEAD")
    branch = git("rev-parse", "--abbrev-ref", "HEAD")
    read = tokens(css)
    if PROJECT.exists():
        shutil.rmtree(PROJECT)
    OUT.mkdir(exist_ok=True)
    shutil.copyfile(UI / "tokens.css", OUT / "tokens.css")
    write(OUT / "tokens.json", json.dumps(dtcg(read), indent=2, ensure_ascii=False) + "\n")
    write(OUT / "README.md", f"# design-system/\n\nKitokey's design system as `scripts/design-export.py` writes it from `ui/tokens.css` and `ui/components.css`, at commit `{sha}`. "
          "`tokens.css`: the code's own. `tokens.json`: the W3C Design Tokens format. `project/`: what Claude Design reads as it is. Never edited by hand.\n")
    write(PROJECT / "tokens.json", json.dumps(project_tokens(read, f"{branch}@{sha}"), indent=2, ensure_ascii=False) + "\n")
    write(PROJECT / "README.md", BOOK.format(sha=sha, date=date))
    (PROJECT / "fonts").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(UI / FONT, PROJECT / "fonts" / FONT)
    shutil.copyfile(UI / "BRICOLAGE-OFL.txt", PROJECT / "fonts" / "BRICOLAGE-OFL.txt")
    (PROJECT / "components").mkdir(parents=True, exist_ok=True)
    shutil.copyfile(UI / "components.css", PROJECT / "components" / "bundle.css")
    for name, (group, height, about, markup) in COMPONENTS.items():
        write(PROJECT / "components" / name / "README.md", f"# {name}\n\n{about}")
        write(PROJECT / "components" / name / "preview.html", f'<!-- @dsCard group="{group}" height={height} -->\n{markup}\n')
    write(PROJECT / "components" / "Cover" / "preview.html", COVER)
    index = {
        "v": 3, "layout": "files", "editing": "code", "createdOnFiles": {"v": 1, "at": date},
        "title": "Kitokey", "namespace": "Kitokey", "libraries": [], "sections": {}, "groups": [], "assetGroups": {},
        "blobs": {}, "docs": {"readme": "project/README.md", "sections": []},
        "lastChange": change(sha, date),
    }
    write(PROJECT / "design-system.json", json.dumps(index, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
