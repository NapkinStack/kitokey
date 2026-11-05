//! The trial's rule (PDR-0005): over only once 14 days have passed since the day Kitokey first
//! started on a trial licence and 10 meetings made here since are listed — whichever comes last.
//! Both are kept in the folder of meetings, which outlives the package:
//!
//! - `.kitokey-trial`, the day, written once and never again: seconds since 1970, one line;
//! - `.kitokey-trial-meetings`, the name of each meeting's folder this Kitokey started, one a line,
//!   added to and never rewritten. A meeting copied in from elsewhere was not started here, so its
//!   name is not there, and it counts for nothing.
//!
//! On a doubt — a file or the folder that cannot be read, a clock earlier than the day — the
//! trial is not over: a recording is never refused on a doubt.

use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    path::Path,
};

/// The file of the folder of meetings that keeps the trial's first day.
pub const DAY_FILE: &str = ".kitokey-trial";
/// The file of the folder of meetings that keeps the names of the meetings made here.
pub const MADE_FILE: &str = ".kitokey-trial-meetings";
/// The days the trial lasts at least, and the meetings it holds at least.
pub const DAYS: i64 = 14;
pub const MEETINGS: usize = 10;

/// Keeps `now` as the trial's first day, unless a day is kept already: the day kept is given
/// back. The folder of meetings is made when it is not there. A day's file that does not read is
/// an error, and is left as it is.
///
/// The day appears whole or not at all: its line is written to a file of its own beside it, then
/// linked under the day's name — which fails when a day is there —, then that file goes. A start
/// cut short leaves no day, never an empty one, and the next start keeps its own.
pub fn begin(meetings: &Path, now: i64) -> io::Result<i64> {
    fs::create_dir_all(meetings)?;
    let file = meetings.join(DAY_FILE);
    if fs::symlink_metadata(&file).is_err() {
        let (written, mut beside) = beside(meetings)?;
        let whole = beside.write_all(format!("{now}\n").as_bytes()).and_then(|()| beside.sync_all());
        // Closed before it is linked or removed: Windows removes no file still open.
        drop(beside);
        let linked = whole.and_then(|()| fs::hard_link(&written, &file));
        let _ = fs::remove_file(&written);
        match linked {
            Ok(()) => return Ok(now),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    day(meetings).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "the trial's day does not read"))
}

/// A file of this process's own beside the day, made new: one an earlier start left is never taken.
fn beside(meetings: &Path) -> io::Result<(std::path::PathBuf, fs::File)> {
    for count in 0..1000 {
        let path = meetings.join(format!("{DAY_FILE}-{}-{count}.tmp", std::process::id()));
        match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("no name free beside the trial's day"))
}

/// Keeps the name of a meeting's folder this Kitokey started. A name that is not one folder's,
/// in the folder of meetings, is refused.
pub fn made(meetings: &Path, name: &str) -> io::Result<()> {
    let mut parts = Path::new(name).components();
    let one = matches!((parts.next(), parts.next()), (Some(std::path::Component::Normal(_)), None));
    if !one || name.contains(['\n', '\r', '/', '\\']) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no folder of the meetings"));
    }
    let mut file = fs::OpenOptions::new().append(true).create(true).open(meetings.join(MADE_FILE))?;
    file.write_all(format!("{name}\n").as_bytes())
}

/// What is left of the trial: its days — a day begun is a day left — and its meetings. It is over
/// once neither is left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Left {
    pub days: u32,
    pub meetings: u32,
}

impl Left {
    /// Nothing left: neither a day nor a meeting.
    pub fn over(self) -> bool {
        self.days == 0 && self.meetings == 0
    }
}

/// What is left of the trial at `now`, in seconds since 1970. Before its first day — none kept yet
/// —, the 14 days whole and the 10 meetings. None on a doubt — a day, a list of the meetings made
/// here or a folder of meetings that cannot be read —: a count that may be wrong is never said.
pub fn left(meetings: &Path, now: i64) -> Option<Left> {
    const DAY: i64 = 86_400;
    if fs::symlink_metadata(meetings.join(DAY_FILE)).is_err_and(|error| error.kind() == io::ErrorKind::NotFound) {
        return Some(Left {
            days: DAYS as u32,
            meetings: MEETINGS as u32,
        });
    }
    let first = day(meetings)?;
    // A clock earlier than the day is the day itself.
    let past = now.saturating_sub(first).clamp(0, DAYS * DAY);
    let days = (DAYS * DAY - past + DAY - 1) / DAY;
    let made: HashSet<String> = match fs::read_to_string(meetings.join(MADE_FILE)) {
        Ok(made) => made.lines().map(str::to_owned).collect(),
        // Not there: no meeting was made here.
        Err(error) if error.kind() == io::ErrorKind::NotFound => HashSet::new(),
        Err(_) => return None,
    };
    // A folder that does not list lists no meeting: none would be counted, wrongly.
    fs::read_dir(meetings).ok()?;
    let counted = crate::sessions::list(meetings, None)
        .iter()
        .filter(|meeting| meeting.instant.is_some_and(|started| started >= first))
        .filter(|meeting| {
            meeting
                .folder
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| made.contains(name))
        })
        .count();
    Some(Left {
        days: days as u32,
        meetings: MEETINGS.saturating_sub(counted) as u32,
    })
}

/// Whether the trial is over at `now`, in seconds since 1970: nothing is left of it.
pub fn over(meetings: &Path, now: i64) -> bool {
    left(meetings, now).is_some_and(Left::over)
}

/// The trial's first day, as kept; none when it is not there or does not read.
fn day(meetings: &Path) -> Option<i64> {
    fs::read_to_string(meetings.join(DAY_FILE)).ok()?.strip_suffix('\n')?.parse().ok()
}
