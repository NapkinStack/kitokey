//! Minutes in a structure (D111): `--structure <name>` (`summarising/v1`) writes `minutes/v5`, its
//! sections the structure's, each named by its id; without it, `minutes/v4` as before. Against a
//! provider played on this machine: no online service, no meeting of anyone.

mod common;

use std::{fs, path::Path, process::Output};

use common::{answering, program, provider, providing};

const TRANSCRIPT: &str = concat!(
    r#"{"type": "header", "contract": "transcript/v1", "engine": "an-engine", "model": "a-model", "language": "fr", "context": null}"#,
    "\n",
    r#"{"type": "segment", "speaker": "others", "start": 0.0, "end": 3.2, "text": "Hier j'ai fini la recette, aujourd'hui je livre."}"#,
    "\n",
);

const STAND_UP: &str = "## Avancées\n\n- Recette finie, livraison aujourd'hui [00:01]\n\n## Blocages\n\nAucun.\n\n## Actions\n\n- Livrer — non précisé — aujourd'hui [00:02]\n";

fn session() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    folder
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
fn front_matter(minutes: &str) -> serde_json::Map<String, serde_json::Value> {
    let inside = minutes.strip_prefix("---\n").unwrap().split_once("\n---\n").unwrap().0;
    inside
        .lines()
        .map(|line| {
            let (key, value) = line.split_once(": ").unwrap();
            let value = if value.starts_with(['"', '[']) {
                serde_json::from_str(value).unwrap()
            } else {
                serde_json::Value::String(value.into())
            };
            (key.to_string(), value)
        })
        .collect()
}

