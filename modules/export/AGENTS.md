# export — local instructions

> Only what is **specific to this module**.
> Never duplicate a kernel rule (`/AGENTS.md`): that is wasted context and a source of
> divergence.
>
> Indicative budget: 100 lines.

## Responsibility

Delivers the files of a session to where the user wants them.

## What this module does not do

- It owns no sentence. A heading or a speaker's name a file shows comes from the caller, the
  shell's `words.rs` (`Labels`).
- It makes nothing: no transcript, no summary. It reads what `transcription` and `summary` wrote.

## Internal conventions nobody could guess

<Naming, layout, choices that would surprise an outsider.>

## Business invariants

- It reads the contracts alone — `session-audio`, `minutes` v1 to v4, `transcript/v1` —, never
  another module's code.
- A file absent is a part absent, never an error: a meeting without minutes still writes.
- The summary's sections are a list read by their place, whatever their number: v5's named
  sections add to it and break nothing (`tests/markdown.rs`).
- A transcript's text is written as said: every ASCII punctuation escaped, so that no reader
  takes it for markup and a `|` stays in its cell.
- A model's raw HTML in the summary is said as text in a page, never run (`tests/html.rs`).

## Known traps

<What has already broken here. Every entry should become a test - see
.nstack/handbook/10-measurement.md §4.>

## Areas not to change without approval

<What, and why.>

## Non-standard commands

<Only what departs from the MANIFEST verbs.>
