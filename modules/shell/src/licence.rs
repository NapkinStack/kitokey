//! Kitokey's licence, as the shell sees it, and the trial's gate before a recording (PDR-0005,
//! D83). The gate is the same on every system; only reading the Store's licence is Windows' own.

use std::path::Path;

use crate::trial;

/// What the licence is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Licence {
    Bought,
    Trial,
    /// The Store answers a licence that is not active.
    Inactive,
    /// Not installed from the Store: the zip, a package signed for a run, Linux, macOS.
    NotFromStore,
}

/// The licence now, read anew at each call: one bought while Kitokey is open is bought at the next
/// recording. An error is a licence that could not be read.
///
/// With `KITOKEY_LICENCE` naming a file, in a build for trying Kitokey only, the file's word
/// stands in for it — `bought`, `trial`, `inactive` or `none`, anything else not read —: the
/// window's end-to-end scenario holds the gate on Linux. A build made for release has no such
/// reading.
pub fn read() -> Result<Licence, String> {
    #[cfg(debug_assertions)]
    if let Some(file) = std::env::var_os("KITOKEY_LICENCE") {
        return stand_in(Path::new(&file));
    }
    // The Store's licence is read on Windows by D83's last pull request; until then, none is.
    Ok(Licence::NotFromStore)
}

#[cfg(debug_assertions)]
fn stand_in(file: &Path) -> Result<Licence, String> {
    match std::fs::read_to_string(file).map_err(|error| error.to_string())?.trim() {
        "bought" => Ok(Licence::Bought),
        "trial" => Ok(Licence::Trial),
        "inactive" => Ok(Licence::Inactive),
        "none" => Ok(Licence::NotFromStore),
        other => Err(format!("the stand-in licence says {other:?}")),
    }
}

/// Whether a recording starting now is refused: on a trial licence whose trial is over, and on
/// nothing else. A licence not read, not active or not from the Store records, as before the trial
/// existed: a recording is never refused on a doubt.
pub fn refused(licence: &Result<Licence, String>, meetings: &Path, now: i64) -> bool {
    matches!(licence, Ok(Licence::Trial)) && trial::over(meetings, now)
}
