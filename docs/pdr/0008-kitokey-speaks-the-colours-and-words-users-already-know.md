# PDR-0008 — Kitokey speaks the colours and the words its users already know

> The framework's words: `.nstack/handbook/glossary.md`.

- **Status**: Proposed — the decider's choices of 2026-11-26, relayed by the coordinator the same day, written
  here by lina (framer of cycle 12). The decider's approval of the pull request that carries it accepts it, the
  forge recording its day.
- **Date**: 2026-11-26
- **Decision makers**: the decider, `@napkinstack-admin`
- **Modules affected**: `shell` (every surface of the window and the bar).

> The PDR describes **what the product must do and why**, never its implementation.

---

## User problem

A person who records their calls opens Kitokey between two meetings. An independent audit of the app
(2026-11-26, `kitokey-team/audit-2026-11-26/`, 52 screenshots) found that the same colour says several things —
green means « success », « the others' voice », « ready » and « confidential on » —, that red is kept for
recording only while « Mettre à la corbeille » looks like any other button, and that one word, « modèle », names
both the AI that writes and the shape of a summary. The person has to learn Kitokey's own code instead of
reading what every other app already taught them.

## Goal

Every colour and every word of Kitokey means one thing, the thing the market already taught its users.

## Out of scope

- The design system's values themselves (tokens, sizes, radii): cycle 12's D120 and its ADR if one is due.
- Summary formats (templates) as a feature: *Later* in cycle 12; only their word is decided here.
- Any brand's colour: brands stay as `ui/brands/BRANDS.md` says (#1238).

---

## Prior art

| Product / reference | Solution chosen | What we keep from it |
|---|---|---|
| Apple HIG, [Buttons](https://developer.apple.com/design/human-interface-guidelines/buttons) and [Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts) | A destructive button takes the system red; an alert with a destructive action includes Cancel, which is the default | Red for an action that deletes, never without a confirmation whose default is to cancel |
| GNOME, libadwaita [style classes](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/style-classes.html), `.destructive-action`; [GNOME HIG, buttons](https://developer.gnome.org/hig/patterns/controls/buttons.html) | « destructive colors », a warning to the user; one suggested or destructive button per view | The same red role on Linux, one per view |
| Firefox private browsing, [Mozilla bug 588655](https://bugzilla.mozilla.org/show_bug.cgi?id=588655) (2010) | Purple as « an ambient and peripheral cue » that private browsing is on, with its mask | Violet and a lock for « Confidentielle », Kitokey's private mode |
| Recording indicators: iOS's red dot while recording ([TidBITS, 2018](https://tidbits.com/2018/05/23/tipbits-make-sure-your-iphone-is-capturing-video/)); Kitokey's own test `red_says_a_recording_and_nothing_else` (`modules/shell/tests/look.rs`) | A red dot says « this is being recorded » | The red dot for a recording, as today |
| Two voices: the audit's design-system proposal (§3), blue and amber as the colour-blind-safe pair | — *(an assumption of the audit, not read at a primary source: to verify in D120)* | « Vous » blue; « Les autres » teal in light, golden amber in dark |
| Granola, [model selection](https://docs.granola.ai/help-center/getting-more-from-your-notes/understanding-model-selection-in-granola-chat), « Auto »; Granola and Notion call summary shapes [templates](https://docs.granola.ai/help-center/taking-notes/customise-notes-with-templates) | One word for the AI, another for the shape; « Auto » by default | « moteur » for the AI, « format » for the shape, never « modèle » for both |

**The convention the user already knows:** red deletes and records, a private mode wears its own colour,
the AI is chosen « Auto » unless one cares.

---

## Options considered

| Option | What the user experiences | Cost | Chosen? |
|---|---|---|---|
| Do nothing | Green for four meanings; « Corbeille » as quiet as « Copier »; « modèle » for two things | 0 | No |
| The audit's proposal: no red but recording; a neutral « Corbeille » with a confirmation | Red stays unambiguous, but a deletion looks like any action | Low | No — the decider: follow what the market adopted |
| **Follow the market's roles** (below) | Each colour and word means what other apps taught | D120's tokens; one test changed | **Yes** |

## Decision

1. **Two voices**: « Vous » (the microphone) is drawn in **blue**, « Les autres » (the call's sound) in **teal**
   in the light theme and **golden amber** in the dark one (the decider, on the mock-up, 2026-11-26) — in the
   waveform, the meters, the legend and the transcript. Green no longer means a voice.
2. **Red** says two things only: **a recording** (its live dot alone — Stop is calm and neutral, the decider on
   the mock-up, 2026-11-26), and **an action that destroys** (« Mettre à la
   corbeille », « Supprimer », « Oublier la destination »), which always opens a confirmation whose default is
   « Annuler ». A destructive action is never the primary action of a view. The test that reserved red to
   recording (`red_says_a_recording_and_nothing_else`) is rewritten to these two roles, never dropped.
3. **Violet with the lock** says **« Confidentielle »** — its switch on, its pill, its banner — and nothing else.
4. **Green** says **success** only: ready, sent, saved.
5. **Words**: an AI that transcribes or writes is a **moteur**; the shape of a summary is a **format**. « Modèle »
   is no longer said to the user for either.
6. **Design first**: the cycle that carries these choices recomposes what exists before adding features.

---

## Expected behaviour

**Business rules:** a colour is a role, set once in the design system's tokens (cycle 12's D120); a component
reads the role, never a value.

**Acceptance criteria** *(cycle 12's D120 and D124 hold them)*:

- [ ] Given the design system's tokens, when the window draws a waveform, a meter or a transcript line, then « Vous » takes the role blue and « Les autres » teal in the light theme and golden amber in the dark one, each 3:1 against its ground at least
- [ ] Given any button whose action deletes, when it is pressed, then a confirmation opens, « Annuler » focused, and the button bears the destructive role; no other button bears it
- [ ] Given « Confidentielle » on, when any surface shows it, then it is violet with the lock, and violet appears nowhere else
- [ ] Given the window's sentences in both languages, when they are read by `tests/words.rs`, then « modèle » / « model » names no AI and no summary shape shown to the user

---

## Success criterion

> We will consider this was the right call if **the screenshots of cycle 12's closure show each role in one
> meaning only, checked by its tests, and no report from a user confuses a deletion, a recording or a
> confidential meeting** is observed before **2027-03-31**.

How it is observed: cycle 12's tests and screenshots; issues filed by users.

If the criterion is not met: adjust the role that confused, by a new PDR.

---

## Removal condition

> This decision will be removed if **a platform Kitokey ships on adopts other roles for these colours** (a new
> Apple HIG or GNOME HIG), in which case Kitokey follows the platform.

---

## Impacts

- **Existing users**: the others' voice turns from green to amber; a deletion asks once more.
- **Modules and contracts**: `shell` only; no contract.
- **Support and documentation**: the site's screenshots are made again from the window's e2e (D103's way).
- **Data**: none.
