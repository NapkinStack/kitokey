//! export: a meeting's folder read into one model, then written as a file (D108).
//!
//! It reads three contracts and nothing else: `session-audio` for when the meeting started,
//! `minutes` v1 to v4 — and v5's named sections as they come — for the summary, `transcript/v1`
//! for what was said. The words a file shows that no contract carries come from the caller.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::Value;

/// What the caller asks to be written.
#[derive(Clone, Copy, Debug)]
pub struct Include {
    pub summary: bool,
    pub transcript: bool,
    pub speakers: bool,
    pub times: bool,
}

impl Include {
    pub fn all() -> Self {
        Include {
            summary: true,
            transcript: true,
            speakers: true,
            times: true,
        }
    }
}

/// The words a file shows that no contract carries: the shell's, in the user's language.
#[derive(Clone, Debug)]
pub struct Labels {
    pub transcript: String,
    pub me: String,
    pub others: String,
    pub time: String,
    pub speaker: String,
    pub text: String,
}

/// A section of the summary: its title as the model wrote it, its body as Markdown.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub title: String,
    pub body: String,
}

/// Who spoke a segment: the track it was heard on (`transcript/v1`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Speaker {
    Me,
    Others,
}

/// A stretch of speech, its start in seconds from the session's start.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub start: f64,
    pub speaker: Speaker,
    pub text: String,
}

/// A meeting, as every format writes it.
#[derive(Clone, Debug, Default)]
pub struct Meeting {
    /// The summary's `title`, when it holds one within `minutes`' bounds.
    pub title: Option<String>,
    /// `started`, as the session wrote it.
    pub started: Option<String>,
    /// The transcript's `duration`, in seconds.
    pub duration: Option<f64>,
    pub transcript_model: Option<String>,
    pub summary_model: Option<String>,
    /// The summary's sections, in their order: read by their place, whatever their number.
    pub sections: Vec<Section>,
    /// The segments in the meeting's order.
    pub segments: Vec<Segment>,
}

impl Meeting {
    /// Reads a meeting's folder; a file absent is a part absent, never an error.
    pub fn read(folder: &Path) -> io::Result<Self> {
        let mut meeting = Meeting::default();
        if let Some(session) = optional(&folder.join("session.json"))? {
            let session: Value = serde_json::from_str(&session).map_err(io::Error::other)?;
            meeting.started = session["started"].as_str().map(str::to_string);
        }
        if let Some(minutes) = optional(&folder.join("minutes.md"))? {
            read_minutes(&minutes, &mut meeting);
        }
        if let Some(transcript) = optional(&folder.join("transcript.jsonl"))? {
            read_transcript(&transcript, &mut meeting);
        }
        Ok(meeting)
    }
}

fn optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// `minutes`: a front matter of `key: value` lines, a value in double quotes read as JSON;
/// then sections of second level, found by their place.
fn read_minutes(text: &str, meeting: &mut Meeting) {
    let mut lines = text.lines();
    if lines.next() == Some("---") {
        for line in lines.by_ref().take_while(|line| *line != "---") {
            let Some((key, value)) = line.split_once(':') else { continue };
            let value = value.trim();
            let value = if value.starts_with('"') {
                serde_json::from_str::<String>(value).ok()
            } else {
                Some(value.to_string())
            };
            let value = value.filter(|value| !value.is_empty() && value != "null");
            match key {
                "model" => meeting.summary_model = value,
                "title" => meeting.title = value.filter(|title| a_title(title)),
                _ => {}
            }
        }
    } else {
        lines = text.lines();
    }
    for line in lines {
        if let Some(title) = line.strip_prefix("## ") {
            meeting.sections.push(Section {
                title: title.trim().to_string(),
                body: String::new(),
            });
        } else if let Some(section) = meeting.sections.last_mut() {
            section.body.push_str(line);
            section.body.push('\n');
        }
    }
    for section in &mut meeting.sections {
        section.body = section.body.trim().to_string();
    }
}

/// `minutes`' bounds for a title: one line, not blank, no control character, 80 characters at most.
fn a_title(title: &str) -> bool {
    !title.trim().is_empty() && !title.chars().any(char::is_control) && title.chars().count() <= 80
}

/// `transcript/v1`: its header's model, its segments sorted by `start`, its `complete` duration.
fn read_transcript(text: &str, meeting: &mut Meeting) {
    for line in text.lines() {
        let Ok(line) = serde_json::from_str::<Value>(line) else { continue };
        match line["type"].as_str() {
            Some("header") => meeting.transcript_model = line["model"].as_str().map(str::to_string),
            Some("segment") => {
                let speaker = if line["speaker"] == "me" { Speaker::Me } else { Speaker::Others };
                if let (Some(start), Some(text)) = (line["start"].as_f64(), line["text"].as_str()) {
                    meeting.segments.push(Segment {
                        start,
                        speaker,
                        text: text.to_string(),
                    });
                }
            }
            Some("complete") => meeting.duration = line["duration"].as_f64(),
            _ => {}
        }
    }
    meeting.segments.sort_by(|a, b| a.start.total_cmp(&b.start));
}

