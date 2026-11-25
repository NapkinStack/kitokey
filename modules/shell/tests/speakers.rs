//! The names a person gives a meeting's speakers (#1205): `speakers.json` of `contracts/session-audio/v3`, written
//! whole by the shell on the user's word, read back as the contract says — a name it refuses reads as absent, the rest
//! still read; a file that is no JSON object reads as no file — and kept to a v3 meeting, the version that names it.

use std::{fs, io::ErrorKind, path::Path};

use kitokey_shell::speakers::{self, Speakers};

fn meeting(contract: &str) -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    fs::write(
        folder.path().join("session.json"),
        format!(r#"{{"contract":"{contract}","started":"2026-11-11T14:05:00+01:00","stopped":"2026-11-11T14:35:00+01:00","tracks":{{}}}}"#),
    )
    .unwrap();
    folder
}

fn named(me: Option<&str>, others: Option<&str>) -> Speakers {
    Speakers {
        me: me.map(str::to_string),
        others: others.map(str::to_string),
    }
}

fn file(folder: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(folder.join("speakers.json")).unwrap()).unwrap()
}

#[test]
fn both_sides_named_are_written_and_read_back() {
    let folder = meeting("session-audio/v3");
    let written = speakers::write(folder.path(), &named(Some("Sarah"), Some("L’équipe produit"))).unwrap();
    assert_eq!(written, named(Some("Sarah"), Some("L’équipe produit")));
    assert_eq!(
        file(folder.path()),
        serde_json::json!({"me": "Sarah", "others": "L’équipe produit"})
    );
    assert_eq!(speakers::read(folder.path()), written);
    // Written whole: nothing left beside it.
    let left: Vec<_> = fs::read_dir(folder.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(left.len(), 2, "{left:?}");
}

#[test]
fn a_name_is_trimmed_and_a_blank_one_gives_the_side_back_its_word() {
    let folder = meeting("session-audio/v3");
    let written = speakers::write(folder.path(), &named(Some("  Sarah "), Some("   "))).unwrap();
    assert_eq!(written, named(Some("Sarah"), None));
    assert_eq!(file(folder.path()), serde_json::json!({"me": "Sarah"}));
    // Both blank: no name left, no file.
    assert_eq!(speakers::write(folder.path(), &named(Some(""), None)).unwrap(), Speakers::default());
    assert!(!folder.path().join("speakers.json").exists());
}

#[test]
fn a_name_the_contract_refuses_is_not_written() {
    let folder = meeting("session-audio/v3");
    speakers::write(folder.path(), &named(Some("Sarah"), None)).unwrap();
    let long = "a".repeat(61);
    for refused in [named(Some("Sa\nrah"), None), named(None, Some("Hu\rgo")), named(Some(&long), None)] {
        let error = speakers::write(folder.path(), &refused).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput, "{refused:?}");
    }
    // Sixty characters, not bytes: accented names at the bound still go.
    let sixty = "é".repeat(60);
    assert_eq!(
        speakers::write(folder.path(), &named(Some(&sixty), None)).unwrap().me.as_deref(),
        Some(sixty.as_str())
    );
}

#[test]
fn only_a_v3_meeting_holds_names() {
    for older in ["session-audio/v1", "session-audio/v2"] {
        let folder = meeting(older);
        assert!(!speakers::offered(folder.path()), "{older}");
        let error = speakers::write(folder.path(), &named(Some("Sarah"), None)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Unsupported, "{older}");
        assert!(!folder.path().join("speakers.json").exists());
    }
    assert!(speakers::offered(meeting("session-audio/v3").path()));
    let none = tempfile::tempdir().unwrap();
    assert!(!speakers::offered(none.path()));
}

#[test]
fn read_takes_what_the_contract_allows_and_nothing_else() {
    let folder = meeting("session-audio/v3");
    let at = folder.path().join("speakers.json");
    // A name refused reads as absent, the rest still reads; voices are not the shell's to show yet.
    fs::write(&at, r#"{"me":"Sarah","others":"   ","voices":{"remote/1":"Hugo"},"extra":1}"#).unwrap();
    assert_eq!(speakers::read(folder.path()), named(Some("Sarah"), None));
    fs::write(&at, r#"{"me":42,"others":"Camille, Hugo et Léa"}"#).unwrap();
    assert_eq!(speakers::read(folder.path()), named(None, Some("Camille, Hugo et Léa")));
    fs::write(&at, format!(r#"{{"me":"{}"}}"#, "b".repeat(61))).unwrap();
    assert_eq!(speakers::read(folder.path()), Speakers::default());
    // No JSON object: no file.
    for text in ["[\"Sarah\"]", "not json", ""] {
        fs::write(&at, text).unwrap();
        assert_eq!(speakers::read(folder.path()), Speakers::default(), "{text}");
    }
    fs::remove_file(&at).unwrap();
    assert_eq!(speakers::read(folder.path()), Speakers::default());
}

#[test]
fn renaming_keeps_the_voices_another_producer_wrote() {
    // The shell writes no voice; one already there (a later producer's, by the person's word) is kept as it was.
    let folder = meeting("session-audio/v3");
    fs::write(folder.path().join("speakers.json"), r#"{"me":"Old","voices":{"remote/1":"Hugo"}}"#).unwrap();
    speakers::write(folder.path(), &named(Some("Sarah"), None)).unwrap();
    assert_eq!(
        file(folder.path()),
        serde_json::json!({"me": "Sarah", "voices": {"remote/1": "Hugo"}})
    );
}

#[test]
fn each_side_is_said_by_its_name_else_by_the_words_given() {
    // What the shell hands kitokey-export (exporting/v1's `me` and `others`): the names kept, else its words.
    assert_eq!(named(Some("Sarah"), None).sides("Vous", "Les autres"), ("Sarah", "Les autres"));
    assert_eq!(
        named(None, Some("L’équipe produit")).sides("You", "Others"),
        ("You", "L’équipe produit")
    );
    assert_eq!(Speakers::default().sides("Vous", "Les autres"), ("Vous", "Les autres"));
}
