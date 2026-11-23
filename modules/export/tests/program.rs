//! `kitokey-export`, the program `exporting/v1` describes, run as the shell runs it: the session's
//! folder and the choices of form in its arguments, where to write and the file's words on its input,
//! one JSON line on its output, one sentence on its error stream when it fails.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use kitokey_export::{Include, Labels, Meeting, docx, html, markdown};
use serde_json::{Value, json};

const MINUTES: &str = "---\ncontract: minutes/v4\nmade: 2026-11-20T10:05:00+01:00\nprovider: \"local\"\nmodel: \"qwen3:8b\"\ncontext: null\ntitle: \"Prix & essai\"\n---\n\n## Points forts\n\nLe prix est fixé.\n\n## Actions\n\n- Écrire la page — Sam — vendredi\n\n## Points clés\n\n- 49 €\n";

const TRANSCRIPT: &str = concat!(
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":1.0,"end":3.0,"text":"Bonjour à tous."}"#,
    "\n",
);

/// A meeting's folder named as a session's is (`session-audio`), with its minutes and transcript.
fn meeting(dir: &Path) -> std::path::PathBuf {
    let folder = dir.join("2026-11-20_09-30");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("minutes.md"), MINUTES).unwrap();
    fs::write(folder.join("transcript.jsonl"), TRANSCRIPT).unwrap();
    folder
}

fn words(to: &Path) -> Value {
    json!({"to": to, "name": "Réunion du 20 nov. 2026", "transcript": "Transcription", "me": "Moi",
           "others": "Les autres", "time": "Heure", "speaker": "Qui", "text": "Texte"})
}

struct Ran {
    success: bool,
    code: Option<i32>,
    lines: Vec<Value>,
    said: String,
}

fn run(args: &[&str], input: &str) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kitokey-export"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the program starts");
    // A program that refuses its arguments ends before reading its input: a pipe closed under the write is
    // that end, and the run is judged by its exit and its sentence, as `exporting/v1` judges a refusal (#1068).
    if let Err(error) = child.stdin.take().unwrap().write_all(input.as_bytes()) {
        assert_eq!(
            error.kind(),
            std::io::ErrorKind::BrokenPipe,
            "the input could not be written: {error}"
        );
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).expect("its output is UTF-8");
    Ran {
        success: output.status.success(),
        code: output.status.code(),
        lines: stdout
            .lines()
            .map(|line| serde_json::from_str(line).expect("each line one JSON object"))
            .collect(),
        said: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn labels() -> Labels {
    Labels {
        transcript: "Transcription".into(),
        me: "Moi".into(),
        others: "Les autres".into(),
        time: "Heure".into(),
        speaker: "Qui".into(),
        text: "Texte".into(),
    }
}

#[test]
fn it_writes_the_file_whole_where_the_input_says_and_says_so_in_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    let out = dir.path().join("out");
    fs::create_dir(&out).unwrap();
    let to = out.join("réunion.md");
    fs::write(&to, "an older file, replaced").unwrap();

    let ran = run(
        &[folder.to_str().unwrap(), "--format", "md", "--lang", "fr"],
        &words(&to).to_string(),
    );

    assert!(ran.success, "{}", ran.said);
    let written = fs::read_to_string(&to).unwrap();
    let expected = markdown(&Meeting::read(&folder).unwrap(), Include::all(), &labels());
    assert_eq!(written, expected);
    assert_eq!(ran.lines, [json!({"event": "written", "file": to, "bytes": written.len()})]);
    assert!(ran.said.is_empty(), "nothing on the error stream: {}", ran.said);
    let left: Vec<_> = fs::read_dir(&out).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(left, [to.file_name().unwrap()], "no file beside it left behind");
}

