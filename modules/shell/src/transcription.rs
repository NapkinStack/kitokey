//! A transcript, made: the program that hears a session is started on its folder and waited
//! for. The shell reads none of what it hears — the program writes `transcript.jsonl`
//! (`contracts/transcript/v1`) in the session's folder, and says in one sentence what failed.

use std::{
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
    time::Duration,
};
use tracing::{error, info};

/// How a transcript ended.
#[derive(Debug, Clone, PartialEq)]
pub enum Made {
    /// Written whole, in this session's folder.
    Written(PathBuf),
    /// Not written, or not whole: the program's own sentence, when it gave one. Run again,
    /// the program goes on from where it was.
    Failed(String),
}

/// Who hears it: a Whisper model file, or Apple's SpeechAnalyzer in a language (D42), as
/// `kitokey-transcription` takes them.
#[derive(Debug, Clone, PartialEq)]
pub enum Hearer {
    Model(PathBuf),
    SpeechAnalyzer {
        language: String,
    },
    /// A service the user named (D43): its address, its model — the language is the meeting's,
    /// told when it says one (D65).
    Service {
        provider: String,
        model: String,
    },
}

/// Why a service named to hear is not saved (D43).
#[derive(Debug, Clone, PartialEq)]
pub enum Refused {
    /// No `http://` or `https://` address of a host.
    Address,
    /// No model named.
    Model,
    /// A host elsewhere than this computer, the user's agreement to it not given.
    NotAgreed(String),
}

/// A service named to hear, checked before anything is saved: its host, and the host the user
/// agreed to — none for this computer's own, where nothing of a meeting leaves it (ADR-0013).
pub fn service_named(provider: &str, model: &str, agreed: bool) -> Result<(String, Option<String>), Refused> {
    let provider = provider.trim();
    let host = crate::providers::host(provider).ok_or(Refused::Address)?;
    if model.trim().is_empty() {
        return Err(Refused::Model);
    }
    if crate::providers::local(provider) {
        return Ok((host, None));
    }
    if !agreed {
        return Err(Refused::NotAgreed(host));
    }
    Ok((host.clone(), Some(host)))
}

/// The keyring's account of a transcription service's key: apart from the minutes', which is
/// the host alone — one host may hear and write with two keys.
pub fn key_account(host: &str) -> String {
    format!("transcription {host}")
}

impl Hearer {
    /// Who hears, as the settings chose it: Apple's SpeechAnalyzer only while this Mac still offers
    /// it with its model there (`apple_ready`, asked only then) — otherwise the Whisper model, so
    /// that a choice the Mac no longer answers never fails every transcript.
    pub fn chosen(engine: crate::settings::Engine, model: &Path, language: &str, apple_ready: impl FnOnce() -> bool) -> Hearer {
        match engine {
            crate::settings::Engine::SpeechAnalyzer if apple_ready() => Hearer::SpeechAnalyzer { language: language.into() },
            _ => Hearer::Model(model.to_path_buf()),
        }
    }

    /// Who hears, as the settings say it: a service only once the user agreed to its host — or it
    /// is this computer —, as the minutes are sent (ADR-0013); else as `chosen` says.
    pub fn of(settings: &crate::settings::Settings, language: &str, apple_ready: impl FnOnce() -> bool) -> Hearer {
        if settings.engine == crate::settings::Engine::Service
            && let (Some(provider), Some(model)) = (&settings.transcription_provider, &settings.transcription_model)
        {
            let host = crate::providers::host(provider);
            let agreed = crate::providers::local(provider) || (host.is_some() && host == settings.transcription_agreed);
            if agreed {
                return Hearer::Service {
                    provider: provider.clone(),
                    model: model.clone(),
                };
            }
        }
        let engine = match settings.engine {
            crate::settings::Engine::Service => crate::settings::Engine::Whisper,
            engine => engine,
        };
        Hearer::chosen(engine, &settings.model, language, apple_ready)
    }

    /// Its name, as the page says who hears.
    pub fn name(&self) -> String {
        match self {
            Hearer::Model(model) => named(model),
            Hearer::SpeechAnalyzer { .. } => "SpeechAnalyzer".into(),
            Hearer::Service { model, .. } => model.clone(),
        }
    }
}

