---
goal: "Kitokey is simple and beautiful: one design system in its code, and the screens a newcomer meets — the first run to their first summary, a meeting's page, the engine, Settings, the bar — recomposed on it, in words read without learning Kitokey's own"
status: accepted          # the decider's approval of this pull request accepts it (#1255)
appetite_weeks: 1         # the decider, 2026-11-26, relayed by the coordinator: one week at most, done well and fast
start: 2026-11-26         # cycle 11's early closure (#1257), on the decider's word of 2026-11-26
end: 2026-12-03           # start + appetite: the first day the cycle is over — the circuit breaker
deliverables:
  - id: D120
    title: "Kitokey's design system lives in its code — one source of tokens, its components, a gallery — and a test refuses any value outside it; an export carries it to Claude Design, the code staying the source"
    module: shell
    state: ready
    acceptance:
      - "Given `ui/tokens.css`, when the shell is built, then it is the only file that holds a colour, a font size, a radius, a shadow, a duration or a spacing — primitives, roles, and one dark block shared by `prefers-color-scheme` and `[data-theme=dark]` —; held by a test of `tests/look.rs` that fails on any hex, `rgb()`, `hsl()` or `color-mix()` outside it, and on any of those properties not read from a `var(--…)`, the literals `0`, `1px`, `2px`, `100%` and `50%` aside; its count of violations kept in a file it can only lower — a ratchet —, and zero at the cycle's closure"
      - "Given the roles of PDR-0008, when the tokens are read, then « Vous » is blue and « Les autres » amber, red is the record's and the destructive action's, violet with the lock « Confidentielle », green success only; each pair of text and ground reaches 4.5:1 and each control's edge 3:1 in both themes; held by `tests/look.rs`, its test `red_says_a_recording_and_nothing_else` rewritten to the two roles of red, never dropped"
      - "Given `ui/components.css`, which reads roles only, when the window's e2e opens the gallery (`#/gallery`), then every component of D120's section — among them the toast, the banner, the dialog, the option card, the progress and the engine card — shows in each of its states, in French and English, light and dark, captured at 420 and 1100 px; axe-core reports no violation of its WCAG 2.2 AA rules there; the shots are linked from the pull request"
      - "Given the five inline « copié » and « Enregistré » notes of today (`window.js`: the copies of the summary, the transcript and a line, the floating « Enregistré » of Settings), when their action succeeds, then a toast says it (`role=status`, gone after 5 s) and no inline note remains; held by the e2e"
      - "Given `scripts/design-export`, when it runs, then it writes `design-system/` — `README.md` with the principles and the commit it came from, `tokens.json` in the W3C Design Tokens format generated from `tokens.css`, `tokens.css`, a preview per component cut from the gallery —; and a check warns when the published copy's commit is not `HEAD`'s. Designs explored in Claude Design come back as a pull request on `tokens.css` or `components.css`, never as an edit of the export"
  - id: D121
    title: "A meeting's page is recomposed: its actions in words in one bar, « Partager » always there with the export to PDF and Markdown, « Refaire » a menu with its versions, the summary and the transcript as tabs, the rest folded in « Détails »"
    module: shell
    state: ready
    acceptance:
      - "Given a meeting's page, when it is drawn, then one bar of actions holds « Partager ▾ », « Agenda (n) » once a date is found, « Refaire ▾ » and « ⋯ », each with its words and its icon — no action of the bar is an icon alone, its accessible name beginning with its visible words (WCAG 2.5.3) —; the focus order is the sidebar, the page's header and its actions, the tabs, the content; held by the e2e and axe-core, in both languages and themes"
      - "Given a meeting with a summary, when « Partager » opens, then it offers its channels — with none, « Ajouter un canal », opening the channel's form in a sheet of the page, not Settings —, « Copier », « Exporter en PDF » and « Exporter en Markdown », which write the file through `exporting/v1` (D108, D109) where the user chooses; a confidential meeting's sends are asked again as PDR-0007 says; a send is said once (#1252); held by the e2e, the file's first bytes read"
      - "Given the redo controls of today — « Refaire avec X », « Revenir à la version précédente (X) » and its explanation, on each card —, when « Refaire ▾ » opens, then it offers « Refaire… », opening a dialog with the engine card (D122), and « Versions », listing the kept ones; going back to a version says it in a toast with « Annuler »; « Refaire avec » never names the engine that already made the version shown; held by the e2e"
      - "Given a summary or a transcript with a kept version (D113), when « Versions » opens, then each version shows with its line of provenance — when it was made, its engine and its model, how long it took, its cost when known (D93, D97) —, and can be seen, made current, or deleted after a confirmation, « Annuler » focused, its button destructive (PDR-0008); the current one cannot be deleted; held by the e2e, the files on disk read after each gesture"
      - "Given a window narrower than 1400 px, when the page is drawn, then the summary and the transcript are two tabs — « Compte rendu », « Transcription », WAI-ARIA's tabs pattern —; at 1400 px and more, side by side; each card keeps its line of provenance — when, engine and model, how long, cost when known (D93, D97) —; the context, the spoken language and the folder live in « Détails de la réunion », folded; « Confidentielle » is a pill of the header, violet with the lock (PDR-0008); the player shows only when its audio plays; a failure is said in one banner above the tabs; at 420 px the player and the tabs are in the first screen; held by the e2e at 420, 1100 and 1920 px"
      - "Given « Mettre à la corbeille » in « ⋯ », when it is chosen, then a dialog asks « Mettre cette réunion à la corbeille ? », says what goes and that it can be restored, « Annuler » focused, its confirm button destructive (PDR-0008); held by the e2e, Escape keeping the meeting"
  - id: D122
    title: "One engine card, the same in the first run, in Settings and in « Refaire », « Auto » by default"
    module: shell
    state: ready
    acceptance:
      - "Given the first run, Settings and the « Refaire » dialog, when each offers an engine for transcribing or writing, then each draws D120's engine card — its name in words, where it runs (« Sur cet ordinateur » or « En ligne »), its state (« Prêt », « À télécharger · 874 Mo », « Clé manquante ») —, the engine's id, address and size behind « Détails »; held by the e2e reading one structure in the three places"
      - "Given a new install, when no engine was chosen, then each job is on « Auto », the card saying which engine « Auto » is; a choice in « Refaire » applies to that meeting only, « Par défaut » marked; for each meeting the engine and its model are chosen apart, for the transcription and for the summary separately (D96), the card's « Détails » offering the model; the compact bar chooses no engine; held by the e2e and the shell's tests"
  - id: D123
    title: "Settings grouped in a few tabs, everything technical folded away, and an « À propos »"
    module: shell
    state: ready
    acceptance:
      - "Given Settings, when they open, then their sections are tabs — « Enregistrement », « Écoute et rédaction », « Partage », « Stockage et confidentialité », « Général » —, WAI-ARIA's tabs pattern; each setting of today found in one tab, listed in D123's section with its tab; the channels moved to « Partage »; « Général » ends on « À propos » — the version, the licence, the credits, the updates —; held by the e2e"
      - "Given each tab at rest, when its text is read, then it holds no address, port, path, engine id or byte count: those live under « Avancé », folded in each tab, and the developer mode's under it; at 420 px a banner stays inside the pane, no label is said twice (« Modèle » over « Modèle de transcription »), no button holds an emoji; D95's measure holds at 1920, 840, 600 and 420 px; held by the e2e"
  - id: D124
    title: "The window's words rewritten to one voice: no raw data, no contradiction, one word per concept"
    module: shell
    state: ready
    acceptance:
      - "Given `modules/shell/docs/voice.md`, when it is read, then it carries the voice of `audit-2026-11-26/messages.md` §2 — rules V1 to V10 —, its message patterns (§3) and its terminology (§5): « moteur », « format », « canal », « confidentielle »; no « — » in its rules or its templates, « modèle » nowhere for an engine or a format"
      - "Given every sentence of `src/words.rs`, in both languages, when `tests/words.rs` reads them, then none holds a host, a port, a path, a raw engine id or a count of bytes under 1 Mo, none holds « — », and « modèle » / « model » names neither an engine nor a format (PDR-0008); its exceptions named one by one, each under « Avancé », « Détails » or the developer mode"
      - "Given the contradictions the audit found, when the words are read with the product, then each is settled: a confidential meeting's send says what D119 does — asked once more, then sent —, never « jamais envoyée »; « trop long » offers « Exporter en PDF », which D121 builds, not an export that does not exist; making room says one thing — what goes to the trash and can come back —, never both « rien n'est conservé » and « rien n'est supprimé »; held by a test of `tests/words.rs` per contradiction"
      - "Given the 40 rewrites of `messages.md` §4 and the 10 of `ux-ia.md` §4, when the window is read, then each is in place, or its replacement and its reason are in D124's section; every error says what happened, what is safe, then one action — no path, no shortcut hint in an error; monospace only on a timecode, a key or an id under « Avancé »; held by the e2e reading the surfaces of D121 to D127"
  - id: D125
    title: "The compact bar simplified"
    module: shell
    state: ready
    acceptance:
      - "Given the bar at rest, when it is drawn, then it holds the context's field (« De quoi parle l'appel ? Qui est là ? »), record and expand; while it records, the time, the two voices' meters (blue, amber), stop, and one disclosure for the context, « Confidentielle » (the same switch as the page's) and the microphone — no engine, no language; a failure is said in two lines then « Ouvrir », never cut in a word; no focus ring shows until the keyboard is used; held by the e2e, axe-core and its shots, in both languages and themes"
  - id: D126
    title: "A newcomer reaches their first summary in three screens: a choice, a check while it downloads, « Prêt », and an example meeting to try"
    module: shell
    state: ready
    acceptance:
      - "Given a first run, when it starts, then it is three screens: (1) two option cards, « Sur cet ordinateur » and « En ligne », « Je ne sais pas » choosing « Sur cet ordinateur », the engine's download starting at once in the background; (2) the sound checked inline, the two meters moving, a progress under it saying what is prepared, its size and the time left; (3) « Prêt. Enregistrez votre premier appel », the record button and the shortcut, and « Essayer sur un exemple »; held by the e2e, each screen shot in both languages and themes"
      - "Given a new install, when its settings are first written, then « Après un enregistrement » is « Transcrire, puis rédiger le compte rendu », so that the first meeting has its summary without a click; a settings file that already says another choice keeps it; held by `settings.rs`'s tests"
      - "Given « Essayer sur un exemple », when it is chosen, then a two-voice clip of about 45 s bundled with Kitokey, invented for it, runs through the real transcription and summary and opens as a meeting marked « Exemple », deletable as any; held by the e2e with its engines' stand-ins, its size recorded against D102's package size"
  - id: D127
    title: "Finding a meeting by what was said: a search in the summaries and the transcripts, its passages shown, a result opening the meeting at its line"
    module: shell
    state: ready
    acceptance:
      - "Given meetings whose summaries and transcripts hold a word, when it is typed in the search (the sidebar's, or Ctrl+K once D128 is merged), then the results are grouped — « Titres », « Comptes rendus », « Transcriptions » —, each with its passage, the word marked, and the transcript's with its time; Enter opens the meeting scrolled to that line, the player cued there; held by the e2e"
      - "Given 500 meetings of an hour, made for the test, when a word is searched, then the first results show within the budget D105 sets for an interaction, measured on the release package; the method — a scan of the files or an index — chosen by that measure and written in D127's section"
      - "Given a meeting without a title of its own, when its summary exists, then its row and its page take a title from the summary's first highlight, marked as Kitokey's, the date as its subtitle; the user's own title wins; held by the e2e"
  - id: D128
    title: "Kitokey by the keyboard: a set of shortcuts, a sheet that lists them, and a command palette"
    module: shell
    state: ready
    acceptance:
      - "Given the window, when its shortcuts are pressed, then Ctrl+R records or stops, Ctrl+K opens the palette, Ctrl+F searches in the page, Ctrl+, opens Settings, Ctrl+Shift+C copies the summary, Esc goes back, J and K go to the next and previous meeting, « ? » opens the sheet that lists them; each shown in its menu entry and its tooltip; none taken while a field has the focus; held by the e2e, each shortcut pressed"
      - "Given the palette (Ctrl+K), when a word is typed, then it lists the actions that match — Enregistrer, Partager vers…, Refaire, Exporter, a section of Settings — and the meetings D127 finds, moved through by the arrows, Enter acting; it follows WAI-ARIA's combobox pattern; held by the e2e and axe-core"
  - id: D129
    title: "The summary can be written in a format the user made: `summarising/v1` expanded to take a format's sections, the built-in six unchanged"
    module: summary
    state: ready
    acceptance:
      - "Given a format the user made — its name, its sections, each an id and what it holds, the one that holds the actions named —, when `kitokey-summary` is given it, then it writes `minutes/v5` in that format; held by the module's tests on two formats made for them"
      - "Given `summarising/v1` expanded by its expand step (03-contracts.md), when the six built-in structures or no structure are asked, then they work as before, and a format it cannot read is a wrong argument with nothing written; held by the contract's tests and the comparator (ADR-0005) on its schema"
  - id: D130
    title: "Formats are managed: made, duplicated, renamed, changed, deleted — the built-in ones hidden only —, one the default, one chosen in « Refaire »"
    module: shell
    state: ready
    acceptance:
      - "Given « Écoute et rédaction › Formats » in Settings (D123), when the user manages them, then a format can be created, duplicated, renamed, changed — its sections, each a title and what it holds, one holding the actions — and deleted after a confirmation (PDR-0008); a built-in format can be hidden and shown again, never changed or deleted; one is the default; the formats are kept in the user's settings folder, never sent anywhere; held by the e2e and the shell's tests"
      - "Given « Refaire… » on a summary (D121), when its dialog opens, then it offers the formats — the default marked —, a choice applying to that meeting; the summary is written in the format chosen through D129's `summarising/v1`; the summary's line of provenance names its format; held by the e2e with the summary's stand-in"
