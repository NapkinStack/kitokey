"""What a package of Kitokey owes the authors of what it ships (D102), as Windows' zip and package
lay it beside the programs: Kitokey's LICENSE, where the MPL crates' sources are, and each
program's THIRD-PARTY-NOTICES.html, which its module's release left in the artefacts, named by the
program. mac/build.sh, deb/build.sh and kitokey.spec lay the same names; scripts/test.sh reads all five.
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))
# The repository's LICENSE, and install's own notices.
LICENSE = os.path.join(HERE, "..", "..", "..", "LICENSE")
MPL = os.path.join(HERE, "..", "notices", "MPL-SOURCES.txt")
# Each module's page, by the program it ships.
PAGES = (("shell", "kitokey"), ("capture", "kitokey-capture"),
         ("transcription", "kitokey-transcription"), ("summary", "kitokey-summary"),
         ("export", "kitokey-export"))


def files(artefacts):
    """(the file to read, its name in the package), for each one owed."""
    owed = [(LICENSE, "LICENSE.txt"), (MPL, "MPL-SOURCES.txt")]
    owed += [(os.path.join(artefacts, module, "THIRD-PARTY-NOTICES.html"), f"THIRD-PARTY-NOTICES-{program}.html")
             for module, program in PAGES]
    return owed
