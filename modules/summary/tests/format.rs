//! Minutes in a format a person made (D129): `--format <file>` (`summarising/v1`) writes `minutes/v5` in it —
//! its structure, its sections in their order, each `about` as the person wrote it, its actions —; a format it
//! cannot read is a wrong argument, nothing leaves and nothing is written. Two formats made for these tests,
//! against a provider played on this machine: no online service, no meeting of anyone.

mod common;

use std::{fs, path::Path, process::Output};

use common::{answering, program, provider};
use serde_json::{Value, json};

const TRANSCRIPT: &str = concat!(
    r#"{"type": "header", "contract": "transcript/v1", "engine": "an-engine", "model": "a-model", "language": "fr", "context": null}"#,
    "\n",
    r#"{"type": "segment", "speaker": "others", "start": 0.0, "end": 3.2, "text": "Le client veut la maquette vendredi, Léa l'envoie."}"#,
    "\n",
);

/// A client's weekly call, in French, the actions third; an `about` holding quotes, as a person may write one.
fn client_weekly() -> Value {
    json!({
        "structure": "client_weekly",
        "sections": [
            {"id": "contexte", "about": "Où en est le client, en deux phrases « sans jargon »."},
            {"id": "demandes", "about": "Ce que le client a demandé, une puce par demande."},
            {"id": "suites", "about": "Ce que chacun fera."}
        ],
        "actions": "suites"
    })
}

/// Notes without actions: one section.
fn notes() -> Value {
    json!({"structure": "notes", "sections": [{"id": "points", "about": "Every point raised, one bullet each."}]})
}

const WEEKLY: &str = "## Contexte\n\nLe client attend la maquette.\n\n## Demandes\n\n- La maquette vendredi [00:01]\n\n## Suites\n\n- Envoyer la maquette — Léa — vendredi [00:02]\n";
const NOTES: &str = "## Points\n\n- La maquette vendredi [00:01]\n";

fn session() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    folder
}

/// The format written beside the session, as the shell keeps one: a file of its own.
fn written(folder: &Path, format: &str) -> String {
    let file = folder.join("format.json");
    fs::write(&file, format).unwrap();
    file.to_str().unwrap().to_string()
}

fn run(folder: &Path, address: &str, args: &[&str]) -> Output {
    program()
        .arg(folder)
        .args(args.iter().map(|arg| arg.replace("ADDRESS", address)))
        .env_remove("KITOKEY_PROVIDER_KEY")
        .output()
        .unwrap()
}

/// The front matter's keys, as `minutes/v5` says a reader takes them.
fn front_matter(minutes: &str) -> serde_json::Map<String, Value> {
    let inside = minutes.strip_prefix("---\n").unwrap().split_once("\n---\n").unwrap().0;
    inside
        .lines()
        .map(|line| {
            let (key, value) = line.split_once(": ").unwrap();
            let value = if value.starts_with(['"', '[']) {
                serde_json::from_str(value).unwrap()
            } else {
                Value::String(value.into())
            };
            (key.to_string(), value)
        })
        .collect()
}

