# Kitokey — design system

Exported from the code by `modules/shell/scripts/design-export.py`, from commit `c3d7b1d16fecf66680311b812ad297146ada82d7` (2026-11-27T20:08:45+01:00). The code is the
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
