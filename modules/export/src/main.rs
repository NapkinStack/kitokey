//! `kitokey-export`: a meeting's folder written as one file, as `exporting/v1` says.
//!
//! The arguments hold no word of the meeting: the session's folder, named by its day and hour, and the
//! choices of form. Where to write and the file's words come on the input, one JSON object: a process's
//! arguments are public to the machine's users. The file is written beside its place, then renamed over
//! it: whole or not at all, as `pandoc -o` replaces a file.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kitokey_export::{Include, Labels, Meeting, docx, html, markdown, srt, vtt};
use serde_json::{Value, json};

/// The version this program says (`--version`): the package's, named when it is built (`KITOKEY_VERSION`,
/// D78), Cargo's when unset or empty.
const VERSION: &str = match option_env!("KITOKEY_VERSION") {
    Some(version) if !version.is_empty() => version,
    _ => env!("CARGO_PKG_VERSION"),
};

/// The formats this version writes. `exporting/v1` names `pdf` too: it joins this list as its writer is
/// delivered (D108), and until then is a wrong argument, which the shell never gives — it offers only
/// the formats this program writes.
const WRITTEN: &[&str] = &["md", "html", "docx", "srt", "vtt"];

/// Why the program ends in failure: its sentence, and the code a person acts on, when there is one.
struct Refused(String, Option<&'static str>);

fn main() -> ExitCode {
    if std::env::args_os().skip(1).eq(["--version"]) {
        println!("kitokey-export {VERSION}");
        return ExitCode::SUCCESS;
    }
    match export() {
        Ok(written) => {
            println!("{written}");
            ExitCode::SUCCESS
        }
        Err(Refused(why, code)) => {
            eprintln!("kitokey-export: {why}");
            match code {
                // A cause of the contract: its line, then the failure.
                Some(code) => {
                    println!("{}", json!({"event": "failed", "code": code}));
                    ExitCode::FAILURE
                }
                // A wrong argument or input — a format not written yet among them —: a usage error, 2, as
                // pandoc's and typst's, before the contract is entered.
                None => ExitCode::from(2),
            }
        }
    }
}

const USAGE: &str = "usage: kitokey-export <session-folder> --format <format> --lang <language> \
     [--without-summary] [--without-transcript] [--without-speakers] [--without-times], \
     where to write and the file's words on the input";

/// What the arguments ask: the folder, the format, the language, what is left out.
struct Asked {
    folder: PathBuf,
    format: String,
    lang: String,
    include: Include,
}

fn asked() -> Result<Asked, Refused> {
    let wrong = |why: &str| Refused(format!("{why}; {USAGE}"), None);
    let mut args = Vec::new();
    for arg in std::env::args_os().skip(1) {
        args.push(arg.into_string().map_err(|_| wrong("an argument is not text"))?);
    }
    // Started bare, it says its usage alone, as the other programs do.
    if args.is_empty() {
        return Err(Refused(USAGE.into(), None));
    }
    let (mut folder, mut format, mut lang) = (None, None, None);
    let mut include = Include::all();
    let mut given = std::collections::HashSet::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        // An option given twice is a mistake, never the last one winning (as kitokey-summary, #1042).
        if arg.starts_with("--") && !given.insert(arg.clone()) {
            return Err(wrong(&format!("{arg} given twice")));
        }
        match arg.as_str() {
            "--format" => format = Some(args.next().ok_or_else(|| wrong("--format without its value"))?),
            "--lang" => lang = Some(args.next().ok_or_else(|| wrong("--lang without its value"))?),
            "--without-summary" => include.summary = false,
            "--without-transcript" => include.transcript = false,
            "--without-speakers" => include.speakers = false,
            "--without-times" => include.times = false,
            option if option.starts_with("--") => return Err(wrong(&format!("{option} is no option of this program"))),
            _ if folder.is_some() => return Err(wrong("one session's folder only")),
            _ => folder = Some(PathBuf::from(arg)),
        }
    }
    let folder = folder.ok_or_else(|| wrong("no session's folder"))?;
    let format = format.ok_or_else(|| wrong("no --format"))?;
    let lang = lang.ok_or_else(|| wrong("no --lang"))?;
    if !WRITTEN.contains(&format.as_str()) {
        return Err(wrong(&format!("this version writes {}, not « {format} »", WRITTEN.join(", "))));
    }
    // A BCP 47 tag: letters, digits and hyphens.
    if lang.is_empty() || lang.len() > 35 || !lang.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err(wrong("--lang is not a language's tag"));
    }
    Ok(Asked {
        folder,
        format,
        lang,
        include,
    })
}

