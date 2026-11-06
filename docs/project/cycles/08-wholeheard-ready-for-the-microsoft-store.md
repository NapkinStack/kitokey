---
goal: "On Windows, Wholeheard is ready for the Store: its trial ends without locking a meeting, it starts with the session from its package, and the colleague on Windows records a real call from it"
status: accepted          # proposed | accepted | closed | stopped — the decider accepts
accepted_on: 2026-11-03   # « 08 accepté, défauts du cadreur »; started at cycle 07's closure
appetite_weeks: 2         # the time the decider WANTS to spend, not an estimate
start: 2026-11-05         # YYYY-MM-DD
end: 2026-11-19           # start + appetite: the first day the cycle is over — the circuit breaker
deliverables:
  - id: D83
    title: "On Windows, once the Store's trial is over, no new meeting is recorded, and every meeting is still read and exported"
    module: shell
    state: ready
    acceptance:
      - "Given the trial's rule, when the shell's tests run, then the trial counts only the meetings the window lists that were made since Wholeheard first started on a trial licence — that day kept in a file of the folder of meetings, written once —, and is over only once 14 days have passed since that day and 10 such meetings are there; never at 9 meetings after 30 days, never at 10 meetings after 13 days; meetings copied there from elsewhere count for nothing; a folder that cannot be read, or a clock earlier than that day, is never over (PDR-0005)"
      - "Given Wholeheard on a Store licence that is a trial, the trial over — a stand-in licence that the tests give, never read by a build made for release; the trial's gate is the same on every system, only reading the Store's licence is Windows' own, so the window's e2e on Linux holds it —, when a recording is started from the window, the tray or a call's offer, then nothing is recorded, the window says why in one sentence of `words.rs`, in French and in English, and offers « Acheter », which asks the Store for the purchase or, that failing, opens Wholeheard's page in the Store — the purchase itself held by the stand-in only; every meeting is still read, played, corrected and exported; held by the window's e2e"
      - "Given the same stand-in licence turning into a bought one while Wholeheard is open, when the next recording starts, then it records, without a restart; given a recording in progress when the limit is reached, then it is never cut"
      - "Given a licence that cannot be read or that is not active, a package not installed from the Store, the zip, or Linux, when a recording starts, then it records as today; a licence that could not be read is said once in the log"
      - "Given Settings on a trial licence, when they show, then they say what is left of the trial, in days and in meetings — before the first day, the 14 days whole; once 10 meetings are made and days are left, the days alone; given `docs/store/listing.md`, then it says in French and in English how the trial works and what stays once it is over"
      - "Given GitHub's Windows runner and the package `trial.ps1` installs, when Wholeheard reads its licence there, then the run records what the Store answers for a package it did not install, and a recording is not refused; the Store's real trial is the decider's to see, after the first submission"
  - id: D84
    title: "On Windows, Wholeheard starts with the session from its package, and Settings say what Windows does"
    module: shell
    state: ready
    acceptance:
      - "Given the package installed and Wholeheard started once, when the user signs in again, then Wholeheard starts in the notification area alone, no window (D17), and Windows' own Startup settings list it under its name; held by the colleague on #482, a runner cannot sign in again"
      - "Given Settings' start-with-the-session switch on Windows, when it shows and is changed, then it shows Windows' state of Wholeheard's startup task and changes it there; a task turned off by the user in Windows is shown off and never turned back on by Wholeheard — Settings say where to turn it back on —, one turned off by a policy is shown off and one turned on by a policy shown on, the switch disabled in both, said in one sentence"
      - "Given the zip, not a package, when Settings show, then they do not offer the switch, and nothing is written under `AppData\\Roaming\\autostart`, where today's code writes a Linux entry Windows never reads; given Linux, D32's entry as today"
      - "Given the shell's tests, when they run, then each startup task state maps to what Settings show; given the Windows workflow, when `trial.ps1` installs the package, then the run reads Wholeheard's startup task and records its state after a first start"
  - id: D85
    title: "Spike — does Whisper large-v3 trained for French hear a meeting better than today's model?"
    module: transcription
    state: ready
    acceptance:
      - "Given `bofenghuang/whisper-large-v3-french` as whisper.cpp reads it (`ggml-model-q5_0.bin`, MIT) and today's recommended `ggml-large-v3-turbo-q8_0.bin`, in a spike outside the shipped program, the spike's input diffed first against what the step really reads — and D76's own input diffed the same way, its 98.3 % measured again if they differ —, when each hears the module's bench, the trial meeting and the decider's call kept in `temp/real-calls` — nothing of which leaves the machine or is published —, then the module's log issue records for each: word errors where a reference exists, inventions on a silent track, duplicated words, time against the recording's length, peak memory and the model's size"
      - "Given that record, when it is read against D76's threshold — at least two points fewer word errors on the trial meeting *(framer's, no source)*, no more than one point worse on the bench, fewer inventions on silence, faster than the call lasted on this machine's CPU —, then the framer proposes an ADR to offer it in the models list (ADR-0010); below it, what was missing is recorded. A refuted result is a delivered one"
  - id: D54
    title: "On Windows, Wholeheard installs from one MSIX package the Microsoft Store can take, and removing it keeps the meetings"
    module: install
    state: ready
    acceptance:
      - "Given the Windows workflow on main, when it runs, then it leaves beside the zip Wholeheard-<version>-x64.msix, made with the Windows SDK's MakeAppx from an AppxManifest.xml that names Wholeheard, its version in four parts the last 0, its logos, the identity the decider reserved in Partner Center — or a placeholder the run says is one —, `runFullTrust` and the microphone, and the shell started with `--window`; the four programs inside it"
      - "Given that package signed for the run with a certificate made for it, when GitHub's Windows runner installs it (`Add-AppxPackage`), then Wholeheard's process starts from it with its window; when a later version is installed over it, then a meeting folder placed under Documents\\Wholeheard and the settings are still there; and when it is removed (`Remove-AppxPackage`), then the meeting folder is still there"
      - "Given the package, when its notes for certification are read (`modules/install/windows/certification.md`), then they say why Wholeheard needs `runFullTrust` and the microphone — recording a call's two sides —, and that its settings and downloaded models live in the package's own data, which Windows removes with it, its meetings never"
      - "Given the colleague of cycle 04's D53 on their Windows 11, when they install the package signed for the run and record a real call of five minutes from it, then the cycle records whether both tracks hold the call's speech — capture working from inside a package, PDR-0004's removal condition"
  - id: D57
    title: "The Store's listing and Wholeheard's privacy policy are written, in French and in English"
    module: null
    state: ready
    acceptance:
      - "Given the repository, when `docs/privacy.md` is read on GitHub, then it says in French and in English what Wholeheard records, where it keeps it, what leaves the machine only when the user chose a service and to which host, that nothing is sent otherwise, and how to remove it all — its URL the one Partner Center is given"
      - "Given `docs/store/listing.md`, when it is read, then it holds the Store's description, its short features and the screenshots it needs, in French and in English, said as PDR-0001's promise — the decider reads it and says it claims nothing the product does not do"
  - id: D53
    title: "On Windows, Wholeheard opens from one zip the forge made — and does a call recorded there hold both sides?"
    module: install
    state: ready
    acceptance:
      - "Given a run of the Windows workflow on main, or one started by hand, when it ends, then each program's tests pass on GitHub's Windows runner, and it leaves one artifact, Wholeheard-<version>-windows-x64.zip, holding wholeheard.exe and the three programs side by side, kept 7 days as the Mac's"
      - "Given that zip on the colleague's Windows 11, when they unpack it and open wholeheard.exe — past SmartScreen's warning on an unsigned program —, then Wholeheard's icon is in the notification area and its window opens from it, with nothing installed before it and no terminal"
      - "Given a call recorded there for at least five minutes, when it stops, then the cycle records, for each track — local, the microphone; remote, what Windows plays: whether it holds the call's speech, and what Windows asked before it recorded — read against D39's threshold: both tracks hold the call's speech as the colleague hears it at the start, the middle and the end of each, and capture's code holds no line added for Windows; below it, what was missing is recorded for the next framing. A refuted result is a delivered one"
  - id: D86
    title: "Wholeheard becomes Kitokey, and nothing of the user's is lost on the way"
    module: shell
    state: ready
    acceptance:
      - "Given Wholeheard installed and used — settings, meetings, the Whisper model, keys in the keyring —, when the user installs Kitokey's package over it (`dnf install` of the .rpm, `apt install` of the .deb), then it replaces Wholeheard's, and Kitokey starts with the same settings, the same meetings where they were, the model not fetched again and the keys kept; a carrying over cut halfway resumes at the next start and never loses a meeting"
      - "Given the repository, when it is searched for Wholeheard, then the product, its programs, its packages, its application id, its folders and its variables are named Kitokey; Wholeheard and Summit are named only where history is told and in the code that carries over what the user held under them"
      - "Given the window, the tray, the applications menu and the notifications, in French and in English, when they are read, then they say Kitokey; the promise's sentence is PDR-0001's, unchanged"
      - "Given D78, when its source package is made and built, then the package is `kitokey` and the Copr project `kitokey`: `dnf copr enable <owner>/kitokey` then `dnf install kitokey`"
  - id: D78
    title: "On Fedora, Wholeheard installs from Copr with dnf, and each new version comes with the system's updates"
    module: install
    state: ready
    acceptance:
      - "Given a release tag on main, when the forge's workflow runs, then it makes a source package from that tag that builds the four programs from source, the crates inside it, and Copr builds it for each Fedora release Copr still supports, its internet access left off; a build that fails fails the workflow and names the release that failed"
      - "Given the package's header, when it is read, then its licence is the repository's, GPL-3.0, and its summary and description are those of today's package"
      - "Given a clean Fedora 44 container, when `dnf copr enable <owner>/wholeheard` then `dnf install wholeheard` run, then Wholeheard is installed, each program answers `--version` with the tag's version, and D80's removal holds on it"
      - "Given the previous version installed from Copr, when the next tag is built and `dnf upgrade` runs, then the new version replaces it and the meetings and settings stay"
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
  - id: D27
    title: "Wholeheard installs on Ubuntu from one package, its programs built on the system they ship to"
    module: install
    state: ready
    acceptance:
      - "Given Ubuntu 24.04, when a user installs Wholeheard from one package file (`sudo apt install ./kitokey_<version>_amd64.deb`), then Wholeheard is in the applications menu and starts in the tray with nothing built and no variable set, and removing it leaves the user's meetings and settings"
      - "Given a release for a system — Fedora or Ubuntu —, when the programs are built, then they are built in a container of that system, so that no package depends on the machine that made it"
