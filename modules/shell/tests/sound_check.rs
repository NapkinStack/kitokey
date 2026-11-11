//! « Vérifier le son » (D88), tried with a program played here: a script that says what
//! `contracts/recording/v1`'s `listen` says. No sound, no device, nobody heard.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};

use kitokey_shell::sound_check::{Heard, SoundCheck};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Held across every script written and every program started here: a program started while
/// another thread's script is open for writing inherits that descriptor until it execs, and the
/// script then cannot be run — "text file busy" (ETXTBSY, the fork/exec race). A test that fails
/// does not poison it for the others.
fn spawning() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The program played here, given what `listen` is given. One at a time: a script still open for
/// writing in one thread while another starts a program cannot be run — "text file busy".
fn played(folder: &Path, script: &str, microphone: Option<&str>) -> SoundCheck {
    let _held = spawning();
    let path = folder.join("kitokey-capture");
    fs::write(&path, format!("#!/bin/bash\nhere=$(dirname \"$0\")\n{script}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    SoundCheck::start(&path, microphone).unwrap()
}

/// What the check has heard once `until` holds, or after three seconds.
fn once(check: &SoundCheck, until: impl Fn(&Heard) -> bool) -> Heard {
    let given = Instant::now();
    loop {
        let heard = check.now();
        if until(&heard) || given.elapsed() > Duration::from_secs(3) {
            return heard;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_check_listens_and_says_each_device_its_level_and_a_headset_then_stops_as_asked() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"printf '%s\n' "$*" > "$here/args"
echo '{"event":"device","source":"me","device":"AirPods Pro","headset":true}'
echo '{"event":"device","source":"others","device":"AirPods Pro"}'
echo '{"event":"level","source":"me","level":0.61}'
echo '{"event":"level","source":"others","level":0.38}'
read line
echo "$line" > "$here/told""#,
        Some("bluez_input.unknown#3f6a"),
    );
    let heard = once(&check, |heard| heard.levels[1] > 0.0);
    assert_eq!(heard.devices, [Some("AirPods Pro".into()), Some("AirPods Pro".into())]);
    assert_eq!(heard.levels, [0.61, 0.38]);
    assert!(heard.headset, "the Bluetooth sentence is owed");
    assert_eq!(heard.failed, None);
    let asked = Instant::now();
    check.stop();
    assert!(asked.elapsed() < Duration::from_secs(1), "{:?}", asked.elapsed());
    assert_eq!(
        fs::read_to_string(folder.path().join("args")).unwrap(),
        "listen --microphone bluez_input.unknown#3f6a\n"
    );
    assert_eq!(fs::read_to_string(folder.path().join("told")).unwrap(), "stop\n");
}

#[test]
fn no_headset_is_said_unless_capture_says_one() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"[ "$*" = listen ] || exit 9
echo '{"event":"device","source":"me","device":"Digital Microphone","headset":false}'
echo '{"event":"device","source":"others","device":"Speaker","headset":true}'
echo '{"event":"level","source":"me","level":0.2}'
read line"#,
        None,
    );
    let heard = once(&check, |heard| heard.levels[0] > 0.0);
    assert!(!heard.ended && heard.failed.is_none(), "{heard:?}");
    // `others`' is no microphone's: only `me`'s says the panel's sentence.
    assert!(!heard.headset, "{heard:?}");
    check.stop();
}

#[test]
fn a_source_that_could_not_be_opened_is_said_unavailable() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"echo '{"event":"unavailable","source":"me","reason":"the system names no default device"}'
echo '{"event":"device","source":"others","device":"Speaker"}'
read line"#,
        None,
    );
    let heard = once(&check, |heard| heard.devices[1].is_some());
    assert_eq!(heard.unavailable, [true, false]);
    assert_eq!(heard.devices, [None, Some("Speaker".into())]);
    check.stop();
}

#[test]
fn the_test_sound_is_heard_when_the_meeting_s_side_rises_above_silence_after_it_was_asked() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"echo '{"event":"device","source":"others","device":"Speaker"}'
while ! read -t 0.05 line; do
  if [ -e "$here/loud" ]; then level=0.8; else level=0.1; fi
  echo '{"event":"level","source":"others","level":'$level'}'
