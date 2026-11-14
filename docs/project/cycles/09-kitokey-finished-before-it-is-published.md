---
goal: "Before Kitokey is published, a meeting weighs half what it does, its two sources are tried before a recording, and recording is a minimal bar in Kitokey's own look"
status: closed            # proposed | accepted | closed | stopped — the decider accepts
accepted_on: 2026-11-07   # « …enchaine », the plan before publishing; started at cycle 08's closure
appetite_weeks: 2         # the time the decider WANTS to spend, not an estimate
start: 2026-11-07         # cycle 08's closure
end: 2026-11-21           # start + appetite: the first day the cycle is over — the circuit breaker
deliverables:
  - id: D87
    title: "A meeting's tracks are compressed without loss once its recording stops: it weighs less, and nothing of what was recorded is lost"
    module: capture
    state: accepted
    acceptance:
      - "Given a recording stopped, when capture ends, then — under `session-audio/v3`, the third version `v2`'s page already names for it — each track is kept as FLAC in the meeting's folder in place of its WAV — the same rate, channels and bit depth —, its samples decoded equal, one by one, to the WAV's, which is removed only once that is checked; `recording/v1`'s page says which event tells that the tracks are final, and no step reads a track before it — a making started on `stopped` reads the WAV whole or the FLAC whole, never one being removed; held by capture's e2e, the second form's notes decoded and compared"
      - "Given capture killed (`SIGKILL`) at chosen points while it compresses — before the FLAC is whole, after, before and after the WAV is removed —, when the meeting is read again, then it is read whole — from its WAV until the FLAC is checked, from its FLAC after; no meeting is ever left with neither a whole WAV nor a whole FLAC"
      - "Given a meeting kept as FLAC, when it is transcribed, summarised, played, its waveform drawn, exported or made again, then each step reads it as it reads a WAV today; given a meeting recorded before, kept as WAV, then it is read as before — the readers of `session-audio/v3` merged before capture writes it, as `contracts/AGENTS.md` asks"
      - "Given the bench's meeting and an hour recorded by the e2e, when they are compressed, then the sheet records what each weighed as WAV and as FLAC and the time compressing took against the recording's length; the FLAC weighs at most 60 % of the WAV on the bench's meeting *(framer, to confirm: cycle 02 measured about 48 %)*"
  - id: D88
    title: "Before a recording, the user tries their microphone and the meeting's sound, and sees that Kitokey hears both"
    module: shell
    state: accepted
    acceptance:
      - "Given capture asked to listen without recording, when it runs, then it says, as `recording/v1` says while recording, each source's `device` and its `level` about ten times a second, writes nothing anywhere, and stops on « stop » or a TERM signal within one second; held by capture's e2e on devices made for the run — a note played into each, its level seen, no file written — in a pull request of capture's own, before the shell's"
      - "Given « Vérifier le son » / \"Check the sound\" opened from the bar or from Settings, when it shows, then it shows, for « Votre micro » and « Le son de la réunion », the device's name and a meter that moves as sound comes; the microphone is opened only while it shows — an active stream, the departure from ADR-0015 this deliverable writes in that record first: a Bluetooth headset then switches to its call profile while the panel shows, which the panel says in one sentence, and WirePlumber restores it once closed; during a recording ADR-0015 holds unchanged —, and no stream of Kitokey's is left open within one second of closing it — held by the window's e2e, read in `pw-dump`; with it closed and no recording, Kitokey holds no stream"
      - "Given the panel and no application holding a microphone, when the user asks « Jouer un son de test », then a short sound plays on the system's default output and the panel says, in one sentence of `words.rs`, whether Kitokey heard it on the meeting's side, and what to do when it did not; given an application holding a microphone — a call —, then no sound is played and the panel says to play something in the call and watch the meter; held by the window's e2e on devices made for the run"
      - "Given the panel, when the user starts a recording from it, then the recording starts from there, the panel closes and the meters go on in the bar; given the decider's AirPods on their Fedora, before a real call, then they see both sides move, and their music or call is back to its quality once the panel is closed — an answer in the acceptance issue"
  - id: D89
    title: "Kitokey has a look of its own: a mark, one set of icons, compact controls, one accent"
    module: shell
    state: deferred   # closed early, 2026-11-13: merged (#712, #720 to #741), its criteria not all held; its holder's `nstack deliver`, or the next framing
    acceptance:
      - "Given a new folder of drawings under `docs/project/drawings/` drawn before anything is built — with D90's boards, in one round —, when the decider reads it, then it shows in both themes Kitokey's mark, the tray's icon at rest and while recording, the full window's header and a meeting's page with the icons and sizes below; what the decider accepts there becomes this deliverable's bar, its date in the cycle"
      - "Given the window, the bar and Settings, when the shell's tests run, then every icon is one of Lucide's, inline, 16 or 20 CSS px; each screen has one labelled primary action, every secondary action is an icon button with a tooltip and an accessible name in French and in English; controls are 32 CSS px high, 28 in the bar, on a 4 px grid; red is used for recording alone; the contrast of each pair of colours the theme uses is checked by a test in both themes"
      - "Given the accepted mark, when the packages are made, then the applications menu, the Store's logos, the deb's and the rpm's icons and the tray's two states are drawn from it, one SVG in the repository; given a recording, then the tray's icon shows it, on GNOME too, where the AppIndicator extension gives a tray; without it, GNOME's own microphone indicator and the bar are the signs, said in the README — that indicator says a microphone is in use, a call's too, not that Kitokey records, and whether a passive stream lights it is not verified *(framer, to confirm)*"
      - "Given the window's e2e, when it runs, then its screenshots in both themes, at 1920 and 420 CSS px, are in the sheet beside the accepted boards"
  - id: D90
    title: "Recording is a minimal bar: the essentials, and the rest one gesture away"
    module: shell
    state: deferred   # closed early, 2026-11-13: merged (#712, #740, #743, #744, #752, #768), criterion 3 on Windows and macOS unproven; its holder's `nstack deliver`, or the next framing
    acceptance:
      - "Given D89's round of drawings, when the decider reads it, then it holds the bar at rest, recording, and opened — its size and content are the boards', the last meeting no longer in it; what the decider accepts there becomes this deliverable's bar"
      - "Given the bar while recording, when it shows, then it holds the time recorded in tabular figures, a thin meter for each side with its health — a side silent for two minutes said in words, never as an error —, « Arrêter », and a way to the full window; opened, it shows the context, the microphone, « Confidentiel », « Vérifier le son » (D88) and « Avancé » (D91) — keyboard alone, both themes, French and English, nothing overflowing"
      - "Given Windows or macOS, when a recording runs, then the bar stays above the other windows, can be moved, and is opened where it was left; given GNOME under Wayland, which lets no application keep itself above (tauri #3117), then the bar is an ordinary window the user may keep above from its window menu, and the tray's icon shows the recording where a tray is (D89) — GNOME's microphone indicator where none is"
      - "Given the bar hidden by the user during a recording, when anything happens but the stop, then it does not come back by itself; the shortcut still stops the recording"
  - id: D91
    title: "A meeting's context and its models are chosen before it is recorded, the settings' choice by default"
    module: shell
    state: accepted
    acceptance:
      - "Given the bar at rest, when the user types a context and then starts a recording, then that context is the new meeting's `context.txt`, under D59's rules — 400 characters, written whole once the meeting's `session.json` is there; another meeting being made does not close the field; given no recording started, then it is kept for the next one until the user clears it, and said as such"
      - "Given « Avancé » / \"Advanced\", closed by default, in the opened bar and in *Refaire…*, when the user opens it, then it offers the transcription's model — the models installed and the service when one is set — and the minutes' model, each set to Settings' choice; a choice made there is that meeting's alone, kept with the meeting as a contract allows — a version first when none does —, used when it is made and made again, and Settings are unchanged"
      - "Given a meeting whose own models were chosen, when its page shows, then it says which models made its transcript and its minutes"
  - id: D52
    title: "Spike — can a transcript followed live, on the machine alone, keep up during a call and be worth reading?"
    module: null
    state: deferred   # closed early, 2026-11-13: spike not started; cut first
    acceptance:
      - "Given a measuring program of the bench — a session's two tracks heard as they would arrive, in spans of 5 s and of 10 s, by `ggml-small-q8_0` (the list's \"faster\" model), and by Kyutai's `stt-1b-en_fr` streaming model as it hears, continuously — one that does not run on this machine's processor recorded as such, a result —, when it runs on the decider's Fedora while capture records a stand-in call on devices made for the run, then the transcription's log issue records, for each: the time to hear a span against its length, the processor and memory it took, the delay from a word said to the word shown, and capture's tracks against a recording of the same call without it — gaps of zeros, blocks lost"
      - "Given SUMM-RE's passages of the bench (`bench.real`), when they are heard that way and scored, then the record gives the share of words missed or misheard by the preview, side by side with the transcript of record's — the recommended model, whole — on the same passages"
      - "Given that record, when it is read against the threshold *(framer, to confirm)* — each span heard in at most half its length, a streaming model hearing at least twice as fast as the sound comes, capture's tracks no worse than without it, and the preview's words missed or misheard at most 15 points above the transcript of record's —, then the next framing may build the preview, its ADR first; below it, what fell short is recorded. A refuted result is a delivered one"
  - id: D84
    title: "On Windows, Wholeheard starts with the session from its package, and Settings say what Windows does"
    module: shell
    state: deferred   # closed early, 2026-11-13: #790 merged, #788 and #791 open, criterion 1 a person's (#795)
    acceptance:
      - "Given the package installed and Wholeheard started once, when the user signs in again, then Wholeheard starts in the notification area alone, no window (D17), and Windows' own Startup settings list it under its name; held by the colleague on #482, a runner cannot sign in again"
      - "Given Settings' start-with-the-session switch on Windows, when it shows and is changed, then it shows Windows' state of Wholeheard's startup task and changes it there; a task turned off by the user in Windows is shown off and never turned back on by Wholeheard — Settings say where to turn it back on —, one turned off by a policy is shown off and one turned on by a policy shown on, the switch disabled in both, said in one sentence"
      - "Given the zip, not a package, when Settings show, then they do not offer the switch, and nothing is written under `AppData\\Roaming\\autostart`, where today's code writes a Linux entry Windows never reads; given Linux, D32's entry as today"
      - "Given the shell's tests, when they run, then each startup task state maps to what Settings show; given the Windows workflow, when `trial.ps1` installs the package, then the run reads Wholeheard's startup task and records its state after a first start"
  - id: D85
    title: "Spike — does Whisper large-v3 trained for French hear a meeting better than today's model?"
    module: transcription
    state: deferred   # closed early, 2026-11-13: spike not started
    acceptance:
      - "Given `bofenghuang/whisper-large-v3-french` as whisper.cpp reads it (`ggml-model-q5_0.bin`, MIT) and today's recommended `ggml-large-v3-turbo-q8_0.bin`, in a spike outside the shipped program, the spike's input diffed first against what the step really reads — and D76's own input diffed the same way, its 98.3 % measured again if they differ —, when each hears the module's bench, the trial meeting and the decider's call kept in `temp/real-calls` — nothing of which leaves the machine or is published —, then the module's log issue records for each: word errors where a reference exists, inventions on a silent track, duplicated words, time against the recording's length, peak memory and the model's size"
      - "Given that record, when it is read against D76's threshold — at least two points fewer word errors on the trial meeting *(framer's, no source)*, no more than one point worse on the bench, fewer inventions on silence, faster than the call lasted on this machine's CPU —, then the framer proposes an ADR to offer it in the models list (ADR-0010); below it, what was missing is recorded. A refuted result is a delivered one"
  - id: D54
    title: "On Windows, Wholeheard installs from one MSIX package the Microsoft Store can take, and removing it keeps the meetings"
    module: install
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given the Windows workflow on main, when it runs, then it leaves beside the zip Wholeheard-<version>-x64.msix, made with the Windows SDK's MakeAppx from an AppxManifest.xml that names Wholeheard, its version in four parts the last 0, its logos, the identity the decider reserved in Partner Center — or a placeholder the run says is one —, `runFullTrust` and the microphone, and the shell started with `--window`; the four programs inside it"
      - "Given that package signed for the run with a certificate made for it, when GitHub's Windows runner installs it (`Add-AppxPackage`), then Wholeheard's process starts from it with its window; when a later version is installed over it, then a meeting folder placed under Documents\\Wholeheard and the settings are still there; and when it is removed (`Remove-AppxPackage`), then the meeting folder is still there"
      - "Given the package, when its notes for certification are read (`modules/install/windows/certification.md`), then they say why Wholeheard needs `runFullTrust` and the microphone — recording a call's two sides —, and that its settings and downloaded models live in the package's own data, which Windows removes with it, its meetings never"
      - "Given the colleague of cycle 04's D53 on their Windows 11, when they install the package signed for the run and record a real call of five minutes from it, then the cycle records whether both tracks hold the call's speech — capture working from inside a package, PDR-0004's removal condition"
  - id: D57
    title: "The Store's listing and Wholeheard's privacy policy are written, in French and in English"
    module: null
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given the repository, when `docs/privacy.md` is read on GitHub, then it says in French and in English what Wholeheard records, where it keeps it, what leaves the machine only when the user chose a service and to which host, that nothing is sent otherwise, and how to remove it all — its URL the one Partner Center is given"
      - "Given `docs/store/listing.md`, when it is read, then it holds the Store's description, its short features and the screenshots it needs, in French and in English, said as PDR-0001's promise — the decider reads it and says it claims nothing the product does not do"
  - id: D53
    title: "On Windows, Wholeheard opens from one zip the forge made — and does a call recorded there hold both sides?"
    module: install
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given a run of the Windows workflow on main, or one started by hand, when it ends, then each program's tests pass on GitHub's Windows runner, and it leaves one artifact, Wholeheard-<version>-windows-x64.zip, holding wholeheard.exe and the three programs side by side, kept 7 days as the Mac's"
      - "Given that zip on the colleague's Windows 11, when they unpack it and open wholeheard.exe — past SmartScreen's warning on an unsigned program —, then Wholeheard's icon is in the notification area and its window opens from it, with nothing installed before it and no terminal"
      - "Given a call recorded there for at least five minutes, when it stops, then the cycle records, for each track — local, the microphone; remote, what Windows plays: whether it holds the call's speech, and what Windows asked before it recorded — read against D39's threshold: both tracks hold the call's speech as the colleague hears it at the start, the middle and the end of each, and capture's code holds no line added for Windows; below it, what was missing is recorded for the next framing. A refuted result is a delivered one"
  - id: D86
    title: "Wholeheard becomes Kitokey, and nothing of the user's is lost on the way"
    module: shell
    state: accepted
    acceptance:
      - "Given Wholeheard installed and used — settings, meetings, the Whisper model, keys in the keyring —, when the user installs Kitokey's package over it (`dnf install` of the .rpm, `apt install` of the .deb), then it replaces Wholeheard's, and Kitokey starts with the same settings, the same meetings where they were, the model not fetched again and the keys kept; a carrying over cut halfway resumes at the next start and never loses a meeting"
      - "Given the repository, when it is searched for Wholeheard, then the product, its programs, its packages, its application id, its folders and its variables are named Kitokey; Wholeheard and Summit are named only where history is told and in the code that carries over what the user held under them"
      - "Given the window, the tray, the applications menu and the notifications, in French and in English, when they are read, then they say Kitokey; the promise's sentence is PDR-0001's, unchanged"
      - "Given D78, when its source package is made and built, then the package is `kitokey` and the Copr project `kitokey`: `dnf copr enable <owner>/kitokey` then `dnf install kitokey`"
  - id: D78
    title: "On Fedora, Wholeheard installs from Copr with dnf, and each new version comes with the system's updates"
    module: install
    state: accepted
    acceptance:
      - "Given a release tag on main, when the forge's workflow runs, then it makes a source package from that tag that builds the four programs from source, the crates inside it, and Copr builds it for each Fedora release Copr still supports, its internet access left off; a build that fails fails the workflow and names the release that failed"
      - "Given the package's header, when it is read, then its licence is the repository's, GPL-3.0, and its summary and description are those of today's package"
      - "Given a clean Fedora 44 container, when `dnf copr enable <owner>/wholeheard` then `dnf install wholeheard` run, then Wholeheard is installed, each program answers `--version` with the tag's version, and D80's removal holds on it"
      - "Given the previous version installed from Copr, when the next tag is built and `dnf upgrade` runs, then the new version replaces it and the meetings and settings stay"
  - id: D66
    title: "A recording holds the microphone the call holds, and never touches a Bluetooth headset's profile"
    module: capture
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given a call holding a microphone other than the default input, when a recording runs, then `me` records that one — its name in `session.json` and the `device` event — and, when the call changes it, follows it, the track going on; held on virtual devices by capture's e2e, a stand-in call on a source of its own"
      - "Given a source carrying `bluez5.loopback=true`, when capture records it, then its stream says its version's passive value (ADR-0015, as corrected by #574, merged after this amendment), the source never runs from it alone and no profile switch comes from it, within one known window: a default input turning into a headset during a recording stays on an active stream until a reading of the graph closes it, within a few seconds, and for as long as the graph does not read; in that window WirePlumber's decision to switch came 4 times in 10 measured, on a stand-in with no card; held by capture's e2e on a stand-in source, read in `pw-dump` — WirePlumber's decision testable on a stand-in carrying `device.id` —, and `others` is unchanged. Given no call holding a microphone, then capture records the meeting's sound and says in one sentence that the microphone waits when the default input is such a source; records the default input, as today, when it is not"
      - "Given a call that lets go of the microphone, when it leaves, then capture lets go of it too: within what WirePlumber's restore takes, it holds no HFP profile by a stream of its own and the source no longer runs — held by the same e2e, the source's state in `pw-dump`, and by the decider's trial, the card's profile"
      - "Given a transport failure during a recording — a source taken away, on the e2e's stand-in —, when it fails, then the recording continues, the track's gap is marked as `session-audio/v2` allows (`clock.jsonl`'s `silence`, `changed` with `device: null`), and `recording/v1` says it as its page says now, by `device` with `null`; a reason in the event is D73's, in *Later*; the shell's words are in their own pull request"
      - "Given the decider's AirPods and a real WhatsApp Web call, when they start and stop a recording during it, at least five times, then the call is never cut, no `Failure in Bluetooth audio transport` is logged within two seconds of capture opening or closing a stream, and `local.wav` holds their voice each time; an acceptance issue says what was seen"
  - id: D56
    title: "The list of models read again is refused unless the decider's key signed it"
    module: shell
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given `registry/models.json` and its signature `models.json.minisig` on main, made with minisign by the decider's key — the public key built into the shell, the private one never in a repository —, when the list is read again on the user's gesture, then it is offered only when its signature verifies; a list unsigned, signed by another key, or changed after it was signed is refused, and the shipped list stays, said in one sentence (ADR-0010, *Rejected alternatives*)"
      - "Given a pull request that changes the list, when its checks run, then a list whose signature does not verify against the built-in key is refused; until the decider's key is given, no list is read again, and Settings and the check say it waits on that key — the colleagues' builds lose today's reading again, said in the cycle's closure"
      - "Given ADR-0010, when D56 is merged, then the record says its option 4 adopted, `minisign-verify` declared in the shell's manifest with its exit strategy, and what a signature does not stop: an older list, signed, offered again"
      - "Given the shell's tests, when they run, then a list signed by a key made for the test is offered, and the three refusals above are held, without the decider's key"
  - id: D41
    title: "On Linux, a call heard on an output other than the system's default is recorded there"
    module: capture
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given a call — an application other than capture holding a microphone — whose sound plays on an output other than the system's default, when it is recorded, then the remote track holds what the call plays, and capture says `device` with that output's name"
      - "Given the call's sound moved to another output while recording, or back to the default, when it moves, then the remote track follows it within two seconds and the move is kept in `session.json` with when"
      - "Given no call, a call whose sound plays on the default, or on several outputs at once, when it is recorded, then the remote track records the system's default output, as D31 does"
      - "Given a graph that cannot be read or a stream that cannot be moved, when it is recorded, then the recording goes on from the default, and one warning is logged"
      - "Given capture's e2e, when a stand-in call plays a note on an output of its own that is not the default, then the remote track holds that note — on devices made for the run, nobody recorded"
  - id: D27
    title: "Wholeheard installs on Ubuntu from one package, its programs built on the system they ship to"
    module: install
    state: deferred   # closed early, 2026-11-13: a person's trial, key or reading left; its gesture in cycle 10's What needs a person
    acceptance:
      - "Given Ubuntu 24.04, when a user installs Wholeheard from one package file (`sudo apt install ./kitokey_<version>_amd64.deb`), then Wholeheard is in the applications menu and starts in the tray with nothing built and no variable set, and removing it leaves the user's meetings and settings"
      - "Given a release for a system — Fedora or Ubuntu —, when the programs are built, then they are built in a container of that system, so that no package depends on the machine that made it"
