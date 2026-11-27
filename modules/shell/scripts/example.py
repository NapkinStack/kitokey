"""Makes the example meeting a newcomer tries (D126): one session folder per language the window
speaks, as `session-audio/v3` says, its two voices made by ElevenLabs Multilingual v2 through
OpenRouter's speech API. Invented for Kitokey: no real meeting, no real person.

Run from `modules/shell`, with the key in the environment, never written anywhere:

    OPENROUTER_API_KEY=… python3 scripts/example.py <speech-folder>

Each turn's speech is kept in `<speech-folder>/<language>/turnNN-<side>.mp3`, as the API answered it,
and asked again only when missing: a second run lays the tracks out from those files, with no key.
It writes `ui/example/<language>/`: `session.json`, `clock.jsonl`, `local.flac` (« Vous »),
`remote.flac` (« Les autres ») and `title.txt`. Needs `ffmpeg` (with soxr) for the FLAC.

The service's synthesis is not deterministic: the same words, voices and speed give other bytes on
another run. ElevenLabs says « All our Default voices will expire on December 31, 2026 »
(https://elevenlabs.io/docs/capabilities/voices, read on 2026-11-27); whether the four voices here
are Default voices is not established. If they are, the clips already made stay, but after that
date this script can no longer make them again with these voices. Terms of the sound, and what they
do not establish: `ui/example/LICENSE`.
"""

import array
import json
import os
import subprocess
import sys
import urllib.request
from pathlib import Path

# The speech model, as OpenRouter names it, its speed, and the address that makes speech.
MODEL = "elevenlabs/eleven-multilingual-v2"
SPEED = 0.95
SPEECH = "https://openrouter.ai/api/v1/audio/speech"

# The rate the tracks are kept at: the 44.1 kHz MP3 the service sends holds nothing above about
# 16 kHz, and 24 kHz keeps the voices to 12 kHz under the repository's 500 KB a file. The gap
# between two turns, in seconds.
RATE = 24_000
GAP = 0.45

# Each language: its title, the service's voice for the local track (« Vous ») and the remote one,
# and its turns.
MEETINGS = {
    "fr": {
        "title": "Exemple : point sur la version 2",
        "local": "lily",
        "remote": "eric",
        "turns": [
            ("local", "Bonjour Hugo. On fait le point sur la sortie de la version deux ?"),
            ("remote", "Oui. Les tests passent depuis hier soir. Il reste la page d'aide à relire."),
            ("local", "D'accord. Je la relis demain matin, et je te l'envoie avant midi."),
            ("remote", "Parfait. De mon côté, je prépare les notes de version pour jeudi."),
            ("local", "Et pour le support, on prévient l'équipe quand ?"),
            ("remote", "Vendredi, après la mise en ligne. Je leur écris un message court."),
            ("local", "Très bien. Donc : la page d'aide demain, les notes jeudi, le support vendredi."),
            ("remote", "C'est ça. On se reparle lundi prochain pour le bilan."),
            ("local", "Ça marche. Merci Hugo, bonne journée."),
        ],
    },
    "en": {
        "title": "Example: version 2 check-in",
        "local": "will",
        "remote": "matilda",
        "turns": [
            ("local", "Hi Jess. Shall we go over the release of version two?"),
            ("remote", "Sure. The tests have passed since last night. The help page still needs a review."),
            ("local", "Okay. I'll review it tomorrow morning and send it to you before noon."),
            ("remote", "Great. On my side, I'll write the release notes for Thursday."),
            ("local", "And when do we tell the support team?"),
            ("remote", "On Friday, once it's live. I'll send them a short note."),
            ("local", "Perfect. So: the help page tomorrow, the notes on Thursday, support on Friday."),
            ("remote", "That's it. Let's catch up next Monday to see how it went."),
            ("local", "Sounds good. Thanks Jess, have a good day."),
        ],
    },
}

# The session's instant 0, fixed so that the folder made again is the same.
STARTED = "2026-11-26T10:00:00+01:00"


