# ADR-0018 — A module `send` of its own, the one to reach a channel

> The framework's words: `.nstack/handbook/glossary.md`.

- **Status**: Proposed — by sam (session wholeheard-sam-13), author of D110, 2026-11-20. The criticality is the
  decider's word, written here with its day.
- **Date**: 2026-11-20
- **Decision makers**: the decider, `@napkinstack-admin`; the boundary settled with the coordinator, 2026-11-20
- **Scope**: module `send` — its capability, its boundary, its criticality
- **Reversibility**: easy before the shell consumes it; then a contract's change (`posting/v1`)

---

## Context

D110 sends a meeting's summary to a channel, framed by
[PDR-0007](../pdr/0007-a-summary-leaves-for-a-channel-on-a-gesture-never-a-confidential-one.md) and
[`posting/v1`](../../contracts/posting/v1/README.md). Cycle 11 named `export` as its module, and
[ADR-0001](./0001-split-summit-into-four-modules.md) named `export` « the first to be raised when a destination
outside the machine is added ». But `export` writes a local file and calls no network (`exporting/v1`: « It
contacts nothing »); D108 and D109 go through it.

## Decision

1. **A module of its own, `send`**, the program `kitokey-send`, providing `posting/v1`. It consumes
   `session-audio` and `minutes` by their contracts, as `export` does; it shares no code with it.
2. **`export` stays `standard`**: a file written on the machine, approved between agents, D108 and D109 not
   waiting on the decider. The risk of the network and of a secret is `send`'s alone.
3. **Its criticality: `high`** (proposed): its failure costs a person a day — a summary sent where it
   should not go, or a channel's address leaked — but it holds no key of its own (the shell keeps the
   address in the keyring; `send` receives it for one send) and it places no order and moves no money,
   which `critical` names. Below, `standard` would let an agent approve a change to what leaves the
   machine. Its cost: a runbook, the e2e of the journey, each pull request approved by the decider.
4. **Its one connection: ureq 3**, as `summary`'s, given a resolver of ours (`Judging`) that judges every
   address a name resolves to before ureq connects to one of them — no second resolution —, no proxy, no
   redirect. **The address read by `url`** (WHATWG), so that IDNA and an IPv4 in hex are what is judged.

## Rejected alternatives

| Alternative | Why not |
|---|---|
| `--send` on `kitokey-export` (cycle 11's module) | `export` would rise to `high`: every D108 and D109 change through the decider, for a risk they do not carry |
| The shell sends itself | The shell would hold a second HTTP client and the SSRF rules; a program tested alone, by its contract, is the repository's convention (ADR-0001) |
| `reqwest` | A runtime (tokio) for one POST; ureq is already the repository's (summary) |
| Judging the address once, then letting the client resolve | A name can answer public to the check and local to the connection (DNS rebinding): the judgement must be the connection's own resolution |

## Consequences

- `install` packs a sixth program, `kitokey-send`, beside the others (its own pull request, after this one).
- The shell's pull request (D110, criterion 2) starts it, keeps the address in the keyring and records each
  send.
- ureq's resolver API is « unversioned » (it may change in a minor release): an upgrade of ureq is read
  against `Judging` and its tests (`modules/send/AGENTS.md`).
