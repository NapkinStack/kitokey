---
goal: "Kitokey is simple and beautiful: one design system in its code, and the screens a buyer uses most — a meeting's page, the engine, Settings, the bar — recomposed on it, in words a newcomer reads without learning Kitokey's own"
status: proposed          # proposed | accepted | closed | stopped — the decider accepts
appetite_weeks: 3         # (framer, to confirm) — the decider says how many weeks they WANT to spend (Open questions)
start: 2026-12-04         # cycle 11's end; moves to cycle 11's closure when it comes first (framing.md §3)
end: 2026-12-25           # start + appetite: the first day the cycle is over — the circuit breaker
deliverables:
  - id: D120
    title: "Kitokey's design system lives in its code — one source of tokens, its components, a gallery — and a test refuses any value outside it; an export carries it to Claude Design, the code staying the source"
    module: shell
    state: ready
    acceptance:
      - "Given `ui/tokens.css`, when the shell is built, then it is the only file that holds a colour, a font size, a radius, a shadow, a duration or a spacing — primitives, roles, and one dark block shared by `prefers-color-scheme` and `[data-theme=dark]` —; held by a test of `tests/look.rs` that fails on any hex, `rgb()`, `hsl()` or `color-mix()` outside it, and on any of those properties not read from a `var(--…)`, the literals `0`, `1px`, `2px`, `100%` and `50%` aside; its count of violations kept in a file it can only lower — a ratchet —, and zero at the cycle's closure"
      - "Given the roles of PDR-0008, when the tokens are read, then « Vous » is blue and « Les autres » amber, red is the record's and the destructive action's, violet with the lock « Confidentielle », green success only; each pair of text and ground reaches 4.5:1 and each control's edge 3:1 in both themes; held by `tests/look.rs`, its test `red_says_a_recording_and_nothing_else` rewritten to the two roles of red, never dropped"
      - "Given `ui/components.css`, which reads roles only, when the window's e2e opens the gallery (`#/gallery`), then every component — button in its variants, field, list, segmented control, switch, card, row, chip, badge, banner, tabs, menu, dialog, toast, empty state — shows in each of its states, in French and English, light and dark, captured at 420 and 1100 px; axe-core reports no violation of its WCAG 2.2 AA rules there; the shots are linked from the pull request"
      - "Given `scripts/design-export`, when it runs, then it writes `design-system/` — `README.md` with the principles and the commit it came from, `tokens.json` in the W3C Design Tokens format generated from `tokens.css`, `tokens.css`, a preview per component cut from the gallery —; and a check warns when the published copy's commit is not `HEAD`'s. Designs explored in Claude Design come back as a pull request on `tokens.css` or `components.css`, never as an edit of the export"
  - id: D121
    title: "A meeting's page is recomposed: its actions in words in one bar, « Partager » always there with the export to PDF and Markdown, « Agenda (n) », the summary and the transcript as tabs, « Confidentielle » a pill"
    module: shell
    state: ready
    acceptance:
      - "Given a meeting's page, when it is drawn, then one bar of actions holds « Partager ▾ », « Agenda (n) » once a date is found, « Refaire ▾ » and « ⋯ », each with its words and its icon — no action of the bar is an icon alone, its accessible name beginning with its visible words (WCAG 2.5.3) —; held by the e2e and axe-core, in both languages and themes"
      - "Given a meeting with a summary, when « Partager » opens, then it offers its channels — with none, « Ajouter Slack, Discord… », opening the channel's form in a sheet of the page, not Settings —, « Copier », and « Exporter en PDF » and « Exporter en Markdown », which write the file through `exporting/v1` (D108, D109) where the user chooses; a confidential meeting's sends are asked again as PDR-0007 says; held by the e2e, the file's first bytes read"
      - "Given a window narrower than 1400 px, when the page is drawn, then the summary and the transcript are two tabs — « Compte rendu », « Transcription », WAI-ARIA's tabs pattern, the arrows moving between them —; at 1400 px and more, side by side; « Confidentielle » a pill of the page's header, violet with the lock (PDR-0008), that turns it on and off; a failure or a redo that failed said in one banner above the tabs, never between versions; held by the e2e at 420, 1100 and 1920 px"
      - "Given « Mettre à la corbeille » in « ⋯ », when it is chosen, then a confirmation opens, « Annuler » focused, its button destructive (PDR-0008); held by the e2e, Escape keeping the meeting"
  - id: D122
    title: "One engine card, the same in the first run, in Settings and in « Refaire », « Auto » by default"
    module: shell
    state: ready
    acceptance:
      - "Given the first run, Settings and « Refaire », when each offers an engine for transcribing or writing, then each draws the same component — its name in words, where it runs (« Sur cet ordinateur » or « Service en ligne »), its state (« Prêt », « À télécharger · 874 Mo », « Clé manquante ») —, the engine's id, address and size behind « Détails »; held by the e2e reading one structure in the three places"
      - "Given a new install, when no engine was chosen, then each job is on « Auto » — the engine this computer has ready, else the one the first run's answer set — and the card says which engine « Auto » is; a choice in « Refaire » applies to that meeting only, « Par défaut » marked; the compact bar chooses no engine; held by the e2e and the shell's tests"
  - id: D123
    title: "Settings grouped in a few tabs, everything technical folded away"
    module: shell
    state: ready
    acceptance:
      - "Given Settings, when they open, then their sections are tabs — « Enregistrement », « Écoute et rédaction », « Partage », « Stockage et confidentialité », « Général » *(framer, to confirm: the audit's five)* —, WAI-ARIA's tabs pattern; each setting of today found in one tab, listed in D123's section with its tab; the channels moved from « Après un enregistrement » to « Partage »; held by the e2e"
      - "Given each tab at rest, when its text is read, then it holds no address, port, path, engine id or byte count: those live under « Avancé », folded; D95's measure holds at 1920, 840, 600 and 420 px; held by the e2e"
  - id: D124
    title: "The window's words rewritten: no raw data in front of the user"
    module: shell
    state: ready
    acceptance:
      - "Given every sentence of `src/words.rs`, in both languages, when `tests/words.rs` reads them, then none holds a host, a port, a path, a raw engine id or a count of bytes, and « modèle » / « model » names neither an engine nor a format (PDR-0008); its exceptions named one by one, each under « Avancé » or « Détails »"
      - "Given the surfaces of D121 to D123 and the list, when the e2e reads their text and styles, then monospace appears only on a timecode, a key or an id under « Avancé »; the audit's ten sentences (§4 of `ux-ia.md`) are each replaced, their before and after in D124's section; every error says what happened, then one action"
  - id: D125
    title: "The compact bar simplified"
    module: shell
    state: ready
    acceptance:
      - "Given the bar, when it is drawn at rest, then it holds the mark, the time, the two voices' meters (blue, amber), record or stop, and three icon buttons named in words for assistive technology; its field says what it is for (« De quoi parle l'appel ? Qui est là ? »); its options choose no engine; no focus ring shows until the keyboard is used; held by the e2e, axe-core and its shots, in both languages and themes"
