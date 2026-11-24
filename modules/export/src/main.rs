//! `kitokey-export`: a meeting's folder written as one file, as `exporting/v1` says.
//!
//! The arguments hold no word of the meeting: the session's folder, named by its day and hour, and the
//! choices of form. Where to write and the file's words come on the input, one JSON object: a process's
//! arguments are public to the machine's users. The file is written beside its place, then renamed over
//! it: whole or not at all, as `pandoc -o` replaces a file.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use kitokey_export::{Data, Dated, Face, FontInfo, Include, Labels, Meeting, chosen, docx, html, ics, markdown, pdf, srt, vtt};
use serde_json::{Value, json};

/// The version this program says (`--version`): the package's, named when it is built (`KITOKEY_VERSION`,
/// D78), Cargo's when unset or empty.
const VERSION: &str = match option_env!("KITOKEY_VERSION") {
    Some(version) if !version.is_empty() => version,
    _ => env!("CARGO_PKG_VERSION"),
};

/// The formats this version writes, every one `exporting/v1` names (D108, D117). A format a later version of
/// the contract names is a wrong argument until it joins this list.
const WRITTEN: &[&str] = &["md", "html", "docx", "srt", "vtt", "pdf", "ics"];

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

/// The input: one JSON object, where to write and the file's words, each on one line, and the dates an
/// `.ics` is made of, which `ics` alone reads (`input.schema.json`).
fn input(format: &str) -> Result<(PathBuf, String, Labels, Vec<Dated>), Refused> {
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
    let dates = match format {
        "ics" => dates(object.get("dates")).map_err(|why| wrong(why.into()))?,
        _ => Vec::new(),
    };
    Ok((PathBuf::from(field("to")?), name, labels, dates))
}

/// The input's `dates`, as `input.schema.json` says them: what a schema cannot say — an `id` twice, a day
/// no calendar has — is the writer's to refuse. Why, in words of the contract, never of the input.
fn dates(dates: Option<&Value>) -> Result<Vec<Dated>, &'static str> {
    let items = dates.and_then(Value::as_array).filter(|items| !items.is_empty());
    let items = items.ok_or("« dates » missing, empty or no array")?;
    fn one_line(value: &Value) -> Option<&str> {
        value.as_str().filter(|text| !text.is_empty() && !text.contains(['\n', '\r']))
    }
    items
        .iter()
        .map(|item| {
            let id = item["id"].as_str().filter(|id| {
                let bytes = id.as_bytes();
                (1..=64).contains(&bytes.len())
                    && bytes[0] != b'-'
                    && bytes.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
            });
            let optional = |name: &str, line: bool| match item.get(name) {
                None | Some(Value::Null) => Ok(None),
                Some(value) => value
                    .as_str()
                    .filter(|text| !text.is_empty() && !(line && text.contains(['\n', '\r'])))
                    .map(|text| Some(text.to_string()))
                    .ok_or(()),
            };
            Ok(Dated {
                id: id.ok_or("an item's « id » missing or not [a-z0-9-]")?.to_string(),
                date: item["date"].as_str().ok_or("an item's « date » missing")?.to_string(),
                what: one_line(&item["what"])
                    .ok_or("an item's « what » missing, empty or on more than one line")?
                    .to_string(),
                who: optional("who", true).map_err(|()| "an item's « who » empty, not text or on more than one line")?,
                passage: optional("passage", false).map_err(|()| "an item's « passage » empty or not text")?,
            })
        })
        .collect()
}

