//! The language a meeting is spoken in (D65): one line, a code of ISO 639-1.

use std::fs;

use kitokey_shell::spoken::{FILE, code, keep_next, mark, read, read_next, taken};

fn folder(name: &str) -> std::path::PathBuf {
    let made = std::env::temp_dir().join(format!("kitokey-spoken-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&made);
    fs::create_dir_all(&made).unwrap();
    made
}

#[test]
fn a_code_is_two_lower_case_letters_and_nothing_else() {
    assert_eq!(code(" fr\n"), Some("fr".to_string()));
    assert_eq!(code("zh"), Some("zh".to_string()));
    for not in ["", "FR", "fr-FR", "fra", "f", "f1", "é1", "  ", "zz", "qq", "xx"] {
        assert_eq!(code(not), None, "{not:?}");
    }
}

#[test]
fn said_written_whole_read_back_removed_and_refused_before_session_json_or_for_no_code() {
    let dir = folder("mark");
    assert_eq!(mark(&dir, Some("en")).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    assert!(!dir.join(FILE).exists());
    fs::write(dir.join("session.json"), "{}").unwrap();
    mark(&dir, Some("en")).unwrap();
    assert_eq!(fs::read_to_string(dir.join(FILE)).unwrap(), "en\n");
    assert_eq!(read(&dir), Some("en".to_string()));
    assert_eq!(mark(&dir, Some("English")).unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(read(&dir), Some("en".to_string()));
    mark(&dir, None).unwrap();
    assert!(!dir.join(FILE).exists());
    mark(&dir, None).unwrap();
    assert_eq!(read(&dir), None);
    // Written by hand, not a code: the meeting names none of its own.
    fs::write(dir.join(FILE), "Français\n").unwrap();
    assert_eq!(read(&dir), None);
    assert!(!dir.join(".language.part").exists());
    fs::remove_dir_all(dir).unwrap();
}

/// The next meeting's language chosen in the bar at rest (D99): kept in its file, read back from it
/// alone as a Kitokey started again reads it; a text that is no code refused, the one kept left.
#[test]
fn the_next_meetings_language_is_kept_across_starts_none_removes_it_and_no_code_is_refused() {
    let dir = folder("next");
    let file = dir.join("settings").join("next-language.txt");
    assert_eq!(read_next(&file), None);
    keep_next(&file, Some("en")).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "en\n");
    assert_eq!(read_next(&file), Some("en".to_string()));
    assert_eq!(
        keep_next(&file, Some("English")).unwrap_err().kind(),
        std::io::ErrorKind::InvalidInput
    );
    assert_eq!(read_next(&file), Some("en".to_string()));
    assert!(!file.with_extension("part").exists());
    keep_next(&file, None).unwrap();
    assert!(!file.exists());
    keep_next(&file, None).unwrap();
    // Written by hand, not a code: none kept.
    fs::write(&file, "Deutsch\n").unwrap();
    assert_eq!(read_next(&file), None);
    fs::remove_dir_all(dir).unwrap();
}

/// Taken by the meeting once its folder holds its session (D99): written there, then none kept; none
/// chosen leaves the meeting's own as it is; refused before `session.json`, the next one kept.
#[test]
fn the_next_meetings_language_is_taken_by_the_meeting_then_none_is_kept() {
    let dir = folder("taken");
    let mut next = Some("de".to_string());
    assert_eq!(taken(&dir, &mut next).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    assert_eq!(next.as_deref(), Some("de"));
    fs::write(dir.join("session.json"), "{}").unwrap();
    taken(&dir, &mut next).unwrap();
    assert_eq!(read(&dir), Some("de".to_string()));
    assert_eq!(next, None);
    taken(&dir, &mut next).unwrap();
    assert_eq!(read(&dir), Some("de".to_string()));
    fs::remove_dir_all(dir).unwrap();
}