---

# Cycle 12 — Kitokey, simple and beautiful

> The framework's words: `.nstack/handbook/glossary.md`.

> Copied to `docs/project/cycles/NN-<slug>.md` at framing (`.nstack/playbooks/framing.md`). One accepted cycle at a
> time; no automatic extension.

## Goal

The decider said **go** on 2026-11-26 for a cycle « Kitokey simple et beau », design first: what exists is
recomposed before anything new is added. An independent audit of that day
(`kitokey-team/audit-2026-11-26/`: `ux-ia.md`, `design-system.md`, `benchmark.md`) found 15 looks of button,
17 font sizes, 23 spacings, green meaning four things, an engine chosen in four places with four vocabularies,
and internals — hosts, ports, paths, ids — in calls to action. The decider's choices of colour and words are
[PDR-0008](../../pdr/0008-kitokey-speaks-the-colours-and-words-users-already-know.md).

```mermaid
flowchart LR
    D120["D120 · the design system<br/>tokens · components · gallery<br/>the test that refuses the rest"]:::first --> D121["D121 · a meeting's page"]
    D120 --> D122["D122 · one engine card"]
    D120 --> D123["D123 · Settings in tabs"]
    D120 --> D125["D125 · the compact bar"]
    D122 --> D123
    D124["D124 · the words"]:::across -.- D121 & D122 & D123 & D125
    classDef first fill:#1e3a8a,color:#fff
    classDef across fill:#374151,color:#fff
```