done"#,
        None,
    );
    let heard = once(&check, |heard| heard.levels[1] > 0.0);
    assert_eq!(heard.test, None, "no test sound asked yet");
    check.mark();
    let heard = once(&check, |heard| heard.test.is_some());
    assert_eq!(heard.test, Some(false), "under silence's threshold: not heard");
    fs::write(folder.path().join("loud"), "").unwrap();
    assert_eq!(once(&check, |heard| heard.test == Some(true)).test, Some(true));
    // Asked again: what was heard before counts for nothing.
    fs::remove_file(folder.path().join("loud")).unwrap();
    std::thread::sleep(Duration::from_millis(150));
    check.mark();
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(check.now().test, Some(false));
    check.stop();
}

#[test]
fn a_check_that_cannot_listen_says_its_program_s_sentence() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"echo "kitokey-capture: no source could be opened: nothing to listen to" >&2
exit 1"#,
        None,
    );
    let heard = once(&check, |heard| heard.ended);
    assert!(heard.ended);
    // Its sentence is the log's, never the page's: the panel says its own.
    assert!(serde_json::to_value(&heard).unwrap().get("failed").is_none());
    assert_eq!(
        heard.failed.as_deref(),
        Some("kitokey-capture: no source could be opened: nothing to listen to")
    );
    check.stop();
}

#[test]
fn a_program_that_does_not_stop_is_ended_within_a_second_and_a_half() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"trap '' TERM INT
echo $$ > "$here/pid"
echo '{"event":"device","source":"others","device":"Speaker"}'
exec 0<&-
while true; do sleep 0.05; done"#,
        None,
    );
    once(&check, |heard| heard.devices[1].is_some());
    let pid = fs::read_to_string(folder.path().join("pid")).unwrap().trim().to_string();
    let asked = Instant::now();
    check.stop();
    assert!(asked.elapsed() < Duration::from_millis(1500), "{:?}", asked.elapsed());
    assert!(!Path::new(&format!("/proc/{pid}")).exists(), "the program is gone");
}

#[test]
fn a_later_microphone_said_without_headset_is_not_known_to_be_one() {
    let folder = tempfile::tempdir().unwrap();
    let check = played(
        folder.path(),
        r#"echo '{"event":"device","source":"me","device":"AirPods Pro","headset":true}'
while [ ! -e "$here/moved" ]; do sleep 0.02; done
echo '{"event":"device","source":"me","device":"Digital Microphone"}'
read line"#,
        None,
    );
    assert!(once(&check, |heard| heard.headset).headset);
    fs::write(folder.path().join("moved"), "").unwrap();
    let heard = once(&check, |heard| heard.devices[0].as_deref() == Some("Digital Microphone"));
    // recording/v1: absent, not known — the Bluetooth sentence is no longer owed.
    assert!(!heard.headset, "{heard:?}");
    check.stop();
}

// ---- One listening at a time, never beside a recording (ADR-0015's amendment), and never left behind.

use kitokey_shell::sound_check::{Refused, Slot, open};

/// A program that listens until told, as `listen` does.
const LISTENS: &str = r#"echo '{"event":"device","source":"others","device":"Speaker"}'
read line"#;