/// A transcript in the making. Dropped, it stops nothing; ended, the program is stopped, and
/// what it had written is kept for the next run.
pub struct Transcription {
    program: Arc<Mutex<Child>>,
}

impl Transcription {
    /// Starts the program that hears on a session's folder, with who hears. How it ended
    /// comes once on the receiver: `Written` only when it ended well with the transcript there.
    pub fn start(program: &Path, folder: &Path, hearer: &Hearer) -> io::Result<(Self, Receiver<Made>)> {
        Self::start_with(program, folder, hearer, None, None)
    }

    /// The same, a service's key handed in the program's environment, never its arguments; and the
    /// language the meeting is spoken in, when it or the settings say one (D65) — told to Whisper
    /// and a service, and to SpeechAnalyzer when it hears it.
    pub fn start_with(
        program: &Path,
        folder: &Path,
        hearer: &Hearer,
        key: Option<&str>,
        spoken: Option<&str>,
    ) -> io::Result<(Self, Receiver<Made>)> {
        let (trace, making) = crate::log::making();
        let _making = making.enter();
        let mut command = Command::new(program);
        command.arg(folder).env("KITOKEY_TRACE", &trace);
        let model = match hearer {
            Hearer::Model(model) => {
                command.arg("--model").arg(model);
                if let Some(spoken) = spoken {
                    command.args(["--language", spoken]);
                }
                named(model)
            }
            Hearer::SpeechAnalyzer { language } => {
                // It hears French and English, and detects none: the system's language otherwise (D64).
                let language = spoken.filter(|spoken| matches!(*spoken, "fr" | "en")).unwrap_or(language);
                command.args(["--engine", "speechanalyzer", "--language", language]);
                "SpeechAnalyzer".to_string()
            }
            Hearer::Service { provider, model, .. } => {
                command.args(["--provider", provider, "--model", model]);
                // None said: the service detects it — never the app's language (D64, D65).
                if let Some(spoken) = spoken {
                    command.args(["--language", spoken]);
                }
                model.clone()
            }
        };
        command.env_remove("KITOKEY_PROVIDER_KEY");
        if let Some(key) = key {
            command.env("KITOKEY_PROVIDER_KEY", key);
        }
        info!(session = %named(folder), model = %model, "shell.transcript");
        // A group of its own: ended, whatever it started ends with it.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .inspect_err(|error| error!(detail = %error, "shell.transcript_failed"))?;
        let (Some(mut output), Some(errors)) = (child.stdout.take(), child.stderr.take()) else {
            return Err(io::Error::other("the program that hears could not be listened to"));
        };
        // Its output is read to its end, so that it never fills and holds the program up.
        thread::spawn(move || io::copy(&mut output, &mut io::sink()));
        let program = Arc::new(Mutex::new(child));
        let (tell, made) = mpsc::channel();
        let (waited, folder) = (Arc::clone(&program), folder.to_path_buf());
        // Its last sentence on the error stream is why it failed, kept as it is read, apart from
        // the wait: a program of its own may keep that stream open after it ended.
        let (last, read) = (Arc::new(Mutex::new(String::new())), mpsc::channel());
        let (kept, (done, all_read)) = (Arc::clone(&last), read);
        thread::spawn(move || {
            last_line(errors, &kept);
            let _ = done.send(());
        });
        let making = making.clone();
        thread::spawn(move || {
            let _making = making.enter();
            // Asked, not waited on with the program held: `end` may need it meanwhile.
            let status = loop {
                match waited.lock().map(|mut child| child.try_wait()) {
                    Ok(Ok(None)) => thread::sleep(Duration::from_millis(100)),
                    Ok(Ok(Some(status))) => break Some(status),
                    _ => break None,
                }
            };
            let written = status.is_some_and(|status| status.success()) && folder.join("transcript.jsonl").is_file();
            // A second for what it wrote last to be read; then what was read is what it said.
            let _ = all_read.recv_timeout(Duration::from_secs(1));
            let why = last.lock().map(|said| said.clone()).unwrap_or_default();
            match written {
                true => info!("shell.transcript_written"),
                false => error!(detail = %crate::log::without(&why, folder.parent()), status = ?status, "shell.transcript_failed"),
            }
            let _ = tell.send(if written { Made::Written(folder) } else { Made::Failed(why) });
        });
        Ok((Self { program }, made))
    }

