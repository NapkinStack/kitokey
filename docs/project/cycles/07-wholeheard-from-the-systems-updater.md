---
goal: "On Linux, Wholeheard comes from the system's updater: Fedora users install it with dnf from Copr and get each new version with their updates, removing it keeps their meetings, and the Linux trials that cycles 04 to 06 carried are run"
status: accepted          # proposed | accepted | closed | stopped — the decider accepts
appetite_weeks: 2         # the time the decider WANTS to spend, not an estimate
start: 2026-11-03         # YYYY-MM-DD
end: 2026-11-17           # start + appetite: the first day the cycle is over — the circuit breaker
deliverables:
  - id: D80
    title: "Removing Wholeheard's package leaves the meetings and the settings, and nothing else of it"
    module: install
    state: ready
    acceptance:
      - "Given the .rpm on Fedora 44 and the .deb on Ubuntu 24.04, each in a throwaway container, when Wholeheard is installed, its first start run as a user, a meeting written, then the package removed (`dnf remove`, `apt remove`), then no file the package installed is left, the user's meetings and settings are as they were, and what is left in the user's home is exactly what `docs/privacy.md` names"
      - "Given the same package installed again, when Wholeheard starts, then it reads the meetings and settings it left"
      - "Given the install module's e2e, when it runs, then both systems' removals above are held"
  - id: D86
    title: "Wholeheard becomes Kitokey, and nothing of the user's is lost on the way"
    module: shell
    state: ready
    acceptance:
      - "Given Wholeheard installed and used — settings, meetings, the Whisper model, keys in the keyring —, when the user installs Kitokey's package over it (`dnf install` of the .rpm, `apt install` of the .deb), then it replaces Wholeheard's, and Kitokey starts with the same settings, the same meetings where they were, the model not fetched again and the keys kept; a carrying over cut halfway resumes at the next start and never loses a meeting"
      - "Given the repository, when it is searched for Wholeheard, then the product, its programs, its packages, its application id, its folders and its variables are named Kitokey; Wholeheard and Summit are named only where history is told and in the code that carries over what the user held under them"
      - "Given the window, the tray, the applications menu and the notifications, in French and in English, when they are read, then they say Kitokey; the promise's sentence is PDR-0001's, unchanged"
      - "Given D78, when its source package is made and built, then the package is `kitokey` and the Copr project `kitokey`: `dnf copr enable <owner>/kitokey` then `dnf install kitokey`"
  - id: D82
    title: "A microphone chosen that cannot be kept says so, and the earlier choice stays"
    module: shell
    state: ready
    acceptance:
      - "Given the small window and settings that cannot be written, when the user chooses a microphone, then the window says it was not kept, the name shows the earlier choice, and the next recording takes the earlier choice; held by the window's e2e on stand-in devices"
  - id: D78
    title: "On Fedora, Wholeheard installs from Copr with dnf, and each new version comes with the system's updates"
    module: install
    state: ready
    acceptance:
      - "Given a release tag on main, when the forge's workflow runs, then it makes a source package from that tag that builds the four programs from source, the crates inside it, and Copr builds it for each Fedora release Copr still supports, its internet access left off; a build that fails fails the workflow and names the release that failed"
      - "Given the package's header, when it is read, then its licence is the repository's, GPL-3.0, and its summary and description are those of today's package"
      - "Given a clean Fedora 44 container, when `dnf copr enable <owner>/wholeheard` then `dnf install wholeheard` run, then Wholeheard is installed, each program answers `--version` with the tag's version, and D80's removal holds on it"
      - "Given the previous version installed from Copr, when the next tag is built and `dnf upgrade` runs, then the new version replaces it and the meetings and settings stay"
  - id: D79
    title: "Spike — can a PPA build Wholeheard for Ubuntu 24.04 with what the archive offers?"
    module: install
    state: ready
    acceptance:
      - "Given a source package of a release tag, its crates vendored in it, when it is built offline in an Ubuntu 24.04 container with only the archive's packages as build dependencies, then the module's log issue records whether it builds, the archive's rustc and cargo against what the four programs need, the time it took, the size of the source package, and what had to be added"
      - "Given that record, when it is read against the threshold — it builds offline with the archive's toolchain, or with one trusted PPA named as a build dependency *(framer's, no source)* —, then the framer proposes the PPA as a deliverable of the next framing; below it, what was missing is recorded. A refuted result is a delivered one"
  - id: D66
    title: "A recording holds the microphone the call holds, and never touches a Bluetooth headset's profile"
    module: capture
    state: ready
    acceptance:
      - "Given a call holding a microphone other than the default input, when a recording runs, then `me` records that one — its name in `session.json` and the `device` event — and, when the call changes it, follows it, the track going on; held on virtual devices by capture's e2e, a stand-in call on a source of its own"
      - "Given a source carrying `bluez5.loopback=true`, when capture records it, then its stream says its version's passive value (ADR-0015, as corrected by #574, merged after this amendment), the source never runs from it alone and no profile switch comes from it, within one known window: a default input turning into a headset during a recording stays on an active stream until a reading of the graph closes it, within a few seconds, and for as long as the graph does not read; in that window WirePlumber's decision to switch came 4 times in 10 measured, on a stand-in with no card; held by capture's e2e on a stand-in source, read in `pw-dump` — WirePlumber's decision testable on a stand-in carrying `device.id` —, and `others` is unchanged. Given no call holding a microphone, then capture records the meeting's sound and says in one sentence that the microphone waits when the default input is such a source; records the default input, as today, when it is not"
      - "Given a call that lets go of the microphone, when it leaves, then capture lets go of it too: within what WirePlumber's restore takes, it holds no HFP profile by a stream of its own and the source no longer runs — held by the same e2e, the source's state in `pw-dump`, and by the decider's trial, the card's profile"
      - "Given a transport failure during a recording — a source taken away, on the e2e's stand-in —, when it fails, then the recording continues, the track's gap is marked as `session-audio/v2` allows (`clock.jsonl`'s `silence`, `changed` with `device: null`), and `recording/v1` says it as its page says now, by `device` with `null`; a reason in the event is D73's, in *Later*; the shell's words are in their own pull request"
      - "Given the decider's AirPods and a real WhatsApp Web call, when they start and stop a recording during it, at least five times, then the call is never cut, no `Failure in Bluetooth audio transport` is logged within two seconds of capture opening or closing a stream, and `local.wav` holds their voice each time; an acceptance issue says what was seen"
  - id: D56
    title: "The list of models read again is refused unless the decider's key signed it"
    module: shell
    state: ready
    acceptance:
      - "Given `registry/models.json` and its signature `models.json.minisig` on main, made with minisign by the decider's key — the public key built into the shell, the private one never in a repository —, when the list is read again on the user's gesture, then it is offered only when its signature verifies; a list unsigned, signed by another key, or changed after it was signed is refused, and the shipped list stays, said in one sentence (ADR-0010, *Rejected alternatives*)"
      - "Given a pull request that changes the list, when its checks run, then a list whose signature does not verify against the built-in key is refused; until the decider's key is given, no list is read again, and Settings and the check say it waits on that key — the colleagues' builds lose today's reading again, said in the cycle's closure"
      - "Given ADR-0010, when D56 is merged, then the record says its option 4 adopted, `minisign-verify` declared in the shell's manifest with its exit strategy, and what a signature does not stop: an older list, signed, offered again"
      - "Given the shell's tests, when they run, then a list signed by a key made for the test is offered, and the three refusals above are held, without the decider's key"
  - id: D41
    title: "On Linux, a call heard on an output other than the system's default is recorded there"
    module: capture
    state: ready
    acceptance:
      - "Given a call — an application other than capture holding a microphone — whose sound plays on an output other than the system's default, when it is recorded, then the remote track holds what the call plays, and capture says `device` with that output's name"
      - "Given the call's sound moved to another output while recording, or back to the default, when it moves, then the remote track follows it within two seconds and the move is kept in `session.json` with when"
      - "Given no call, a call whose sound plays on the default, or on several outputs at once, when it is recorded, then the remote track records the system's default output, as D31 does"
      - "Given a graph that cannot be read or a stream that cannot be moved, when it is recorded, then the recording goes on from the default, and one warning is logged"
      - "Given capture's e2e, when a stand-in call plays a note on an output of its own that is not the default, then the remote track holds that note — on devices made for the run, nobody recorded"
  - id: D63
    title: "A service that hears is chosen as the minutes' is: from a list, its transcription models offered"
    module: shell
    state: ready
    acceptance:
      - "Given Settings, when the user chooses who hears online, then the services the minutes offer that publish OpenAI's `audio/transcriptions` — OpenRouter, OpenAI, and Mistral if it does — are offered by name, their address filled in as the minutes' are, and « Un service que je choisis » keeps an address of their own"
      - "Given a service chosen, when Settings show it, then its transcription models are offered to choose from — OpenRouter's read from its public list, `models?output_modalities=transcription`, asked with no key and nothing of a meeting; the others' that give segments and their times named in the repository —, a model typed by hand still possible, and *Tester* says whether the one chosen gives segment times, the list not saying it; its key kept in the system's keyring as the minutes' is"
      - "Given a service and a model chosen, when the user tests it or a meeting is transcribed, then it is as D43 says — a sound of the test's own, the host agreed first, a model that gives no segment times refused and said —, with the service's own address"
      - "Given the shell's tests and the window's e2e, when they run, then a stand-in service listing its models is offered, chosen and tested, and its key never written in the settings"
  - id: D27
    title: "Wholeheard installs on Ubuntu from one package, its programs built on the system they ship to"
    module: install
    state: ready
    acceptance:
      - "Given Ubuntu 24.04, when a user installs Wholeheard from one package file (`sudo apt install ./kitokey_<version>_amd64.deb`), then Wholeheard is in the applications menu and starts in the tray with nothing built and no variable set, and removing it leaves the user's meetings and settings"
      - "Given a release for a system — Fedora or Ubuntu —, when the programs are built, then they are built in a container of that system, so that no package depends on the machine that made it"
