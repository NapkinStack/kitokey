//! The example meeting a newcomer tries (D126): bundled with Kitokey, one per language the window
//! speaks, placed in the meetings' folder as a meeting like any — its two tracks, its clock, its
//! title saying it is an example —, dated when it is tried.

use std::fs;

use kitokey_shell::{
    example,
    sessions::{self, State},
    words::Language,
};

const NOW: &str = "2026-12-01T09:30:00+01:00";

#[test]
fn the_example_is_placed_as_a_meeting_of_its_language_dated_when_tried() {
    let meetings = tempfile::tempdir().unwrap();
    for (language, title, spoken) in [
        (Language::French, "Exemple : point sur la version 2", "fr"),
        (Language::English, "Example: version 2 check-in", "en"),
    ] {
        let folder = example::place(meetings.path(), language, NOW).unwrap();
        assert!(folder.starts_with(meetings.path()));
        for file in ["session.json", "clock.jsonl", "local.flac", "remote.flac"] {
            assert!(folder.join(file).is_file(), "{language:?}: {file}");
        }
        assert_eq!(fs::read_to_string(folder.join("title.txt")).unwrap().trim(), title);
        assert_eq!(fs::read_to_string(folder.join("language")).unwrap().trim(), spoken);
        // Listed as a meeting ended, started when it was tried, of about 45 seconds.
        let listed = sessions::list(meetings.path(), None);
        let it = listed.iter().find(|listed| listed.folder == folder).unwrap();
        assert_eq!(it.state, State::Done);
        assert_eq!(it.started, NOW);
        let seconds = sessions::length(&folder).unwrap();
        assert!((30.0..=60.0).contains(&seconds), "{seconds}");
    }
}

#[test]
fn an_example_tried_again_is_another_meeting() {
    let meetings = tempfile::tempdir().unwrap();
    let first = example::place(meetings.path(), Language::French, NOW).unwrap();
    let second = example::place(meetings.path(), Language::French, NOW).unwrap();
    assert_ne!(first, second);
    assert_eq!(sessions::list(meetings.path(), None).len(), 2);
}

#[test]
fn the_meetings_folder_not_made_yet_is_made() {
    let home = tempfile::tempdir().unwrap();
    let meetings = home.path().join("Kitokey");
    let folder = example::place(&meetings, Language::English, NOW).unwrap();
    assert!(folder.join("session.json").is_file());
}

/// What the examples add to the package, kept under a ceiling: D102's budget is read against it.
#[test]
fn the_examples_weigh_no_more_than_their_ceiling() {
    let bytes: usize = example::BUNDLED
        .iter()
        .map(|(_, files)| files.iter().map(|(_, bytes)| bytes.len()).sum::<usize>())
        .sum();
    assert!(bytes <= 3_000_000, "{bytes} bytes");
}
