//! D108's SRT and VTT: the transcript as subtitles, each segment a cue between its `start` and its
//! `end`; read back by a subtitle parser (aspasia).

use std::fs;

use aspasia::{Format, SubRipSubtitle, Subtitle, WebVttSubtitle, detect_format_from_str};
use kitokey_export::{Include, Labels, Meeting, srt, vtt};

const TRANSCRIPT: &str = concat!(
    r#"{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"large-v3-turbo","language":"fr"}"#,
    "\n",
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":4.5,"end":6.07,"text":"a|b <i>c</i> & d --> e"}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":1.0,"end":4.02,"text":"Bonjour à tous."}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":3725.5,"end":3727.0,"text":"Après une heure."}"#,
    "\n",
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":9.0,"end":8.0,"text":"Fin avant le début."}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":10.0,"end":10.0,"text":"Sans durée."}"#,
    "\n",
    r#"{"type":"complete","duration":3730.0,"made":"2026-11-20T10:05:00+01:00"}"#,
    "\n",
);

fn labels() -> Labels {
    Labels {
        transcript: "Transcription".into(),
        me: "Moi".into(),
        others: "Les <autres> & co".into(),
        time: "Temps".into(),
        speaker: "Qui".into(),
        text: "Texte".into(),
    }
}

fn meeting() -> Meeting {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    Meeting::read(dir.path()).unwrap()
}

fn ms(moment: aspasia::Moment) -> i64 {
    i64::from(moment)
}

#[test]
fn srt_gives_each_segment_a_numbered_cue_between_its_start_and_its_end_in_the_meetings_order() {
    let file = srt(&meeting(), Include::all(), &labels());
    assert!(matches!(detect_format_from_str(&file), Ok(Format::SubRip)), "{file}");
    let read: SubRipSubtitle = file.parse().unwrap();
    let cues: Vec<_> = read.events().iter().map(|e| (e.line_number, ms(e.start), ms(e.end))).collect();
    assert_eq!(
        cues,
        [(1, 1_000, 4_020), (2, 4_500, 6_070), (3, 3_725_500, 3_727_000)],
        "a segment whose end is not after its start is left out: {file}"
    );
    assert!(
        file.starts_with("1\n00:00:01,000 --> 00:00:04,020\n[Moi] Bonjour à tous.\n\n2\n"),
        "{file}"
    );
    assert!(file.contains("\n01:02:05,500 --> 01:02:07,000\n"), "{file}");
}

#[test]
fn vtt_gives_each_segment_a_cue_its_speaker_a_voice_and_its_text_escaped() {
    let file = vtt(&meeting(), Include::all(), &labels(), "fr");
    assert!(file.starts_with("WEBVTT\n\n"), "{file}");
    assert!(matches!(detect_format_from_str(&file), Ok(Format::WebVtt)), "{file}");
    let read: WebVttSubtitle = file.parse().unwrap();
    let cues: Vec<_> = read.events().iter().map(|c| (ms(c.start), ms(c.end), c.text.as_str())).collect();
    assert_eq!(
        cues,
        [
            (1_000, 4_020, "<v Moi>Moi\u{a0}: Bonjour à tous.</v>"),
            (
                4_500,
                6_070,
                "<v Les &lt;autres&gt; &amp; co>Les &lt;autres&gt; &amp; co\u{a0}: a|b &lt;i&gt;c&lt;/i&gt; &amp; d --&gt; e</v>"
            ),
            (3_725_500, 3_727_000, "<v Moi>Moi\u{a0}: Après une heure.</v>"),
        ],
        "{file}"
    );
    // The voice is the speaker's for a machine; most players show none of it (a browser's <track>,
    // libass): the name is said in the cue too, as SDH does, and no markup of the meeting is left raw.
    assert!(
        !file.contains("<i>") && !file.contains("</i>"),
        "a transcript's markup raw in a cue: {file}"
    );
}

#[test]
fn a_vtt_cue_says_its_speaker_in_its_text_punctuated_as_its_language_writes() {
    let english = vtt(&meeting(), Include::all(), &labels(), "en");
    assert!(english.contains("\n<v Moi>Moi: Bonjour à tous.</v>\n"), "{english}");
    let quebec = vtt(&meeting(), Include::all(), &labels(), "fr-CA");
    assert!(quebec.contains("\n<v Moi>Moi\u{a0}: Bonjour à tous.</v>\n"), "{quebec}");
}

#[test]
fn without_speakers_a_cue_is_its_text_alone_and_without_the_transcript_there_is_no_cue() {
    let mut include = Include::all();
    include.speakers = false;
    let srt_file = srt(&meeting(), include, &labels());
    let vtt_file = vtt(&meeting(), include, &labels(), "fr");
    assert!(
        srt_file.starts_with("1\n00:00:01,000 --> 00:00:04,020\nBonjour à tous.\n\n"),
        "{srt_file}"
    );
    assert!(
        vtt_file.contains("\n00:00:01.000 --> 00:00:04.020\nBonjour à tous.\n\n"),
        "{vtt_file}"
    );
    assert!(!srt_file.contains("Moi") && !vtt_file.contains("<v"), "{srt_file}{vtt_file}");

    include.transcript = false;
    assert_eq!(srt(&meeting(), include, &labels()), "");
    assert_eq!(vtt(&meeting(), include, &labels(), "fr"), "WEBVTT\n");
}