#[test]
fn html_is_written_its_language_the_options_and_its_name_the_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    // No title from the summary: the page is named by the input's `name`.
    fs::write(folder.join("minutes.md"), MINUTES.replace("title: \"Prix & essai\"\n", "")).unwrap();
    let to = dir.path().join("réunion.html");

    let ran = run(
        &[folder.to_str().unwrap(), "--format", "html", "--lang", "fr-CA"],
        &words(&to).to_string(),
    );

    assert!(ran.success, "{}", ran.said);
    let written = fs::read_to_string(&to).unwrap();
    let expected = html(
        &Meeting::read(&folder).unwrap(),
        Include::all(),
        &labels(),
        "fr-CA",
        "Réunion du 20 nov. 2026",
    );
    assert_eq!(written, expected);
    assert!(written.contains("<html lang=\"fr-CA\">") && written.contains("<title>Réunion du 20 nov. 2026</title>"));
    assert_eq!(ran.lines, [json!({"event": "written", "file": to, "bytes": written.len()})]);
}

#[test]
fn docx_is_written_its_name_the_inputs_and_read_back_as_the_library_writes_it() {
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    // No title from the summary: the document is named by the input's `name`.
    fs::write(folder.join("minutes.md"), MINUTES.replace("title: \"Prix & essai\"\n", "")).unwrap();
    let to = dir.path().join("réunion.docx");

    let ran = run(
        &[folder.to_str().unwrap(), "--format", "docx", "--lang", "fr"],
        &words(&to).to_string(),
    );

    assert!(ran.success, "{}", ran.said);
    let written = fs::read(&to).unwrap();
    let expected = docx(
        &Meeting::read(&folder).unwrap(),
        Include::all(),
        &labels(),
        "Réunion du 20 nov. 2026",
    )
    .unwrap();
    let body = |bytes: &[u8]| -> Value {
        serde_json::from_str::<Value>(&docx_rs::read_docx(bytes).expect("a document Word's format reads").json()).unwrap()["document"]
            .clone()
    };
    assert_eq!(body(&written), body(&expected));
    let first = &body(&written)["children"][0]["data"];
    assert_eq!(first["property"]["style"], "Heading1");
    assert_eq!(
        first["children"][0]["data"]["children"][0]["data"]["text"],
        "Réunion du 20 nov. 2026"
    );
    assert_eq!(ran.lines, [json!({"event": "written", "file": to, "bytes": written.len()})]);
}

#[test]
fn what_is_left_out_is_the_options_choice_in_any_order() {
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    let to = dir.path().join("m.md");
    let ran = run(
        &[
            "--without-speakers",
            "--lang",
            "en",
            folder.to_str().unwrap(),
            "--without-times",
            "--format",
            "md",
        ],
        &words(&to).to_string(),
    );
    assert!(ran.success, "{}", ran.said);
    let include = Include {
        speakers: false,
        times: false,
        ..Include::all()
    };
    assert_eq!(
        fs::read_to_string(&to).unwrap(),
        markdown(&Meeting::read(&folder).unwrap(), include, &labels())
    );
}

#[test]
fn a_cause_a_person_acts_on_is_its_one_line_and_a_sentence_and_nothing_is_written() {
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    let empty = dir.path().join("2026-11-20_10-00");
    fs::create_dir(&empty).unwrap();
    let nowhere = dir.path().join("gone").join("m.md");
    let to = dir.path().join("m.md");
    for (args, to, code) in [
        (
            vec![folder.to_str().unwrap(), "--format", "md", "--lang", "fr"],
            &nowhere,
            "file.unwritable",
        ),
        (
            vec![dir.path().join("absent").to_str().unwrap(), "--format", "md", "--lang", "fr"],
            &to,
            "meeting.unreadable",
        ),
        (
            vec![empty.to_str().unwrap(), "--format", "md", "--lang", "fr"],
            &to,
            "meeting.empty",
        ),
        (
            vec![
                folder.to_str().unwrap(),
                "--format",
                "md",
                "--lang",
                "fr",
                "--without-summary",
                "--without-transcript",
            ],
            &to,
            "meeting.empty",
        ),
    ] {
        let ran = run(&args, &words(to).to_string());
        assert_eq!(ran.code, Some(1), "{code}: a cause of the contract ends at 1");
        assert_eq!(ran.lines, [json!({"event": "failed", "code": code})], "{code}");
        assert!(
            ran.said.starts_with("kitokey-export: ") && ran.said.lines().count() == 1,
            "{code}: {}",
            ran.said
        );
        assert!(!to.exists(), "{code}: nothing written");
    }
}

