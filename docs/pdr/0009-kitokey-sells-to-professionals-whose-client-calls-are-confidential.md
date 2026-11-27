# PDR-0009 — Kitokey sells to professionals whose client calls are confidential: one promise, one price, the Store's page

> The framework's words: `.nstack/handbook/glossary.md`.

- **Status**: Proposed — the decider's choices of 2026-11-26, relayed by the coordinator the same day. The
  decider's approval of the pull request that carries it accepts it, the forge recording its day.
- **Date**: 2026-11-26
- **Decision makers**: the decider, `@napkinstack-admin`
- **Modules affected**: `site` (its message, sections and call to action); `shell` (the end of the trial on macOS).

> The PDR describes **what the product must do and why**, never its implementation.

---

## User problem

The site was written before the app's redesign (PDR-0008, mock-up v6) and before anyone chose whom it sells to.
An independent audit of 2026-11-26 found it shows the old app on every slide, says privacy four times, lists
eight uses, and promises a search and exports the window does not offer. A visitor cannot tell in five seconds
whether Kitokey is for them, what it costs, or where to get it on macOS and Windows.

## Goal

A professional who records client calls understands on the first screen that Kitokey writes their minutes
without anything leaving the computer, that it is paid once, and how to try it.

## Out of scope

- The site's visual design and its build: a mock-up, then a deliverable, after D121 to D126 are merged.
- Teams, health care, students: not addressed at launch.
- Any compliance claim (GDPR, HIPAA, HDS, professional secrecy): the site describes the architecture only.

---

## Prior art

| Product | Solution chosen | What we keep from it |
|---|---|---|
| Summit AI Notes (summitnotes.app, read 2026-11-26) | Local on the Mac, aimed at consultants and lawyers, 149.99 $ lifetime | The segment, named on the first screen |
| Talat (talat.app, read 2026-11-26) | Local, macOS and Windows, 189 $ lifetime, the trial's content said under the button | The price shown; the trial said plainly |
| MacWhisper (macwhisper.com, read 2026-11-26) | 64 € pay once | One payment as an argument |
| Granola, Otter, Fathom | Cloud, subscription, badges and logos as proof | Not followed: Kitokey proves privacy by its architecture |

Sources and the full comparison: `kitokey-team/site-audit/analyse-marche-et-message.md` (twelve products, each
with its URL).

## Options considered

| Option | Benefit for the user | Cost / risk | Verdict |
|---|---|---|---|
| Professionals in confidential client calls, French and English | The product fits as it is: two sides of a call, kept on the computer, minutes to hand over | A niche; willingness to pay inferred from competitors, not surveyed | **Chosen** |
| Employees under a « no bot, no cloud » policy | Same product, through the Microsoft Store | Some policies forbid any AI note-taking | Secondary |
| Teams | Larger market | Served free by Granola, Fathom, tl;dv, with team features Kitokey lacks | Rejected at launch |

## Decision

1. **Whom the site sells to first**: independents and small firms whose client calls are confidential
   (consultants, lawyers, accountants, coaches, HR, finance), in **French and English**. Second: employees under a
   « no bot, no cloud » policy.
2. **The promise**: FR « Vos rendez-vous clients ont leur compte rendu. Rien ne quitte votre ordinateur. » —
   EN « Every client call gets its minutes. Nothing leaves your computer. » Four pillars: on your computer,
   invisible in the call, minutes to hand over, paid once.
3. **The price**: **69 € once**, the same on macOS and Windows, after a free trial (within PDR-0001's range).
4. **The call to action before the macOS and Windows release**: the Microsoft Store's page as soon as it exists;
   no mailing list of ours (PDR-0001).
5. **On macOS, the meetings stay readable after the trial**, as on Windows (PDR-0005): recording stops, nothing
   made is locked.

## Expected behaviour

| Situation | What the person sees or gets |
|---|---|
| A lawyer lands on the site | The promise, one button for their system, « Sans compte. Sans abonnement. Rien ne quitte votre ordinateur. » |
| They look for the price | 69 €, once, and what the trial holds |
| Their trial ends on macOS | Their meetings, transcripts and minutes stay readable and exportable; recording asks for the purchase |

## Success criterion

- 2026-12-31: the site's new version is published with this promise, the price and the Store's button, every
  claim on it true of the app on main (checked by an audit like 2026-11-26's).
- 2027-06-30: the trial's conversion read from the Store's figures, without telemetry (PDR-0003).

## Removal condition

If conversion after six months on sale stays below what keeps the product alive, the segment and the price are
reopened by a new PDR.

## Impacts

- **Existing users**: none; Linux stays free (PDR-0001).
- **Modules and contracts**: `site` rewrites its message; `shell` holds the end of the trial on macOS as on
  Windows. No contract.
- **Support and documentation**: the FAQ answers « Does my client see anything? » and « Is it compatible with my
  professional secrecy? » by the architecture, never by a promise of compliance.
- **Data**: none.
