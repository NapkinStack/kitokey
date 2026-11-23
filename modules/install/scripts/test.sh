#!/usr/bin/env bash
# The spec, the .deb's recipe and the Mac's app, read: every file it installs is in %files and the other way round, the desktop entry
# carries the application's id (GNOME's shortcut portal finds Kitokey by it), and nothing is
# installed outside /usr — so removing the package never touches a user's meetings or settings.
set -euo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
python3 - "$here/kitokey.spec" "$here/deb/build.sh" "$here/deb/control.in" "$here/mac" <<'PY'
import json, os, plistlib, re, subprocess, sys, tempfile, zipfile
spec = open(sys.argv[1], encoding="utf-8").read()
macros = {"%{_bindir}": "/usr/bin", "%{_datadir}": "/usr/share", "%{buildroot}": ""}
def expand(path):
    for macro, value in macros.items():
        path = path.replace(macro, value)
    return path
installed = {expand(m) for m in re.findall(r"^install -D\S* \S+ (\S+)$", spec, re.M)}
# Its conditions and the licence, shipped when built from source (D78), are not files it installs.
files = {expand(line.strip()) for line in spec.split("%files", 1)[1].splitlines()
         if line.strip() and not line.startswith(("%if", "%endif", "%license"))}
failed = 0
# What every package owes the authors of what it ships, by its name there (D102): windows/notices.py.
OWED = ["LICENSE.txt", "MPL-SOURCES.txt"] + [f"THIRD-PARTY-NOTICES-{program}.html" for program in
        ("kitokey", "kitokey-capture", "kitokey-transcription", "kitokey-summary", "kitokey-export")]
def pages(artefacts):
    """A fixture's pages, as each module's release leaves its own."""
    for module in ("shell", "capture", "transcription", "summary", "export"):
        os.makedirs(os.path.join(artefacts, module), exist_ok=True)
        open(os.path.join(artefacts, module, "THIRD-PARTY-NOTICES.html"), "w").write(f"<p>{module}'s crates</p>")
def check(what, held, got=""):
    global failed
    print(f"  {'ok  ' if held else 'FAIL'}  {what}" + ("" if held else f"\n        got: {got}"))
    failed |= not held
check("what is installed is what %files lists", installed == files, installed ^ files)
for program in ("kitokey", "kitokey-capture", "kitokey-transcription", "kitokey-summary", "kitokey-export"):
    check(f"{program} in /usr/bin, beside the others", f"/usr/bin/{program}" in files)
check("the desktop entry is named as the application's id", "/usr/share/applications/com.napkinstack.Kitokey.desktop" in files)
check("its icon, under the same name", "/usr/share/icons/hicolor/512x512/apps/com.napkinstack.Kitokey.png" in files)
check("nothing outside /usr: the user's meetings and settings are never the package's", all(f.startswith("/usr/") for f in files), files)
check("its licence is the repository's, GPL-3.0", re.search(r"^License:\s+GPL-3\.0-only$", spec, re.M) is not None)
check("what it owes its third parties is in its licences' folder, from source or not (D102)",
      all(f"/usr/share/licenses/kitokey/{name}" in files for name in OWED), sorted(files))
check("pw-metadata required: capture runs it, no library names it", re.search(r"^Requires:\s+pipewire-utils$", spec, re.M) is not None)
check("the tray's library required: the shell opens it at run time, no link names it",
      re.search(r"^Requires:.*\blib(ayatana-)?appindicator-gtk3\b", spec, re.M) is not None)
pairs = re.findall(r"^install -D\S* %\{artefacts\}/\S+/(\S+) %\{buildroot\}%\{_bindir\}/(\S+)$", spec, re.M)
check("each program installs its own artefact", len(pairs) == 5 and all(a == b for a, b in pairs), pairs)
scriptlets = re.findall(r"^%(pre|post|preun|postun|pretrans|posttrans|trigger\w*)\b", spec, re.M)
check("no scriptlet: installing and removing touch nothing but the package's files", not scriptlets, scriptlets)
# The .deb (D27): made by deb/build.sh from the same artefacts — what it installs is judged on the
# package installed, by the e2e, against the spec's %files.
build = open(sys.argv[2], encoding="utf-8").read()
control = open(sys.argv[3], encoding="utf-8").read()
check("pw-metadata and the tray's library named in its dependencies: run and opened, never linked",
      re.search(r"pipewire-bin, libayatana-appindicator3-1 \| libappindicator3-1", build) is not None)