---

# Cycle 08 — Wholeheard ready for the Microsoft Store

> The framework's words: `.nstack/handbook/glossary.md`.

> **Accepted by the decider on 2026-11-03**, on their word that day in the session that proposed it: « 08
> accepté, défauts du cadreur ». Each open question below is settled by its default, and PDR-0005 is accepted
> with it. Started at [cycle 07's closure](07-wholeheard-from-the-systems-updater.md#closure), on
> 2026-11-05, which moved six deliverables here.
>
> Proposed by the framer on 2026-11-03, on the decider's word « cadre le cycle 08 ». On 2026-10-31 the
> decider asked for one cycle per system: Linux first (cycle 07), then Windows, then macOS. This is Windows'.

## Goal

[PDR-0004](../../pdr/0004-windows-through-the-microsoft-store.md) sells Wholeheard on Windows through the
Microsoft Store. The package exists, and the certification kit passes it, with one optional test failed
(D54, D77; `modules/install/windows/certification.md`). Three things stand between it and a first
submission. First, the trial PDR-0004 chose would lock the meetings once over, so the Store's trial is
re-decided, in [PDR-0005](../../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md), and
built. Second, on Windows, today's « Démarrer avec la session » switch writes a Linux entry that Windows
never reads. Third, nobody has yet recorded a real call from the package.

```mermaid
flowchart LR
    P["PDR-0005<br/>the trial re-decided"]:::word --> T["D83 the trial,<br/>no meeting locked"]:::own
    S["D84 started with<br/>the session, from the package"]:::own
    F["D85 spike: Whisper<br/>trained for French?"]:::spike
    C["D53 · D54: the colleague's<br/>real call on Windows"]:::wait
    L["D57: the listing read"]:::wait
    T --> R["A package ready<br/>for its first submission"]:::end8
    S --> R
    C --> R
    L --> R
    A["The decider's<br/>Partner Center account"]:::word --> R
    classDef word fill:#1e3a8a,color:#fff
    classDef own fill:#065f46,color:#fff
    classDef spike fill:#5b21b6,color:#fff
    classDef wait fill:#92400e,color:#fff
    classDef end8 fill:#0f766e,color:#fff
```

**Legend** — blue: the decider's word or account · green: built by agents alone · violet: a spike · amber: a
person's trial or reading · teal: the outcome.

## Deliverables

**Cut first** if the breaker nears: D85, then D84. D83 is never cut while PDR-0005 stands: without it, the
Store's trial cannot be set at all.

### Moved from cycle 06

D53, D54 and D57. Their criteria are copied as they stand. Their state and pull requests are in
[cycle 06's closure](06-the-minutes-you-can-check.md#closure); their sections are in cycles 04 to 06. D58 is
not moved: its criteria hold on the three systems, so it is accepted in the Mac cycle. Its Windows part — the
first run's assistant opening once — is asked on #482 with D53's. D57's privacy page was brought current by
#624; what is left of it is the decider's reading of the listing.

### Moved from cycle 07

D78, D86, D66, D56, D41 and D27, at cycle 07's closure, on the decider's word of 2026-11-05. Their criteria
are copied as they stand, their sections are in [cycle 07](07-wholeheard-from-the-systems-updater.md), and
*What needs a person* below gives each its gesture. D78's and D86's code is merged: what is left is the
Fedora group `napkinstack-labs`, then a tag, then the first publication, which holds D78's criteria 3 and 4 and
D86's criterion 4. The end date does not move for them.

### D83

- **Its decision**: [PDR-0005](../../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md),
  proposed with this cycle. Refused, D83 is dropped. Partner Center then gets PDR-0004's 15-day trial, with
  no code, and its lock is written in the listing.
- **Prior Art Gate.** Microsoft's own way:
  - `StoreContext.GetAppLicenseAsync` reads the licence, and `StoreAppLicense.IsTrial` and `IsActive` say
    what it is;
  - `OfflineLicensesChanged` follows a purchase made while the application is open;
  - `RequestPurchaseAsync` buys;
  - a desktop application **must** give the `StoreContext` its window (`IInitializeWithWindow`), or it "will return inaccurate data or errors" ([*In-app purchases and trials*](https://learn.microsoft.com/en-us/windows/uwp/monetize/in-app-purchases-and-trials)): Wholeheard gives it its main window, hidden or not, before any reading or purchase, since a recording may start from the tray (D17);
  - `RequestPurchaseAsync` is reported never to start in some desktop applications (Microsoft Q&A 24011), hence the Store's page as the fallback.

  Source: [*Implement a trial version of your app*](https://learn.microsoft.com/en-us/windows/uwp/monetize/implement-a-trial-version-of-your-app),
  read 2026-11-03.
- The trial is counted from the folder of meetings, which survives the package's removal. No date is kept in
  the package's own data, which Windows removes with it.
- A new dependency, if the Store's API needs one, is declared in the shell's manifest with its exit
  (`security.md`).
- The shell's words for the trial are in this deliverable's pull requests. The listing's lines go in their
  own pull request, with no module; `trial.ps1`'s lines in an `install` pull request of their own.
- [Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) §10.8.4 asks a
  product to say the terms of its trial, and to warn in advance if it restricts the user's content: the
  listing says it; nothing the user made is restricted.
- The Store's real trial is seen only once Wholeheard is published, even hidden (*In-app purchases and
  trials*, *Testing*): after D54's submission, likely after this cycle's end — *Dependency with a date*.

### D84

- **Prior Art Gate.** A packaged desktop application declares a `windows.startupTask` in its manifest and
  reads and changes it through `StartupTask`. "If the task was disabled by the user using Task Manager, this
  method will not override their choice" ([`StartupTask`](https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptask),
  read 2026-11-03). Windows' Startup settings are then the setting, as the autostart folder is on Linux (D32).
- The manifest's extension belongs to `install`, with `Enabled="true"`: Windows turns it on at the first
  start, as D32 writes Linux's entry once. It goes in a pull request of its own, as does `trial.ps1`'s reading;
  the shell reads and changes the task.
- Whether the startup task inherits the application's `uap10:Parameters="--window"` is not documented: the
  colleague's sign-in on #482 is what shows it, and D17 asks for the tray alone.
- The same defect exists on macOS, where the entry is written under `Application Support`: it belongs to the
  Mac cycle.

### D85

- [The market read of 2026-11-03](../reviews/2026-11-03-market.md), row 3:
  - the model is a drop-in for whisper.cpp, under the MIT licence;
  - it claims fewer hallucinations, the defect the decider's real call showed;
  - no new engine is needed.
- As D76 did: no tuning, the model's defaults as they are (the decider, 2026-10-17). Nothing of the call is
  sent or published. Nothing enters the shipped program; offering the model is an ADR's.
- D76's 98.3 % of words wrong looks like an input that is not the product's. That input is diffed first,
  because a spike whose input is not the product's measures nothing.

## What needs a person

| Deliverable | Who | The literal gesture |
|---|---|---|
| PDR-0005 | The decider | Read [PDR-0005](../../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md) and say « PDR-0005 acceptée », or what changes |
| D57 | The decider | Read [`docs/store/listing.md`](../../store/listing.md) once D83's lines are in it; say « l'annonce ne promet rien de faux », or which line does |
| D54, submission | The decider | In [Partner Center](https://partner.microsoft.com/dashboard), with a company's account: reserve the product's name and give an agent the identity Partner Center shows. Set *Free trial* to **Unlimited** (PDR-0005). Read the Store's licence terms beside GPL-3.0, answer the age questionnaire, and submit the package an agent built from that identity |
| D53, D54 on Windows | The colleague on Windows 11 | [#482](https://github.com/NapkinStack/wholeheard/issues/482), updated first by an agent with the run's link, as cycle 06 says: the package signed for the run; the first run's assistant opens once (D58); a real call of five minutes; then sign out and in again: Wholeheard in the notification area, no window (D84). Answered there |
| D78, D86 | The decider | Once Fedora's infrastructure has made the group `napkinstack-labs`: activate it in [Copr](https://copr.fedorainfracloud.org) (*My Groups*), create its project `kitokey`, delete the project under your own account from its *Settings*. Then say it: an agent checks the project answers, tags `v<version>` on `main` on your word, and watches the publication |
| D66, trial | The decider | As cycle 06 says, on a build of `main`: AirPods, a WhatsApp Web call, five starts and stops, the answers on [#494](https://github.com/NapkinStack/kitokey/issues/494); then « ADR-0015 acceptée », or what falls |
| D41 | The decider | [#449](https://github.com/NapkinStack/kitokey/issues/449): a call on the earbuds, the default on the speaker |
| D56 | The decider | As [cycle 07](07-wholeheard-from-the-systems-updater.md#what-needs-a-person) says, in a terminal of your own, never through an agent: a minisign key, its `.pub` alone given to an agent |
| D27 | Anyone on Ubuntu 24.04 | `nstack claim D27`, then [#255](https://github.com/NapkinStack/kitokey/issues/255): the `.deb` of a build of `main`, one real call of five minutes, the answer there |

## Dependency with a date

- **An agent remakes the Windows builds from `main` and updates #482 within 48 hours of the acceptance**,
  and again whenever an artifact is about to expire. This was missed in cycles 06 and 07.
- **The decider's Partner Center account**, a company's: asked for within 5 days of the acceptance. Missed,
  D54 is delivered up to its package and the submission waits.
- **The colleague's call on #482**: 10 days after the acceptance. Missed, D53 and D54 stay `deferred`, and D84's
  first criterion waits.
- **The Store's real trial**: seen once the first submission is published, even hidden — the decider installs
  Wholeheard from the Store, sees the trial, buys. 2026-12-15 *(framer's, no source)*; missed, PDR-0005's
  criteria are read at the cycle that follows.

## Out of scope

- macOS: cycle 09, with D38, D39, D42 and D58.
- A signed zip, winget, or a download outside the Store: the Store signs (PDR-0004).
- Making the trial impossible to work around: the code is GPL-3.0 (PDR-0005).
- The price's figure: the decider's, in Partner Center.

## Later

- **Cycle 09, the finish before publishing**: proposed on 2026-11-06 ([its page](09-kitokey-finished-before-it-is-published.md)), from the decider's feedback that day.
- **Cycle 10, macOS**: D38, D39, D42, D58 on a Mac, ADR-0012's reading, notarisation. Also the start with the
  session on macOS, today written where macOS never reads it (D84's note), and Foresight read again (the market
  review of 2026-11-03, row 6).
- **The name**: Kitokey, chosen by the decider on 2026-11-04 ([PDR-0006](../../pdr/0006-the-product-is-named-kitokey.md)).
  The decider's trademark search and domains come before the Store's reservation and Copr's first publication.
- Everything in cycle 07's *Later*, unchanged, and the market review's rows 1, 2, 4, 5, 7 and 8.

## Open questions

**Blocking:** none. Each has a default the deliverable is built with.

**Not blocking**, each *(framer, to confirm)*:

- **The appetite**: two weeks. Default: two, never stretched.
- **PDR-0005**: default, accepted with the cycle. Refused, D83 is dropped (*D83* above).
- **The version submitted**: the Store refuses a version whose first part is 0. Default: 1.0.0 for the
  first submission only, as cycle 07 left it.
- **D85's second measure**: re-measuring D76 only if the inputs differ; dropped first if the breaker nears.
- **D85 in a cycle for Windows**: a spike with no system, so agents work while the cycle waits on people.
  Default: kept, and cut first.

## Closure

<Written at the end.>

**Documents brought current:** <at the closure>
