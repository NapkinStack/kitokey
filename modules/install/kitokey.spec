# Kitokey's package for Fedora. Two ways, one header: given `artefacts`, it packs what each module's
# release left in dist/ (scripts/release.sh); not given, it builds the four programs from the source
# package copr/srpm.sh makes — the crates vendored in it, no network —, as Copr does (D78).
# rpmbuild finds the libraries every program links and requires their packages itself.
%if %{undefined artefacts}
%global artefacts dist
%global from_source 1
# Built in release mode, without symbols to gather: no debuginfo package.
%global debug_package %{nil}
%endif
Name:           kitokey
Version:        %{kitokey_version}
Release:        1%{?dist}
# Wholeheard became Kitokey (D86), as Summit became Wholeheard (D28): installing Kitokey takes the
# place of either; the user's settings, meetings and keys are carried over by Kitokey itself at its
# first start. Every Wholeheard ever made is below 0.3.0.
Obsoletes:      wholeheard < 0.3.0
Provides:       wholeheard = %{version}-%{release}
Obsoletes:      summit < 0.2.0
Provides:       summit = %{version}-%{release}
Summary:        Records a meeting, transcribes it and writes its minutes, on your machine
License:        GPL-3.0-only
ExclusiveArch:  x86_64 aarch64
%if 0%{?from_source}
Source0:        kitokey-%{version}.tar.xz
# What each module's containers/fedora-*.Containerfile builds it with, Rust from Fedora's.
BuildRequires:  python3 cargo rust gcc gcc-c++ make cmake clang clang-devel pkgconf-pkg-config
BuildRequires:  webkit2gtk4.1-devel gtk3-devel libayatana-appindicator-gtk3-devel librsvg2-devel
BuildRequires:  pipewire-devel alsa-lib-devel vulkan-headers vulkan-loader-devel glslc
%endif
# Run, not linked: capture asks PipeWire which device the default stands for.
Requires:       pipewire-utils
# Opened at run time, never linked: the shell's tray icon (ayatana's first, the older one else).
Requires:       (libayatana-appindicator-gtk3 or libappindicator-gtk3)
# GNOME shows a tray icon only with this extension.
Recommends:     gnome-shell-extension-appindicator
# The graphics card's Vulkan driver: the transcript made there, a tenth of the meeting's length;
# without it, on the processor. Installed on a desktop already.
Recommends:     mesa-vulkan-drivers

%description
Kitokey records both sides of a call from the system tray, transcribes it on the machine with
Whisper, and writes its minutes with the model you choose. Your meetings stay in your folder.

%if 0%{?from_source}
%prep
%autosetup

%build
# Each program as its module releases it (`commands.release`, as `nstack release` runs it), into
# dist/<module>: assemble.sh, written by copr/srpm.sh. Each says the package's version (`--version`).
KITOKEY_VERSION=%{version} CARGO_NET_OFFLINE=true bash assemble.sh
%endif

%install
install -Dm755 %{artefacts}/shell/kitokey %{buildroot}%{_bindir}/kitokey
install -Dm755 %{artefacts}/capture/kitokey-capture %{buildroot}%{_bindir}/kitokey-capture
install -Dm755 %{artefacts}/transcription/kitokey-transcription %{buildroot}%{_bindir}/kitokey-transcription
install -Dm755 %{artefacts}/summary/kitokey-summary %{buildroot}%{_bindir}/kitokey-summary
# Lighter without their symbols; a program that is no binary is left as it is.
for program in %{buildroot}%{_bindir}/*; do strip "$program" 2>/dev/null || :; done
install -Dm644 %{artefacts}/shell/com.napkinstack.Kitokey.desktop %{buildroot}%{_datadir}/applications/com.napkinstack.Kitokey.desktop
install -Dm644 %{artefacts}/shell/com.napkinstack.Kitokey.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/com.napkinstack.Kitokey.png

%files
%if 0%{?from_source}
%license LICENSE
%endif
%{_bindir}/kitokey
%{_bindir}/kitokey-capture
%{_bindir}/kitokey-transcription
%{_bindir}/kitokey-summary
%{_datadir}/applications/com.napkinstack.Kitokey.desktop
%{_datadir}/icons/hicolor/512x512/apps/com.napkinstack.Kitokey.png
