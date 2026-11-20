//! Each summary written, written to the folder the user chose too (D109): by `kitokey-export`, as
//! `contracts/exporting/v1` starts it — Markdown, the summary alone.

use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// What the program is given on its input: where to write, and the file's words (`exporting/v1`).
#[derive(serde::Serialize)]
pub struct Input<'a> {
    pub to: &'a Path,
    pub name: &'a str,
    pub transcript: &'a str,
    pub me: &'a str,
    pub others: &'a str,
    pub time: &'a str,
    pub speaker: &'a str,
    pub text: &'a str,
}

/// How long the program is given: a Markdown of a summary takes it well under a second; past this,
/// it is stopped and the export said failed, the next summary never kept waiting behind it.
const BOUND: Duration = Duration::from_secs(60);

/// Writes `folder`'s summary as Markdown at `input.to`: done once the program said `written` and
/// ended well; else why, for the logs — its own sentence, or its code.
pub fn markdown(program: &Path, folder: &Path, language: &str, input: &Input) -> Result<(), String> {
    let mut child = Command::new(program)
        .arg(folder)
        .args(["--format", "md", "--lang", language, "--without-transcript"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("kitokey-export could not be started ({error})"))?;
    let given = serde_json::to_vec(input).map_err(|error| error.to_string())?;
    // Written, then closed: the end of its input is the end of the object.
    let sent = child.stdin.take().map(|mut stdin| stdin.write_all(&given));
    let until = Instant::now() + BOUND;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < until => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("kitokey-export did not end within {} s", BOUND.as_secs()));
            }
        }
    };
    let (mut out, mut err) = (String::new(), String::new());
    child.stdout.take().map(|mut stdout| stdout.read_to_string(&mut out));
    child.stderr.take().map(|mut stderr| stderr.read_to_string(&mut err));
    let last = out
        .lines()
        .last()
        .and_then(|line| serde_json::from_str::<serde_json::Value>(line).ok());
    let event = last.as_ref().and_then(|line| line.get("event")?.as_str());
    if status.success() && event == Some("written") {
        return Ok(());
    }
    if let Some(Err(error)) = sent {
        return Err(format!("kitokey-export's input could not be written ({error})"));
    }
    let code = last.as_ref().and_then(|line| line.get("code")?.as_str()).unwrap_or("none");
    let said = err.trim();
    Err(if said.is_empty() {
        format!("kitokey-export failed, code {code}")
    } else {
        format!("{said} (code {code})")
    })
}