**Legend** — blue: first, the others read its tokens and components; arrows: what a deliverable waits on;
grey, dotted: the words, rewritten with each screen and checked across them.

**How the work moves**: screen by screen, each pull request within **800 lines**, each with its screenshots
before and after. The design system's test starts in « report » mode — a count it can only lower — and is
strict at the closure.

## Deliverables

### D120

The source is the audit's `design-system.md` §3 to §6, amended by PDR-0008: red is also the destructive
action's, with a confirmation (the audit had kept red for recording alone). The values the audit proposes —
the 7-step type scale, the 4 px grid, the six radii, the three elevations — are *(framer, to confirm)* in
D120's first pull request, with the gallery's shots; the roles are PDR-0008's. The blue and amber pair is said
colour-blind-safe by the audit without a primary source: D120's first pull request checks it with a simulator
of the three deficiencies and keeps the shots.

Order: (1) `tokens.css` and the old names kept as aliases — no visible change, the shots identical; (2) the
buttons and controls — 15 looks become the button's variants; (3) the gallery and the export. The screens are
D121 to D125's.

**Claude Design**: `scripts/design-export` writes the folder; publishing it as the « Design System » artifact
is a gesture (*What needs a person*). The code stays the source; the artifact is never edited.

### D121

The audit's `ux-ia.md` §3. The export to PDF and Markdown is `exporting/v1`'s, which the shell already consumes
(D109): no contract changes. « Partager » with no channel opens the same form Settings has, in a sheet.

### D122

The audit's `ux-ia.md` §2, *one model, two jobs*: « Écouter » turns the sound into text, « Rédiger » turns the
text into a summary; each done by a **moteur** (PDR-0008), on this computer or by a service. « Auto » follows
Granola, whose help tells its users to leave it on
([Granola](https://docs.granola.ai/help-center/getting-more-from-your-notes/understanding-model-selection-in-granola-chat)).
What « Auto » picks when nothing is ready is an open question.

### D123

The audit's `ux-ia.md` §2, its five tabs. D123's first pull request lists every setting of today with its tab.

### D124

The audit's `ux-ia.md` §4: its six rules and its ten sentences. Each screen's pull request rewrites its own
words; D124 holds the test across them, and closes once every surface's text passes.

### D125

The audit's `design-system.md` §4, *Compact bar*, and `ux-ia.md` §1: a recording offers no engine.

## What needs a person

| Deliverable | Who | The literal gesture |
|---|---|---|
| The cycle | The decider | Read this file and PDR-0008 from their pull request; say the appetite in weeks; approve the pull request, or say what changes |
| D120 | The decider | Read the gallery's shots from D120's pull request; say « charte ok », or what changes |
| D120 | An agent | Once D120 is merged: publish `design-system/` as the « Design System » artifact, updated in place at each release, its commit in its README |
| D121, D123 | The decider | Read the before and after shots of each screen's pull request; say « ok », or what changes |

## Out of scope

- New features: summary formats, importing a file, Outlook, speaker names by passage (#1230's S2 and S3) — *Later*.
- The site: it follows the app's tokens once D120 is merged, in a cycle of its own.
- Any brand beyond `ui/brands/BRANDS.md` (#1238).

## Later

- **Formats** (templates) of a summary — the audit's `ux-ia.md` §5 and `benchmark.md` §3; « format », never
  « modèle » (PDR-0008).
- **Importing an audio or video file** — `benchmark.md` §5.
- **Outlook** in « Agenda » — cycle 11's D115 made its link; its trial waits on a colleague.
- **Names by passage** — #1230's S2 (the contract step of `transcript/v1`) and S3 (a service's speakers).
- Auto-titles from the summary (#1240), « Dates mentionnées »'s count following the panel's day (marc, #1249).
- Illustrations of the empty states and the first run's aurora (`design-system.md` §5).

## Open questions

**Blocking:**

- **The appetite**: how many weeks does the decider want to spend? Proposed: three *(framer, to confirm)*.

**Not blocking:**

- What « Auto » picks when no engine is ready on the computer and none was chosen (D122): the first run's
  answer, or nothing until one is ready?
- The five tabs of Settings (D123): kept as the audit names them, or fewer?
- The display face of the audit (Bricolage Grotesque, OFL, the site's) for titles in the app (D120): yes or no?

## Closure

<Written at the end.>

**Documents brought current:** <at the closure>