check("its control names the package, its version and its dependencies as computed",
      all(f"{field}: @" in control for field in ("Version", "Architecture", "Depends")) and "Package: kitokey" in control)
check("the .deb lays the same notices in its documentation's folder (D102)",
      all(re.search(rf'^install -Dm644 \S+ "\$root/usr/share/doc/kitokey/{re.escape(name)}"$', build, re.M) for name in OWED), build)
# Its copyright file (#1118): Debian Policy 12.5 — /usr/share/doc/kitokey/copyright, a plain file in UTF-8,
# saying where the sources are, the GPL referred to /usr/share/common-licenses —, in DEP-5 1.0's form.
# A minimised Ubuntu keeps it alone of that folder: it points to the notices beside it.
check("the .deb lays its copyright file, a plain file, in its documentation's folder (Debian Policy 12.5)",
      re.search(r'^install -Dm644 /deb/copyright "\$root/usr/share/doc/kitokey/copyright"$', build, re.M) is not None, build)
copyright_path = os.path.join(os.path.dirname(sys.argv[3]), "copyright")
try:
    copyright = open(copyright_path, encoding="utf-8").read()
except (OSError, UnicodeDecodeError) as error:
    copyright = ""
    print(f"        {copyright_path}: {error}")
stanzas = [dict(re.findall(r"^([A-Za-z-]+):[ \t]*(.*)$", stanza, re.M)) for stanza in copyright.strip().split("\n\n")] if copyright else [{}]
check("it is DEP-5 1.0: its Format line first, then a Files stanza with Copyright and License",
      copyright.startswith("Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\n")
      and any(s.get("Files") == "*" and s.get("Copyright") and s.get("License") for s in stanzas[1:]), stanzas)
check("it says where the sources are: the repository",
      stanzas[0].get("Source") == "https://github.com/NapkinStack/kitokey", stanzas[0])
check("Kitokey's licence, the GPL-3.0 only, referred to /usr/share/common-licenses/GPL-3",
      any(s.get("Files") == "*" and s.get("License") == "GPL-3" for s in stanzas)
      and "/usr/share/common-licenses/GPL-3" in copyright, stanzas)
check("it names each notice laid beside it, which carries its third parties' licences",
      all(f"/usr/share/doc/kitokey/{name}" in copyright for name in OWED if name != "LICENSE.txt"), copyright)
scripts = [f for f in os.listdir(os.path.dirname(sys.argv[3])) if f in ("preinst", "postinst", "prerm", "postrm", "triggers")]
check("no maintainer script: installing and removing touch nothing but the package's files", not scripts, scripts)
check("the graphics card's Vulkan driver recommended by both: the transcript is made there",
      re.search(r"^Recommends:\s+mesa-vulkan-drivers$", spec, re.M) is not None and "mesa-vulkan-drivers" in control)
check("Kitokey takes Wholeheard's place, and Summit's still: the rpm obsoletes and provides both, the .deb breaks and replaces both (D86, D28)",
      re.search(r"^Obsoletes:\s+wholeheard < 0\.3\.0$", spec, re.M) is not None
      and re.search(r"^Provides:\s+wholeheard = %\{version\}-%\{release\}$", spec, re.M) is not None
      and re.search(r"^Obsoletes:\s+summit < 0\.2\.0$", spec, re.M) is not None
      and re.search(r"^Breaks: summit \(<< 0\.2\.0\), wholeheard \(<< 0\.3\.0\)$", control, re.M) is not None
      and re.search(r"^Replaces: summit \(<< 0\.2\.0\), wholeheard \(<< 0\.3\.0\)$", control, re.M) is not None)
# The Mac's app (D38): made on a Mac by mac/build.sh; its Info.plist and French strings written
# here by the same mac/info.py, and read.
mac = sys.argv[4]
macbuild = open(os.path.join(mac, "build.sh"), encoding="utf-8").read()
for program in ("shell/kitokey", "capture/kitokey-capture", "transcription/kitokey-transcription", "summary/kitokey-summary",
                "export/kitokey-export",
                # Apple's SpeechAnalyzer, through transcription's Swift program (D42, ADR-0012).
                "transcription/kitokey-speech"):
    check(f"{program.split('/')[1]} in Contents/MacOS, beside the others", program in macbuild)
