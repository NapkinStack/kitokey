//! The program as a shell starts it, where it opens no device: what it refuses is logged,
//! never with a session's path. What a recording logs is the e2e's (scripts/e2e.sh).

use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// A run refused before any device is opened, and the lines of its log.
fn refused(args: &[&str], logs: &Path) -> (String, Vec<serde_json::Value>, String) {
    // No sound server within its reach, no ALSA device defined, and a short time: were a guard to fail,
    // it opens no real device and does not stay (seen under strace: no `/dev/snd` opened, no
    // source-output, "no source could be opened").
    let nowhere = tempfile::tempdir().unwrap();
    let alsa = nowhere.path().join("alsa.conf");
    fs::write(&alsa, "").unwrap();
    let mut run = Command::new(env!("CARGO_BIN_EXE_kitokey-capture"))
        .args(args)
        .env("KITOKEY_LOG_DIR", logs)
        .env("KITOKEY_TRACE", "making-9")
        // The lowest level a user may ask: a failure is logged, with its id.
        .env("KITOKEY_LOG", "error")
        .env("PIPEWIRE_REMOTE", "kitokey-none")
        .env("PULSE_SERVER", format!("unix:{}/none", nowhere.path().display()))
        .env("XDG_RUNTIME_DIR", nowhere.path())
        .env("ALSA_CONFIG_PATH", &alsa)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let asked = Instant::now();
    while run.try_wait().unwrap().is_none() && asked.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = run.kill();
    let output = run.wait_with_output().unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    let files: Vec<_> = fs::read_dir(logs).unwrap().map(|entry| entry.unwrap().path()).collect();
    assert_eq!(files.len(), 1, "{files:?}");
    let name = files[0].file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("capture.") && name.ends_with(".jsonl"), "{name}");
    let text = fs::read_to_string(&files[0]).unwrap();
    let lines = text.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    (String::from_utf8_lossy(&output.stderr).into_owned(), lines, text)
}

#[test]
fn a_refusal_is_logged_with_its_detail_under_the_makings_id() {
    let logs = tempfile::tempdir().unwrap();
    let (err, lines, _) = refused(&["listen"], logs.path());
    assert!(err.contains("usage: kitokey-capture"), "{err}");
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["message"], "capture.failed");
    assert_eq!(lines[0]["level"], "ERROR");
    assert_eq!(lines[0]["span"]["trace"], "making-9");
    assert_eq!(lines[0]["span"]["program"], "capture");
    assert!(lines[0]["detail"].as_str().unwrap().starts_with("usage: kitokey-capture"));
}

#[test]
fn a_microphone_named_is_refused_with_the_first_form_which_follows_nothing() {
    let logs = tempfile::tempdir().unwrap();
    let sessions = tempfile::tempdir().unwrap();
    let (err, _, _) = refused(
        &["record", sessions.path().to_str().unwrap(), "--as", "v1", "--microphone", "any"],
        logs.path(),
    );
    assert!(err.contains("usage: kitokey-capture"), "{err}");
    assert_eq!(fs::read_dir(sessions.path()).unwrap().count(), 0);
}

#[test]
fn a_sessions_folder_that_cannot_hold_one_is_logged_without_its_path() {
    let logs = tempfile::tempdir().unwrap();
    // A file where the sessions' folder should be: refused before any device is opened.
    let not_a_folder = tempfile::NamedTempFile::new().unwrap();
    let sessions = not_a_folder.path().join("sessions");
    let (_, lines, text) = refused(&["record", sessions.to_str().unwrap()], logs.path());
    assert_eq!(lines.last().unwrap()["message"], "capture.failed", "{text}");
    assert!(!text.contains(&*not_a_folder.path().to_string_lossy()), "{text}");
}

/// A log folder whose day file cannot take a byte: every write to it fails.
#[cfg(unix)]
fn full_logs(program: &str) -> tempfile::TempDir {
    let logs = tempfile::tempdir().unwrap();
    let today = std::process::Command::new("date").args(["-u", "+%F"]).output().unwrap();
    let today = String::from_utf8(today.stdout).unwrap();
    std::os::unix::fs::symlink("/dev/full", logs.path().join(format!("{program}.{}.jsonl", today.trim()))).unwrap();
    logs
}

#[cfg(unix)]
#[test]
fn a_log_that_cannot_be_written_says_nothing_on_the_error_stream() {
    let logs = full_logs("capture");
    let output = Command::new(env!("CARGO_BIN_EXE_kitokey-capture"))
        .arg("listen")
        .env("KITOKEY_LOG_DIR", logs.path())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(err.starts_with("kitokey-capture: usage"), "{err}");
}

/// A refusal's run with this environment, nothing else of the machine's: where its log went.
fn logged_with(env: &[(&str, &Path)], texts: &[(&str, &str)]) -> Vec<std::path::PathBuf> {
    let nowhere = tempfile::tempdir().unwrap();
    let mut run = Command::new(env!("CARGO_BIN_EXE_kitokey-capture"));
    run.arg("listen")
        .env_clear()
        .env("PIPEWIRE_REMOTE", "kitokey-none")
        .env("XDG_RUNTIME_DIR", nowhere.path());
    for (name, value) in env {
        run.env(name, value);
    }
    for (name, value) in texts {
        run.env(name, value);
    }
    assert!(!run.output().unwrap().status.success());
    let mut files = Vec::new();
    for (_, folder) in env {
        for entry in walk(folder) {
            files.push(entry);
        }
    }
    files.sort();
    files.dedup();
    files
}

fn walk(folder: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else { return Vec::new() };
    entries
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() { walk(&path) } else { vec![path] }
        })
        .collect()
}

#[test]
fn the_names_before_d86_are_read_no_more() {
    // Every shell passes Kitokey's names since D86: Wholeheard's are no one's, and ignored.
    let (old, state) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let files = logged_with(
        &[
            ("WHOLEHEARD_LOG_DIR", old.path()),
            ("XDG_STATE_HOME", state.path()),
            ("HOME", state.path()),
        ],
        &[("WHOLEHEARD_TRACE", "old-1"), ("WHOLEHEARD_LOG", "off")],
    );
    assert!(
        files.iter().all(|file| !file.starts_with(old.path())),
        "a log where WHOLEHEARD_LOG_DIR says: {files:?}"
    );
    for file in &files {
        assert!(
            !fs::read_to_string(file).unwrap().contains("old-1"),
            "the id read from WHOLEHEARD_TRACE"
        );
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    assert_eq!(files.len(), 1, "WHOLEHEARD_LOG=off taken: {files:?}");
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn unasked_the_log_is_in_kitokeys_folder_of_the_systems_state() {
    let state = tempfile::tempdir().unwrap();
    let files = logged_with(&[("XDG_STATE_HOME", state.path()), ("HOME", state.path())], &[]);
    assert_eq!(files.len(), 1, "{files:?}");
    assert_eq!(files[0].parent().unwrap(), state.path().join("kitokey/logs"), "{files:?}");
}