#[test]
fn a_format_given_writes_minutes_v5_in_it_each_about_as_the_person_wrote_it() {
    let folder = session();
    let format = written(folder.path(), &client_weekly().to_string());
    let (address, received) = provider("200 OK", answering(WEEKLY, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--provider", "ADDRESS", "--model", "a-model", "--format", &format],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    // The model is asked the format's sections, in their order, in the person's words; the actions' section
    // as every structure asks it, so that its items say what — who — by when (minutes/v5).
    let (_, asked) = received.recv().unwrap();
    let asked: Value = serde_json::from_str(&asked).unwrap();
    let content = asked["messages"][0]["content"].as_str().unwrap();
    assert!(content.contains("exactly 3 sections"), "{content}");
    let (first, second) = (
        content.find("1. Où en est le client, en deux phrases « sans jargon ».").unwrap(),
        content.find("2. Ce que le client a demandé, une puce par demande.").unwrap(),
    );
    assert!(first < second, "{content}");
    assert!(content.contains("3. Every task the meeting agreed on or asked for"), "{content}");

    let minutes = fs::read_to_string(folder.path().join("minutes.md")).unwrap();
    let keys = front_matter(&minutes);
    assert_eq!(keys["contract"], "minutes/v5");
    assert_eq!(keys["structure"], "client_weekly");
    assert_eq!(keys["sections"], client_weekly()["sections"]);
    assert_eq!(keys["actions"], "suites");
    assert!(minutes.ends_with(WEEKLY), "{minutes}");
}

#[test]
fn a_format_without_actions_writes_none() {
    let folder = session();
    let format = written(folder.path(), &notes().to_string());
    let (address, _received) = provider("200 OK", answering(NOTES, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--format", &format, "--model", "a-model", "--provider", "ADDRESS"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let minutes = fs::read_to_string(folder.path().join("minutes.md")).unwrap();
    let keys = front_matter(&minutes);
    assert_eq!(keys["structure"], "notes");
    assert_eq!(keys["sections"], notes()["sections"]);
    assert!(!keys.contains_key("actions"), "{minutes}");
}

#[test]
fn an_answer_without_the_formats_number_of_sections_is_not_minutes() {
    let folder = session();
    let format = written(folder.path(), &client_weekly().to_string());
    let (address, received) = provider("200 OK", answering(NOTES, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--provider", "ADDRESS", "--model", "a-model", "--format", &format],
    );
    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("1 section(s) of second level where three are expected"), "{err}");
    assert!(received.try_iter().count() >= 1);
    assert!(!folder.path().join("minutes.md").exists());
}

#[test]
fn a_format_it_cannot_read_is_a_wrong_argument_nothing_leaves_nothing_is_written() {
    let mut weekly = client_weekly();
    let sections = |n: usize| (0..n).map(|n| json!({"id": format!("s{n}"), "about": "x"})).collect::<Vec<_>>();
    let wrong: Vec<(String, &str)> = vec![
        ("not json".into(), "no JSON"),
        (json!({"sections": notes()["sections"]}).to_string(), "no structure"),
        (json!({"structure": "notes"}).to_string(), "no sections"),
        (
            json!({"structure": "general", "sections": notes()["sections"]}).to_string(),
            "one of the six",
        ),
        (
            json!({"structure": "Notes", "sections": notes()["sections"]}).to_string(),
            "a name with capitals",
        ),
        (json!({"structure": "notes", "sections": []}).to_string(), "no section"),
        (json!({"structure": "notes", "sections": sections(7)}).to_string(), "seven sections"),
        (
            json!({"structure": "notes", "sections": [{"id": "Points", "about": "x"}]}).to_string(),
            "an id with capitals",
        ),
        (
            json!({"structure": "notes", "sections": [{"id": "points", "about": ""}]}).to_string(),
            "an empty about",
        ),
        (
            json!({"structure": "notes", "sections": [{"id": "points", "about": "two\nlines"}]}).to_string(),
            "an about on two lines",
        ),
        (
            json!({"structure": "notes", "sections": [{"id": "points", "about": "é".repeat(201)}]}).to_string(),
            "an about of 201 characters",
        ),
        (
            json!({"structure": "notes", "sections": [{"id": "a", "about": "x"}, {"id": "a", "about": "y"}]}).to_string(),
            "an id twice",
        ),
        (
            {
                weekly["actions"] = json!("next");
                weekly.to_string()
            },
            "actions naming no section",
        ),
    ];
    for (format, why) in wrong {
        let folder = session();
        let file = written(folder.path(), &format);
        let (address, received) = provider("200 OK", answering(WEEKLY, "stop"));
        let output = run(
            folder.path(),
            &address,
            &["--provider", "ADDRESS", "--model", "a-model", "--format", &file],
        );
        assert!(!output.status.success(), "{why}");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("kitokey-summary: "), "{why}");
        assert!(output.stdout.is_empty(), "{why}: nothing said sent");
        assert_eq!(received.try_iter().count(), 0, "{why}");
        assert!(!folder.path().join("minutes.md").exists(), "{why}");
    }
    // Six sections and 200 characters are within the bounds.
    let folder = session();
    let six = json!({"structure": "six", "sections": [{"id": "a", "about": "é".repeat(200)}]}).to_string();
    let file = written(folder.path(), &six);
    let (address, _received) = provider("200 OK", answering(NOTES, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--provider", "ADDRESS", "--model", "a-model", "--format", &file],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn a_format_with_a_structure_or_a_file_it_cannot_open_is_a_wrong_argument() {
    let folder = session();
    let format = written(folder.path(), &notes().to_string());
    let missing = folder.path().join("none.json");
    for (args, why) in [
        (vec!["--structure", "general", "--format", format.as_str()], "with --structure"),
        (vec!["--format", missing.to_str().unwrap()], "a file it cannot open"),
        (vec!["--format", format.as_str(), "--format", format.as_str()], "--format twice"),
    ] {
        let (address, received) = provider("200 OK", answering(NOTES, "stop"));
        let mut all = vec!["--provider", "ADDRESS", "--model", "a-model"];
        all.extend(args);
        let output = run(folder.path(), &address, &all);
        assert!(!output.status.success(), "{why}");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("kitokey-summary: "), "{why}");
        assert!(output.stdout.is_empty(), "{why}");
        assert_eq!(received.try_iter().count(), 0, "{why}");
        assert!(!folder.path().join("minutes.md").exists(), "{why}");
    }
}
