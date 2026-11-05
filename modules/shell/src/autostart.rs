//! Kitokey started with the user's session (D32), as the desktop's convention has it: an
//! entry in the user's autostart folder ([XDG Autostart]). The file is the setting — what the
//! desktop's own tools change of it is what Settings show. Kitokey writes it once by itself,
//! at its first start, and never again: once turned off, it stays off.
//!
//! [XDG Autostart]: https://specifications.freedesktop.org/autostart-spec/latest/
//!
//! On Windows, the session start uses the Windows StartupTask API (uap5:StartupTask in the
//! package manifest) instead of XDG autostart. The Windows StartupTask is managed by Windows
//! and shown in the Task Manager's Startup tab. Kitokey reads and writes its state through
//! Windows.ApplicationModel.StartupTask.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::settings;

/// The entry's name: the package's own desktop entry's.
pub const NAME: &str = "com.napkinstack.Kitokey.desktop";

/// Wholeheard's entry, before D86: its program gone with its package, it starts nothing.
pub const NAME_BEFORE: &str = "dev.wholeheard.Wholeheard.desktop";

/// The tray alone, no window (D17); `TryExec`: a removed package starts nothing.
const ENTRY: &str = "[Desktop Entry]\n\
    Type=Application\n\
    Name=Kitokey\n\
    Comment=Offers to record a call as it starts\n\
    Comment[fr]=Propose d’enregistrer un appel dès qu’il commence\n\
    Exec=kitokey\n\
    TryExec=kitokey\n\
    Icon=com.napkinstack.Kitokey\n\
    Terminal=false\n";

/// Where the entry is on this system: the user's configuration's `autostart` folder.
/// On Windows, this returns None since Windows uses StartupTask API instead.
pub fn file() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        None
    } else {
        dirs::config_dir().map(|folder| folder.join("autostart").join(NAME))
    }
}

/// Whether the session starts Kitokey: the entry is there, and neither hidden nor turned off
/// by GNOME's key.
pub fn on(entry: &Path) -> bool {
    let Ok(text) = fs::read_to_string(entry) else {
        return false;
    };
    let mut group = "";
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            group = line;
            continue;
        }
        if group != "[Desktop Entry]" {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match (key.trim(), value.trim()) {
            ("Hidden", "true") | ("X-GNOME-Autostart-enabled", "false") => return false,
            _ => {}
        }
    }
    true
}

/// Turns it on — the entry written whole — or off — the entry taken away.
pub fn set(entry: &Path, on: bool) -> io::Result<()> {
    if !on {
        return match fs::remove_file(entry) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            done => done,
        };
    }
    if let Some(folder) = entry.parent() {
        fs::create_dir_all(folder)?;
    }
    let draft = entry.with_extension("desktop.part");
    fs::write(&draft, ENTRY).and_then(|()| fs::rename(&draft, entry)).inspect_err(|_| {
        let _ = fs::remove_file(&draft);
    })
}

/// At a start, before `first`: Wholeheard's entry, when there is one, becomes Kitokey's — on if it
/// was on, off if it was turned off — and is taken away. True when there was one.
pub fn carry_over(before: &Path, entry: &Path) -> io::Result<bool> {
    if !before.exists() {
        return Ok(false);
    }
    if on(before) {
        set(entry, true)?;
    }
    set(before, false)?;
    Ok(true)
}

/// At a start: the entry written when the settings say it never was, and that it was, kept in
/// them. True when it was written now. A settings file that cannot be read writes nothing.
pub fn first(entry: &Path, settings_file: &Path) -> io::Result<bool> {
    let mut settings = settings::read(settings_file).map_err(|why| io::Error::new(io::ErrorKind::InvalidData, why))?;
    if settings.autostart.is_some() {
        return Ok(false);
    }
    set(entry, true)?;
    settings.autostart = Some(true);
    settings::write(settings_file, &settings)?;
    Ok(true)
}

/// What the start did of the entry: Wholeheard's carried over, then Kitokey's written once.
#[derive(Debug)]
pub struct AtStart {
    /// `carry_over`'s answer: true when Wholeheard's entry was there.
    pub carried: io::Result<bool>,
    /// `first`'s answer: true when Kitokey's was written now.
    pub written: io::Result<bool>,
}