def asked(text: str, voice: str, mp3: Path) -> None:
    """One turn's speech, asked of the service with the key the environment holds."""
    key = os.environ.get("OPENROUTER_API_KEY")
    if not key:
        sys.exit(f"{mp3} is missing: set OPENROUTER_API_KEY to ask the service for it")
    request = urllib.request.Request(
        SPEECH,
        method="POST",
        data=json.dumps({"model": MODEL, "input": text, "voice": voice, "response_format": "mp3", "speed": SPEED}).encode(),
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=120) as answered:
        mp3.write_bytes(answered.read())


def heard(mp3: Path) -> array.array:
    """A turn's speech at the tracks' rate, 16-bit samples, by soxr."""
    raw = subprocess.run(
        ["ffmpeg", "-loglevel", "error", "-i", str(mp3), "-af", f"aresample={RATE}:resampler=soxr:precision=28",
         "-ac", "1", "-f", "s16le", "-"],
        capture_output=True, check=True,
    ).stdout
    samples = array.array("h")
    samples.frombytes(raw)
    return samples


def track(samples: array.array, out: Path) -> None:
    """A track as `session-audio/v3` keeps one once stopped: FLAC, PCM 24 bits, one channel; the
    speech's own 16 bits held in 24, the low bits zero, which FLAC keeps for nothing."""
    subprocess.run(
        ["ffmpeg", "-loglevel", "error", "-y", "-f", "s16le", "-ar", str(RATE), "-ac", "1", "-i", "-",
         "-af", "aformat=sample_fmts=s32", "-sample_fmt", "s32", "-bits_per_raw_sample", "24",
         "-compression_level", "12", "-map_metadata", "-1", "-fflags", "+bitexact", "-flags:a", "+bitexact", str(out)],
        input=samples.tobytes(), check=True,
    )


def made(speech: Path, language: str, meeting: dict, out: Path) -> None:
    folder = speech / language
    folder.mkdir(parents=True, exist_ok=True)
    sides = {"local": array.array("h"), "remote": array.array("h")}
    gap = array.array("h", [0]) * int(GAP * RATE)
    for n, (side, text) in enumerate(meeting["turns"]):
        mp3 = folder / f"turn{n:02d}-{side}.mp3"
        if not mp3.exists():
            asked(text, meeting[side], mp3)
        turn = heard(mp3)
        other = "remote" if side == "local" else "local"
        sides[side].extend(turn + gap)
        sides[other].extend(array.array("h", [0]) * len(turn) + gap)
    out.mkdir(parents=True, exist_ok=True)
    for side, samples in sides.items():
        track(samples, out / f"{side}.flac")
    seconds = len(sides["local"]) / RATE
    # A point at the first frame, then every ten seconds, on each track.
    clock = [
        {"track": side, "sample": at * RATE, "at": float(at)}
        for at in range(0, int(seconds) + 1, 10)
        for side in ("local", "remote")
    ]
    (out / "clock.jsonl").write_text("".join(json.dumps(point) + "\n" for point in clock), encoding="utf-8")
    stopped_at = int(seconds) + 1
    session = {
        "contract": "session-audio/v3",
        "started": STARTED,
        "stopped": f"2026-11-26T10:00:{stopped_at:02d}+01:00",
        "tracks": {"local": {"device": "Kitokey example"}, "remote": {"device": "Kitokey example"}},
    }
    (out / "session.json").write_text(json.dumps(session, indent=2) + "\n", encoding="utf-8")
    (out / "title.txt").write_text(meeting["title"] + "\n", encoding="utf-8")
    print(f"{language}: {seconds:.1f} s, " + ", ".join(f"{p.name} {p.stat().st_size} B" for p in sorted(out.iterdir())))


def main() -> None:
    speech = Path(sys.argv[1])
    root = Path(__file__).resolve().parent.parent / "ui" / "example"
    for language, meeting in MEETINGS.items():
        made(speech, language, meeting, root / language)


if __name__ == "__main__":
    main()
