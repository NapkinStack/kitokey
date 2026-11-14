//! A recording left running by a Kitokey ended before this one, found again (#773): what is kept of
//! it, and what is found. Played by a script that writes a session as `session-audio` describes it and
//! stops on a termination signal as `recording/v1` says. No sound, no device, nobody recorded.
#![cfg(target_os = "linux")]

use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::Receiver,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use kitokey_shell::recording::{self, Found, Kept, Said};

/// A session that started 90 seconds ago, its `session.json` written whole at once.
const SESSION: &str = r#"
folder="$1/2026-11-13_09-00"; mkdir -p "$folder"
since=$(date -d '-90 seconds' +%Y-%m-%dT%H:%M:%S%:z)
write() { printf '{"contract":"session-audio/v3","started":"%s"%s,"tracks":{}}\n' "$since" "$1" > "$folder/session.json.part"; mv "$folder/session.json.part" "$folder/session.json"; }
stopped() { write ",\"stopped\":\"$(date +%Y-%m-%dT%H:%M:%S%:z)\""; }
"#;

/// The recorder played: it writes its session, then records until a termination signal, which
/// stops it as the line `stop` would — its `stopped` written —; `then` runs before the session is
/// written, and may end it otherwise. Returned once it says it is ready; the caller waits for it.
fn recorder(sessions: &Path, then: &str) -> Child {
    let script = format!(
        "{SESSION}\ntrap '' PIPE\ntrap 'stopped; exit 0' TERM\n{then}\nwrite ''\ntouch \"$1/ready\"\nwhile :; do sleep 0.1 & wait $!; done\n"
    );
    let mut child = Command::new("sh")
        .args(["-c", &script, "sh"])
        .arg(sessions)
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    let ready = (0..100).any(|_| {
        std::thread::sleep(Duration::from_millis(50));
        sessions.join("ready").is_file()
    });
    if !ready {
        let _ = child.kill();
        let _ = child.wait();
        panic!("the recorder played did not start");
    }
    child
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

fn session(sessions: &Path) -> std::path::PathBuf {
    sessions.join("2026-11-13_09-00")
}

/// Kept as the shell keeps it once the recording said `started`, then found as a Kitokey started
/// again finds it.
fn kept_and_found(sessions: &Path, child: &Child) -> (std::path::PathBuf, Option<Found>) {
    let file = sessions.join("recording.json");
    let kept = Kept::of(child.id(), &session(sessions)).expect("a program running here has a start");
    recording::keep(&file, &kept).unwrap();
    assert!(!sessions.join("recording.part").exists(), "kept whole, nothing left beside");
    (file.clone(), recording::find(&file, now()))
}

fn end(said: &Receiver<Said>) -> Said {
    said.recv_timeout(Duration::from_secs(10)).expect("its end is said")
}

#[test]
fn a_recording_left_running_is_found_counted_from_its_start_and_stopped_by_a_signal() {
    let sessions = tempfile::tempdir().unwrap();
    let mut child = recorder(sessions.path(), "");
    let (file, found) = kept_and_found(sessions.path(), &child);
    let Found {
        mut recording,
        said,
        folder,
        stopping,
        seconds,
    } = found.expect("found running");
    assert_eq!(folder, session(sessions.path()));
    assert!(!stopping);
    assert!((89.0..120.0).contains(&seconds), "counted from the session's start: {seconds}");
    assert_eq!(recording.id(), child.id());
    assert!(!recording.ended());
    // Nothing is said before it ends: its output went with the shell that read it.
    assert!(said.recv_timeout(Duration::from_millis(500)).is_err());
    recording.stop().unwrap();
    // Ended and not yet waited for — as a program whose parent is gone may stay a moment — is ended.
    match end(&said) {
        Said::Stopped(stopped, length) => {
            assert_eq!(stopped, folder);
            assert!((89.0..120.0).contains(&length), "its length, from start to stop: {length}");
        }
        other => panic!("said {other:?}"),
    }
    assert!(recording.ended());
    assert!(child.wait().unwrap().success(), "stopped as the contract says, not killed");
    // The file is the shell's to forget, once the recording's end is told.
    assert!(file.is_file());
    recording::forget(&file).unwrap();
    recording::forget(&file).unwrap();
    assert!(!file.exists());
}

#[test]
fn a_recording_found_that_ends_without_stopping_is_said_to_have_failed() {
    let sessions = tempfile::tempdir().unwrap();
    let mut child = recorder(sessions.path(), "");
    let (_, found) = kept_and_found(sessions.path(), &child);
    let found = found.expect("found running");
    child.kill().unwrap();
    assert_eq!(end(&found.said), Said::Failed(String::new()));
    let _ = child.wait();
}

#[test]
fn a_recording_found_already_stopping_is_said_so_and_its_end_followed() {
    let sessions = tempfile::tempdir().unwrap();
    // Told to stop by the shell that was ended: `stopped` written, still keeping its tracks.
    let mut child = recorder(sessions.path(), "stopped; touch \"$1/ready\"; sleep 1; exit 0");
    let (_, found) = kept_and_found(sessions.path(), &child);
    let found = found.expect("found running");
    assert!(found.stopping);
    assert!(matches!(end(&found.said), Said::Stopped(..)));
    let _ = child.wait();
}

#[test]
fn nothing_is_found_of_a_program_gone_and_the_file_is_removed() {
    let sessions = tempfile::tempdir().unwrap();
    let mut child = recorder(sessions.path(), "");
    let kept = Kept::of(child.id(), &session(sessions.path())).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    let file = sessions.path().join("recording.json");
    recording::keep(&file, &kept).unwrap();
    assert!(recording::find(&file, now()).is_none());
    assert!(!file.exists(), "nothing is left to find");
}

#[test]
fn another_program_under_the_number_kept_is_not_found_nor_signalled() {
    let sessions = tempfile::tempdir().unwrap();
    let mut child = recorder(sessions.path(), "");
    // The number given again by the system: what was kept is the start of another real program, started later.
    std::thread::sleep(Duration::from_millis(200));
    let mut other = Command::new("sleep").arg("5").spawn().unwrap();
    let mut kept = Kept::of(child.id(), &session(sessions.path())).unwrap();
    let theirs = recording::since(other.id());
    other.kill().unwrap();
    other.wait().unwrap();
    let file = sessions.path().join("recording.json");
    let found = theirs.filter(|theirs| *theirs != kept.since).map(|theirs| {
        kept.since = theirs;
        recording::keep(&file, &kept).unwrap();
        recording::find(&file, now()).is_some()
    });
    std::thread::sleep(Duration::from_millis(300));
    let running = child.try_wait().unwrap().is_none();
    child.kill().unwrap();
    let _ = child.wait();
    assert_eq!(found, Some(false), "another program's start: not found");
    assert!(!file.exists());
    assert!(running, "left running: it is not the one kept");
}

#[test]
fn a_recording_found_whose_program_is_gone_before_the_stop_is_sent_no_signal() {
    let sessions = tempfile::tempdir().unwrap();
    let mut child = recorder(sessions.path(), "");
    let (_, found) = kept_and_found(sessions.path(), &child);
    let mut found = found.expect("found running");
    // Gone between the start and ■: ended, not yet waited for, its number still the system's.
    unsafe_kill(child.id());
    std::thread::sleep(Duration::from_millis(300));
    let refused = found
        .recording
        .stop()
        .expect_err("no signal sent to a number no longer the recording's");
    assert_eq!(refused.kind(), std::io::ErrorKind::NotFound);
    let _ = child.wait();
}

/// Ended outright, as a crash ends it, never waited for here.
fn unsafe_kill(id: u32) {
    Command::new("kill").args(["-KILL", &id.to_string()]).status().unwrap();
}

#[test]
fn a_program_started_later_has_a_later_start() {
    let own = recording::since(std::process::id()).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let mut child = Command::new("sleep").arg("5").spawn().unwrap();
    let later = recording::since(child.id());
    child.kill().unwrap();
    child.wait().unwrap();
    let later = later.unwrap();
    assert!(own > 0 && later > own, "its start in the system's ticks: {own}, then {later}");
}

#[test]
fn a_kept_file_that_cannot_be_read_is_set_aside_never_deleted() {
    use std::os::unix::fs::PermissionsExt;
    let sessions = tempfile::tempdir().unwrap();
    let file = sessions.path().join("recording.json");
    fs::write(&file, "{}").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&file).is_ok() {
        return; // run as a user who reads anything: there is no unreadable file to make
    }
    let at = now();
    assert!(recording::find(&file, at).is_none());
    assert!(!file.exists(), "nothing is left to find");
    let aside = sessions.path().join(format!("recording.json.unreadable-{at}"));
    fs::set_permissions(&aside, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read_to_string(&aside).unwrap(), "{}", "renamed whole, not removed");
}

