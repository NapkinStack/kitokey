//! The speakers' names a person gave the meeting (#1205, `summarising/v1`, `session-audio/v3`'s `speakers.json`): each
//! line of the request says its side's name beside the transcript's word, and one rule says what that name is — so
//! the minutes name who does what. Without `speakers.json`, or with nothing in it the contract allows, the request is
//! the one sent before, byte for byte.

mod common;

use std::{fs, path::Path};

use common::{MINUTES, answering, program, provider};
use kitokey_summary::{Names, read_transcript, request_in, request_named, speakers};

const TRANSCRIPT: &str = concat!(
    r#"{"type": "header", "contract": "transcript/v1", "engine": "an-engine", "model": "a-model", "language": "fr", "context": null}"#,
    "\n",
    r#"{"type": "segment", "speaker": "me", "track": "local", "voice": null, "start": 3.0, "end": 5.0, "text": "On commence par la 2.4 : Hugo, où en sont les tests ?"}"#,
    "\n",
    r#"{"type": "segment", "speaker": "others", "track": "remote", "voice": null, "start": 10.0, "end": 12.0, "text": "Tout passe depuis la correction du cache."}"#,
    "\n",
);

const RULE: &str = "A name in brackets after \"me\" or \"others\" is the one the user gave that side: write it for them.";

fn named(me: Option<&str>, others: Option<&str>) -> Names {
    Names {
        me: me.map(str::to_string),
        others: others.map(str::to_string),
    }
}

#[test]
fn each_line_says_its_sides_name_and_one_rule_says_what_it_is() {
    let segments = read_transcript(TRANSCRIPT).unwrap();
    let asked = request_named(&segments, None, None, &named(Some("Sarah"), Some("L’équipe produit")));
    assert!(asked.contains("[00:03] me (Sarah): On commence par la 2.4"), "{asked}");
    assert!(asked.contains("[00:10] others (L’équipe produit): Tout passe"), "{asked}");
    assert_eq!(asked.matches(RULE).count(), 1, "{asked}");
    // The rule comes with the others on speakers, before the transcript.
    assert!(asked.find(RULE).unwrap() < asked.find("Transcript:\n").unwrap());
    // One side named: the other keeps the transcript's word alone.
    let asked = request_named(&segments, None, None, &named(None, Some("Camille")));
    assert!(
        asked.contains("[00:03] me: On commence") && asked.contains("[00:10] others (Camille): Tout passe"),
        "{asked}"
    );
}

#[test]
fn without_names_the_request_is_the_one_sent_before() {
    let segments = read_transcript(TRANSCRIPT).unwrap();
    let before = request_in(&segments, Some("Quatre personnes."), None);
    assert_eq!(request_named(&segments, Some("Quatre personnes."), None, &Names::default()), before);
    assert!(!before.contains(RULE) && before.contains("[00:03] me: On commence"));
}

#[test]
fn speakers_json_is_read_as_the_contract_says() {
    let folder = tempfile::tempdir().unwrap();
    let at = folder.path().join("speakers.json");
    assert_eq!(speakers(folder.path()), Names::default());
    fs::write(&at, r#"{"me":"Sarah","others":"   ","voices":{"remote/1":"Hugo"},"extra":1}"#).unwrap();
    assert_eq!(speakers(folder.path()), named(Some("Sarah"), None));
    fs::write(&at, format!(r#"{{"me":42,"others":"{}"}}"#, "a".repeat(61))).unwrap();
    assert_eq!(speakers(folder.path()), Names::default());
    fs::write(&at, r#"{"me":"Sa\nrah","others":"Camille, Hugo et Léa"}"#).unwrap();
    assert_eq!(speakers(folder.path()), named(None, Some("Camille, Hugo et Léa")));
    for text in ["[\"Sarah\"]", "not json", ""] {
        fs::write(&at, text).unwrap();
        assert_eq!(speakers(folder.path()), Names::default(), "{text}");
    }
}

fn sent(folder: &Path) -> String {
    let (address, received) = provider("200 OK", answering(MINUTES, "stop"));
    let output = program()
        .arg(folder)
        .args(["--provider", &address, "--model", "a-model"])
        .env_remove("KITOKEY_PROVIDER_KEY")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let (_, asked) = received.recv().unwrap();
    let asked: serde_json::Value = serde_json::from_str(&asked).unwrap();
    asked["messages"][0]["content"].as_str().unwrap().to_string()
}

#[test]
fn the_program_sends_the_names_the_meeting_holds() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    let before = sent(folder.path());
    assert!(!before.contains(RULE) && before.contains("[00:10] others: Tout passe"), "{before}");
    fs::write(folder.path().join("speakers.json"), r#"{"me":"Sarah","others":"L’équipe produit"}"#).unwrap();
    let asked = sent(folder.path());
    assert!(
        asked.contains("[00:03] me (Sarah): ") && asked.contains("[00:10] others (L’équipe produit): ") && asked.contains(RULE),
        "{asked}"
    );
    // A file that is no JSON object: the request sent before.
    fs::write(folder.path().join("speakers.json"), "[]").unwrap();
    assert_eq!(sent(folder.path()), before);
}