---

# Cycle 12 — Kitokey, simple and beautiful

> The framework's words: `.nstack/handbook/glossary.md`.

> Copied to `docs/project/cycles/NN-<slug>.md` at framing (`.nstack/playbooks/framing.md`). One accepted cycle at a
> time; no automatic extension.

## Goal

The decider said **go** on 2026-11-26 for a cycle « Kitokey simple et beau », design first: what exists is
recomposed before anything new is added. Two independent passes of that day, in
`kitokey-team/audit-2026-11-26/`, are its source: the first — `ux-ia.md`, `design-system.md`, `benchmark.md` —
found 15 looks of button, 17 font sizes, green meaning four things, an engine chosen in four places with four
vocabularies; the second — `screens-and-delight.md`, `messages.md` — found that the search finds titles only,
that the first meeting gets no summary without a click, that the private path needs a program installed by
hand, and that some sentences contradict the product. The decider's choices of colour and words are
[PDR-0008](../../pdr/0008-kitokey-speaks-the-colours-and-words-users-already-know.md).

```mermaid
flowchart LR
    D120["D120 · the design system<br/>tokens · components · gallery<br/>the test that refuses the rest"]:::first --> D126["D126 · the first run"]
    D120 --> D121["D121 · a meeting's page"]
    D120 --> D122["D122 · one engine card"]
    D122 --> D123["D123 · Settings in tabs"]
    D122 --> D126
    D120 --> D125["D125 · the compact bar"]
    D121 --> D127["D127 · the search"]
    D127 --> D128["D128 · keyboard and palette"]
    D124["D124 · the words"]:::across -.- D121 & D123 & D125 & D126 & D127
    classDef first fill:#1e3a8a,color:#fff
    classDef across fill:#374151,color:#fff
```

