//! The trial's rule (PDR-0005, D83): over only once 14 days have passed since the day Kitokey
//! first started on a trial licence and 10 meetings made here since are listed; never on a doubt.

use std::{fs, path::Path};

use kitokey_shell::{
    sessions::instant,
    trial::{self, DAY_FILE, MADE_FILE},
};

const DAY: i64 = 86_400;

/// The day the trial began, in these tests: 2026-11-05 at nine, UTC.
fn first() -> i64 {
    instant("2026-11-05T09:00:00Z").unwrap()
}

/// A meeting as capture leaves it, started `after` seconds past the trial's first day.
fn meeting(meetings: &Path, name: &str, after: i64) {
    let started = first() + after;
    let folder = meetings.join(name);
    fs::create_dir_all(&folder).unwrap();
    let json = format!(
        r#"{{"contract":"session-audio/v2","started":"{}","stopped":"{}","tracks":{{}}}}"#,
        iso(started),
        iso(started + 1800)
    );
    fs::write(folder.join("session.json"), json).unwrap();
}

/// A meeting made here: capture's folder, and its name kept by the shell as it starts it.
fn made(meetings: &Path, name: &str, after: i64) {
    meeting(meetings, name, after);
    trial::made(meetings, name).unwrap();
}

/// `count` meetings made here, a day apart from the first day on.
fn made_here(meetings: &Path, count: usize) {
    for index in 0..count {
        made(meetings, &format!("2026-11-{:02}_10-00", 5 + index), index as i64 * DAY + 3600);
    }
}

/// An instant in seconds since 1970, written as the contract writes one, in UTC.
fn iso(seconds: i64) -> String {
    let (days, rest) = (seconds.div_euclid(DAY), seconds.rem_euclid(DAY));
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// A folder of meetings whose trial began on the first day.
fn begun() -> tempfile::TempDir {
    let meetings = tempfile::tempdir().unwrap();
    assert_eq!(trial::begin(meetings.path(), first()).unwrap(), first());
    meetings
}

#[test]
fn the_tests_write_an_instant_the_shell_reads_back() {
    for seconds in [0, first(), first() + 29 * DAY + 3599] {
        assert_eq!(instant(&iso(seconds)), Some(seconds));
    }
}

#[test]
fn it_is_over_once_fourteen_days_have_passed_and_ten_meetings_are_there() {
    let meetings = begun();
    made_here(meetings.path(), 10);
    assert!(!trial::over(meetings.path(), first() + 14 * DAY - 1));
    assert!(trial::over(meetings.path(), first() + 14 * DAY));
    assert!(trial::over(meetings.path(), first() + 400 * DAY));
}

#[test]
fn never_at_nine_meetings_after_thirty_days() {
    let meetings = begun();
    made_here(meetings.path(), 9);
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
    // The tenth makes it over.
    made(meetings.path(), "2026-11-20_10-00", 15 * DAY);
    assert!(trial::over(meetings.path(), first() + 30 * DAY));
}

#[test]
fn never_at_ten_meetings_after_thirteen_days() {
    let meetings = begun();
    made_here(meetings.path(), 13);
    assert!(!trial::over(meetings.path(), first() + 13 * DAY));
    assert!(!trial::over(meetings.path(), first() + 14 * DAY - 1));
}

#[test]
fn meetings_copied_from_elsewhere_count_for_nothing() {
    let meetings = begun();
    made_here(meetings.path(), 9);
    // Copied in after the first day — from Linux, or synced from another computer —, dated after it.
    for index in 0..5 {
        meeting(
            meetings.path(),
            &format!("2026-11-{:02}_15-00", 6 + index),
            (1 + index) * DAY + 6 * 3600,
        );
    }
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
    // Ten copied and none made here: nothing counts.
    let copied = begun();
    for index in 0..10 {
        meeting(copied.path(), &format!("2026-11-{:02}_15-00", 5 + index), index * DAY + 6 * 3600);
    }
    assert!(!trial::over(copied.path(), first() + 30 * DAY));
}

#[test]
fn a_meeting_counts_once_and_only_while_the_window_lists_it() {
    let meetings = begun();
    made_here(meetings.path(), 9);
    // Its name kept twice is one meeting.
    trial::made(meetings.path(), "2026-11-05_10-00").unwrap();
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
    // A name kept whose meeting the window does not list counts for nothing: deleted, or no session.
    trial::made(meetings.path(), "2026-12-01_10-00").unwrap();
    fs::create_dir_all(meetings.path().join("2026-12-02_10-00")).unwrap();
    trial::made(meetings.path(), "2026-12-02_10-00").unwrap();
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
    made(meetings.path(), "2026-12-03_10-00", 20 * DAY);
    assert!(trial::over(meetings.path(), first() + 30 * DAY));
    fs::remove_dir_all(meetings.path().join("2026-12-03_10-00")).unwrap();
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
}

#[test]
fn a_recording_stopped_after_seconds_or_cut_short_counts_as_the_window_lists_it() {
    // Stopped after three seconds: the window lists it, so it counts.
    let meetings = begun();
    made_here(meetings.path(), 9);
    made(meetings.path(), "2026-11-20_10-00", 15 * DAY);
    let stopped_early = format!(
        r#"{{"contract":"session-audio/v2","started":"{}","stopped":"{}","tracks":{{}}}}"#,
        iso(first() + 15 * DAY),
        iso(first() + 15 * DAY + 3)
    );
    fs::write(meetings.path().join("2026-11-20_10-00/session.json"), stopped_early).unwrap();
    assert!(trial::over(meetings.path(), first() + 30 * DAY));
    // Cut short — a crash, no stop written —: the window lists it, interrupted, so it counts.
    let cut = format!(
        r#"{{"contract":"session-audio/v2","started":"{}","tracks":{{}}}}"#,
        iso(first() + 15 * DAY)
    );
    fs::write(meetings.path().join("2026-11-20_10-00/session.json"), cut).unwrap();
    assert!(trial::over(meetings.path(), first() + 30 * DAY));
}

#[test]
fn a_meeting_started_before_the_first_day_counts_for_nothing() {
    let meetings = begun();
    made_here(meetings.path(), 9);
    made(meetings.path(), "2026-11-04_10-00", -DAY);
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
}

#[test]
fn the_day_is_written_once_and_kept() {
    let meetings = tempfile::tempdir().unwrap();
    // Not there yet, the folder of meetings is made to keep it.
    let folder = meetings.path().join("Kitokey");
    assert_eq!(trial::begin(&folder, first()).unwrap(), first());
    let written = fs::read(folder.join(DAY_FILE)).unwrap();
    assert_eq!(trial::begin(&folder, first() + 20 * DAY).unwrap(), first());
    assert_eq!(fs::read(folder.join(DAY_FILE)).unwrap(), written);
    made_here(&folder, 10);
    assert!(trial::over(&folder, first() + 14 * DAY));
}

/// The files of the folder of meetings whose name is the day's own and that are not it.
fn beside_the_day(meetings: &Path) -> Vec<String> {
    let mut found: Vec<String> = fs::read_dir(meetings)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(DAY_FILE) && name != DAY_FILE && name != MADE_FILE)
        .collect();
    found.sort();
    found
}

