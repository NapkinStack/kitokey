# Kitokey's package for Fedora, made from the artefacts each module's release left in dist/.
# rpmbuild finds the libraries every program links and requires their packages itself.
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
License:        LicenseRef-Proprietary
ExclusiveArch:  x86_64 aarch64
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

%install
install -Dm755 %{artefacts}/shell/kitokey %{buildroot}%{_bindir}/kitokey
install -Dm755 %{artefacts}/capture/kitokey-capture %{buildroot}%{_bindir}/kitokey-capture
install -Dm755 %{artefacts}/transcription/kitokey-transcription %{buildroot}%{_bindir}/kitokey-transcription
install -Dm755 %{artefacts}/summary/kitokey-summary %{buildroot}%{_bindir}/kitokey-summary
# Lighter without their symbols; a program that is no binary is left as it is.
for program in %{buildroot}%{_bindir}/*; do strip "$program" 2>/dev/null || :; done
install -Dm644 %{artefacts}/shell/com.kitokey.Kitokey.desktop %{buildroot}%{_datadir}/applications/com.kitokey.Kitokey.desktop
install -Dm644 %{artefacts}/shell/com.kitokey.Kitokey.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/com.kitokey.Kitokey.png

%files
%{_bindir}/kitokey
%{_bindir}/kitokey-capture
%{_bindir}/kitokey-transcription
%{_bindir}/kitokey-summary
%{_datadir}/applications/com.kitokey.Kitokey.desktop
%{_datadir}/icons/hicolor/512x512/apps/com.kitokey.Kitokey.png