check("the Mac's app lays the same notices in its resources, before its seal (D102)",
      all(re.search(rf'^install -m644 \S+ "\$app/Contents/Resources/{re.escape(name)}"$', macbuild, re.M) for name in OWED)
      and all(macbuild.index(f'"$app/Contents/Resources/{name}"') < macbuild.index('codesign --force --sign - "$app"') for name in OWED))
each, bundle = "codesign --force --sign - \"$program\"", "codesign --force --sign - \"$app\""
check("signed ad hoc, each program then the bundle, last of all: its seal covers what is in it",
      each in macbuild and bundle in macbuild and macbuild.index(each) < macbuild.index(bundle)
      and all(macbuild.index(step) < macbuild.index(bundle) for step in ("info.py", "iconutil")))
with tempfile.TemporaryDirectory() as work:
    app = os.path.join(work, "Kitokey.app")
    subprocess.run([sys.executable, os.path.join(mac, "info.py"), app, "1.2.3"], check=True)
    info = plistlib.load(open(os.path.join(app, "Contents", "Info.plist"), "rb"))
    check("the Info.plist names the application, its id and its version",
          info.get("CFBundleName") == "Kitokey" and info.get("CFBundleIdentifier") == "com.napkinstack.Kitokey"
          and info.get("CFBundleShortVersionString") == "1.2.3", info)
    check("it opens the shell, the program that finds the others beside it", info.get("CFBundleExecutable") == "kitokey", info)
    words = json.load(open(os.path.join(mac, "words.json"), encoding="utf-8"))
    asked = ("NSMicrophoneUsageDescription", "NSAudioCaptureUsageDescription")
    check("macOS asks for the microphone and the sound the Mac plays with Kitokey's sentences",
          all(words["en"].get(key) and info.get(key) == words["en"][key] for key in asked), info)
    strings = open(os.path.join(app, "Contents", "Resources", "fr.lproj", "InfoPlist.strings"), encoding="utf-8").read()
    check("and in French, from the same file", all(words["fr"].get(key) and f'{key} = "{words["fr"][key]}";' in strings for key in asked) and
          info.get("CFBundleLocalizations") == ["en", "fr"], strings)
# Windows' zip (D53): made on Windows by windows/build.py, run here on artefacts of its names.
windows = os.path.join(os.path.dirname(mac), "windows", "build.py")
with tempfile.TemporaryDirectory() as work:
    artefacts, out = os.path.join(work, "artefacts"), os.path.join(work, "out")
    names = ("shell/kitokey.exe", "capture/kitokey-capture.exe",
             "transcription/kitokey-transcription.exe", "summary/kitokey-summary.exe", "export/kitokey-export.exe")
    for name in names:
        os.makedirs(os.path.join(artefacts, os.path.dirname(name)), exist_ok=True)
        open(os.path.join(artefacts, name), "wb").write(name.encode())
    pages(artefacts)
    os.makedirs(out)
    subprocess.run([sys.executable, windows, artefacts, "1.2.3", out], check=True, stdout=subprocess.DEVNULL)
    made = os.listdir(out)
    check("Windows' zip is named by its version and processor", made == ["Kitokey-1.2.3-windows-x64.zip"], made)
    inside = sorted(zipfile.ZipFile(os.path.join(out, made[0])).namelist()) if made else []
    check("the five programs side by side in one folder, where the shell looks for them, and what they owe beside them",
          inside == sorted([f"Kitokey/{name.split('/')[1]}" for name in names] + [f"Kitokey/{owed}" for owed in OWED]), inside)
    shell_page = zipfile.ZipFile(os.path.join(out, made[0])).read("Kitokey/THIRD-PARTY-NOTICES-kitokey.html") if made else b""
    check("each program's page is its module's, as its release left it", shell_page == b"<p>shell's crates</p>", shell_page)
# Windows' package (D54): made on Windows by windows/msix.py; its manifest and what it holds, read here.
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(os.path.dirname(mac), "windows"))
import msix
from xml.etree import ElementTree
ns = {"m": "http://schemas.microsoft.com/appx/manifest/foundation/windows10",
      "uap": "http://schemas.microsoft.com/appx/manifest/uap/windows10",
      "rescap": "http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"}
