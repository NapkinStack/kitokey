# install

What this module is for, who owns it and how critical it is: its
[MANIFEST.yaml](./MANIFEST.yaml), the one place they are written.

```mermaid
flowchart LR
    M["shell · capture · transcription · summary<br/>each module's release"]:::mod --> D["dist/&lt;module&gt;/<br/>their programs, the desktop entry, the icon"]:::art
    D --> B["install's release<br/>rpmbuild or dpkg, in a container<br/>of the system KITOKEY_TARGET names;<br/>the Mac's app on a Mac"]:::this
    B --> R["dist/install/<br/>kitokey-‹version›.x86_64.rpm · kitokey_‹version›_amd64.deb ·<br/>Kitokey-‹version›-macos-arm64.zip, made on a Mac ·<br/>Kitokey-‹version›-windows-x64.zip and -x64.msix, made on Windows"]:::out
    classDef mod fill:#374151,color:#fff
    classDef art fill:#92400e,color:#fff
    classDef this fill:#065f46,color:#fff
    classDef out fill:#1e3a8a,color:#fff
```

**Legend** — grey: the modules whose programs it packs · amber: their release artefacts ·
green: this module · blue: what a user installs. It provides and consumes no contract; why it
exists: ADR-0006.

## Getting started

Needs `podman`. From the repository's root:

```bash
nstack release                                    # for this Fedora: every module, then the rpm
sudo dnf install ./dist/install/kitokey-*.x86_64.rpm

KITOKEY_TARGET=ubuntu-24.04 nstack release         # each program built in an Ubuntu container, then the .deb
sudo apt install ./dist/install/kitokey_*_amd64.deb
```

`KITOKEY_TARGET` names the system the package is for — `fedora-44`, `ubuntu-24.04` — for every
module at once: each program is built in a container of that system, so that it links what that
system has (D27).

Removing it keeps what is yours (D80). `bash modules/install/scripts/e2e.sh <the package>`, with
`KITOKEY_TARGET` naming its system, installs it in a throwaway container, starts Kitokey as a user,
writes a meeting, removes the package, then installs it again:

```mermaid
flowchart LR
    I["installed"]:::step --> F["first start, as a user<br/>a meeting written"]:::step --> R["removed"]:::step
    R --> C{"its files gone ·<br/>the home as it was ·<br/>new there: privacy.md's paths"}:::check
    C --> A["installed again,<br/>started: the meeting listed,<br/>no welcome again"]:::check
    classDef step fill:#374151,color:#fff
    classDef check fill:#065f46,color:#fff
```

**Legend** — grey: what the e2e does · green: what it holds. The paths Kitokey may leave are read
from [`docs/privacy.md`](../../docs/privacy.md)'s Linux line.

## On Fedora, from Copr (D78)

```bash
sudo dnf copr enable @napkinstack-labs/kitokey
sudo dnf install kitokey          # each new version then comes with the system's updates
```

```mermaid
flowchart LR
    T["a tag v‹version›<br/>on main"]:::step --> S["copr/srpm.sh<br/>the sources at the tag,<br/>their crates vendored"]:::step
    S --> B["copr/build.sh<br/>Copr builds it, no network,<br/>each Fedora it supports"]:::copr
    B --> U{"the version Copr held,<br/>upgraded with dnf ·<br/>the meetings stay"}:::check
    B --> E{"installed from Copr,<br/>each release: D80's e2e,<br/>each program's --version"}:::check
    classDef step fill:#374151,color:#fff
    classDef copr fill:#1e3a8a,color:#fff
    classDef check fill:#065f46,color:#fff
```

**Legend** — grey: made by the forge's workflow · blue: Copr · green: what it holds.

`.github/workflows/copr.yml` runs on a tag `v‹version›` on `main`, the shell's version. With no
artefacts given, `kitokey.spec` builds from the source package (`from_source`): each module this
one assembles released by its own `release` command, as `nstack release` does (`assemble.sh`,
written by `srpm.sh`), each program saying the tag's version (`--version`, `KITOKEY_VERSION`); it
ships `LICENSE`. The project
`@napkinstack-labs/kitokey` belongs to a Fedora group, so that installing names no person; the token is
the forge's secret `COPR_CONFIG`. Copr builds on x86_64 only: transcription's whisper.cpp is built
for x86-64-v3 (`modules/transcription/.cargo/config.toml`). By hand, from the repository's root:

```bash
bash modules/install/copr/srpm.sh v0.2.0 0.2.0 dist/copr             # the source package, in a Fedora container
KITOKEY_TARGET=fedora-44 KITOKEY_VERSION=0.2.0 bash modules/install/scripts/e2e.sh copr:@napkinstack-labs/kitokey
```

On a Mac, `nstack release` makes `dist/install/Kitokey-<version>-macos-arm64.zip`: the app,
its four programs in `Contents/MacOS` and transcription's `kitokey-speech` (D42), signed ad hoc, not by Apple. GitHub's macOS runner makes
it on every change to a program on `main` (`.github/workflows/macos-app.yml`, the job *the app*; on a pull request only when it changes install) and keeps it 7
days on the run. Opened the first time through System Settings → Privacy & Security → Open
Anyway (D38).

On Windows, `nstack release` also makes `dist/install/Kitokey-<version>-x64.msix`, the package the
Microsoft Store takes (D54, PDR-0004), unsigned; it needs the Windows SDK. `windows-app.yml` signs it
for the run with a certificate it throws away, installs it, upgrades it and removes it
(`windows/trial.ps1`) — at each start, it prints what the Store answers for a package it did
not install (`shell.licence_read`, D83), and fails if that read would refuse a recording —, has the Windows App Certification Kit test it (`windows/certify.ps1`, each
failed test named in the run), and keeps it 7 days with the public certificate and the kit's report,
`appcert-report.xml`: a colleague runs, as
administrator, `Import-Certificate .\Kitokey-this-run.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople`,
then opens the package. What certification reads: [`windows/certification.md`](./windows/certification.md).