#[test]
fn a_file_left_beside_the_day_by_an_earlier_start_is_never_the_day() {
    let meetings = tempfile::tempdir().unwrap();
    // What a start cut short may have left, under any name this process could take too.
    let stale = [
        format!("{DAY_FILE}-{}-0.tmp", std::process::id()),
        format!("{DAY_FILE}-{}-1.tmp", std::process::id()),
        format!("{DAY_FILE}-1-0.tmp"),
    ];
    for name in &stale {
        fs::write(meetings.path().join(name), "").unwrap();
    }
    assert_eq!(trial::begin(meetings.path(), first()).unwrap(), first());
    assert_eq!(
        fs::read_to_string(meetings.path().join(DAY_FILE)).unwrap(),
        format!("{}\n", first())
    );
    // Its own is gone; the others are left as they were.
    let mut left = stale.to_vec();
    left.sort();
    assert_eq!(beside_the_day(meetings.path()), left);
    for name in &stale {
        assert_eq!(fs::read_to_string(meetings.path().join(name)).unwrap(), "");
    }
}

/// Run alone, in a process that may write no byte to a file (`a_day_cut_short_is_never_kept`): a
/// day kept is given back, and none is an error.
#[test]
fn a_start_that_cannot_write() {
    let Some(meetings) = std::env::var_os("KITOKEY_TRIAL_CANNOT_WRITE") else {
        return;
    };
    let meetings = Path::new(&meetings);
    let kept = fs::read_to_string(meetings.join(DAY_FILE)).ok();
    match kept {
        Some(day) => assert_eq!(trial::begin(meetings, first() + 5 * DAY).unwrap().to_string(), day.trim_end()),
        None => assert!(trial::begin(meetings, first()).is_err()),
    }
}

