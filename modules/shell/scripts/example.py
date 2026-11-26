"""Makes the example meeting a newcomer tries (D126): one session folder per language the window
speaks, as `session-audio/v3` says, its two voices made by Piper. Invented for Kitokey: no real
meeting, no real person.

Run once, from `modules/shell`, with the voices downloaded (licences in `ui/example/LICENSE`):

    uv run --no-project --with piper-tts==1.8.0 python -m piper.download_voices \
        fr_FR-mls-medium en_US-joe-medium en_GB-cori-medium --download-dir <voices>
    uv run --no-project --with piper-tts==1.8.0 python scripts/example.py <voices>

It writes `ui/example/<language>/`: `session.json`, `clock.jsonl`, `local.flac` (« Vous »),
`remote.flac` (« Les autres ») and `title.txt`. Needs `ffmpeg` for the FLAC.
"""

import array
import json
import subprocess
import sys
import tempfile
import wave
from pathlib import Path

from piper import PiperVoice, SynthesisConfig

# The rate the tracks are kept at, and the gap between two turns, in seconds.
RATE = 16_000
GAP = 0.45

# Each language: its title, its pace, its voices — a Piper model and its speaker, `None` for one of a
# single speaker — for the local track and the remote one, and its turns.
MEETINGS = {
    "fr": {
        "title": "Exemple : point sur la version 2",
        # Speaker 3698 (a pitch of about 170 Hz) is « Vous »; 1840 (about 105 Hz) is Hugo.
        "local": ("fr_FR-mls-medium", "3698"),
        "remote": ("fr_FR-mls-medium", "1840"),
        # MLS's readers are slow for a meeting: a little faster, to about 45 s.
        "pace": 0.72,
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
        "pace": 1.0,
        "local": ("en_US-joe-medium", None),
        "remote": ("en_GB-cori-medium", None),
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


def spoken(voice: PiperVoice, speaker: str | None, text: str, pace: float, noise: tuple[float, float] | None) -> array.array:
    """One turn, as 16-bit samples at the voice's own rate; `pace` below 1 speaks faster; `noise`, Piper's
    noise and noise width, the voice's own when none."""
    config = SynthesisConfig(
        speaker_id=None if speaker is None else voice.config.speaker_id_map[speaker],
        length_scale=pace,
        **({} if noise is None else {"noise_scale": noise[0], "noise_w_scale": noise[1]}),
    )
    samples = array.array("h")
    for chunk in voice.synthesize(text, syn_config=config):
        samples.frombytes(chunk.audio_int16_bytes)
    return samples


def track(samples: array.array, rate: int, out: Path) -> None:
    """A track as `session-audio/v3` keeps one once stopped: FLAC, PCM 24 bits, one channel."""
    with tempfile.TemporaryDirectory() as scratch:
        raw = Path(scratch) / "track.wav"
        with wave.open(str(raw), "wb") as written:
            written.setnchannels(1)
            written.setsampwidth(2)
            written.setframerate(rate)
            written.writeframes(samples.tobytes())
        subprocess.run(
            # Resampled to 16 bits, then held in 24: the voices' own precision, the low bits zero —
            # which FLAC keeps for nothing, where a resampling's noise there would double the file.
            ["ffmpeg", "-loglevel", "error", "-y", "-i", str(raw),
             "-af", f"aresample={RATE},aformat=sample_fmts=s16,aformat=sample_fmts=s32",
             "-sample_fmt", "s32", "-bits_per_raw_sample", "24", "-compression_level", "12",
             "-map_metadata", "-1", "-fflags", "+bitexact", "-flags:a", "+bitexact", str(out)],
            check=True,
        )


def made(voices: Path, language: str, meeting: dict, out: Path) -> None:
    loaded = {side: PiperVoice.load(str(voices / f"{meeting[side][0]}.onnx")) for side in ("local", "remote")}
    rate = loaded["local"].config.sample_rate
    assert loaded["remote"].config.sample_rate == rate, "both voices at one rate"
    sides = {"local": array.array("h"), "remote": array.array("h")}
    gap = array.array("h", [0]) * int(GAP * rate)
    for side, text in meeting["turns"]:
        turn = spoken(loaded[side], meeting[side][1], text, meeting["pace"], meeting.get("noise"))
        other = "remote" if side == "local" else "local"
        sides[side].extend(turn + gap)
        sides[other].extend(array.array("h", [0]) * len(turn) + gap)
    out.mkdir(parents=True, exist_ok=True)
    for side, samples in sides.items():
        track(samples, rate, out / f"{side}.flac")
    seconds = len(sides["local"]) / rate
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
    voices = Path(sys.argv[1])
    root = Path(__file__).resolve().parent.parent / "ui" / "example"
    for language, meeting in MEETINGS.items():
        made(voices, language, meeting, root / language)


if __name__ == "__main__":
    main()