#[test]
fn a_folder_in_the_kept_file_s_place_is_set_aside_whole_and_keeping_works_again() {
    let sessions = tempfile::tempdir().unwrap();
    let file = sessions.path().join("recording.json");
    fs::create_dir_all(file.join("inside")).unwrap();
    fs::write(file.join("inside/x"), "x").unwrap();
    let at = now();
    assert!(recording::find(&file, at).is_none());
    assert!(!file.exists(), "nothing is left to find");
    let aside = sessions.path().join(format!("recording.json.unreadable-{at}"));
    assert_eq!(
        fs::read_to_string(aside.join("inside/x")).unwrap(),
        "x",
        "what it held, kept intact"
    );
    let kept = Kept::of(std::process::id(), &session(sessions.path())).unwrap();
    recording::keep(&file, &kept).unwrap();
    assert!(file.is_file());
}

#[test]
fn a_file_that_does_not_read_finds_nothing_and_is_removed() {
    let sessions = tempfile::tempdir().unwrap();
    let file = sessions.path().join("recording.json");
    assert!(recording::find(&file, now()).is_none(), "nothing kept: nothing found");
    for text in ["", "not json", r#"{"id":1}"#, r#"{"id":"1","since":2,"folder":"x"}"#] {
        fs::write(&file, text).unwrap();
        assert!(recording::find(&file, now()).is_none(), "{text}");
        assert!(!file.exists(), "{text}");
    }
}

#[test]
fn a_session_that_does_not_read_is_found_from_now_and_its_end_said_failed() {
    let sessions = tempfile::tempdir().unwrap();
    let mut child = recorder(sessions.path(), "");
    fs::write(session(sessions.path()).join("session.json"), "{").unwrap();
    let (_, found) = kept_and_found(sessions.path(), &child);
    let found = found.expect("the program is the one kept: found");
    assert_eq!(found.seconds, 0.0);
    child.kill().unwrap();
    assert_eq!(end(&found.said), Said::Failed(String::new()));
    let _ = child.wait();
}

#[test]
fn a_program_has_a_start_while_it_runs_and_none_once_ended() {
    assert!(recording::since(std::process::id()).is_some());
    let mut child = Command::new("true").spawn().unwrap();
    std::thread::sleep(Duration::from_millis(300));
    // Ended, not yet waited for: no start, it is no program running.
    assert_eq!(recording::since(child.id()), None);
    child.wait().unwrap();
    assert_eq!(recording::since(child.id()), None);
}
