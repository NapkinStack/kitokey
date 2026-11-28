"""Run on Windows by scripts/release.sh, and by trial.ps1 for a later version: Kitokey-<version>-x64.msix,
made with the Windows SDK's MakeAppx from AppxManifest.xml.in, identity.json, the six programs and
Assets/ (D54). Unsigned: the Store signs what it publishes; outside it, whoever installs it signs it.
Refused while identity.json holds a placeholder (D102): the identity is Partner Center's, or there is no package.

Usage: msix.py <artefacts> <version> <out>
"""
import glob
import json
import os
import re
import shutil
import subprocess
import sys

import notices as owed

HERE = os.path.dirname(os.path.abspath(__file__))
PROGRAMS = ("shell/kitokey.exe", "capture/kitokey-capture.exe",
            "transcription/kitokey-transcription.exe", "summary/kitokey-summary.exe", "export/kitokey-export.exe", "send/kitokey-send.exe")


def four_parts(version):
    """A package's version: the application's three numbers, then the 0 the Store keeps for itself."""
    if not re.fullmatch(r"\d+\.\d+\.\d+", version) or any(int(part) > 65535 for part in version.split(".")):
        raise ValueError(f"not three numbers under 65536: {version}")
    return f"{version}.0"


def identity():
    with open(os.path.join(HERE, "identity.json"), encoding="utf-8") as file:
        return json.load(file)


def refusal(who):
    """Why no package is made from this identity, or None: Partner Center's three values, or nothing."""
    if who.get("placeholder", True):
        return (f"msix: identity.json holds a placeholder ({who.get('name')}): paste the three values Partner Center's "
                "Product identity shows (Package/Identity/Name, Package/Identity/Publisher, "
                "Package/Properties/PublisherDisplayName) and set placeholder to false")
    empty = [key for key in ("name", "publisher", "publisher_display_name") if not who.get(key)]
    if empty:
        return f"msix: identity.json has no {', '.join(empty)}: Partner Center's Product identity gives each"
    return None


def manifest(version, who):
    with open(os.path.join(HERE, "AppxManifest.xml.in"), encoding="utf-8") as file:
        text = file.read()
    for key, value in (("NAME", who["name"]), ("VERSION", four_parts(version)), ("PUBLISHER", who["publisher"]),
                       ("PUBLISHER_DISPLAY_NAME", who["publisher_display_name"])):
        text = text.replace(f"@{key}@", value)
    return text


def notices(version, who):
    """What this package cannot do yet, for the run to say."""
    said = []
    if version.startswith("0."):
        said.append(f"version {four_parts(version)}: the Store refuses a first part 0, "
                    "the application's version must reach 1 before a submission")
    return said


def stage(artefacts, version, folder):
    """The package's content, as MakeAppx reads it."""
    os.makedirs(folder)
    for program in PROGRAMS:
        shutil.copy(os.path.join(artefacts, program), folder)
    shutil.copytree(os.path.join(HERE, "Assets"), os.path.join(folder, "Assets"))
    # What the programs owe the authors of what they ship, beside them (D102).
    for source, name in owed.files(artefacts):
        shutil.copy(source, os.path.join(folder, name))
    with open(os.path.join(folder, "AppxManifest.xml"), "w", encoding="utf-8") as file:
        file.write(manifest(version, identity()))


def makeappx():
    found = sorted(glob.glob(r"C:\Program Files (x86)\Windows Kits\10\bin\10.*\x64\makeappx.exe"))
    if not found:
        sys.exit("msix: no Windows SDK's makeappx.exe under Windows Kits\\10\\bin")
    return found[-1]


def main(artefacts, version, out):
    refused = refusal(identity())
    if refused:
        sys.exit(refused)
    folder = os.path.join(out, f"msix-{version}")
    shutil.rmtree(folder, ignore_errors=True)
    stage(artefacts, version, folder)
    path = os.path.join(out, f"Kitokey-{version}-x64.msix")
    subprocess.run([makeappx(), "pack", "/o", "/h", "SHA256", "/d", folder, "/p", path], check=True, timeout=600)
    shutil.rmtree(folder)
    for line in notices(version, identity()):
        print(f"::warning title=The Windows package::{line}")
    print(path)


if __name__ == "__main__":
    main(*sys.argv[1:4])
