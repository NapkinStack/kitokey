//! « Vérifier le son » (D88): the program that records, asked only to listen (`contracts/recording/v1`,
//! *Listening*) while the panel shows — each source's device and level, whether the microphone is a
//! Bluetooth headset's, and whether a test sound played was heard on the meeting's side. Nothing is
//! recorded; the microphone is open only between `start` and `stop`.

use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;

use crate::recording::{self, ABOVE_SILENCE, Said, Source};

/// What the panel shows, as the program last said it.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Heard {
    /// Each side's device, the microphone then the meeting's: none until named.
    pub devices: [Option<String>; 2],
    /// Each side said it could not be opened.
    pub unavailable: [bool; 2],
    /// How loud each side is now, from 0 to 1.
    pub levels: [f32; 2],
    /// capture said the microphone listened to is a Bluetooth headset's — it may switch to its call
    /// profile while the panel shows; false when it said not, or nothing: not known.
    pub headset: bool,
    /// Whether the meeting's side rose above silence since a test sound was asked; none, none asked.
    pub test: Option<bool>,
    /// The program ended: on its own, it could not listen.
    pub ended: bool,
    /// It ended in failure: its last sentence, for the log — the panel says its own.
    #[serde(skip)]
    pub failed: Option<String>,
}

#[derive(Default)]
struct State {
    heard: Heard,
    /// The meeting's side's loudest since the test sound was asked.
    peak: Option<f32>,
    /// The last line of its error stream, once its output closed.
    complaint: Option<String>,
}

/// The program listening, for as long as the panel shows.
pub struct SoundCheck {
    program: Mutex<Child>,
    input: Option<ChildStdin>,
    state: Arc<Mutex<State>>,
}

/// How long the program is given to stop once told, before it is killed: `recording/v1` promises one second.
const STOPS_WITHIN: Duration = Duration::from_secs(1);

impl SoundCheck {
    /// Starts the program listening — the microphone the user chose, or none for Auto. What it says is
    /// read on a thread of its own, to its end.
    pub fn start(program: &Path, microphone: Option<&str>) -> std::io::Result<Self> {
        let mut program = Command::new(program)
            .arg("listen")
            .args(microphone.into_iter().flat_map(|name| ["--microphone", name]))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let (output, errors) = (program.stdout.take(), program.stderr.take());
        let (Some(output), Some(errors)) = (output, errors) else {
            let _ = program.kill();
            let _ = program.wait();
            return Err(std::io::Error::other("the program could not be listened to"));
        };
        let input = program.stdin.take();
        let state = Arc::new(Mutex::new(State::default()));
        // Its error stream read as it comes, so that it never fills: the last line is its sentence.
        let complaint = thread::spawn(move || {
            let lines = BufReader::new(errors).lines().map_while(Result::ok);
            lines.filter(|line| !line.trim().is_empty()).last()
        });
        let told = state.clone();
        thread::spawn(move || {
            for line in BufReader::new(output).lines().map_while(Result::ok) {
                heard(&mut told.lock().unwrap(), &line);
            }
            let complaint = complaint.join().ok().flatten();
            told.lock().unwrap().complaint = complaint;
        });
        Ok(Self {
            program: Mutex::new(program),
            input,
            state,
        })
    }

    /// What the panel shows now.
    pub fn now(&self) -> Heard {
        let ended = self.program.lock().unwrap().try_wait().ok().flatten();
        let mut state = self.state.lock().unwrap();
        if let Some(status) = ended
            && !state.heard.ended
        {
            state.heard.ended = true;
            if !status.success() {
                let said = state.complaint.clone().unwrap_or_default();
                tracing::warn!(detail = %said, "shell.sound_check_failed");
                state.heard.failed = Some(said);
            }
        }
        Heard {
            test: state.peak.map(|peak| peak > ABOVE_SILENCE),
            ..state.heard.clone()
        }
    }

    /// A test sound is about to play: the meeting's side is listened to for it from now on.
    pub fn mark(&self) {
        self.state.lock().unwrap().peak = Some(0.0);
    }

    /// Tells the program to stop, and waits for it to end — killed past a second: once this returns,
    /// nothing of it is open.
    pub fn stop(mut self) {
        if let Some(mut input) = self.input.take() {
            let _ = input.write_all(b"stop\n").and_then(|()| input.flush());
        }
        let program = self.program.get_mut().unwrap();
        let told = Instant::now();
        while matches!(program.try_wait(), Ok(None)) && told.elapsed() < STOPS_WITHIN {
            thread::sleep(Duration::from_millis(10));
        }
        if matches!(program.try_wait(), Ok(None)) {
            tracing::warn!("shell.sound_check_killed");
            let _ = program.kill();
        }
        let _ = program.wait();
    }
}

/// One line of the program, as `recording/v1` says it; anything else passed over.
fn heard(state: &mut State, line: &str) {
    let side = |source: Source| usize::from(source == Source::Others);
    match recording::read(line) {
        Some(Said::Moved(source, name)) => {
            let index = side(source);
            state.heard.devices[index] = name;
            state.heard.unavailable[index] = false;
            // Only the microphone's first `device` says it; absent — on a later one — it is not known
            // (`recording/v1`), and the panel says nothing of a headset. `others`' is never read.
            if source == Source::Me {
                let said = serde_json::from_str::<Value>(line)
                    .ok()
                    .and_then(|event| event["headset"].as_bool());
                state.heard.headset = said == Some(true);
            }
        }
        Some(Said::Lost(source)) => state.heard.devices[side(source)] = None,
        Some(Said::Unavailable(source, _)) => state.heard.unavailable[side(source)] = true,
        Some(Said::Level(source, level)) => {
            state.heard.levels[side(source)] = level;
            if source == Source::Others
                && let Some(peak) = &mut state.peak
            {
                *peak = peak.max(level);
            }
        }
        _ => {}
    }
}