/// Runs `a_start_that_cannot_write` on a folder of meetings, in a process that may write no byte
/// to a file (`ulimit -f 0`), the signal that would stop it ignored.
#[cfg(unix)]
fn start_that_cannot_write(meetings: &Path) {
    let run = std::process::Command::new("bash")
        .arg("-c")
        .arg("trap '' XFSZ; ulimit -f 0; exec \"$0\" --exact a_start_that_cannot_write --test-threads 1")
        .arg(std::env::current_exe().unwrap())
        .env("KITOKEY_TRIAL_CANNOT_WRITE", meetings)
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&run.stdout);
    assert!(run.status.success() && said.contains("1 passed"), "{said}");
}

/// A start whose line is cut short — a disk full, a crash — leaves no day: never an empty or
/// partial one, which would read as no day for ever.
#[cfg(unix)]
#[test]
fn a_day_cut_short_is_never_kept() {
    let meetings = tempfile::tempdir().unwrap();
    start_that_cannot_write(meetings.path());
    assert!(!meetings.path().join(DAY_FILE).exists());
    assert!(beside_the_day(meetings.path()).is_empty());
    // The next start keeps its day; a start that cannot write then still reads it.
    assert_eq!(trial::begin(meetings.path(), first() + DAY).unwrap(), first() + DAY);
    start_that_cannot_write(meetings.path());
    assert_eq!(
        fs::read_to_string(meetings.path().join(DAY_FILE)).unwrap(),
        format!("{}\n", first() + DAY)
    );
    assert!(beside_the_day(meetings.path()).is_empty());
}

#[test]
fn starts_at_once_keep_one_day() {
    let meetings = tempfile::tempdir().unwrap();
    let starts = 16;
    let together = std::sync::Barrier::new(starts);
    let kept: Vec<i64> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..starts)
            .map(|index| {
                let (meetings, together) = (meetings.path(), &together);
                scope.spawn(move || {
                    together.wait();
                    trial::begin(meetings, first() + index as i64).unwrap()
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().unwrap()).collect()
    });
    let day = fs::read_to_string(meetings.path().join(DAY_FILE)).unwrap();
    assert!(kept.iter().all(|kept| kept.to_string() == day.trim_end()), "{kept:?} {day:?}");
    assert!(beside_the_day(meetings.path()).is_empty());
}

#[test]
fn without_its_day_the_trial_is_never_over() {
    let meetings = tempfile::tempdir().unwrap();
    made_here(meetings.path(), 10);
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
    // A day that does not read is no day, and is never written again.
    for wrong in ["", "yesterday", "-", "1762333200x"] {
        fs::write(meetings.path().join(DAY_FILE), wrong).unwrap();
        assert!(!trial::over(meetings.path(), first() + 30 * DAY), "{wrong:?}");
        assert!(trial::begin(meetings.path(), first()).is_err(), "{wrong:?}");
        assert_eq!(fs::read_to_string(meetings.path().join(DAY_FILE)).unwrap(), wrong);
    }
}

#[test]
fn a_clock_earlier_than_the_day_is_never_over() {
    let meetings = begun();
    made_here(meetings.path(), 10);
    assert!(!trial::over(meetings.path(), first() - 30 * DAY));
    assert!(!trial::over(meetings.path(), first() - 1));
}

#[test]
fn a_folder_of_meetings_that_is_not_there_is_never_over() {
    let meetings = tempfile::tempdir().unwrap();
    assert!(!trial::over(&meetings.path().join("gone"), first() + 30 * DAY));
}

#[cfg(unix)]
#[test]
fn a_folder_of_meetings_that_cannot_be_read_is_never_over() {
    use std::os::unix::fs::PermissionsExt;
    let meetings = begun();
    made_here(meetings.path(), 10);
    assert!(trial::over(meetings.path(), first() + 30 * DAY));
    let mode = |path: &Path, mode| fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    // Its files open by name, and it does not list: nothing is listed, nothing counts.
    mode(meetings.path(), 0o311);
    let listed = fs::read_dir(meetings.path()).is_ok();
    assert!(!trial::over(meetings.path(), first() + 30 * DAY));
    mode(meetings.path(), 0o755);
    assert!(!listed, "a test run by root reads any folder, and proves nothing here");
    // The day's file, or the meetings made here, that cannot be read.
    for file in [DAY_FILE, MADE_FILE] {
        mode(&meetings.path().join(file), 0o000);
        assert!(!trial::over(meetings.path(), first() + 30 * DAY), "{file}");
        mode(&meetings.path().join(file), 0o644);
    }
    assert!(trial::over(meetings.path(), first() + 30 * DAY));
}

#[test]
fn a_name_that_is_no_folder_of_the_meetings_is_refused() {
    let meetings = begun();
    for wrong in ["", "a/b", "a\\b", "..", "a\nb", "."] {
        assert!(trial::made(meetings.path(), wrong).is_err(), "{wrong:?}");
    }
    assert!(!meetings.path().join(MADE_FILE).exists());
}
