//! D108's Markdown: a meeting's folder read into one model, written as CommonMark with a GFM table
//! and a YAML front matter, then read back by pulldown-cmark.

use std::fs;
use std::path::Path;

use kitokey_export::{Include, Labels, Meeting, markdown};
use pulldown_cmark::{Event, HeadingLevel, MetadataBlockKind, Options, Parser, Tag, TagEnd};

const SESSION: &str = r#"{"contract": "session-audio/v3", "started": "2026-11-20T09:30:00+01:00", "tracks": {}}"#;

const MINUTES_V4: &str = r#"---
contract: minutes/v4
made: 2026-11-20T10:05:00+01:00
provider: "local"
model: "qwen3:8b"
context: null
title: "Prix | essai"
made_in: 3.2
---

## Points forts

Le prix est fixé.

## Actions

- Écrire la page — Sam — vendredi [00:04]

## Points clés

- 49 €
"#;

const MINUTES_V1: &str = "---\ncontract: minutes/v1\nmade: 2026-11-20T10:05:00+01:00\nprovider: \"local\"\nmodel: \"qwen3:8b\"\ncontext: null\n---\n\n## A\n\none\n\n## B\n\ntwo\n\n## C\n\nthree\n";

const TRANSCRIPT: &str = concat!(
    r#"{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"small","language":null,"context":null}"#,
    "\n",
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":4.5,"end":6.0,"text":"Здравствуйте | коллеги"}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":1.0,"end":3.0,"text":"Bonjour à tous."}"#,
    "\n",
    r#"{"type":"complete","duration":65.2}"#,
    "\n",
);

fn labels() -> Labels {
    Labels {
        transcript: "Transcription".into(),
        me: "Moi".into(),
        others: "Les autres".into(),
        time: "Temps".into(),
        speaker: "Qui".into(),
        text: "Texte".into(),
    }
}

fn folder(minutes: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    fs::write(dir.path().join("minutes.md"), minutes).unwrap();
    fs::write(dir.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    dir
}

fn written(dir: &Path, include: Include) -> String {
    markdown(&Meeting::read(dir).expect("the meeting read"), include, &labels())
}

/// What a reader finds: the front matter's text, the headings in order, the table's rows.
struct Read {
    front: Option<String>,
    headings: Vec<(HeadingLevel, String)>,
    rows: Vec<Vec<String>>,
}

fn read(text: &str) -> Read {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    let (mut front, mut headings, mut rows) = (None, vec![], vec![]);
    let (mut in_front, mut heading, mut cell): (bool, Option<(HeadingLevel, String)>, Option<String>) = (false, None, None);
    for event in Parser::new_ext(text, options) {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => in_front = true,
            Event::End(TagEnd::MetadataBlock(_)) => in_front = false,
            Event::Start(Tag::Heading { level, .. }) => heading = Some((level, String::new())),
            Event::End(TagEnd::Heading(_)) => headings.extend(heading.take()),
            Event::Start(Tag::TableRow) | Event::Start(Tag::TableHead) => rows.push(vec![]),
            Event::Start(Tag::TableCell) => cell = Some(String::new()),
            Event::End(TagEnd::TableCell) => rows.last_mut().unwrap().extend(cell.take()),
            Event::Text(said) if in_front => front = Some(said.to_string()),
            Event::Text(said) | Event::Code(said) => {
                if let Some((_, h)) = heading.as_mut() {
                    h.push_str(&said)
                } else if let Some(c) = cell.as_mut() {
                    c.push_str(&said)
                }
            }
            _ => {}
        }
    }
    Read { front, headings, rows }
}

/// The front matter's keys, each value read as JSON, as the writer writes them.
fn front(text: &str) -> serde_json::Map<String, serde_json::Value> {
    text.lines()
        .filter_map(|line| line.split_once(": "))
        .map(|(key, value)| {
            (
                key.to_string(),
                serde_json::from_str(value).unwrap_or_else(|_| panic!("{key}: {value} is not JSON")),
            )
        })
        .collect()
}

#[test]
fn a_meeting_reads_back_as_commonmark_its_front_matter_its_sections_in_order_and_its_transcript_a_table() {
    let dir = folder(MINUTES_V4);
    let out = read(&written(dir.path(), Include::all()));
    let front = front(out.front.as_deref().expect("a YAML front matter"));
    assert_eq!(front["title"], "Prix | essai");
    assert_eq!(front["date"], "2026-11-20T09:30:00+01:00");
    assert_eq!(front["duration"], 65.2);
    assert_eq!(front["models"], serde_json::json!({"transcript": "small", "summary": "qwen3:8b"}));
    let titles: Vec<&str> = out.headings.iter().map(|(_, t)| t.as_str()).collect();
    assert_eq!(titles, ["Prix | essai", "Points forts", "Actions", "Points clés", "Transcription"]);
    assert_eq!(out.headings[0].0, HeadingLevel::H1);
    assert!(out.headings[1..].iter().all(|(level, _)| *level == HeadingLevel::H2));
    // The segments in the meeting's order, a `|` in a text kept inside its cell.
    assert_eq!(
        out.rows,
        [
            vec!["Temps", "Qui", "Texte"],
            vec!["00:01", "Moi", "Bonjour à tous."],
            vec!["00:04", "Les autres", "Здравствуйте | коллеги"]
        ]
    );
}

