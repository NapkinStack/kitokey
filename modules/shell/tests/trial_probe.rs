//! TEMPORARY probe for #705, removed before the fix is reviewed: `trial::begin`'s steps, copied,
//! each failure said with its step and the OS's code, with the names beside the day taken as
//! today (the process id and a count from 0) and taken once each in the process.

use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Barrier, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

const DAY_FILE: &str = ".kitokey-trial";
static ONCE: AtomicU64 = AtomicU64::new(0);

fn beside(meetings: &Path, unique: bool) -> Result<(PathBuf, fs::File), (&'static str, io::Error)> {
    for count in 0..1000u64 {
        let count = if unique { ONCE.fetch_add(1, Ordering::Relaxed) } else { count };
        let path = meetings.join(format!("{DAY_FILE}-{}-{count}.tmp", std::process::id()));
        match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(("create_new", error)),
        }
    }
    Err(("no name", io::Error::other("none")))
}

fn begin(meetings: &Path, now: i64, unique: bool) -> Result<(), (&'static str, io::Error)> {
    let file = meetings.join(DAY_FILE);
    if fs::symlink_metadata(&file).is_err() {
        let (written, mut beside) = beside(meetings, unique)?;
        beside
            .write_all(format!("{now}\n").as_bytes())
            .and_then(|()| beside.sync_all())
            .map_err(|e| ("write", e))?;
        drop(beside);
        let linked = fs::hard_link(&written, &file);
        let removed = fs::remove_file(&written);
        match linked {
            Ok(()) => return removed.map_err(|e| ("remove", e)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(("hard_link", error)),
        }
        removed.map_err(|e| ("remove", e))?;
    }
    fs::read_to_string(&file).map(drop).map_err(|e| ("read", e))
}

fn rounds(unique: bool) -> BTreeMap<String, usize> {
    let failures = Mutex::new(BTreeMap::new());
    for round in 0..300 {
        let meetings = tempfile::tempdir().unwrap();
        let together = Barrier::new(16);
        std::thread::scope(|scope| {
            for index in 0..16 {
                let (meetings, together, failures) = (meetings.path(), &together, &failures);
                scope.spawn(move || {
                    together.wait();
                    if let Err((step, error)) = begin(meetings, round * 100 + index, unique) {
                        *failures
                            .lock()
                            .unwrap()
                            .entry(format!("{step}: {:?} {:?}", error.raw_os_error(), error.kind()))
                            .or_insert(0) += 1;
                    }
                });
            }
        });
    }
    failures.into_inner().unwrap()
}

#[test]
fn probe_705() {
    let today = rounds(false);
    let once = rounds(true);
    assert!(
        today.is_empty() && once.is_empty(),
        "PROBE 705 — names as today: {today:?} — names once each: {once:?}"
    );
}