#[test]
fn a_file_that_cannot_take_its_place_leaves_nothing_beside_it() {
    // `to` an existing folder: the file is written beside it, and cannot be renamed over a folder.
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    let out = dir.path().join("out");
    let taken = out.join("taken.md");
    fs::create_dir_all(&taken).unwrap();
    let ran = run(
        &[folder.to_str().unwrap(), "--format", "md", "--lang", "fr"],
        &words(&taken).to_string(),
    );
    assert_eq!(ran.lines, [json!({"event": "failed", "code": "file.unwritable"})]);
    let left: Vec<_> = fs::read_dir(&out).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(left, [taken.file_name().unwrap()], "no file left half written beside it");
    assert!(fs::read_dir(&taken).unwrap().next().is_none(), "the folder in its place untouched");
}

#[test]
fn a_wrong_argument_or_input_is_a_sentence_alone() {
    let dir = tempfile::tempdir().unwrap();
    let folder = meeting(dir.path());
    let to = dir.path().join("m.md");
    let folder = folder.to_str().unwrap();
    let good = words(&to);
    let mut two_lines = good.clone();
    two_lines["name"] = json!("deux\nlignes");
    let mut missing = good.clone();
    missing.as_object_mut().unwrap().remove("speaker");
    for (args, input) in [
        (vec![folder, "--format", "odt", "--lang", "fr"], good.to_string()),
        // A format the contract names that this version does not write yet: the shell offers only those it does.
        (vec![folder, "--format", "pdf", "--lang", "fr"], good.to_string()),
        (vec![folder, "--format", "md"], good.to_string()),
        (
            vec![folder, "--format", "md", "--lang", "fr", "--to", to.to_str().unwrap()],
            good.to_string(),
        ),
        (vec![folder, "--format", "md", "--lang", "fr\nx"], good.to_string()),
        // An option given twice, as kitokey-summary refuses it (#1042).
        (vec![folder, "--format", "md", "--format", "md", "--lang", "fr"], good.to_string()),
        // An option this program does not know, never taken for the folder.
        (vec!["--format", "md", "--lang", "fr", "--without-everything"], good.to_string()),
        (vec![folder, "--format", "md", "--lang", "fr"], "not JSON".into()),
        (vec![folder, "--format", "md", "--lang", "fr"], two_lines.to_string()),
        (vec![folder, "--format", "md", "--lang", "fr"], missing.to_string()),
    ] {
        let ran = run(&args, &input);
        assert_eq!(
            ran.code,
            Some(2),
            "a wrong argument or input ends at 2, as a usage error: {args:?} {input}"
        );
        assert!(ran.lines.is_empty(), "no line names a wrong argument: {args:?} {:?}", ran.lines);
        assert!(ran.said.starts_with("kitokey-export: "), "{args:?}: {}", ran.said);
        assert!(!to.exists(), "{args:?}: nothing written");
    }
}

#[test]
fn without_an_argument_it_says_its_usage_as_the_other_programs_do() {
    // The package's e2e starts each program bare and reads « <program>: usage » (install's scripts/e2e.sh).
    let ran = run(&[], "");
    assert_eq!(ran.code, Some(2));
    assert!(ran.lines.is_empty());
    assert!(ran.said.starts_with("kitokey-export: usage: kitokey-export "), "{}", ran.said);
}

#[test]
fn it_says_its_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_kitokey-export"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("kitokey-export "));
}