#[test]
fn what_is_included_is_the_callers_choice() {
    let dir = folder(MINUTES_V4);
    let summary_only = read(&written(
        dir.path(),
        Include {
            summary: true,
            transcript: false,
            speakers: true,
            times: true,
        },
    ));
    assert!(summary_only.rows.is_empty(), "no transcript asked, a table written");
    assert!(!summary_only.headings.iter().any(|(_, t)| t == "Transcription"));
    let bare = read(&written(
        dir.path(),
        Include {
            summary: false,
            transcript: true,
            speakers: false,
            times: false,
        },
    ));
    assert_eq!(bare.rows, [vec!["Texte"], vec!["Bonjour à tous."], vec!["Здравствуйте | коллеги"]]);
    assert!(
        !bare.headings.iter().any(|(_, t)| t == "Points forts"),
        "no summary asked, its sections written"
    );
}

#[test]
fn the_summarys_sections_are_a_list_read_by_their_place_whatever_their_number() {
    // v1's three, named as the model named them; v5 adds named sections, read the same way.
    let sections = |minutes: &str| -> Vec<String> {
        let dir = folder(minutes);
        let summary = Include {
            summary: true,
            transcript: false,
            speakers: false,
            times: false,
        };
        read(&written(dir.path(), summary))
            .headings
            .into_iter()
            .filter(|(level, _)| *level == HeadingLevel::H2)
            .map(|(_, t)| t)
            .collect()
    };
    assert_eq!(sections(MINUTES_V1), ["A", "B", "C"]);
    let five = MINUTES_V1.replace("minutes/v1", "minutes/v5") + "\n## D\n\nfour\n\n## E\n\nfive\n";
    assert_eq!(sections(&five), ["A", "B", "C", "D", "E"]);
}

#[test]
fn a_meeting_without_minutes_or_title_is_named_by_its_date_and_still_writes() {
    let dir = folder(MINUTES_V4);
    fs::remove_file(dir.path().join("minutes.md")).unwrap();
    let text = written(dir.path(), Include::all());
    let out = read(&text);
    let front = front(out.front.as_deref().unwrap());
    assert!(!front.contains_key("title"), "a title invented: {text}");
    assert_eq!(front["models"], serde_json::json!({"transcript": "small"}));
    assert_eq!(out.headings.first().map(|(_, t)| t.as_str()), Some("Transcription"));
}

#[test]
fn a_title_out_of_the_minutes_bounds_is_no_title() {
    // minutes/v4: over 80 characters, blank, or holding a control character, a title is absent.
    for title in ["x".repeat(81), "   ".to_string(), "a\\u0007b".to_string()] {
        let minutes = MINUTES_V4.replace(r#"title: "Prix | essai""#, &format!("title: \"{title}\""));
        let dir = folder(&minutes);
        let out = read(&written(dir.path(), Include::all()));
        assert!(
            !front(out.front.as_deref().unwrap()).contains_key("title"),
            "{title:?} taken as a title"
        );
        assert_ne!(out.headings[0].0, HeadingLevel::H1, "{title:?} written as the document's heading");
    }
    let minutes = MINUTES_V4.replace(r#"title: "Prix | essai""#, &format!("title: \"{}\"", "é".repeat(80)));
    let dir = folder(&minutes);
    assert_eq!(
        front(read(&written(dir.path(), Include::all())).front.as_deref().unwrap())["title"],
        "é".repeat(80)
    );
}

#[test]
fn a_time_past_an_hour_says_its_hours_as_the_window_does() {
    // The page's clock says « 01:05:00 » past an hour, « 59:59 » before (R9: one passage, one reading).
    let transcript = concat!(
        r#"{"type":"header","contract":"transcript/v1","engine":"whisper.cpp","model":"small","language":null,"context":null}"#,
        "\n",
        r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":3599.9,"end":3600.5,"text":"avant"}"#,
        "\n",
        r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":3900.0,"end":3901.0,"text":"après"}"#,
        "\n",
    );
    let dir = folder(MINUTES_V4);
    fs::write(dir.path().join("transcript.jsonl"), transcript).unwrap();
    let times: Vec<String> = read(&written(dir.path(), Include::all())).rows[1..]
        .iter()
        .map(|row| row[0].clone())
        .collect();
    assert_eq!(times, ["59:59", "01:05:00"]);
}

#[test]
fn a_text_to_edit_keeps_its_punctuation_where_commonmark_reads_none_of_it_as_markup() {
    // D108 writes a file to edit: « 20 min, http://a.b (ok) » stays as typed; only what CommonMark
    // could take for markup in a line of a cell is escaped, and still reads back as said.
    let transcript = concat!(
        r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":1.0,"end":2.0,"text":"20 min, http://a.b (ok) - fin."}"#,
        "\n",
        r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":3.0,"end":4.0,"text":"*a* _b_ `c` [d](e) <f> a|b &amp; ~g~ \\h"}"#,
        "\n",
    );
    let dir = folder(MINUTES_V4);
    fs::write(dir.path().join("transcript.jsonl"), transcript).unwrap();
    let text = written(dir.path(), Include::all());
    assert!(
        text.contains("| 20 min, http://a.b (ok) - fin. |"),
        "plain punctuation escaped:\n{text}"
    );
    let rows = read(&text).rows;
    assert_eq!(
        rows[2][2], "*a* _b_ `c` [d](e) <f> a|b &amp; ~g~ \\h",
        "markup taken as such:\n{text}"
    );
}