**Legend** — blue: first, the others read its tokens and components; arrows: what a deliverable waits on;
grey, dotted: the words, rewritten with each screen and checked across them.

**One week, done well and fast** (the decider, 2026-11-26): only the backlog's P1 items are criteria; its P2
and P3 items are in *Later*. **D127, D128 and D130 are P2**: in the cycle, built only if the time remains once
D120 to D126 and D129 are merged, deferred at the closure otherwise — never stretching the end.

**Added by the decider, 2026-11-26** (relayed by the coordinator): the kept versions managed (D121, P1), and the
formats managed (D129, D130, P1). A format the user makes changes a contract — `summarising/v1` takes only its six
built-in structures (D111) — so it is two deliverables in two modules, the contract's expand step first, never in
one pull request (kernel §4). The decider keeps D125 (the compact bar) in the cycle, P1. To fit the week,
**D130 slips to P2** — D129, the contract's expand step, stays P1 so that the formats can be built as soon as
time allows, this cycle or the next —; D127 and D128 stay P2. *(framer, to confirm.)*

**How the work moves** — D120 first and blocking; then five in parallel; one pull request per screen, within
**800 lines**, with its screenshots before and after, attested right after its sheet is signed:

| Day | Stream | Who *(framer, proposed)* | Files mostly its own |
|---|---|---|---|
| 1–2 | **D120** — `tokens.css` with the old names as aliases (no visible change), then the button, banner, toast, dialog, option card, progress, engine card; the gallery; the test in report mode | lina | `ui/tokens.css`, `ui/components.css`, `ui/gallery.html`, `tests/look.rs` |
| 2–6 | **D121** — a meeting's page | lina | the page's part of `window.js`, `window.css`'s layout |
| 2–6 | **D126** — the first run, the example meeting, the default « rédiger » | ines | the first run's part of `window.js`, `settings.rs`, the clip under `ui/` |
| 2–5 | **D122** then **D123** — the engine card wired in Settings, the first run and « Refaire »; Settings in tabs | sam | Settings' part of `window.js`, `src/main.rs`'s offer of engines |
| 5–6 | **D125** — the compact bar, its engine choice gone with D122 | sam | the bar's part of `window.js` |
| 2–4 | **D124** — `docs/voice.md`, `src/words.rs`, `tests/words.rs` | marc | `words.rs`, `voice.md`, `tests/words.rs` |
| 4–6 | **D129** — `summarising/v1` expanded, the summary module writing a format it is given | marc | `contracts/summarising/`, `modules/summary/` |
| 6–7 | **D127**, **D128**, **D130** — only if the time remains, D130 first | the first stream done | `window.js`'s search, a palette file, Settings' formats in a file of its own |