#[test]
fn a_structure_given_writes_minutes_v5_its_sections_named_by_their_ids() {
    let folder = session();
    let (address, received) = provider("200 OK", answering(STAND_UP, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--provider", "ADDRESS", "--model", "a-model", "--structure", "stand_up"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    // The model is asked the structure's sections, in their order, and their number.
    let (_, asked) = received.recv().unwrap();
    let asked: serde_json::Value = serde_json::from_str(&asked).unwrap();
    let content = asked["messages"][0]["content"].as_str().unwrap();
    assert!(content.contains("exactly 3 sections"), "{content}");
    let (progress, blockers) = (
        content.find("did and will do next").unwrap(),
        content.find("stops someone").unwrap(),
    );
    assert!(progress < blockers, "{content}");

    let minutes = fs::read_to_string(folder.path().join("minutes.md")).unwrap();
    let keys = front_matter(&minutes);
    assert_eq!(keys["contract"], "minutes/v5");
    assert_eq!(keys["structure"], "stand_up");
    assert_eq!(keys["actions"], "actions");
    let ids: Vec<&str> = keys["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|section| section["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["progress", "blockers", "actions"]);
    for section in keys["sections"].as_array().unwrap() {
        let about = section["about"].as_str().unwrap();
        assert!(
            !about.is_empty() && !about.contains('\n') && about.chars().count() <= 200,
            "{about}"
        );
    }
    // The body is the answer's: one section of second level for each id, in that order.
    assert!(minutes.ends_with(STAND_UP), "{minutes}");
}

#[test]
fn each_structure_the_contract_names_is_summarys_own_with_the_same_ids() {
    // Windows' checkout writes `\r\n` (CI, run 44893481583): the page's paragraphs are found all the same.
    let page = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts/minutes/v5/README.md"))
        .unwrap()
        .replace("\r\n", "\n");
    let table = page[page.find("The six structures this version names").unwrap()..]
        .split("\n\n")
        .nth(1)
        .unwrap();
    let mut named = 0;
    for row in table.lines().skip(2) {
        let cells: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
        let name = cells[0].trim_matches('`');
        let ids: Vec<&str> = cells[1]
            .split(" — ")
            .next()
            .unwrap()
            .split(", ")
            .map(|id| id.trim_matches('`'))
            .collect();
        let structure = kitokey_summary::structure(name).unwrap_or_else(|| panic!("{name} is not summary's"));
        assert_eq!(
            structure.sections.iter().map(|section| section.id).collect::<Vec<_>>(),
            ids,
            "{name}"
        );
        assert_eq!(structure.actions, Some(cells[2].trim_matches('`')), "{name}");
        named += 1;
    }
    assert_eq!(named, 6);
    assert_eq!(kitokey_summary::STRUCTURES.len(), 6);
}

#[test]
fn an_answer_without_the_structures_number_of_sections_is_not_minutes() {
    let folder = session();
    // interview has four sections; the answer, three, asked for once more, then refused.
    let (address, received) = providing(vec![("200 OK", answering(STAND_UP, "stop"))]);
    let output = run(
        folder.path(),
        &address,
        &["--structure", "interview", "--provider", "ADDRESS", "--model", "a-model"],
    );
    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("3 section(s) of second level where 4 are expected"), "{err}");
    assert_eq!(received.try_iter().count(), 2);
    assert!(!folder.path().join("minutes.md").exists());
}

#[test]
fn a_structure_it_does_not_know_is_a_wrong_argument_and_nothing_leaves() {
    let folder = session();
    let (address, received) = provider("200 OK", answering(STAND_UP, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--provider", "ADDRESS", "--model", "a-model", "--structure", "weekly"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("kitokey-summary: "));
    assert!(output.stdout.is_empty(), "nothing said sent");
    assert_eq!(received.try_iter().count(), 0);
    assert!(!folder.path().join("minutes.md").exists());
}

#[test]
fn without_a_structure_the_minutes_are_v4_as_before() {
    let folder = session();
    let (address, _received) = provider("200 OK", answering(common::MINUTES, "stop"));
    let output = run(folder.path(), &address, &["--model", "a-model", "--provider", "ADDRESS"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let minutes = fs::read_to_string(folder.path().join("minutes.md")).unwrap();
    let keys = front_matter(&minutes);
    assert_eq!(keys["contract"], "minutes/v4");
    assert!(!keys.contains_key("structure") && !keys.contains_key("sections") && !keys.contains_key("actions"));
}

#[test]
fn in_a_structure_every_sections_items_have_their_times_checked() {
    let folder = session();
    // A time in no segment, in the first section: gone, as under Actions; one that lies in one, kept.
    let answer = STAND_UP.replace("aujourd'hui [00:01]", "aujourd'hui [09:59]");
    let (address, _received) = provider("200 OK", answering(&answer, "stop"));
    let output = run(
        folder.path(),
        &address,
        &["--provider", "ADDRESS", "--model", "a-model", "--structure", "stand_up"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let minutes = fs::read_to_string(folder.path().join("minutes.md")).unwrap();
    assert!(minutes.contains("- Recette finie, livraison aujourd'hui\n"), "{minutes}");
    assert!(minutes.contains("aujourd'hui [00:02]"), "{minutes}");
}

#[test]
fn an_option_given_twice_is_a_wrong_argument_and_nothing_leaves() {
    let folder = session();
    let (address, received) = provider("200 OK", answering(STAND_UP, "stop"));
    let output = run(
        folder.path(),
        &address,
        &[
            "--provider",
            "ADDRESS",
            "--model",
            "a-model",
            "--structure",
            "stand_up",
            "--structure",
            "general",
        ],
    );
    assert!(!output.status.success() && output.stdout.is_empty());
    assert_eq!(received.try_iter().count(), 0);
    assert!(!folder.path().join("minutes.md").exists());
}

#[test]
fn an_interview_is_asked_never_to_judge_the_candidate() {
    // minutes/v5: « It never judges a person » — an evaluation of a candidate is high-risk under
    // the AI Act (Annex III, 4(a)). The ids are held by the contract; what the model is asked, here.
    let asked = kitokey_summary::request_in(&[], None, kitokey_summary::structure("interview"));
    let interview: Vec<&str> = asked
        .lines()
        .filter(|line| line.starts_with(|c: char| c.is_ascii_digit()))
        .collect();
    assert_eq!(interview.len(), 4, "{asked}");
    assert!(interview[0].contains("Never judge the candidate"), "{asked}");
    assert!(interview[1].contains("never a judgement"), "{asked}");
    for word in ["strength", "weakness", "concern", "score", "rating"] {
        assert!(!asked.to_lowercase().contains(word), "{word}: {asked}");
    }
}

#[test]
fn in_a_structure_the_model_is_given_the_skeleton_to_copy() {
    // D111's second spike: asked by the sections' order alone, qwen3:8b held 3 of the 6 structures
    // (no `## ` line, content in a title); given this skeleton to copy, 6 of 6, every run.
    for structure in &kitokey_summary::STRUCTURES {
        let asked = kitokey_summary::request_in(&[], None, Some(structure));
        let mut skeleton = String::from("\n# <the meeting's title>\n\n");
        for n in 1..=structure.sections.len() {
            skeleton.push_str(&format!("## <section {n}'s title>\n<section {n}'s content>\n\n"));
        }
        assert!(asked.contains(&skeleton), "{}: {asked}", structure.name);
        assert!(asked.contains("copied line for line"), "{}: {asked}", structure.name);
        assert!(
            asked.contains("naming the section, never its content"),
            "{}: {asked}",
            structure.name
        );
        assert!(!asked.contains("in this order:"), "{}: {asked}", structure.name);
        // v4's count of titles is not the structure's: no number of titles but the structure's.
        assert!(!asked.contains("the three titles"), "{}: {asked}", structure.name);
        assert!(asked.contains("the titles included"), "{}: {asked}", structure.name);
    }
    // Without a structure, v4's request as before.
    assert!(!kitokey_summary::request(&[], None).contains("<section 1's title>"));
}