---

# Cycle 07 — Wholeheard from the system's updater, on Linux

> The framework's words: `.nstack/handbook/glossary.md`.

> **Accepted by the decider on 2026-11-03**, closing cycle 06, on their word that day in the session
> that proposed it: « 07 accepté, défauts du cadreur ». Each open question below is settled by its
> default, the decider amending any at will; the dates are that day's, the appetite two weeks.

## Goal

[PDR-0001](../../pdr/0001-wholeheard-open-on-linux-paid-on-macos-and-windows.md) §3 makes Linux free, through
Copr and a PPA, updated by the system. No rival has that on Fedora ([the market read on 2026-10-26](../reviews/2026-10-26-market.md), row 4).
Today a person installs Wholeheard from a file an agent built, and nothing brings them the next version.
The repository is public and `capture` is `critical` (D55): what [PDR-0002](../../pdr/0002-the-repository-opened-the-pace-kept.md)
asks before a first build reaches people outside is done. On 2026-10-31 the decider asked for one cycle per
system: Linux first, then Windows, then macOS. This is Linux's.

```mermaid
flowchart LR
    R["D80 removing it<br/>keeps the meetings"]:::own --> C["D78 Copr,<br/>dnf and its updates"]:::pub
    N["D86 the name<br/>Kitokey"]:::own --> C
    M["D82 a choice not kept,<br/>said"]:::own
    P["D79 spike:<br/>a PPA for Ubuntu?"]:::spike
    T["Trials moved:<br/>D66 D41 D63 D27 · D56's key"]:::wait
    C --> U["A Fedora user<br/>updated by the system"]:::end7
    classDef own fill:#065f46,color:#fff
    classDef pub fill:#1e3a8a,color:#fff
    classDef spike fill:#5b21b6,color:#fff
    classDef wait fill:#92400e,color:#fff
    classDef end7 fill:#0f766e,color:#fff
```