fn export() -> Result<Value, Refused> {
    let asked = asked()?;
    let (to, name, labels, dates) = input(&asked.format)?;
    let unreadable = || Refused("the session's folder could not be read".into(), Some("meeting.unreadable"));
    if !asked.folder.is_dir() {
        return Err(unreadable());
    }
    let meeting = Meeting::read(&asked.folder).map_err(|_| unreadable())?;
    let include = asked.include;
    let unwritable = |error: io::Error| Refused(format!("the file could not be written: {error}"), Some("file.unwritable"));
    // The `.ics` is the dates alone: named by the minutes' title, or the input's name; no summary nor
    // transcript needed, no option of form read.
    if asked.format == "ics" {
        let folder_name = asked.folder.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        let title = meeting.title.as_deref().unwrap_or(&name);
        let written =
            ics(&dates, title, &folder_name, stamp).map_err(|why| Refused(format!("the input is not exporting/v1's: {why}"), None))?;
        let bytes = whole(&to, written.as_bytes()).map_err(unwritable)?;
        return Ok(json!({"event": "written", "file": to, "bytes": bytes}));
    }
    // Subtitles are the transcript alone (`exporting/v1`): a summary makes none.
    let subtitles = matches!(asked.format.as_str(), "srt" | "vtt");
    let transcript = include.transcript && !meeting.segments.is_empty();
    if !(transcript || !subtitles && include.summary && !meeting.sections.is_empty()) {
        return Err(Refused(
            "nothing to write: neither a summary nor a transcript".into(),
            Some("meeting.empty"),
        ));
    }
    let written = match asked.format.as_str() {
        "md" => markdown(&meeting, include, &labels).into_bytes(),
        "html" => html(&meeting, include, &labels, &asked.lang, &name).into_bytes(),
        "docx" => docx(&meeting, include, &labels, &name).map_err(unwritable)?,
        "srt" => srt(&meeting, include, &labels).into_bytes(),
        "vtt" => vtt(&meeting, include, &labels, &asked.lang).into_bytes(),
        "pdf" => {
            let system = System::read();
            let taken = chosen(&meeting, include, &labels, &name, &system.infos(), Some(&system.sans()))
                .map_err(|_| Refused("no font of the system has the meeting's letters".into(), Some("pdf.no_font")))?;
            let faces = system.faces(&taken);
            // typst's message may quote the meeting: the sentence says none of it.
            pdf(&meeting, include, &labels, &asked.lang, &name, &faces)
                .map_err(|_| Refused("the PDF could not be laid out".into(), Some("pdf.failed")))?
        }
        other => unreachable!("{other} is refused with the arguments"),
    };
    let bytes = whole(&to, &written).map_err(unwritable)?;
    Ok(json!({"event": "written", "file": to, "bytes": bytes}))
}

/// The system's fonts: fontconfig's folders on Linux, the system's on macOS and Windows (fontdb). None
/// ships with the program.
struct System {
    fonts: fontdb::Database,
    /// Each face typst reads, and what it is: its family, whether it is variable, its characters — its
    /// file read for this and not kept.
    faces: Vec<(fontdb::ID, FontInfo)>,
}

impl System {
    fn read() -> System {
        let mut fonts = fontdb::Database::new();
        fonts.load_system_fonts();
        let faces = fonts
            .faces()
            .filter_map(|face| {
                let info = fonts.with_face_data(face.id, FontInfo::new)??;
                Some((face.id, info))
            })
            .collect();
        System { fonts, faces }
    }

    /// The system's sans-serif family's name.
    fn sans(&self) -> String {
        self.fonts.family_name(&fontdb::Family::SansSerif).to_string()
    }

    fn infos(&self) -> Vec<FontInfo> {
        self.faces.iter().map(|(_, info)| info.clone()).collect()
    }

    /// The faces `taken`, each file read once.
    fn faces(&self, taken: &[usize]) -> Vec<Face> {
        let mut read: HashMap<PathBuf, Data> = HashMap::new();
        taken
            .iter()
            .filter_map(|&i| {
                let face = self.fonts.face(self.faces[i].0)?;
                let data = match &face.source {
                    fontdb::Source::Binary(data) => data.clone(),
                    fontdb::Source::File(path) | fontdb::Source::SharedFile(path, _) => match read.get(path) {
                        Some(data) => data.clone(),
                        None => {
                            let data: Data = Arc::new(fs::read(path).ok()?);
                            read.insert(path.clone(), data.clone());
                            data
                        }
                    },
                };
                Some(Face { data, index: face.index })
            })
            .collect()
    }
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