New code of a screen goes in a file of its own under `ui/` where it can — the first run's, the palette's —, to
keep the streams off each other's lines in `window.js`. The verifiers rotate: no one verifies their own stream,
and a verifier signs the screen of a stream they do not touch that day.

**The critical path**: D120's tokens and components (two days) → D121, the largest screen (three to four days)
→ its attest and merge, serial on the shell → the closure on day 7. A second path: D122 → D123 → D125 for sam,
who holds three screens. **The risk of the week**: the shell's attests, serial on one machine, five screens
asking in the last two days; each is attested as soon as its sheet is signed. D130 waits on D129's merge. D120 slipping a day takes it from every
stream; its first pull request — the tokens alone, no visible change — merges first so that the others start.
The shell's attests run one at a time on the machine: each pull request is attested as soon as it is signed,
never batched at the end.

**The backlog of the second pass** (`screens-and-delight.md` §7, 24 items): its P1 items are the criteria
below; P2 and P3 are in *Later*, D127 and D128 aside.

## Deliverables

### D120

The source is `design-system.md` §3 to §6, amended by PDR-0008 — red is also the destructive action's, with a
confirmation —, and the component list of `screens-and-delight.md` §3: the 15 looks of button become one
button's variants; inline notes become the toast; the error lines become the banner, one slot per card and one
per page; the radio cards become the option card; the inline trash card becomes the dialog; plus the engine
card (D122's), the progress (two segments with an estimate and « Annuler »), and the empty state. The values —
type scale, 4 px grid, radii, elevations — are *(framer, to confirm)* in D120's first pull request with the
gallery's shots. The blue and amber pair is said colour-blind-safe by the audit without a primary source: D120's
first pull request checks it with a simulator of the three deficiencies and keeps the shots.

Order: (1) `tokens.css`, the old names kept as aliases — no visible change, the shots identical; (2) the
button, the banner, the toast, the dialog, the option card, the progress; (3) the gallery and the export.


**Claude Design**: `scripts/design-export` writes the folder; publishing it as the « Design System » artifact
is a gesture (*What needs a person*). The code stays the source; the artifact is never edited.

### D121

`ux-ia.md` §3, `screens-and-delight.md` §2 B (record, then summary, then share: three clicks) and §5 (density):
the default view is what one acts on, the rest one click away. The export is `exporting/v1`'s, which the shell
consumes already (D109): no contract changes. « Ajouter un canal » opens the same form as Settings, in a sheet.


### D122

`ux-ia.md` §2, *one model, two jobs*: transcribing turns the sound into text, writing turns the text into a
summary, each done by a **moteur** (PDR-0008), on this computer or online. « Auto » follows Granola, whose help
tells its users to leave it on
([Granola](https://docs.granola.ai/help-center/getting-more-from-your-notes/understanding-model-selection-in-granola-chat)).


### D123

`ux-ia.md` §2, its five tabs; `screens-and-delight.md` §5.4, three depths — the tab, « Avancé », the developer
mode (D29). D123's first pull request lists every setting of today with its tab.


### D124

`messages.md` throughout, and `ux-ia.md` §4. Each screen's pull request rewrites its own words; D124 holds
the voice guide, the tests across them, and the three contradictions, and closes once every surface's text
passes. Where a message should become UI (`messages.md` §6) — a badge, a progress, a lock pill —, the screen's
deliverable builds it.


### D125

`design-system.md` §4, *Compact bar*; `screens-and-delight.md` §5.7: a recording offers no engine and no
language.

### D126

`screens-and-delight.md` §2 A and fact 2: today's first run ends on an empty list, and the first meeting is
transcribed but not summarised. The example meeting follows Granola's demo note and Fathom's test call
(`benchmark.md` §1). Its clip is invented for Kitokey — two voices, French and English — with nothing of a real
meeting.

**À décider (the decider, by an ADR)**: how the summary is written on this computer without installing Ollama
by hand — (a) Kitokey bundles or installs the engine's runtime, llama.cpp embedded or Ollama managed by Kitokey
(MacWhisper's and LM Studio's convention); (b) the first run says plainly that the private summary needs Ollama
and about 5 Go, and offers the online path for the first try. Until the decider chooses, D126's first screen
says (b)'s sentence for the private path; nothing of (a) is built in this cycle without the ADR accepted.

### D127

`screens-and-delight.md` §2 C and fact 1: the search finds titles only, and untitled meetings are named by
their date, so it finds dates. The title from the summary is #1240's, moved here: it is what makes the list
readable.

### D128

`screens-and-delight.md` §4, *Shortcuts* and *Command palette*. The global shortcut, which the desktop grants, stays as it is.

## What needs a person

| Deliverable | Who | The literal gesture |
|---|---|---|
| The cycle | The decider | Read this file and PDR-0008 from their pull request; approve it, or say what changes |
| D126 | The decider | Read the ADR « a private summary without installing Ollama by hand » from its pull request; say (a) or (b), or what changes |
| D120 | The decider | Read the gallery's shots from D120's pull request; say « charte ok », or what changes |
| D120 | An agent | Once D120 is merged: publish `design-system/` as the « Design System » artifact, updated in place at each release, its commit in its README |
| D121, D123, D126 | The decider | Read the before and after shots of each screen's pull request; say « ok », or what changes |
| D126 | The decider | On the package an agent built, run the first run from a new account: say how long it took to the first summary, on the example |

## Out of scope

- New features beyond these: importing a file, Outlook, speaker names by passage — *Later*.
- The site: it follows the app's tokens once D120 is merged, in a cycle of its own.
- Any brand beyond `ui/brands/BRANDS.md` (#1238).

## Later

- **Importing an audio or video file**, « Nouveau ▾ » and a file dropped on the window — `benchmark.md` §5, backlog item 23.
- **Outlook** in « Agenda » — cycle 11's D115 made its link; its trial waits on a colleague.
- **Names by passage** — #1230's S2 (the contract step of `transcript/v1`) and S3 (a service's speakers).
- **Quiet sounds**, opt-in, for starting, stopping and ready — backlog item 22 (P3).
- **The P2 items of the backlog** (`screens-and-delight.md` §7): the empty states' illustrations and the micro-interactions (3, 4); empty sections of a summary hidden and the speaker said only when it changes (8) — #1234's « Rien de noté » stays until then —; the first summary's celebration (9); an estimate from the first meeting (11); the trial in the sidebar and « Acheter » when over (13); no button label over 28 characters (16).
- « Dates mentionnées »'s count following the panel's day (marc, #1249).
- The first run's aurora (`design-system.md` §5).

## Open questions

**Blocking:** none — the appetite is one week (the decider, 2026-11-26).

**À décider (by an ADR, before D126 builds the private path):**

- A private summary without installing Ollama by hand: (a) bundled or managed by Kitokey, or (b) said plainly (D126).

**Not blocking:**

- What « Auto » picks when no engine is ready on the computer (D122): the first run's answer, or nothing until one is ready?
- The five tabs of Settings (D123): kept as the audit names them, or fewer?
- The site's display face (Bricolage Grotesque, OFL) for titles in the app (D120): yes or no?
- A settings file of today that says « Transcrire » (D126): kept as the user's choice — the assumption here —, or moved to « rédiger »?

## Closure

<Written at the end.>

**Documents brought current:** <at the closure>