/// At a start, in this order: Wholeheard's entry carried over (D86), then Kitokey's written when
/// the settings say it never was (D32). The carrying over comes first: the settings carried from
/// Wholeheard say its entry was written once, so `first` alone would leave the session starting
/// nothing.
pub fn at_start(entry: &Path, settings_file: &Path) -> AtStart {
    let carried = carry_over(&entry.with_file_name(NAME_BEFORE), entry);
    let written = first(entry, settings_file);
    AtStart { carried, written }
}

/// Windows StartupTask state, matching Windows.ApplicationModel.StartupTaskState.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsStartupState {
    /// The startup task is enabled.
    Enabled,
    /// The startup task is disabled (default state).
    Disabled,
    /// The startup task was disabled by the user in Task Manager.
    DisabledByUser,
    /// The startup task was disabled by system policy.
    DisabledByPolicy,
}

/// Reads the Windows StartupTask state for the current package.
/// Returns None if not on Windows or if the API is unavailable.
#[cfg(target_os = "windows")]
pub fn windows_startup_state() -> io::Result<Option<WindowsStartupState>> {
    use windows::ApplicationModel::StartupTask;
    let task = StartupTask::GetForCurrentPackageAsync()
        .map_err(|e| io::Error::other(e))?
        .join()
        .map_err(|e| io::Error::other(e))?;
    let state = task.State().map_err(|e| io::Error::other(e))?;
    Ok(Some(match state {
        windows::ApplicationModel::StartupTaskState::Enabled => WindowsStartupState::Enabled,
        windows::ApplicationModel::StartupTaskState::Disabled => WindowsStartupState::Disabled,
        windows::ApplicationModel::StartupTaskState::DisabledByUser => WindowsStartupState::DisabledByUser,
        windows::ApplicationModel::StartupTaskState::DisabledByPolicy => WindowsStartupState::DisabledByPolicy,
        _ => return Ok(None),
    }))
}

/// On non-Windows, always returns Ok(None).
#[cfg(not(target_os = "windows"))]
pub fn windows_startup_state() -> io::Result<Option<WindowsStartupState>> {
    Ok(None)
}

/// Enables the Windows StartupTask for the current package.
/// Returns the new state after the enable request.
/// On non-Windows, returns Ok(None).
#[cfg(target_os = "windows")]
pub fn windows_startup_enable() -> io::Result<Option<WindowsStartupState>> {
    use windows::ApplicationModel::StartupTask;
    let task = StartupTask::GetForCurrentPackageAsync()
        .map_err(|e| io::Error::other(e))?
        .join()
        .map_err(|e| io::Error::other(e))?;
    let new_state = task
        .RequestEnableAsync()
        .map_err(|e| io::Error::other(e))?
        .join()
        .map_err(|e| io::Error::other(e))?;
    Ok(Some(match new_state {
        windows::ApplicationModel::StartupTaskState::Enabled => WindowsStartupState::Enabled,
        windows::ApplicationModel::StartupTaskState::Disabled => WindowsStartupState::Disabled,
        windows::ApplicationModel::StartupTaskState::DisabledByUser => WindowsStartupState::DisabledByUser,
        windows::ApplicationModel::StartupTaskState::DisabledByPolicy => WindowsStartupState::DisabledByPolicy,
        _ => return Ok(None),
    }))
}

/// On non-Windows, always returns Ok(None).
#[cfg(not(target_os = "windows"))]
pub fn windows_startup_enable() -> io::Result<Option<WindowsStartupState>> {
    Ok(None)
}

/// Disables the Windows StartupTask for the current package.
/// On non-Windows, returns Ok(()).
#[cfg(target_os = "windows")]
pub fn windows_startup_disable() -> io::Result<()> {
    use windows::ApplicationModel::StartupTask;
    let task = StartupTask::GetForCurrentPackageAsync()
        .map_err(|e| io::Error::other(e))?
        .join()
        .map_err(|e| io::Error::other(e))?;
    task.Disable().map_err(|e| io::Error::other(e))
}

/// On non-Windows, always returns Ok(()).
#[cfg(not(target_os = "windows"))]
pub fn windows_startup_disable() -> io::Result<()> {
    Ok(())
}
