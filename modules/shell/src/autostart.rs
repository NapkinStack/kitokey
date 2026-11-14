//! Kitokey started with the user's session (D32), as the desktop's convention has it: an
//! entry in the user's autostart folder ([XDG Autostart]). The file is the setting — what the
//! desktop's own tools change of it is what Settings show. Kitokey writes it once by itself,
//! at its first start, and never again: once turned off, it stays off.
//!
//! [XDG Autostart]: https://specifications.freedesktop.org/autostart-spec/latest/
//!
//! On Windows, the package declares a startup task Windows turns on at its first start (D84):
//! [`StartupTask`] is read and changed, Windows' Startup settings are the setting, and nothing is
//! written under `AppData\Roaming\autostart`, which Windows never reads. Outside a package — the
//! zip —, there is no task, and Settings offer nothing.
//!
//! [`StartupTask`]: https://learn.microsoft.com/en-us/uwp/api/windows.applicationmodel.startuptask

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

/// Where the entry is on this system: the user's configuration's `autostart` folder; none on
/// Windows, whose session starts the package's task instead.
pub fn file() -> Option<PathBuf> {
    if cfg!(windows) {
        return None;
    }
    dirs::config_dir().map(|folder| folder.join("autostart").join(NAME))
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
    let mut settings = settings::read(settings_file).map_err(|why| io::Error::new(io::ErrorKind::InvalidData, why.detail))?;
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

/// The startup task's `TaskId`, as the package's manifest declares it (`install`, D84).
pub const TASK_ID: &str = "KitokeyStartupTask";

/// Windows' state of the package's startup task, as `StartupTaskState` names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Disabled,
    DisabledByUser,
    Enabled,
    DisabledByPolicy,
    EnabledByPolicy,
}

/// Who holds the switch when Kitokey cannot move it: the user, in Windows' Startup settings — never
/// turned back on by Kitokey —, or a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Held {
    User,
    Policy,
}

/// What Settings show of a task: on or off, and who holds it when Kitokey cannot change it.
pub fn shown(task: Task) -> (bool, Option<Held>) {
    match task {
        Task::Enabled => (true, None),
        Task::Disabled => (false, None),
        Task::DisabledByUser => (false, Some(Held::User)),
        Task::DisabledByPolicy => (false, Some(Held::Policy)),
        Task::EnabledByPolicy => (true, Some(Held::Policy)),
    }
}

/// The package's startup task as Windows holds it now; none outside a package, or off Windows.
pub fn task() -> Result<Option<Task>, String> {
    #[cfg(windows)]
    return windows_task::read();
    #[cfg(not(windows))]
    Ok(None)
}

/// Asks Windows to turn the task on — which it refuses, saying nothing, when the user or a policy
/// turned it off — or turns it off.
pub fn turn(on: bool) -> Result<(), String> {
    #[cfg(windows)]
    return windows_task::turn(on);
    #[cfg(not(windows))]
    {
        let _ = on;
        Err("this system names no folder for the session's start".to_string())
    }
}

/// The startup task, through `Windows.ApplicationModel.StartupTask` (Windows 10, version 1607 and later).
#[cfg(windows)]
mod windows_task {
    use std::{sync::mpsc, thread, time::Duration};

    use windows::{
        ApplicationModel::{Package, StartupTask, StartupTaskState},
        core::{HRESULT, HSTRING},
    };

    use super::{TASK_ID, Task};

    /// `APPMODEL_ERROR_NO_PACKAGE`: the process has no package's identity.
    const NO_PACKAGE: HRESULT = HRESULT::from_win32(15_700);

    /// How long Windows is waited for, on a thread of its own — never the window's.
    const WAIT: Duration = Duration::from_secs(5);

    fn waited<T: Send + 'static>(ask: impl FnOnce() -> windows::core::Result<T> + Send + 'static) -> Result<T, String> {
        let (sent, received) = mpsc::channel();
        thread::spawn(move || {
            let _ = sent.send(ask());
        });
        received
            .recv_timeout(WAIT)
            .map_err(|_| format!("Windows did not answer within {} s", WAIT.as_secs()))?
            .map_err(|error| error.to_string())
    }

    /// The package's task; none when Windows did not start Kitokey from a package.
    fn startup_task() -> windows::core::Result<Option<StartupTask>> {
        match Package::Current() {
            Ok(_) => StartupTask::GetAsync(&HSTRING::from(TASK_ID))?.join().map(Some),
            Err(error) if error.code() == NO_PACKAGE => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn named(state: StartupTaskState) -> Result<Task, String> {
        Ok(match state {
            StartupTaskState::Disabled => Task::Disabled,
            StartupTaskState::DisabledByUser => Task::DisabledByUser,
            StartupTaskState::Enabled => Task::Enabled,
            StartupTaskState::DisabledByPolicy => Task::DisabledByPolicy,
            StartupTaskState::EnabledByPolicy => Task::EnabledByPolicy,
            other => {
                return Err(format!(
                    "Windows names a state of the startup task Kitokey does not know: {}",
                    other.0
                ));
            }
        })
    }

    pub fn read() -> Result<Option<Task>, String> {
        waited(|| startup_task()?.map(|task| task.State()).transpose())?
            .map(named)
            .transpose()
    }

    pub fn turn(on: bool) -> Result<(), String> {
        let turned = waited(move || {
            let Some(task) = startup_task()? else {
                return Ok(false);
            };
            if on {
                task.RequestEnableAsync()?.join()?;
            } else {
                task.Disable()?;
            }
            Ok(true)
        })?;
        turned
            .then_some(())
            .ok_or_else(|| "Kitokey was not started from its package".to_string())
    }
}
