# shell — local instructions

> Only what is **specific to this module**.
> Never duplicate a kernel rule (`/AGENTS.md`): that is wasted context and a source of
> divergence.
>
> Indicative budget: 100 lines.

## Responsibility

Puts Kitokey's four steps in the user's hands: a tray, a global shortcut, notifications, one window.

## What this module does not do

- It records, hears and writes nothing itself: each step is its module's program, started here. A feature that needs a step's code belongs to that step.
- It decides nothing about a session's files: it reads them as their contracts describe them.

## Internal conventions nobody could guess

- Every sentence the user reads is in `src/words.rs`, in each language the interface speaks — English and French, following the system's (the decider, 2026-10-13). They are the product's content: the one place where this repository holds another language than English. A sentence written anywhere else is a defect. They are written to the voice guide, [`docs/voice.md`](docs/voice.md): its principles, its lexicon, its templates and the comment each sentence carries.
- The Whisper models Settings offer are `registry/models.json`, built in (`src/registry.rs`, D40, ADR-0010): a model is added or removed there, never in the code. Each says what it is good for by a key whose sentence is in `src/words.rs`; a new key needs its sentence first. The shell's tests read the file: one that does not read, or names a host other than `registry::HOSTS`, fails them. Its first model stays this build's default (`model::DEFAULT`): a list read again that puts another first is refused, the default being what a transcript waits for and the tray fetches. Read again, a list is taken only signed by the decider's key (`registry::PUBLIC_KEY`, D56): once that key is built in, a change to the file is followed by their `minisign -Sm`, or the shell's tests refuse it. `registry/.gitattributes` keeps the file's bytes as signed, on Windows too.
- Decided before any code (ADR-0002, cycle 01 *D1*): Tauri v2; under GNOME the global shortcut and the window's coming forward go through the desktop's portal, not the toolkit's plugin; the tray's menu holds the actions themselves, since it cannot bring a window forward there.

## Business invariants

- No window opens without a gesture of the user.
- A recording never stops because the shell did.
- Nothing crosses the network before the user was shown what and where, and agreed.
- A log line holds steps, ids, hosts, models and failures — never a word of a meeting, a key, nor
  a session's path beyond its folder's name (`tests/log.rs`, the e2e).

## Known traps

- `ashpd` is taken with its `async-io` feature, never `tokio`: with `tokio` the portal's library switches the desktop bus to that runtime, and every notification then panics inside the toolkit's own runtime ("cannot start a runtime from within a runtime").
- The microphones the small window lists are capture's answer (`recording/v1`, `microphones`, D74), never read from the sound server here; none, nothing is offered: that is what keeps macOS and Windows without a list, no check of the system's name. Whether the one chosen is there is what capture says of it, `microphone_gone` and `microphone_back`, never a description compared: two may share one.
- A notification is shown from the main thread or a plain thread, never from a task of the async runtime.
- The tray's icon is the mark on its tile — blue at rest, red while recording (the decider, 2026-11-08) —, never a glyph alone: a light glyph was invisible on the decider's desktop, whose bar is light, and a red dot on the blue tile is under 3:1. A tray icon is judged on a light bar, a grey one and a dark one. It is drawn by `scripts/icons.sh` from `icons/tray*.svg`, never edited as a PNG.
- A desktop entry whose `Exec` program is not found is no entry at all for the desktop: the portal answers "App info not found" and the shortcut is refused.
- The header's name is Montserrat 700 outlined into `ui/index.html` by `scripts/name.py`, never edited by hand (D89): the font's SIL Open Font License lets its outlines be drawn into the page, no font shipped; Lucide's icons keep their ISC notice in `ui/LUCIDE-LICENSE`.
- The toolkit writes `gen/` at each build: it is ignored. Files are added to a commit by name.
- A name declared twice in one function of `ui/window.js` is a syntax error that leaves the whole window blank, with nothing in the shell's own logs: `check` runs `node --check` on it where Node is there.

## Areas not to change without approval

- None yet.

## Non-standard commands

- `KITOKEY_E2E_ONLY=<scenario>[,…] bash scripts/headless.sh`: the window's e2e scenarios named, by their function in `scripts/e2e_window.py`, played alone (`scripts/e2e_only.py`, as pytest's `-k`); the tray's and `main()`'s own are not. A red check is replayed so before a pull request is blamed for it. A name `main()` does not call is refused before anything is built or started; a blank value is a full run. Such a run says « partial run » first and last, and never exits 0: it is never the module's e2e.