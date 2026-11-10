//! A recording supervised, tried with a program played here: a script that says what
//! `contracts/recording/v1` describes. No sound, no device, nobody recorded.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::{Mutex, mpsc::Receiver},
    time::{Duration, Instant},
};

use kitokey_shell::recording::{self, Check, Checks, Devices, Recording, Said, Source, read};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// A recording by a program played here: a script given the same arguments, `record
/// <sessions>`. One at a time: a script still open for writing in one thread while another
/// starts a program cannot be run — "text file busy".
fn played(folder: &Path, script: &str) -> (Recording, Receiver<Said>) {
    let _held = ONE_AT_A_TIME.lock().unwrap();
    let path = folder.join("kitokey-capture");
    fs::write(&path, format!("#!/bin/sh\nsessions=\"$2\"\n{script}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    Recording::start(&path, folder, None).unwrap()
}

/// Everything a recording says until it closes — a test that waits ten seconds has failed.
fn all(heard: &Receiver<Said>) -> Vec<Said> {
    let mut said = Vec::new();
    while let Ok(event) = heard.recv_timeout(Duration::from_secs(10)) {
        said.push(event);
    }
    said
}

#[test]
fn a_recording_is_started_listened_to_and_stopped_as_its_contract_says() {
    let folder = tempfile::tempdir().unwrap();
    let (mut recording, heard) = played(
        folder.path(),
        r#"[ "$1" = record ] || exit 9
echo '{"event":"started","folder":"'"$sessions"'/2026-10-13_14-05","sources":{"me":"Mic","others":"Speaker"}}'
echo 'a line that is no event'
printf 'neither is one that is not text: caf\351\n'
printf 'nor does one end the listening on the other stream: caf\351\n' >&2
echo '{"event":"of-a-later-version","source":"me"}'
echo '{"event":"silent","source":"others"}'
echo '{"event":"heard","source":"others"}'
echo '{"event":"level","source":"me","level":0.42}'
echo '{"event":"level","source":"others"}'
echo '{"event":"level","source":"others","level":1.5}'
read line
[ "$line" = stop ] && echo '{"event":"stopped","folder":"'"$sessions"'/2026-10-13_14-05","seconds":1912.4,"new":"field"}'
echo '{"event":"silent","source":"me"}'"#,
    );
    let session = folder.path().join("2026-10-13_14-05");
    let before = [
        Said::Started(session.clone(), Some("Mic".into()), Some("Speaker".into())),
        Said::Silent(Source::Others),
        Said::Heard(Source::Others),
        // A level read; one without its level is no event.
        Said::Level(Source::Me, 0.42),
        // The meeting's own, and never louder than the loudest.
        Said::Level(Source::Others, 1.0),
    ];
    for expected in before {
        assert_eq!(heard.recv_timeout(Duration::from_secs(10)).unwrap(), expected);
    }
    // Nothing more until it is told to stop: it is still recording.
    assert!(heard.recv_timeout(Duration::from_millis(300)).is_err() && !recording.ended());
    recording.stop().unwrap();
    assert_eq!(all(&heard), [Said::Stopped(session, 1912.4)]);
}

#[test]
fn a_recording_that_ends_by_itself_is_said_to_have_failed_with_its_own_sentence() {
    let folder = tempfile::tempdir().unwrap();
    // Refused before it started: the contract's "unavailable", then one sentence and a failure.
    let (_recording, heard) = played(
        folder.path(),
        r#"echo '{"event":"unavailable","source":"me","reason":"the system names no default device"}'
head -c 3000000 /dev/zero | tr '\0' x >&2
printf '\nan earlier line, and one that is not text: caf\351\n' >&2
echo 'kitokey-capture: no source could be opened: nothing to record' >&2
exit 1"#,
    );
    assert_eq!(
        all(&heard),
        [
            Said::Unavailable(Source::Me, "the system names no default device".into()),
            Said::Failed("kitokey-capture: no source could be opened: nothing to record".into()),
        ]
    );

    // Killed while it recorded: it says nothing of it, and that is said too.
    let (_recording, heard) = played(
        folder.path(),
        r#"echo '{"event":"started","folder":"'"$sessions"'/s","sources":{"me":"Mic","others":"Speaker"}}'; kill -9 $$"#,
    );
    assert_eq!(
        all(&heard),
        [
            Said::Started(folder.path().join("s"), Some("Mic".into()), Some("Speaker".into())),
            Said::Failed(String::new())
        ]
    );

    // A program that is not there is an error at once, not a recording.
    assert!(Recording::start(&folder.path().join("nowhere"), folder.path(), None).is_err());
}

#[test]
fn a_recording_goes_on_when_the_shell_lets_go_of_it() {
    let folder = tempfile::tempdir().unwrap();
    let (recording, heard) = played(
        folder.path(),
        r#"echo '{"event":"started","folder":"'"$sessions"'/s","sources":{"me":"Mic","others":"Speaker"}}'
sleep 1
touch "$sessions/still-recording""#,
    );
    assert_eq!(
        heard.recv_timeout(Duration::from_secs(10)).unwrap(),
        Said::Started(folder.path().join("s"), Some("Mic".into()), Some("Speaker".into()))
    );
    drop(recording);
    std::thread::sleep(Duration::from_millis(1800));
    assert!(
        folder.path().join("still-recording").exists(),
        "the program was stopped with the shell's hold on it"
    );
}

#[test]
fn a_line_is_read_as_the_contract_describes_it_or_passed_over() {
    assert_eq!(read(r#"{"event":"silent","source":"me"}"#), Some(Said::Silent(Source::Me)));
    // Exact zeros are no quiet room (#361): a silence that says nothing came is told apart.
    assert_eq!(read(r#"{"event":"silent","source":"me","nothing":true}"#), Some(Said::Nothing));
    assert_eq!(
        read(r#"{"event":"silent","source":"me","nothing":false}"#),
        Some(Said::Silent(Source::Me))
    );
    // An output plays exact zeros whenever nothing plays: the meeting's side stays a silence.
    assert_eq!(
        read(r#"{"event":"silent","source":"others","nothing":true}"#),
        Some(Said::Silent(Source::Others))
    );
    // The microphone, as the system names it; a default named for itself, or none, says nothing.
    let long = "Mic ".repeat(30);
    let cut = format!("{}…", &long[..80]);
    for (me, named) in [
        ("Headset Mic", Some("Headset Mic")),
        // A tab and a line break, as JSON escapes them.
        (r"  Headset\t\nMic ", Some("Headset Mic")),
        (long.as_str(), Some(cut.as_str())),
        ("default_input", None),
        ("unavailable", None),
        ("unnamed", None),
        ("", None),
        ("   ", None),
    ] {
        let line = format!(r#"{{"event":"started","folder":"/s","sources":{{"me":"{me}","others":"Speaker"}}}}"#);
        assert_eq!(
            read(&line),
            Some(Said::Started("/s".into(), named.map(str::to_string), Some("Speaker".into()))),
            "{me}"
        );
    }
    // Another device, named as `started` names one; none left (D31).
    assert_eq!(
        read(r#"{"event":"device","source":"me","device":"  Headset\tMic"}"#),
        Some(Said::Moved(Source::Me, Some("Headset Mic".into())))
    );
    assert_eq!(
        read(r#"{"event":"device","source":"others","device":"unnamed"}"#),
        Some(Said::Moved(Source::Others, None))
    );
    assert_eq!(
        read(r#"{"event":"device","source":"others","device":null}"#),
        Some(Said::Lost(Source::Others))
    );
    assert_eq!(
        read(r#"{"event":"stopped","folder":"/s","seconds":3}"#),
        Some(Said::Stopped("/s".into(), 3.0))
    );
    for passed_over in [
        "",
        "stopped",
        "[]",
        r#"{"event":"silent"}"#,
        r#"{"event":"silent","source":"someone"}"#,
        r#"{"event":"started"}"#,
        r#"{"event":7}"#,
        r#"{"event":"device","source":"me"}"#,
        r#"{"event":"device","source":"me","device":7}"#,
        r#"{"event":"device","device":"Mic"}"#,
        r#"{"source":"me"}"#,
    ] {
        assert_eq!(read(passed_over), None, "{passed_over}");
    }
}

#[test]
fn what_each_side_is_recorded_from_is_followed() {
    // The output too, as `started` names it (D33).
    assert_eq!(
        read(r#"{"event":"started","folder":"/s","sources":{"me":"Mic","others":"  Speaker\tout"}}"#),
        Some(Said::Started("/s".into(), Some("Mic".into()), Some("Speaker out".into())))
    );
    let mut devices = Devices::default();
    assert_eq!(devices.names(), [None, None]);
    devices.heard(&Said::Started("/s".into(), Some("Mic".into()), Some("Speaker".into())));
    assert_eq!(devices.names(), [Some("Mic".into()), Some("Speaker".into())]);
    devices.heard(&Said::Moved(Source::Me, Some("AirPods Pro".into())));
    devices.heard(&Said::Level(Source::Me, 0.5));
    devices.heard(&Said::Silent(Source::Others));
    assert_eq!(devices.names(), [Some("AirPods Pro".into()), Some("Speaker".into())]);
    devices.heard(&Said::Lost(Source::Others));
    assert_eq!(devices.names(), [Some("AirPods Pro".into()), None]);
    devices.heard(&Said::Moved(Source::Others, None));
    devices.heard(&Said::Unavailable(Source::Me, "no microphone".into()));
    assert_eq!(devices.names(), [None, None]);
}

/// A check after what a recording said, side by side (D49).
fn checked(said: &[Said]) -> [Check; 2] {
    let mut checks = Checks::default();
    for event in said {
        checks.heard(event);
    }
    checks.now()
}

#[test]
fn a_check_turns_when_a_side_rises_above_silence() {
    use Source::{Me, Others};
    // Nothing yet, or levels that stay at silence: still listening.
    assert_eq!(checked(&[]), [Check::Listening, Check::Listening]);
    assert_eq!(
        checked(&[Said::Level(Me, 0.0), Said::Level(Others, 0.005)]),
        [Check::Listening, Check::Listening]
    );
    // Each side its own.
    assert_eq!(checked(&[Said::Level(Others, 0.4)]), [Check::Listening, Check::Heard]);
    assert_eq!(
        checked(&[Said::Level(Me, 0.4), Said::Level(Me, 0.0)]),
        [Check::Heard, Check::Listening]
    );
}

#[test]
fn a_side_said_silent_is_silent_until_capture_says_it_is_heard() {
    use Source::{Me, Others};
    assert_eq!(checked(&[Said::Silent(Me), Said::Silent(Others)]), [Check::Silent, Check::Silent]);
    assert_eq!(checked(&[Said::Nothing]), [Check::Nothing, Check::Listening]);
    // A level does not overrule what capture said: its own measure of a silence holds.
    assert_eq!(
        checked(&[Said::Silent(Me), Said::Level(Me, 0.5)]),
        [Check::Silent, Check::Listening]
    );
    assert_eq!(checked(&[Said::Nothing, Said::Level(Me, 0.5)]), [Check::Nothing, Check::Listening]);
    assert_eq!(checked(&[Said::Silent(Me), Said::Heard(Me)]), [Check::Heard, Check::Listening]);
    assert_eq!(checked(&[Said::Nothing, Said::Heard(Me)]), [Check::Heard, Check::Listening]);
    // Heard a while ago, silent since: the sentence comes back.
    assert_eq!(
        checked(&[Said::Level(Others, 0.2), Said::Silent(Others)]),
        [Check::Listening, Check::Silent]
    );
}

#[test]
fn a_side_with_no_device_waits_and_is_checked_again_when_one_comes() {
    use Source::{Me, Others};
    let waiting = Said::Unavailable(Me, "none".into());
    assert_eq!(checked(std::slice::from_ref(&waiting)), [Check::Waiting, Check::Listening]);
    // What the recording keeps in its place is no news: no check, no silence said.
    assert_eq!(
        checked(&[
            waiting.clone(),
            Said::Silent(Me),
            Said::Nothing,
            Said::Level(Me, 0.5),
            Said::Heard(Me)
        ]),
        [Check::Waiting, Check::Listening]
    );
    assert_eq!(
        checked(&[Said::Level(Others, 0.5), Said::Lost(Others)]),
        [Check::Listening, Check::Waiting]
    );
    // A device come, or changed: nothing is known of it yet.
    assert_eq!(
        checked(&[waiting, Said::Moved(Me, Some("Casque".into()))]),
        [Check::Listening, Check::Listening]
    );
    assert_eq!(
        checked(&[Said::Level(Me, 0.5), Said::Moved(Me, Some("Casque".into()))]),
        [Check::Listening, Check::Listening]
    );
}

#[test]
fn a_check_is_said_to_the_page_by_its_name() {
    assert_eq!(
        serde_json::to_value([Check::Listening, Check::Heard, Check::Silent, Check::Nothing, Check::Waiting]).unwrap(),
        serde_json::json!(["listening", "heard", "silent", "nothing", "waiting"])
    );
}

#[test]
fn a_silence_said_once_stays_said_after_a_device_changes() {
    let silent = Said::Silent(Source::Me);
    let moved = Said::Moved(Source::Me, Some("Autre".into()));
    assert_eq!(checked(&[silent.clone(), moved.clone()])[0], Check::Silent);
    assert_eq!(checked(&[Said::Nothing, moved.clone()])[0], Check::Nothing);
    // Said while it waited: told when a device comes.
    let lost = Said::Lost(Source::Me);
    assert_eq!(checked(&[lost.clone(), silent.clone()])[0], Check::Waiting);
    assert_eq!(checked(&[lost, silent.clone(), moved.clone()])[0], Check::Silent);
    // Heard, the silence is over: a device changed listens again.
    assert_eq!(checked(&[silent, Said::Heard(Source::Me), moved])[0], Check::Listening);
}

#[test]
fn a_call_taking_and_leaving_the_microphone_turns_the_check_each_time() {
    let (lost, taken) = (Said::Lost(Source::Me), Said::Moved(Source::Me, Some("AirPods".into())));
    let waiting = Said::Unavailable(Source::Me, "waits".into());
    assert_eq!(checked(&[waiting.clone(), taken.clone()])[0], Check::Listening);
    assert_eq!(checked(&[waiting.clone(), taken.clone(), lost.clone()])[0], Check::Waiting);
    assert_eq!(checked(&[waiting, taken.clone(), lost, taken])[0], Check::Listening);
}

#[test]
fn a_level_under_a_voice_is_no_sound_yet() {
    assert_eq!(checked(&[Said::Level(Source::Me, 0.2)])[0], Check::Listening);
    assert_eq!(checked(&[Said::Level(Source::Me, 0.4)])[0], Check::Heard);
}

#[test]
fn a_microphone_named_is_given_to_the_program_as_the_contract_says() {
    let folder = tempfile::tempdir().unwrap();
    let held = ONE_AT_A_TIME.lock().unwrap();
    let path = folder.path().join("kitokey-capture");
    fs::write(&path, "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$2/args\"\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let (mut started, said) = Recording::start(&path, folder.path(), Some("-a name with spaces")).unwrap();
    drop(held);
    while said.recv_timeout(Duration::from_secs(5)).is_ok() {}
    assert!(
        started.ended() || {
            std::thread::sleep(Duration::from_millis(200));
            started.ended()
        }
    );
    assert_eq!(
        fs::read_to_string(folder.path().join("args"))
            .unwrap()
            .lines()
            .skip(2)
            .collect::<Vec<_>>(),
        ["--microphone", "-a name with spaces"]
    );
}

#[test]
fn a_microphone_gone_is_read_and_a_microphone_line_is_no_event_of_a_recording() {
    assert_eq!(recording::read(r#"{"event":"microphone_gone"}"#), Some(Said::MicrophoneGone));
    assert_eq!(recording::read(r#"{"event":"microphone_back"}"#), Some(Said::MicrophoneBack));
    assert_eq!(recording::read(r#"{"event":"microphone","name":"a","description":"b"}"#), None);
}

/// What `microphones` answers, by a program played here.
fn listed(script: &str) -> std::io::Result<Vec<recording::Listed>> {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("kitokey-capture");
    let held = ONE_AT_A_TIME.lock().unwrap();
    fs::write(&path, format!("#!/bin/sh\n[ \"$1\" = microphones ] || exit 9\n{script}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    let listed = recording::microphones(&path);
    drop(held);
    listed
}

#[test]
fn the_microphones_are_what_the_program_lists_each_made_fit_to_show() {
    let got = listed(
        r#"echo '{"event":"microphone","name":"alsa","description":"Digital   Microphone"}'
echo '{"event":"an event of a later version"}'
echo '{"event":"device","source":"me","name":"device_name","description":"A device"}'
echo '{"event":"started","name":"started_name","description":"A start"}'
echo 'no event'
echo '{"event":"microphone","name":"","description":"No name"}'
echo '{"event":"microphone","name":"odd","description":"unnamed"}'"#,
    )
    .unwrap();
    assert_eq!(
        got,
        [recording::Listed {
            name: "alsa".into(),
            description: "Digital Microphone".into()
        }]
    );
}

#[test]
fn no_line_is_nothing_to_choose_from_and_a_failure_is_said() {
    assert_eq!(listed("").unwrap(), []);
    assert!(listed("echo cannot >&2; exit 1").is_err());
}

#[test]
fn a_program_that_does_not_answer_the_list_is_given_up_on() {
    let started = std::time::Instant::now();
    assert!(listed("sleep 30").is_err());
    assert!(started.elapsed() < Duration::from_secs(10), "{:?}", started.elapsed());
}

#[test]
fn a_side_is_said_silent_for_as_long_as_capture_has_not_heard_it() {
    let start = Instant::now();
    let mut checks = Checks::default();
    let later = |seconds| start + Duration::from_secs(seconds);
    checks.heard_at(&Said::Level(Source::Others, 0.9), start);
    assert_eq!(checks.silent_for(later(5)), [None, None]);
    // Capture says it after ten seconds of silence (`recording/v1`): the shell counts from there.
    checks.heard_at(&Said::Silent(Source::Others), later(10));
    assert_eq!(checks.silent_for(later(10)), [None, Some(10.0)]);
    assert_eq!(checks.silent_for(later(120)), [None, Some(120.0)]);
    // A device changed is no sound: the silence goes on.
    checks.heard_at(&Said::Moved(Source::Others, Some("Casque".into())), later(60));
    assert_eq!(checks.silent_for(later(120)), [None, Some(120.0)]);
    checks.heard_at(&Said::Heard(Source::Others), later(130));
    assert_eq!(checks.silent_for(later(200)), [None, None]);
    // Exact zeros are a device's state, said at once, not a silence counted.
    checks.heard_at(&Said::Nothing, later(140));
    assert_eq!(checks.silent_for(later(400)), [None, None]);
    // No device: nothing recorded, no silence to count.
    checks.heard_at(&Said::Lost(Source::Others), later(150));
    checks.heard_at(&Said::Silent(Source::Others), later(160));
    assert_eq!(checks.silent_for(later(400)), [None, None]);
    // A device back, the side still silent: said again, counted from the device back.
    checks.heard_at(&Said::Moved(Source::Others, Some("Casque".into())), later(400));
    assert_eq!(checks.silent_for(later(400)), [None, Some(10.0)]);
    assert_eq!(checks.silent_for(later(510)), [None, Some(120.0)]);
}

#[test]
fn a_new_silence_is_counted_from_its_own_start() {
    let start = Instant::now();
    let mut checks = Checks::default();
    let later = |seconds| start + Duration::from_secs(seconds);
    checks.heard_at(&Said::Silent(Source::Me), later(10));
    checks.heard_at(&Said::Heard(Source::Me), later(100));
    assert_eq!(checks.silent_for(later(110)), [None, None]);
    // Silent again: two minutes from this silence, not from the one before.
    checks.heard_at(&Said::Silent(Source::Me), later(300));
    assert_eq!(checks.silent_for(later(310)), [Some(20.0), None]);
}
