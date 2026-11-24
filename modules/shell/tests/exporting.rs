//! The dates ticked on a meeting's page, written as an `.ics` by `kitokey-export` (D115, `exporting/v1`'s
//! `ics` and `dates`): the program started as the contract says, the dates on its input, each on one line where
//! the contract wants one; and the Markdown export unchanged, no `dates` on its input.
#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use kitokey_shell::exporting::{self, Dated, Input};
use serde_json::{Value, json};

/// A `kitokey-export` that keeps its arguments and its input beside it, writes the file and says `written`.
fn program(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("kitokey-export");
    fs::write(
        &path,
        "#!/bin/sh\nhere=$(dirname \"$0\")\nprintf '%s\\n' \"$*\" > \"$here/args\"\ncat > \"$here/input\"\n\
         to=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))[\"to\"])' \"$here/input\")\n\
         printf 'BEGIN:VCALENDAR' > \"$to\"\necho '{\"event\": \"written\", \"file\": \"'\"$to\"'\", \"bytes\": 15}'\n",
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn input<'a>(to: &'a Path, dates: Option<&'a [Dated]>) -> Input<'a> {
    Input {
        to,
        name: "Point client",
        transcript: "Transcription",
        me: "Moi",
        others: "Les autres",
        time: "Heure",
        speaker: "Qui",
        text: "Texte",
        dates,
    }
}

#[test]
fn the_ticked_dates_go_to_kitokey_export_as_an_ics_each_on_one_line_where_the_contract_wants_one() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("2026-11-24_23-30");
    fs::create_dir(&folder).unwrap();
    let to = dir.path().join("dates.ics");
    let dates = [
        Dated::new(
            1,
            "2026-11-26",
            "Envoyer le devis\nvite",
            Some("Inès\r"),
            Some("Tu envoies le devis jeudi ?\nOui, jeudi."),
        ),
        Dated::new(3, "2026-12-12", "Rappeler Jiří", None, None),
    ];
    exporting::ics(&program(dir.path()), &folder, "fr", &input(&to, Some(&dates))).unwrap();
    let args = fs::read_to_string(dir.path().join("args")).unwrap();
    assert_eq!(args.trim(), format!("{} --format ics --lang fr", folder.display()));
    let given: Value = serde_json::from_str(&fs::read_to_string(dir.path().join("input")).unwrap()).unwrap();
    assert_eq!(
        given["dates"],
        json!([
            {"id": "a1", "date": "2026-11-26", "what": "Envoyer le devis vite", "who": "Inès", "passage": "Tu envoies le devis jeudi ?\nOui, jeudi."},
            {"id": "a3", "date": "2026-12-12", "what": "Rappeler Jiří", "who": null, "passage": null},
        ])
    );
    assert_eq!(given["name"], "Point client");
    assert!(to.is_file());
}

#[test]
fn the_markdown_export_gives_no_dates() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("2026-11-24_23-30");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("minutes.md"), "x").unwrap();
    let to = dir.path().join("x.md");
    exporting::markdown(&program(dir.path()), &folder, "fr", &input(&to, None)).unwrap();
    let given: Value = serde_json::from_str(&fs::read_to_string(dir.path().join("input")).unwrap()).unwrap();
    assert!(given.get("dates").is_none(), "{given}");
    assert!(fs::read_to_string(dir.path().join("args")).unwrap().contains("--format md"));
}