    /// Stops the program now: Kitokey is quitting. What it had heard stays written.
    pub fn end(&self) {
        if let Ok(mut child) = self.program.lock() {
            // Only while it runs: once it is waited for, its group's number may be another's.
            #[cfg(unix)]
            if let (Ok(None), Ok(group)) = (child.try_wait(), i32::try_from(child.id())) {
                // SAFETY: a signal sent to the group this program leads, which it was started in.
                unsafe { libc::kill(-group, libc::SIGKILL) };
            }
            let _ = child.kill();
        }
    }

    /// Stops the program on the user's word, « Annuler » (D92): within two seconds, without
    /// waiting here. What it had written is the shell's to settle once it is said ended.
    pub fn cancel(&self) {
        (self.stopper())();
    }

    /// The same, to be held by the making's token (`crate::cancel::Cancel::hold`).
    pub fn stopper(&self) -> impl FnOnce() + Send + 'static {
        let program = Arc::clone(&self.program);
        move || {
            thread::spawn(move || cancel(&program));
        }
    }
}

/// How long a program told to stop is given before it is killed, with what it started: the two
/// seconds « Annuler » promises hold it, the shell's waiting a tenth of a second at a time (D92).
const GRACE: Duration = Duration::from_secs(1);

/// Stops a program that leads a group of its own, and whatever it started: TERM to the group, a
/// second's grace, then KILL to what is left of it. The group is taken while its leader runs, its
/// number its own then; within the grace, no other process can have been given it.
#[cfg(unix)]
pub(crate) fn cancel(program: &Mutex<Child>) {
    let running = |child: &mut Child| matches!(child.try_wait(), Ok(None));
    let Some(group) = program
        .lock()
        .ok()
        .and_then(|mut child| running(&mut child).then(|| i32::try_from(child.id()).ok()).flatten())
    else {
        return;
    };
    // SAFETY: signals sent to the group this program leads, which it was started in.
    unsafe { libc::kill(-group, libc::SIGTERM) };
    let until = std::time::Instant::now() + GRACE;
    // SAFETY: signal 0 sends nothing: it asks whether the group still holds a process.
    while std::time::Instant::now() < until && unsafe { libc::kill(-group, 0) } == 0 {
        thread::sleep(Duration::from_millis(50));
    }
    // SAFETY: as above; none left, it reaches no one.
    unsafe { libc::kill(-group, libc::SIGKILL) };
}

/// Elsewhere, the program alone: no step starts one of its own there — the one program a step
/// starts, SpeechAnalyzer's helper, is macOS's alone.
#[cfg(not(unix))]
pub(crate) fn cancel(program: &Mutex<Child>) {
    if let Ok(mut child) = program.lock() {
        let _ = child.kill();
    }
}

/// A file's or a folder's own name, never its path: the path says whose meetings and where.
pub(crate) fn named(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Reads a stream to its end, keeping its last line that is not blank as it goes.
pub(crate) fn last_line(stream: impl Read, kept: &Mutex<String>) {
    for line in BufReader::new(stream).split(b'\n').map_while(Result::ok) {
        let line = String::from_utf8_lossy(&line).trim().to_string();
        if let (false, Ok(mut kept)) = (line.is_empty(), kept.lock()) {
            *kept = line;
        }
    }
}

/// The last session of a folder of sessions: the one whose name comes last — a session is
/// named by when it started — among those that hold a `session.json`.
pub fn latest(sessions: &Path) -> Option<PathBuf> {
    let folders = std::fs::read_dir(sessions).ok()?.filter_map(Result::ok).map(|entry| entry.path());
    folders.filter(|folder| folder.join("session.json").is_file()).max()
}