/// The script alone; whoever runs it holds `spawning` across the start.
fn script(folder: &Path) -> std::path::PathBuf {
    let _held = spawning();
    let path = folder.join("kitokey-capture");
    fs::write(&path, format!("#!/bin/bash\n{LISTENS}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn no_listening_opens_while_a_recording_runs() {
    let recording = Mutex::new(Some(()));
    let slot = Mutex::new(Slot::default());
    let refused = open(&recording, &slot, "main", || panic!("nothing is started"));
    assert!(matches!(refused, Err(Refused::Recording)));
    assert!(!slot.lock().unwrap().is_open());
}

#[test]
fn a_recording_that_starts_while_a_listening_opens_finds_it_to_stop() {
    let folder = tempfile::tempdir().unwrap();
    let program = script(folder.path());
    let recording: Mutex<Option<()>> = Mutex::new(None);
    let slot = Mutex::new(Slot::default());
    std::thread::scope(|scope| {
        let opening = scope.spawn(|| {
            open(&recording, &slot, "main", || {
                // Slow to start, as a program is: a recording asked meanwhile waits for it.
                std::thread::sleep(Duration::from_millis(300));
                let _held = spawning();
                SoundCheck::start(&program, None)
            })
        });
        std::thread::sleep(Duration::from_millis(50));
        // A recording's start, as `start_recording` does it: the recording's lock, then the listening's.
        let started = recording.lock().unwrap();
        let found = slot.lock().unwrap().take();
        drop(started);
        assert!(found.is_some(), "the listening was stored before the recording could start");
        found.unwrap().stop();
        assert!(matches!(opening.join().unwrap(), Ok(None)));
    });
}

#[test]
fn a_listening_unasked_too_long_is_given_up_and_one_asked_is_kept() {
    let folder = tempfile::tempdir().unwrap();
    let program = script(folder.path());
    let (recording, slot) = (Mutex::new(None::<()>), Mutex::new(Slot::default()));
    open(&recording, &slot, "main", || {
        let _held = spawning();
        SoundCheck::start(&program, None)
    })
    .unwrap();
    let unasked = Duration::from_millis(200);
    std::thread::sleep(Duration::from_millis(120));
    assert!(slot.lock().unwrap().asked().is_some());
    std::thread::sleep(Duration::from_millis(120));
    assert!(slot.lock().unwrap().abandoned(unasked).is_none(), "asked 120 ms ago");
    std::thread::sleep(Duration::from_millis(150));
    let gone = slot.lock().unwrap().abandoned(unasked).expect("unasked 270 ms: given up");
    gone.stop();
    assert!(!slot.lock().unwrap().is_open());
}

#[test]
fn a_listening_ends_with_the_window_that_opened_it_alone() {
    let folder = tempfile::tempdir().unwrap();
    let program = script(folder.path());
    let (recording, slot) = (Mutex::new(None::<()>), Mutex::new(Slot::default()));
    open(&recording, &slot, "main", || {
        let _held = spawning();
        SoundCheck::start(&program, None)
    })
    .unwrap();
    assert!(
        slot.lock().unwrap().of_window("small").is_none(),
        "another window's end is not its own"
    );
    slot.lock().unwrap().of_window("main").expect("its window gone: it goes").stop();
    assert!(!slot.lock().unwrap().is_open());
}

#[test]
fn a_close_while_a_listening_opens_wins() {
    let folder = tempfile::tempdir().unwrap();
    let program = script(folder.path());
    let recording: Mutex<Option<()>> = Mutex::new(None);
    let slot = Mutex::new(Slot::default());
    std::thread::scope(|scope| {
        let opening = scope.spawn(|| {
            open(&recording, &slot, "main", || {
                std::thread::sleep(Duration::from_millis(300));
                let _held = spawning();
                SoundCheck::start(&program, None)
            })
        });
        std::thread::sleep(Duration::from_millis(50));
        // The panel closed while its listening starts: nothing to take yet.
        assert!(slot.lock().unwrap().take().is_none());
        assert!(matches!(opening.join().unwrap(), Err(Refused::Closed)));
    });
    assert!(!slot.lock().unwrap().is_open(), "the listening started meanwhile is not kept");
}

#[test]
fn a_listening_started_for_a_panel_closed_meanwhile_is_gone_when_the_opening_answers() {
    let folder = tempfile::tempdir().unwrap();
    let program = {
        let _held = spawning();
        let path = folder.path().join("kitokey-capture");
        fs::write(&path, format!("#!/bin/bash\necho $$ > \"$(dirname \"$0\")/pid\"\n{LISTENS}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    };
    let recording: Mutex<Option<()>> = Mutex::new(None);
    let slot = Mutex::new(Slot::default());
    std::thread::scope(|scope| {
        let opening = scope.spawn(|| {
            open(&recording, &slot, "main", || {
                let _held = spawning();
                let started = SoundCheck::start(&program, None);
                // Its program running, the panel closes before it is stored.
                std::thread::sleep(Duration::from_millis(300));
                started
            })
        });
        std::thread::sleep(Duration::from_millis(100));
        assert!(slot.lock().unwrap().take().is_none());
        assert!(matches!(opening.join().unwrap(), Err(Refused::Closed)));
    });
    let pid = fs::read_to_string(folder.path().join("pid")).unwrap().trim().to_string();
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists(),
        "the program started meanwhile is gone"
    );
}
