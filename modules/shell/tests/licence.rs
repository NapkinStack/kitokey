//! The licence as the shell sees it, and the trial's gate before a recording (PDR-0005, D83):
//! refused only on a trial licence whose trial is over; anything else records, as today.

use std::{fs, path::Path};

use kitokey_shell::{
    licence::{self, Licence},
    sessions::instant,
    trial::{self, Left},
};

const DAY: i64 = 86_400;

/// The day the trial began, in these tests.
fn first() -> i64 {
    instant("2026-11-05T09:00:00Z").unwrap()
}

/// A folder of meetings whose trial began on the first day, with ten meetings made here the
/// next day: over 14 days after the first.
fn ten_made() -> tempfile::TempDir {
    let meetings = tempfile::tempdir().unwrap();
    trial::begin(meetings.path(), first()).unwrap();
    for index in 0..10 {
        let name = format!("2026-11-06_10-{index:02}");
        let folder = meetings.path().join(&name);
        fs::create_dir_all(&folder).unwrap();
        let json = format!(
            r#"{{"contract":"session-audio/v2","started":"2026-11-06T10:{index:02}:00Z","stopped":"2026-11-06T10:{index:02}:30Z","tracks":{{}}}}"#
        );
        fs::write(folder.join("session.json"), json).unwrap();
        trial::made(meetings.path(), &name).unwrap();
    }
    meetings
}

#[test]
fn a_recording_is_refused_on_a_trial_licence_once_its_trial_is_over() {
    let meetings = ten_made();
    assert!(licence::refused(&Ok(Licence::Trial), meetings.path(), first() + 14 * DAY));
    assert!(licence::refused(&Ok(Licence::Trial), meetings.path(), first() + 400 * DAY));
}

#[test]
fn a_trial_not_over_records() {
    let meetings = ten_made();
    assert!(!licence::refused(&Ok(Licence::Trial), meetings.path(), first() + 14 * DAY - 1));
    // No trial's day kept in this folder of meetings: never over.
    let none = tempfile::tempdir().unwrap();
    assert!(!licence::refused(&Ok(Licence::Trial), none.path(), first() + 400 * DAY));
}

#[test]
fn a_licence_bought_not_active_not_read_or_not_from_the_store_records_as_today() {
    let meetings = ten_made();
    for licence in [
        Ok(Licence::Bought),
        Ok(Licence::Inactive),
        Ok(Licence::NotFromStore),
        Err("the Store did not answer".to_string()),
    ] {
        assert!(!licence::refused(&licence, meetings.path(), first() + 400 * DAY), "{licence:?}");
    }
}

#[test]
fn settings_say_what_is_left_on_a_trial_licence_alone() {
    let meetings = ten_made();
    let trial = Ok(Licence::Trial);
    assert_eq!(
        licence::left(&trial, meetings.path(), first() + 3 * DAY),
        Some(Left { days: 11, meetings: 0 })
    );
    // Over: nothing is left to say; the refusal says the rest.
    assert_eq!(licence::left(&trial, meetings.path(), first() + 14 * DAY), None);
    // Before its first day, the trial whole.
    let none = tempfile::tempdir().unwrap();
    assert_eq!(licence::left(&trial, none.path(), first()), Some(Left { days: 14, meetings: 10 }));
    for licence in [
        Ok(Licence::Bought),
        Ok(Licence::Inactive),
        Ok(Licence::NotFromStore),
        Err("the Store did not answer".to_string()),
    ] {
        assert_eq!(licence::left(&licence, meetings.path(), first() + 3 * DAY), None, "{licence:?}");
        assert_eq!(licence::left(&licence, none.path(), first()), None, "{licence:?}");
    }
}

/// What the stand-in says, and what `read` answers then.
fn says(file: &Path, word: &str) -> Result<Licence, String> {
    fs::write(file, word).unwrap();
    licence::read()
}

/// The one test that names the stand-in in the environment: tests run side by side, and the
/// environment is the process's.
#[test]
fn the_stand_in_is_read_anew_at_each_start_and_by_a_build_for_the_tests_alone() {
    // SAFETY: no other test of this file reads or writes the environment.
    unsafe { std::env::remove_var("KITOKEY_LICENCE") };
    assert_eq!(licence::read(), Ok(Licence::NotFromStore));
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("licence");
    // SAFETY: as above.
    unsafe { std::env::set_var("KITOKEY_LICENCE", &file) };
    if cfg!(debug_assertions) {
        // Not there: a licence that could not be read.
        assert!(licence::read().is_err());
        // Read again each time: turned bought while Kitokey is open, it is bought at the next start.
        assert_eq!(says(&file, "trial\n"), Ok(Licence::Trial));
        assert_eq!(says(&file, "bought\n"), Ok(Licence::Bought));
        assert_eq!(says(&file, "inactive"), Ok(Licence::Inactive));
        assert_eq!(says(&file, "none"), Ok(Licence::NotFromStore));
        assert!(says(&file, "unread").is_err());
    } else {
        // A build made for release never reads it: `cargo test --release --test licence`.
        for word in ["trial", "bought", "inactive", "unread"] {
            assert_eq!(says(&file, word), Ok(Licence::NotFromStore), "{word}");
        }
    }
}