/// The input: one JSON object, where to write and the file's words, each on one line (`input.schema.json`).
fn input() -> Result<(PathBuf, String, Labels), Refused> {
    let wrong = |why: String| Refused(format!("the input is not exporting/v1's: {why}"), None);
    let mut text = String::new();
    io::stdin().read_to_string(&mut text).map_err(|error| wrong(error.to_string()))?;
    let object: Value = serde_json::from_str(&text).map_err(|error| wrong(error.to_string()))?;
    let field = |name: &str| -> Result<String, Refused> {
        match object.get(name).and_then(Value::as_str) {
            Some(value) if !value.is_empty() && !value.contains(['\n', '\r']) => Ok(value.to_string()),
            _ => Err(wrong(format!("« {name} » missing, empty or on more than one line"))),
        }
    };
    // `name` is the meeting's, for a format that names it (HTML's title): required by the contract, unused by Markdown.
    let name = field("name")?;
    let labels = Labels {
        transcript: field("transcript")?,
        me: field("me")?,
        others: field("others")?,
        time: field("time")?,
        speaker: field("speaker")?,
        text: field("text")?,
    };
    Ok((PathBuf::from(field("to")?), name, labels))
}

fn export() -> Result<Value, Refused> {
    let asked = asked()?;
    let (to, name, labels) = input()?;
    let unreadable = || Refused("the session's folder could not be read".into(), Some("meeting.unreadable"));
    if !asked.folder.is_dir() {
        return Err(unreadable());
    }
    let meeting = Meeting::read(&asked.folder).map_err(|_| unreadable())?;
    let include = asked.include;
    // Subtitles are the transcript alone (`exporting/v1`): a summary makes none.
    let subtitles = matches!(asked.format.as_str(), "srt" | "vtt");
    let transcript = include.transcript && !meeting.segments.is_empty();
    if !(transcript || !subtitles && include.summary && !meeting.sections.is_empty()) {
        return Err(Refused(
            "nothing to write: neither a summary nor a transcript".into(),
            Some("meeting.empty"),
        ));
    }
    let unwritable = |error: io::Error| Refused(format!("the file could not be written: {error}"), Some("file.unwritable"));
    let written = match asked.format.as_str() {
        "md" => markdown(&meeting, include, &labels).into_bytes(),
        "html" => html(&meeting, include, &labels, &asked.lang, &name).into_bytes(),
        "docx" => docx(&meeting, include, &labels, &name).map_err(unwritable)?,
        "srt" => srt(&meeting, include, &labels).into_bytes(),
        "vtt" => vtt(&meeting, include, &labels).into_bytes(),
        other => unreachable!("{other} is refused with the arguments"),
    };
    let bytes = whole(&to, &written).map_err(unwritable)?;
    Ok(json!({"event": "written", "file": to, "bytes": bytes}))
}

/// Writes `bytes` at `to` whole or not at all: a file beside it, flushed, then renamed over it.
fn whole(to: &Path, bytes: &[u8]) -> io::Result<usize> {
    let folder = to
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = to.file_name().ok_or_else(|| io::Error::other("no file's name"))?;
    let beside = folder.join(format!(".{}.{}.part", name.to_string_lossy(), std::process::id()));
    let written = (|| {
        let mut file = fs::File::create(&beside)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&beside, to)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&beside);
    }
    written.map(|()| bytes.len())
}