/// The meeting as Markdown: CommonMark, a YAML front matter — title, date, duration, models —, the
/// summary's sections, then the transcript as a GFM table.
pub fn markdown(meeting: &Meeting, include: Include, labels: &Labels) -> String {
    let mut out = String::from("---\n");
    // Each value written as JSON: a JSON string, number or object is YAML too.
    if let Some(title) = &meeting.title {
        out += &format!("title: {}\n", Value::from(title.as_str()));
    }
    if let Some(started) = &meeting.started {
        out += &format!("date: {}\n", Value::from(started.as_str()));
    }
    if let Some(duration) = meeting.duration {
        out += &format!("duration: {}\n", Value::from(duration));
    }
    let mut models = serde_json::Map::new();
    if let Some(model) = &meeting.transcript_model {
        models.insert("transcript".into(), model.as_str().into());
    }
    if let Some(model) = &meeting.summary_model {
        models.insert("summary".into(), model.as_str().into());
    }
    if !models.is_empty() {
        out += &format!("models: {}\n", Value::Object(models));
    }
    out += "---\n";
    if let Some(title) = &meeting.title {
        out += &format!("\n# {}\n", escaped(title));
    }
    if include.summary {
        for section in &meeting.sections {
            out += &format!("\n## {}\n\n{}\n", section.title, section.body);
        }
    }
    if include.transcript && !meeting.segments.is_empty() {
        out += &format!("\n## {}\n\n", escaped(&labels.transcript));
        let mut head = vec![];
        if include.times {
            head.push(labels.time.as_str());
        }
        if include.speakers {
            head.push(labels.speaker.as_str());
        }
        head.push(labels.text.as_str());
        out += &row(head.iter().map(|cell| escaped(cell)));
        out += &row(head.iter().map(|_| "---".to_string()));
        for segment in &meeting.segments {
            let mut cells = vec![];
            if include.times {
                cells.push(time(segment.start));
            }
            if include.speakers {
                cells.push(escaped(match segment.speaker {
                    Speaker::Me => &labels.me,
                    Speaker::Others => &labels.others,
                }));
            }
            cells.push(escaped(&segment.text));
            out += &row(cells.into_iter());
        }
    }
    out
}

fn row(cells: impl Iterator<Item = String>) -> String {
    format!("| {} |\n", cells.collect::<Vec<_>>().join(" | "))
}

/// A time as the meeting's page says it: `mm:ss`, and `hh:mm:ss` once past an hour (R9).
fn time(seconds: f64) -> String {
    let whole = seconds.max(0.0) as u64;
    let (hours, minutes, seconds) = (whole / 3600, whole / 60 % 60, whole % 60);
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// Text said as it is, and left to edit: only what CommonMark and GFM could read as markup inside a
/// line — emphasis, code, a link, HTML, an entity, a strikethrough, a cell's end — is escaped.
fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '&' | '~') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The meeting as an HTML page: `lang` set, `name` its title when the summary gives none, real
/// headings, the summary's Markdown rendered, the transcript a table its columns headed. Written well
/// formed — every element closed, every attribute quoted —, so that an XML reader reads it too.
pub fn html(meeting: &Meeting, include: Include, labels: &Labels, lang: &str, name: &str) -> String {
    let title = text(meeting.title.as_deref().unwrap_or(name));
    let mut out = format!(
        "<!DOCTYPE html>\n<html lang=\"{}\">\n<head>\n<meta charset=\"utf-8\" />\n<title>{title}</title>\n</head>\n<body>\n<h1>{title}</h1>\n",
        text(lang)
    );
    if include.summary {
        for section in &meeting.sections {
            out += &format!("<h2>{}</h2>\n", text(&section.title));
            use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
            // A model's raw HTML is said as text, never run; its links and images are said as their
            // words, never followed nor fetched — as the shell says them; its headings come under
            // the section's own h2, so that the page keeps its outline.
            let events = pulldown_cmark::Parser::new(&section.body).filter_map(|event| match event {
                Event::Html(raw) | Event::InlineHtml(raw) => Some(Event::Text(raw)),
                Event::Start(Tag::Link { .. } | Tag::Image { .. }) | Event::End(TagEnd::Link | TagEnd::Image) => None,
                Event::Start(Tag::Heading { level, id, classes, attrs }) => Some(Event::Start(Tag::Heading {
                    level: level.max(HeadingLevel::H3),
                    id,
                    classes,
                    attrs,
                })),
                Event::End(TagEnd::Heading(level)) => Some(Event::End(TagEnd::Heading(level.max(HeadingLevel::H3)))),
                event => Some(event),
            });
            pulldown_cmark::html::push_html(&mut out, events);
        }
    }
    if include.transcript && !meeting.segments.is_empty() {
        out += &format!("<h2>{}</h2>\n<table>\n<thead>\n<tr>", text(&labels.transcript));
        let mut head = vec![];
        if include.times {
            head.push(&labels.time);
        }
        if include.speakers {
            head.push(&labels.speaker);
        }
        head.push(&labels.text);
        for column in head {
            out += &format!("<th scope=\"col\">{}</th>", text(column));
        }
        out += "</tr>\n</thead>\n<tbody>\n";
        for segment in &meeting.segments {
            out += "<tr>";
            if include.times {
                out += &format!("<td>{}</td>", time(segment.start));
            }
            if include.speakers {
                let who = match segment.speaker {
                    Speaker::Me => &labels.me,
                    Speaker::Others => &labels.others,
                };
                out += &format!("<td>{}</td>", text(who));
            }
            out += &format!("<td>{}</td></tr>\n", text(&segment.text));
        }
        out += "</tbody>\n</table>\n";
    }
    out += "</body>\n</html>\n";
    out
}

/// Text in HTML, as said: the five characters that could be markup written as entities.
fn text(said: &str) -> String {
    said.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