**Legend** — green: built by agents alone · blue: needs the decider's Copr project once · violet: a spike ·
amber: moved, a person's trial or key left · teal: the outcome.

## Deliverables

**Cut first** if the breaker nears: D79, then D82. Never D80: Copr does not publish a package whose
removal is untested. Never D86 while D78 publishes: nothing is published under the name PDR-0006 replaced.

### Moved from cycle 06

D66, D56, D41, D63 and D27. Their criteria are copied as they stand. Their state and pull requests are in
[cycle 06's closure](06-the-minutes-you-can-check.md#closure), and their sections are in cycles 04 to 06.
*What needs a person* below gives each its gesture. D27 is no longer tied to a named colleague: anyone with
an Ubuntu 24.04 machine can take it with `nstack claim D27`, for one short real session.

### D80

- `modules/install/AGENTS.md` already says nothing is installed outside `/usr`. This deliverable proves it
  after the package is removed. [`docs/privacy.md`](../../privacy.md) (#624) names what stays in the user's home:
  the start-at-login entry, the meetings, the settings, and Ollama's models.
- In throwaway containers, as the module's e2e already runs. Nothing is installed on the workstation.

### D86

**Added on the decider's word, 2026-11-04**, in the session that wrote
[PDR-0006](../../pdr/0006-the-product-is-named-kitokey.md): « cadre le livrable de renommage en kitokey », in
this cycle, before D78 (« Cycle 07, avant D78 »). The trademark search done and the domains bought, the same
day: « Déjà fait ». The end date does not move.

- **Prior Art Gate.** D28 renamed Summit to Wholeheard on 2026-10-20 ([cycle 03](03-the-window.md#d28)): one
  pull request per program, the shell's carrying over, the package taking the old one's place, the
  contracts' prose and the repository's pages, each verified by another session. Its lessons stand: a step
  of the carrying over does what is left of it at every start; a settings file with no line end stays
  readable. The keys already move from `dev.summit.Summit` (`modules/shell/src/keys.rs`): Wholeheard's
  become one more name to read from.
- **Before D78 publishes**, so that the first users of Copr install Kitokey and never change repository.
- **The pull requests in flight** (D80, D27, D56, D41, D66, D63) are not waited for: each takes the default
  branch in before its own merge and settles its conflicts, as any pull request does (kernel §2, step 10).
- The repository's name on GitHub is the decider's gesture, once D86 is merged (*What needs a person*).

### D82

- Named by #623's verifier: no check holds `choosePick`'s rollback when the settings cannot be written.

### D78

- **Prior Art Gate.** [Copr](https://docs.copr.fedorainfracloud.org/user_documentation.html) builds from an
  uploaded source package or from a repository's spec, and copr-cli reads an API token from a file. Builds
  run without internet access unless the project turns it on
  ([Copr's project guide](https://www.mintlify.com/fedora-copr/copr/user-guide/creating-projects), a mirror,
  not the official page). Access stays off here, so the crates are vendored in the source package, the usual
  way to build Rust offline.
- Today's `wholeheard.spec` packs programs built beforehand and says `License: LicenseRef-Proprietary`. A
  Copr build compiles them from source, and the licence is the repository's, GPL-3.0 (PDR-0002).
- The token is a secret of the forge. Setting it is the decider's: GitHub's settings are theirs.

### D79

- [Launchpad's builders have no outbound network](https://answers.launchpad.net/launchpad/+question/691015):
  everything needed is in the source package or a build dependency. Whether Ubuntu 24.04's archive rustc
  builds the four programs is unknown, hence a spike. It is answered in a container before anyone opens a
  Launchpad account.

## What needs a person

| Deliverable | Who | The literal gesture |
|---|---|---|
| D86 | The decider | Done on 2026-11-04: the repository is `NapkinStack/kitokey`, GitHub redirecting the old address |
| D78 | The decider | Sign in to [Copr](https://copr.fedorainfracloud.org/) with a Fedora account and create the project `kitokey` (D86; a project already made as `wholeheard` is deleted from its *Settings*). Copy the text [its API page](https://copr.fedorainfracloud.org/api/) gives into a file `copr`, then, at the root of a checkout: `gh secret set COPR_CONFIG < copr` and delete the file. Tell an agent the owner's name |
| D66, trial | The decider | As cycle 06 says, on the build of this cycle's first tag: AirPods, a WhatsApp Web call, five starts and stops, the answers on [#494](https://github.com/NapkinStack/wholeheard/issues/494) |
| ADR-0015 | The decider | After that trial, say « ADR-0015 acceptée », or what falls |
| D41 | The decider | [#449](https://github.com/NapkinStack/wholeheard/issues/449): a call on the earbuds, the default on the speaker |
| D63 | The decider | [#463](https://github.com/NapkinStack/wholeheard/issues/463): a real service heard, with their key |
| D56 | The decider | In a terminal of your own, never through an agent: `sudo dnf install minisign`, then, at the root of a checkout of `main`, `mkdir -m 700 -p ~/.minisign` and `minisign -G -p kitokey-models.pub -s ~/.minisign/kitokey-models.key` (named after D86): a passphrase kept in your password manager, the `.key` file copied offline, never `-W`, which keeps it without a passphrase. Give an agent `kitokey-models.pub` alone. Sign when asked: `minisign -Sm modules/shell/registry/models.json -s ~/.minisign/kitokey-models.key`, and give an agent `models.json.minisig`: it commits it |
| D27 | Anyone on Ubuntu 24.04 | `nstack claim D27`, then [#255](https://github.com/NapkinStack/wholeheard/issues/255): `sudo apt install ./wholeheard_<version>_amd64.deb`, one real call of five minutes, the answer there |

## Dependency with a date

- **An agent remakes the builds from `main` and updates #255, #449, #463 and #494** within 48 hours of the
  acceptance, the files with them. Cycle 06 missed it.
- **The decider's Copr project and its token**: 5 days after the acceptance, so that D78 is tried before the
  end. Missed, D78 is delivered up to its second criterion and publishing waits.

## Out of scope

- A PPA built and published: D79 asks first whether it can be.
- Flathub, Snap, AUR: Flathub does not sell (PDR-0004), and nobody asked for the others.
- Windows and macOS: their cycles come next.
- Signing the packages with a key of the project's own: Copr signs with its own.

## Later

- **Cycle 08, Windows**: D53, D54, D57, D58 on Windows, D77's kit on a person's Windows, the Store's account.
- **Cycle 09, macOS**: D38, D39, D42, D58 on a Mac, ADR-0012's reading, notarisation.
- A PPA, if D79 says it can be built. Debian once a channel exists (PDR-0001).
- Tests in `summary`, `transcription`, `platform` and `contracts` that still name a later version's number
  get `vnext`: one pull request each, out of cycle.
- The window's AA scenario, failed twice in a whole e2e and never alone.
- Every item of cycle 06's *Later*, unchanged.

## Open questions

**Blocking:** none. Each has a default the deliverable is built with.

**Not blocking**, each *(framer, to confirm)*:

- **The appetite**: two weeks. Default: two, never stretched.
- **Which Fedora releases**: those Copr still supports at the tag. Default: 43 and 44, x86_64; aarch64 only
  if it costs nothing more.
- **The first Copr build's version**: the `0.2.x` already in the packages. Default: kept; 1.0.0 stays for the
  Store's first submission.
- **D27 without a named colleague**: default, anyone may take it; not taken by the end, it goes to *Later*.

## Closure

<Written at the end.>

**Documents brought current:** <at the closure>