outcome: shipped
ended_on: 2026-11-13
---

# Cycle 09 — Kitokey finished before it is published

> The framework's words: `.nstack/handbook/glossary.md`.

> **Accepted by the decider on 2026-11-07**, on their word that day in the session that published Kitokey on
> Copr: « je te laisse le poste libre 1h, avance en autonomie le projet, mets a jour au fur et a mesure. nous
> avions mis en place un plan a afaire avant la publication, enchaine ». Each open question below is settled by
> its default. Started at [cycle 08's closure](08-wholeheard-ready-for-the-microsoft-store.md#closure), the
> same day, which moved eleven deliverables here.
>
> **Proposed by the framer on 2026-11-06**, on the decider's word that day — « analyse le marché, les forces,
> les faiblesses, les tendances pour avoir le meilleur produit et cadre le cycle » —, after their feedback of
> the same day, given from the package installed on their Fedora (*Goal* below). Its source is
> [the review of 2026-11-06](../reviews/2026-11-06-market.md), by an independent session. It stays
> `proposed` until the decider accepts it: one accepted cycle at a time, cycle 08 running. It takes the place
> cycle 08's *Later* gave macOS, which becomes cycle 10.

## Goal

The decider's feedback of 2026-11-06, in their words, the source of every deliverable:

| # | The decider said | What was there | Here |
|---|---|---|---|
| 1 | « gérer une identité graphique propre … les boutons comme "Démarrer un enregistrement" sont trop grands … des icônes adaptées » | Five Lucide icons and a checked contrast (cycle 06, D69); no mark, no rule of size | D89 |
| 2 | « les sons d'origine sont volumineux, à gérer avant de commencer à publier … compressé sans perdre en qualité » | 2.07 GB an hour as written, about 1.0 as FLAC (cycle 02, D9); compressing deferred by every cycle since | D87 |
| 3 | « la fenêtre compacte est trop intrusive, trop grande (le last meeting n'a pas trop d'intérêt) … juste une "barre" avec l'essentiel » | The small window, 380 × 400 (D34, D35), filled since by D49, D50, D59, D60, D74 | D90 |
| 4 | « pouvoir tester les sources micro et l'audio avant de lancer un enregistrement » | Meters while recording only (D22, D49) | D88 |
| 5 | « le "contexte", ce serait utile de pouvoir le noter avant de lancer l'enregistrement » | D59's field, while recording and after | D91 |
| 6 | « la transcription "live" lorsque c'est possible » | Their idea of 2026-10-26; D52 framed in cycle 05, moved to *Later* twice, never run | D52 |
| 7 | « une config "default" mais choisir les modèles de chaque étape pour chaque réunion (un bouton "avancé") » | Asked on 2026-10-27 (D60) and 2026-10-29 (D65); answered then: no page of settings per meeting; *Refaire…* changes the model after | D91 |

**Why now.** Each is seen from the first day by whoever installs Kitokey: a meeting of an hour that takes two
gigabytes, a meeting lost because a side was never heard — the botless recorder's own failure, which nobody
checks before the call ([review §1](../reviews/2026-11-06-market.md)) —, a window that takes a corner of the
screen during the call. Copr and the Store publish to people who never met the decider: it comes before.

```mermaid
flowchart LR
    R["Drawings, one round<br/>D89 + D90"]:::draw --> A{"Decider reads"}:::person --> I["D89 the look"]:::build --> B["D90 the bar"]:::build --> M["D91 context and models<br/>before the start"]:::build
    L["capture listens<br/>(D88, its own PR)"]:::build --> P["D88 Vérifier le son"]:::build --> B
    F["readers of FLAC<br/>(D87, first)"]:::build --> W["capture writes FLAC<br/>(D87)"]:::build
    S["D52 spike, live"]:::spike
    classDef draw fill:#92400e,color:#fff
    classDef person fill:#1e3a8a,color:#fff
    classDef build fill:#065f46,color:#fff
    classDef spike fill:#374151,color:#fff
```

**Legend** — orange: drawn first · blue: the decider's reading · green: built, arrows the order · grey: a
spike, alone, cut first.

## Deliverables

### D87

- ADR-0004's plan since cycle 02: the tracks written as delivered while recording — a growing WAV stays
  readable if anything stops —, compressed without loss once stopped. FLAC is the field's convention for
  lossless audio; nobody else read keeps a lossless master ([review §3](../reviews/2026-11-06-market.md)):
  it is what lets a meeting be heard again and made again, the promise Granola's deleted audio cannot keep.
- Lossy (Opus) is not this deliverable: the decider asked for no loss, ADR-0004 refuses it at capture, and no
  study of Whisper on Opus in French meetings was found. Its bench and a compact copy of old meetings are
  *Later*.
- The order is a contract's — `session-audio/v3`, as `v2`'s page says a compressed track would be, and `recording/v1`'s page for the event that says the tracks are final: the readers first — transcription, the shell (playback, waveform), export,
  the bench —, then capture. The encoder is a new dependency: `security.md`, its exit strategy written.
  FLAC plays in WebKitGTK and WebView2; WKWebView is cycle 10's to check *(review, unverified)*.

### D88

- The one check no rival does: whether the call's sound is captured before the call
  ([review §1](../reviews/2026-11-06-market.md)). Zoom, Meet and Teams test the microphone inside a window
  the user opened; the convention kept here: the microphone opened only while the panel shows — GNOME's and
  macOS's indicator then light for a reason the user just gave —, never a meter at rest, never at each start.
- ADR-0015 opens the microphone passively so that a recording never switches a headset's profile: a passive stream on a source no call runs hears nothing, so a test before the call must open it. The departure is limited to the panel, the user's gesture, before a call; it is written in ADR-0015 before it is built.
- capture's command to listen is an addition to `recording/v1`'s page, in capture's own pull request; the
  shell's in its own. The test sound is never played while an application holds a microphone: on
  loudspeakers it would reach the call.
- The first run's assistant (D58) may offer it; whether it does is the drawing's.

### D89

- No agency, no custom font, no component library: the system's font, Lucide alone, one accent, a 4 px grid,
  a monogram drawn once in SVG ([review §6](../reviews/2026-11-06-market.md)). What users of the leaders
  complain about is intrusion, rarely looks: playful by motion on a change of state, never by sound.
- One round of drawings with D90, in a folder of its own, frozen; the decider reads it before either is
  built (*Dependency with a date*). The author draws; the decider's word is the bar.
- The trademark search for Kitokey (cycle 08's *Later*) comes before the mark is published.

### D90

- It replaces the small window (D34, D35) while recording and at rest. The leaders converge on one live sign,
  one way back, a drag, and a way to hide it; what users resent is a bar that comes back once hidden (Zoom) or
  cannot be turned off (Teams) ([review §2](../reviews/2026-11-06-market.md)).
- GNOME under Wayland lets no application keep itself above or place itself ([tauri #3117](https://github.com/tauri-apps/tauri/issues/3117)):
  there the tray's icon is the always-visible sign where the AppIndicator extension gives a tray (cycle 03: a Fedora without it has none), GNOME's microphone indicator otherwise, and no extension or XWayland is used to force it.
- Today's `always_on_top` while recording (`modules/shell/src/main.rs`) has no effect there; the drawing says
  what the bar is on each system.

### D91

- The decider's question asked a third time: answered now with one closed disclosure, not a page of settings.
  The review advises no choice at the start ([§5](../reviews/2026-11-06-market.md)); the departure is the
  decider's, three times: the disclosure is in the opened bar, never on the start button, and the same one is
  in *Refaire…*.
- The context typed before the start is D59's field moved earlier: its rules unchanged.

### D52

- Framed in cycle 05 (#461), moved to *Later* twice; its section there holds its reasons. Kyutai's
  `stt-1b-en_fr`, the one open streaming model found that hears French ([review §4](../reviews/2026-11-06-market.md)),
  added as a second candidate. The market's design leader moved live AI aside (Granola); the decider's idea is
  measured before anything is built.
- A measure whose product is a record, outside the shipped program: `module: null`, held by a line of the transcription's log issue (#129). Its program is kept outside the repository, or merged without carrying the deliverable.
- On devices made for the run, while the decider is not on a call. **Cut first** when the breaker nears.

### Moved from cycle 08

D84, D85, D54, D57, D53, D86, D78, D66, D56, D41 and D27, at cycle 08's closure, on the decider's word of
2026-11-07. Their criteria are copied as they stand, their sections are in
[cycle 08](08-wholeheard-ready-for-the-microsoft-store.md) and the cycles before it, and *What needs a person*
below gives each its gesture. D78's criterion 4 is held by the next tag, which carries D87; D86's criterion 1 by
the decider's word on the rpm installed over Wholeheard, given on 2026-11-08. They do not count against the appetite: the cut order
below is unchanged, and the end date does not move for them.

| Deliverable | What holds it, made before this cycle or by its release |
|---|---|
| D78 | #657, #658, #659, #661, #662 (cycle 07: each program's `--version`, the source package and `copr.yml`), #686 (the group's project). Criterion 4, `dnf upgrade` from 0.2.0: the tag v0.2.1's run [42417841678](https://github.com/NapkinStack/kitokey/actions/runs/42417841678) — « upgraded with dnf, kitokey 0.2.0 to 0.2.1 », « the meetings and the settings stay as they were » —, its version set by #711 |
| D86 | Criterion 1: the decider's word, 2026-11-08, in the working session, on the rpm installed over Wholeheard on 2026-11-06 — « oui, mes réunions s'affichent » —, after an agent read on their disk the settings, the meetings, the model and the key file under Kitokey's names. Criterion 2: #648, #649, #650, #651, #652, #653, #654, #663, #664, #665, #666. Criterion 3: #651, #654. Criterion 4: #652, #686 |

## What needs a person

| Deliverable | Who | The literal gesture |
|---|---|---|
| PDR-0005 | The decider | Read [PDR-0005](../../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md) and say « PDR-0005 acceptée », or what changes |
| D57 | The decider | Read [`docs/store/listing.md`](../../store/listing.md); say « l'annonce ne promet rien de faux », or which line does |
| D54, submission | The decider | In [Partner Center](https://partner.microsoft.com/dashboard), with a company's account: reserve the product's name and give an agent the identity Partner Center shows. Set *Free trial* to **Unlimited** (PDR-0005). Read the Store's licence terms beside GPL-3.0, answer the age questionnaire, and submit the package an agent built from that identity |
| D53, D54 on Windows, D84 | The colleague on Windows 11 | [#482](https://github.com/NapkinStack/wholeheard/issues/482), updated first by an agent with the run's link: the package signed for the run; a real call of five minutes; then sign out and in again: Kitokey in the notification area, no window. Answered there |
| D66, trial | The decider | On a build of `main`: AirPods, a WhatsApp Web call, five starts and stops, the answers on [#494](https://github.com/NapkinStack/kitokey/issues/494); then « ADR-0015 acceptée », or what falls |
| D41 | The decider | [#449](https://github.com/NapkinStack/kitokey/issues/449): a call on the earbuds, the default on the speaker |
| D56 | The decider | As [cycle 07](07-wholeheard-from-the-systems-updater.md#what-needs-a-person) says, in a terminal of your own, never through an agent: a minisign key, its `.pub` alone given to an agent |
| D27 | Anyone on Ubuntu 24.04 | `nstack claim D27`, then [#255](https://github.com/NapkinStack/kitokey/issues/255): the `.deb` of a build of `main`, one real call of five minutes, the answer there |
| D89, D90 | The decider | First round [read on 2026-11-08](../drawings/d89-d90-kitokey-look-and-bar/README.md#what-the-decider-answers): second round [read the same day](../drawings/d89-d90-mark-and-bar-buttons/README.md#what-the-decider-answers): third round [read the same day](../drawings/d89-d90-mark-k-and-i/README.md#what-the-decider-answers): the mark [accepted in the sixth](../drawings/d89-d90-mark-k-stem/README.md#what-the-decider-answers), the name [in the seventh](../drawings/d89-d90-name-kitokey/README.md#what-the-decider-answers), the tray's icon on its tile, red while recording (« ok C », 2026-11-08, in the working session): done |
| D88 | The decider | Before a real call, on the AirPods: Kitokey → the bar → « Vérifier le son »; say in the acceptance issue whether both sides moved |

## Dependency with a date

- **The decider's reading of D89–D90's drawings**, within two working days of their pull request. Missed,
  D87, D88 and D52 go on; D89, D90 and D91's bar wait.

- **Moved from cycle 08**: an agent remakes the Windows builds from `main` and updates #482 within 48 hours
  of this acceptance — missed in cycles 06 to 08; the decider's Partner Center account, a company's, asked
  within 5 days. Missed, D53, D54 and D84's first criterion stay where they are.

## Out of scope

- Lossy audio at capture; a format picker; 16-bit tracks to save room (D1's weak voice).
- A meter or a microphone open at rest; a test at each start; Zoom's record-and-replay.
- The live preview built — D52 measures it; a live AI panel over other applications.
- A model picker on the start button; a router choosing models by itself; meeting types.
- A GNOME Shell extension, or XWayland, to keep the bar above.
- A component library, a custom font, a mascot, a theme picker.
- macOS: a later cycle's ([cycle 10's *Later*](10-kitokey-ready-when-you-come-back.md#later), framed 2026-11-13).

## Later

- **macOS**: D38, D39, D42, D58 on a Mac, ADR-0012's reading, notarisation, FLAC in WKWebView —
  everything cycle 08's *Later* gave cycle 09; moved to [cycle 10's *Later*](10-kitokey-ready-when-you-come-back.md#later)
  when cycle 10 was framed on the copy and UX review, 2026-11-13.
- **Opus for old meetings**: a bench first — the trial meeting through today's engine from FLAC and from Opus
  32 and 48 kbit/s —, then, only if it holds and the decider amends ADR-0004, a compact copy after N days
  (about 43 MB an hour) ([review §3](../reviews/2026-11-06-market.md)).
- Meetings recorded before D87 compressed from « Faire de la place ».
- **FLAC written while recording**, in place of a WAV compressed at the stop: only if users' feedback says
  the room a recording takes while it runs matters — about 1 GB an hour a track in 24-bit stereo. A new
  version of `session-audio` and its ADR first: a WAV is readable while written and whole to its last frame
  after a crash, which `session-audio/v2` promises. The decider, 2026-11-07: « on y reviendra si le besoin
  de place remonte par les retours utilisateur ».
- The live preview built, on D52's record and its ADR.
- Meeting types, each with its context and its models (market review of 2026-10-26, 8).
- Everything in cycle 08's *Later*, unchanged.

## Open questions

**Blocking:** none. Each has a default the deliverable is built with.

**Not blocking**, each *(framer, to confirm)*:

- **The appetite**: two weeks. Default: two, never stretched; cut in this order when the breaker nears: D52, D91 whole, then D89's third criterion (the packages' icons).
- **The start**: settled — cycle 08 closed early on 2026-11-07 and moved its eleven deliverables here.
- **Copr's first tag**: settled by its default — `v0.2.0` published on 2026-11-07 without waiting for D87,
  which ships in the next tag.
- **D91 against the review**: a choice before the start is the decider's word against the review's advice.
  Default: built, behind the closed disclosure.

## Closure


**Closed on 2026-11-13, early, shipped, on the decider's word that day:** « on enchaine des que possible ».
[Cycle 10](10-kitokey-ready-when-you-come-back.md), accepted the same day, starts at this closure. One
accepted cycle at a time, so this one closes first, a week before its end. D78, D86, D87, D88 and D91 are
accepted from their records by `nstack deliver --write-states`. As cycle 10 says, the twelve others are
deferred to this cycle's *Later*, never moved there. Their gestures for a person are carried into cycle 10's
*What needs a person*. 5 of 17 accepted.

**D88, the decider's answer** (criterion 4), in the working session on 2026-11-13: « ma dernier reunion été
vraie et faite avec les airpods, le son et le micro sont bon » and « oui, les deux jauges bougeaient ».

**Merged, not delivered.** The criteria are read at `main` 958404b. Each one waits for its holder's
`nstack deliver`, or for the next framing.

| Deliverable | Criterion | Holds? | Pull requests |
|---|---|---|---|
| D89 the look | 1 the drawings read | Yes, the mark and the name accepted in rounds 6 and 7 | #712, #720 to #726 |
| | 2 Lucide, sizes, one accent, contrast in both themes | Yes, by the look's tests (`tests/look.rs`) | #730, #731, #732, #733, #737 |
| | 3 icons from one SVG, the tray's two states | Yes. The README's line on GNOME's microphone indicator is not found | #727, #729 |
| | 4 screenshots in both themes at 1920 and 420 px | Made; not checked in a sheet beside the boards | #738 |
| D90 the bar | 1 the drawings read | Yes | #712, #720 to #726 |
| | 2 recording and opened | Merged; read from the pull requests, not run again here | #740, #743, #752, #765, #768 |
| | 3 above the other windows on Windows and macOS; on GNOME an ordinary window | Not proven on Windows or macOS | #744 |
| | 4 hidden, never back by itself; the shortcut stops | Yes | #744 |
| D84 the startup task | 1 started at sign-in, listed by Windows | No: a person on Windows 11 (#795) | — |
| | 2 Settings show and change the task's state | Yes in unit tests; on Windows, a person's | #790 |
| | 3 the zip offers no switch, no `autostart` file; Linux unchanged | Yes in unit tests | #790 |
| | 4 each state mapped; `trial.ps1` records it | First half yes; second half open | #790; #788 and #791 open |

**Deferred with a reason, one line each:**

- D41, D66 (trial), PDR-0005, D57, D53, D54 and D84 on Windows, D56, D27: a person's trial, key or reading.
  Each gesture is unchanged, in cycle 10's *What needs a person*.
- D89 and D90: the decider's acceptance of the drawings is done. What is left is the table above.
- D52 and D85: agents' spikes, not started; D52 is cut first, as the cut order says.

**Out-of-cycle pull requests** #786, #787, #788, #791, #792, #793, #794, #796, #798 and #800 are unaffected:
none carries a deliverable of this cycle's, and each goes on as it is.

**The goal.** Partly reached: a meeting is kept as FLAC (D87), both sources are tried before a recording (D88),
and the context and models are chosen before it (D91). The look and the bar (D89, D90) are merged, but not
every criterion is proven.

**Success criteria**: none moved.

**Documents brought current:** none by this closure. Cycle 10's front matter: `accepted`, starting today.