uap10 = "{http://schemas.microsoft.com/appx/manifest/uap/windows10/10}"
identity = msix.identity()
written = ElementTree.fromstring(msix.manifest("1.2.3", identity))
said = written.find("m:Identity", ns).attrib
check("the package's version in four parts, the last 0: the Store keeps it", said.get("Version") == "1.2.3.0", said)
for wrong in ("1.2", "1.2.3.4", "1.2.3-rc1", "70000.0.0"):
    try:
        msix.four_parts(wrong)
        check(f"a version not of three numbers under 65536 is refused: {wrong}", False)
    except ValueError:
        check(f"a version not of three numbers under 65536 is refused: {wrong}", True)
check("the identity named is the one identity.json holds, for x64",
      said.get("Name") == identity["name"] and said.get("Publisher") == identity["publisher"]
      and said.get("ProcessorArchitecture") == "x64", said)
check("Kitokey is its name", written.findtext("m:Properties/m:DisplayName", namespaces=ns) == "Kitokey")
# D84: the session can start Kitokey from its package, a startup task — off until the user turns it on in
# Settings (#1085): starting with Windows is asked, never assumed.
uap5 = "{http://schemas.microsoft.com/appx/manifest/uap/windows10/5}"
tasks = [(extension.attrib, extension.find(f"{uap5}StartupTask"))
         for extension in written.iterfind(f"m:Applications/m:Application/m:Extensions/{uap5}Extension", ns)
         if extension.get("Category") == "windows.startupTask"]
check("one startup task, for kitokey.exe, off until the user turns it on, named Kitokey",
      len(tasks) == 1 and tasks[0][0].get("Executable") == "kitokey.exe"
      and tasks[0][0].get("EntryPoint") == "Windows.FullTrustApplication" and tasks[0][1] is not None
      and tasks[0][1].get("TaskId") == "KitokeyStartupTask" and tasks[0][1].get("Enabled") == "false"
      and tasks[0][1].get("DisplayName") == "Kitokey", tasks)
placeholder = {**identity, "placeholder": True}
check("a placeholder identity is said, and that the package is not to be submitted",
      any("placeholder" in line and "not submitted" in line for line in msix.notices("1.2.3", placeholder)))
check("a reserved identity is not said to be one",
      not any("placeholder" in line for line in msix.notices("1.2.3", {**identity, "placeholder": False})))
check("a first part 0 is said: the Store refuses it", any("first part" in line for line in msix.notices("0.2.0", identity))
      and not any("first part" in line for line in msix.notices("1.2.0", identity)))
caps = written.find("m:Capabilities", ns)
check("runFullTrust and the microphone, and nothing else",
      [(c.tag.split("}")[1], c.get("Name")) for c in caps] == [("Capability", "runFullTrust"), ("DeviceCapability", "microphone")]
      and caps.find("rescap:Capability", ns) is not None, ElementTree.tostring(caps))
application = written.find("m:Applications/m:Application", ns).attrib
check("the shell started with its window, full trust",
      application.get("Executable") == "kitokey.exe" and application.get(uap10 + "Parameters") == "--window"
      and application.get(uap10 + "RuntimeBehavior") == "packagedClassicApp" and application.get(uap10 + "TrustLevel") == "mediumIL", application)
family = written.find("m:Dependencies/m:TargetDeviceFamily", ns).attrib
check("for desktops from the build that reads uap10", family.get("Name") == "Windows.Desktop" and family.get("MinVersion") == "10.0.19041.0", family)
visual = written.find("m:Applications/m:Application/uap:VisualElements", ns).attrib
sizes = {written.findtext("m:Properties/m:Logo", namespaces=ns): 50, visual.get("Square150x150Logo"): 150, visual.get("Square44x44Logo"): 44}
with tempfile.TemporaryDirectory() as work:
    artefacts, stage = os.path.join(work, "artefacts"), os.path.join(work, "stage")
    for name in names:
        os.makedirs(os.path.join(artefacts, os.path.dirname(name)), exist_ok=True)
        open(os.path.join(artefacts, name), "wb").write(name.encode())
    pages(artefacts)
    msix.stage(artefacts, "1.2.3", stage)
    held = {os.path.relpath(os.path.join(folder, file), stage).replace(os.sep, "/") for folder, _, files in os.walk(stage) for file in files}
    check("the five programs, the manifest, its logos and what they owe are what it holds, and no more",
          held == {name.split("/")[1] for name in names} | {"AppxManifest.xml"} | {logo.replace("\\", "/") for logo in sizes} | set(OWED), held)
    licence = open(os.path.join(stage, "LICENSE.txt"), encoding="utf-8").read()
    check("Kitokey's own licence among them, the GPL-3.0", "GNU GENERAL PUBLIC LICENSE" in licence and "Version 3" in licence)
    mpl = open(os.path.join(stage, "MPL-SOURCES.txt"), encoding="utf-8").read()
    check("where the MPL crates' sources are, and that their licence governs them (MPL-2.0 §3.2)",
          "https://crates.io/crates/" in mpl and "Mozilla Public License 2.0" in mpl and "3.2" in mpl, mpl)
    os.remove(os.path.join(artefacts, "capture", "THIRD-PARTY-NOTICES.html"))
    try:
        msix.stage(artefacts, "1.2.3", os.path.join(work, "again"))
        check("a module's page missing, no package is made", False)
    except (FileNotFoundError, SystemExit) as refused:
        check("a module's page missing, no package is made", "capture" in str(refused), refused)
    for logo, side in sizes.items():
        head = open(os.path.join(stage, logo.replace("\\", "/")), "rb").read(24) if logo else b""
        check(f"{logo} is a PNG of {side}x{side}", head[:8] == b"\x89PNG\r\n\x1a\n"
              and head[16:24] == side.to_bytes(4, "big") * 2, head)
