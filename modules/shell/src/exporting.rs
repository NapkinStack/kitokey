//! Each summary written, written to the folder the user chose too (D109): by `kitokey-export`, as
//! `contracts/exporting/v1` starts it — Markdown, the summary alone. And the dates a user ticks on a
//! meeting's page, as an `.ics` (D115): `ics`, its `dates` on the input.

use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// What the program is given on its input: where to write, and the file's words (`exporting/v1`).
#[derive(Clone, Copy, serde::Serialize)]
pub struct Input<'a> {
    pub to: &'a Path,
    pub name: &'a str,
    pub transcript: &'a str,
    pub me: &'a str,
    pub others: &'a str,
    pub time: &'a str,
    pub speaker: &'a str,
    pub text: &'a str,
    /// The `.ics`'s events (`exporting/v1`, *The dates of an `.ics`*); absent for any other format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dates: Option<&'a [Dated]>,
}

/// One ticked date as `exporting/v1` takes it: `id` the action's place in the minutes (`a1`, `a2`…), stable for
/// the meeting, which the `.ics` makes its UID of; `what` and `who` on one line, as the contract wants them.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dated {
    pub id: String,
    pub date: String,
    pub what: String,
    pub who: Option<String>,
    pub passage: Option<String>,
}

impl Dated {
    /// The action at `place` (from one) in the minutes' actions: its words on one line, its passage as said.
    pub fn new(place: usize, date: &str, what: &str, who: Option<&str>, passage: Option<&str>) -> Self {
        let one_line = |text: &str| {
            text.split(['\n', '\r'])
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        };
        Dated {
            id: format!("a{place}"),
            date: date.to_string(),
            what: one_line(what),
            who: who.map(one_line).filter(|who| !who.is_empty()),
            passage: passage.map(str::to_string).filter(|passage| !passage.trim().is_empty()),
        }
    }
}

/// How long the program is given: a Markdown of a summary takes it well under a second; past this,
/// it is stopped and the export said failed, the next summary never kept waiting behind it.
const BOUND: Duration = Duration::from_secs(60);

/// Writes `folder`'s summary as Markdown at `input.to`: done once the program said `written` and
/// ended well; else why, for the logs — its own sentence, or its code.
pub fn markdown(program: &Path, folder: &Path, language: &str, input: &Input) -> Result<(), String> {
    exported(
        program,
        folder,
        &["--format", "md", "--lang", language, "--without-transcript"],
        input,
    )
}

/// A meeting's summary exported where the user chose (D121): a PDF or Markdown, the summary alone, as a send carries
/// it (PDR-0007 sends never the transcript); done or why, as `markdown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Pdf,
    Md,
}

impl Format {
    /// The format as `exporting/v1` names it, and the file's extension.
    pub fn name(self) -> &'static str {
        match self {
            Format::Pdf => "pdf",
            Format::Md => "md",
        }
    }
}

/// Writes `folder`'s summary at `input.to` in `format` (D121).
pub fn file(program: &Path, folder: &Path, format: Format, language: &str, input: &Input) -> Result<(), String> {
    exported(
        program,
        folder,
        &["--format", format.name(), "--lang", language, "--without-transcript"],
        input,
    )
}

/// Writes `input.dates` as an `.ics` at `input.to` (D115): done or why, as `markdown`.
pub fn ics(program: &Path, folder: &Path, language: &str, input: &Input) -> Result<(), String> {
    // As the page sends them, each date made what the contract wants: `what` and `who` on one line.
    let dates: Option<Vec<Dated>> = input.dates.map(|dates| {
        dates
            .iter()
            .map(|d| Dated {
                id: d.id.clone(),
                ..Dated::new(0, &d.date, &d.what, d.who.as_deref(), d.passage.as_deref())
            })
            .collect()
    });
    let input = Input {
        dates: dates.as_deref(),
        ..*input
    };
    exported(program, folder, &["--format", "ics", "--lang", language], &input)
}

fn exported(program: &Path, folder: &Path, args: &[&str], input: &Input) -> Result<(), String> {
    let mut child = Command::new(program)
        .arg(folder)
        .args(args)
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