certification = open(os.path.join(os.path.dirname(mac), "windows", "certification.md"), encoding="utf-8").read()
check("the notes for certification say why runFullTrust and the microphone, and what removal keeps",
      all(word in certification for word in ("runFullTrust", "microphone", "both sides", "Documents", "removes")))
# The certification kit's report (D77): written on Windows by appcert.exe, read here from a report it wrote.
import appcert
said = appcert.said(os.path.join(os.path.dirname(mac), "tests", "appcert-report.xml"))
check("the kit's verdict first: its version, the overall result, how many tests and how many failed",
      said[0] == "certification kit 10.0.26100.9457: PASS overall, 2 tests, 1 failed", said)
check("each failed test named, its number, whether the Store requires it, and what the kit saw first",
      any(line.startswith("  FAIL  Blocked executables (test 88, optional): 3 messages, the first:")
          and "CreateProcessW" in line for line in said), said)
check("a test that passed is not named", not any("App manifest" in line for line in said), said)
with tempfile.TemporaryDirectory() as work:
    broken = os.path.join(work, "report.xml")
    open(broken, "w").write("<html>not the kit's</html>")
    for path, why in ((broken, "a file that is not the kit's report"), (os.path.join(work, "none.xml"), "no file")):
        got = appcert.said(path)
        check(f"{why}: one line, saying no report was read", len(got) == 1 and got[0].startswith("certification kit: no report read"), got)
        check(f"{why}: the run fails, no report read (D102)", appcert.verdict(path) == 2)
# D102: the run fails on a required test failed, or on no report at all; an optional test failed is named, never blocking.
check("the kit's report with only an optional test failed passes the run",
      appcert.verdict(os.path.join(os.path.dirname(mac), "tests", "appcert-report.xml")) == 0)
with tempfile.TemporaryDirectory() as work:
    report = open(os.path.join(os.path.dirname(mac), "tests", "appcert-report.xml"), encoding="utf-8").read()
    required = os.path.join(work, "required.xml")
    open(required, "w", encoding="utf-8").write(report.replace("<RESULT><![CDATA[PASS]]></RESULT>", "<RESULT><![CDATA[FAIL]]></RESULT>", 1))
    check("a required test failed fails the run", appcert.verdict(required) == 1)
    unrun = os.path.join(work, "unrun.xml")
    open(unrun, "w", encoding="utf-8").write(report.replace("<RESULT><![CDATA[PASS]]></RESULT>", "<RESULT><![CDATA[NOT RUN]]></RESULT>", 1))
    check("a required test not run fails the run, never skipped silently", appcert.verdict(unrun) == 1)
    check("no report read fails the run: a kit that did not run is never a pass", appcert.verdict(os.path.join(work, "none.xml")) == 2)
    empty = os.path.join(work, "empty.xml")
    open(empty, "w", encoding="utf-8").write(re.sub(r"<REQUIREMENTS>.*</REQUIREMENTS>", "<REQUIREMENTS />", report, flags=re.S))
    check("a report without a test fails the run: a kit that tested nothing is never a pass", appcert.verdict(empty) == 2)
check("the notes for certification say what stands between the package and a submission, the kit's failure in it",
      "## What stands between the package and a submission" in certification and "Blocked executables" in certification)
print("test: passed" if not failed else "test: FAILED")
sys.exit(failed)
PY
