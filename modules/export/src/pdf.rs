//! The meeting as a tagged PDF (PDF/UA-1), written by typst as a library (D108, its spike on #129).
//!
//! Every word the meeting holds reaches typst as a string literal, never as markup: a summary or a
//! transcript can say `#`, `*` or `]` and stay what it says. The fonts are the caller's, in the order
//! it prefers them; this module reads none from the system.

use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
use std::sync::Arc;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};

use typst::text::{Font, FontBook, FontFlags, FontInfo};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

use crate::{Include, Labels, Meeting, Speaker, time};

/// The meeting as a PDF/UA-1 document: its title and language set, « heading 1 » the meeting, « heading
/// 2 » each section and the transcript, a model's own headings under them; the transcript a table its
/// head marked as such. As in the page, a model's raw HTML, links and images are said as their words.
pub fn pdf(meeting: &Meeting, include: Include, labels: &Labels, lang: &str, name: &str, fonts: &[Face]) -> Result<Vec<u8>, String> {
    let fonts: Vec<Font> = fonts
        .iter()
        .filter_map(|face| Font::new(Bytes::new(Shared(face.data.clone())), face.index))
        .collect();
    let book = FontBook::from_fonts(&fonts);
    let mut families: Vec<String> = Vec::new();
    for font in &fonts {
        let family = font.info().family.clone();
        if !families.contains(&family) {
            families.push(family);
        }
    }
    let world = Paper {
        library: LazyHash::new(Library::default()),
        book: LazyHash::new(book),
        fonts,
        main: Source::new(
            FileId::new(RootedPath::new(
                VirtualRoot::Project,
                VirtualPath::new("/meeting.typ").map_err(|error| error.to_string())?,
            )),
            source(meeting, include, labels, lang, name, &families),
        ),
    };
    let document = typst::compile::<PagedDocument>(&world).output.map_err(said)?;
    let standards = PdfStandards::new(&[PdfStandard::Ua_1]).map_err(|error| error.message().to_string())?;
    typst_pdf::pdf(
        &document,
        &PdfOptions {
            standards,
            ..PdfOptions::default()
        },
    )
    .map_err(said)
}

/// A font file's bytes, shared: a system's font mapped once, a collection's faces reading the same bytes.
pub type Data = Arc<dyn AsRef<[u8]> + Send + Sync>;

/// One face of a font file: the file's bytes, and the face's place in it (0 but in a collection).
#[derive(Clone)]
pub struct Face {
    pub data: Data,
    pub index: u32,
}

struct Shared(Data);

impl AsRef<[u8]> for Shared {
    fn as_ref(&self) -> &[u8] {
        (*self.0).as_ref()
    }
}

/// The faces the meeting needs, among `faces` — what the system's are, read without keeping them —, by their place
/// in it: static faces alone. A variable face is never taken — its default instance can be its lightest weight
/// (Thin for Noto Sans CJK); a letter only a variable face has is no letter the program has (`exporting/v1`, the
/// decider on 2026-11-25).
/// The family `sans` names — the system's sans-serif — comes first when it has any; then, each time,
/// the family that has the most of what is still missing, the first given on a tie. Every face of a
/// family taken comes with it, its bold for the headings. A letter no face has is `Err`, that letter:
/// the PDF would show a box for a word. A symbol no face has — an emoji — is drawn as a box, the rest
/// written.
pub fn chosen(
    meeting: &Meeting,
    include: Include,
    labels: &Labels,
    name: &str,
    faces: &[FontInfo],
    sans: Option<&str>,
) -> Result<Vec<usize>, char> {
    let mut missing: Vec<char> = source(meeting, include, labels, "", name, &[])
        .chars()
        .filter(|&c| !invisible(c))
        .collect();
    missing.sort_unstable();
    missing.dedup();
    let mut taken: Vec<usize> = Vec::new();
    let fixed = |i: usize| !faces[i].flags.contains(FontFlags::VARIABLE);
    // The static faces' families, in the order first given.
    let mut families: Vec<&str> = Vec::new();
    for (i, info) in faces.iter().enumerate() {
        if fixed(i) && !families.contains(&info.family.as_str()) {
            families.push(&info.family);
        }
    }
    let of = |family: &str| -> Vec<usize> { (0..faces.len()).filter(|&i| faces[i].family == family && fixed(i)).collect() };
    let has = |family: &str, c: char| of(family).iter().any(|&i| faces[i].coverage.contains(c as u32));
    loop {
        let count = |family: &str| missing.iter().filter(|&&c| has(family, c)).count();
        let best = sans
            .filter(|&sans| taken.is_empty() && families.contains(&sans) && count(sans) > 0)
            .or_else(|| {
                families
                    .iter()
                    .copied()
                    .map(|family| (count(family), family))
                    .filter(|&(n, _)| n > 0)
                    .rev()
                    .max_by_key(|&(n, _)| n)
                    .map(|(_, family)| family)
            });
        let Some(family) = best else { break };
        missing.retain(|&c| !has(family, c));
        taken.extend(of(family));
        families.retain(|&f| f != family);
    }
    match missing.into_iter().find(|c| c.is_alphabetic()) {
        Some(letter) => Err(letter),
        None => Ok(taken),
    }
}

/// A character no face draws: white space, a control, a joiner or a variation selector.
fn invisible(c: char) -> bool {
    c.is_whitespace()
        || c.is_control()
        || matches!(c, '\u{ad}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{fe00}'..='\u{fe0f}' | '\u{feff}' | '\u{e0000}'..='\u{e0fff}')
}

fn said(errors: impl IntoIterator<Item = typst::diag::SourceDiagnostic>) -> String {
    errors
        .into_iter()
        .map(|error| error.message.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

/// The document as typst code: each word a string literal.
fn source(meeting: &Meeting, include: Include, labels: &Labels, lang: &str, name: &str, families: &[String]) -> String {
    let title = meeting.title.as_deref().unwrap_or(name);
    let families: Vec<String> = families.iter().map(|family| literal(family)).collect();
    let mut out = format!(
        "#set document(title: {})\n#set text(lang: {}, font: ({},))\n#heading(level: 1, {})\n",
        literal(title),
        literal(lang),
        families.join(", "),
        literal(title)
    );
    if include.summary {
        for section in &meeting.sections {
            out += &format!("#heading(level: 2, {})\n", literal(&section.title));
            out += &summary(&section.body);
        }
    }
    if include.transcript && !meeting.segments.is_empty() {
        out += &format!("#heading(level: 2, {})\n", literal(&labels.transcript));
        let mut head = vec![];
        if include.times {
            head.push(&labels.time);
        }
        if include.speakers {
            head.push(&labels.speaker);
        }
        head.push(&labels.text);
        let mut columns = vec!["auto"; head.len() - 1];
        columns.push("1fr");
        out += &format!("#table(columns: ({},), stroke: none, table.header(", columns.join(", "));
        for column in head {
            out += &format!("[#strong({})], ", literal(column));
        }
        out += "),\n";
        for segment in &meeting.segments {
            if include.times {
                out += &format!("[#{}], ", literal(&time(segment.start)));
            }
            if include.speakers {
                let who = match segment.speaker {
                    Speaker::Me => &labels.me,
                    Speaker::Others => &labels.others,
                };
                out += &format!("[#{}], ", literal(who));
            }
            out += &format!("[#{}],\n", literal(&segment.text));
        }
        out += ")\n";
    }
    out
}

/// A section's Markdown as typst code: paragraphs, emphasis, lists, quotes and code kept; a model's raw
/// HTML said as text; its links and images as their words; its headings under the section's own.
fn summary(markdown: &str) -> String {
    let mut out = String::new();
    let mut code: Option<String> = None;
    for event in pulldown_cmark::Parser::new(markdown) {
        if let Some(block) = &mut code {
            match event {
                Event::End(TagEnd::CodeBlock) => {
                    out += &format!("#raw(block: true, {})\n", literal(block));
                    code = None;
                }
                Event::Text(text) => block.push_str(&text),
                _ => {}
            }
            continue;
        }
        match event {
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => out += &format!("#{}", literal(&text)),
            Event::Code(code) => out += &format!("#raw({})", literal(&code)),
            Event::SoftBreak => out += "#\" \"",
            Event::HardBreak => out += "#linebreak()",
            Event::Rule => out += "\n#line(length: 100%)\n",
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => out += "\n\n",
            Event::Start(Tag::Heading { level, .. }) => {
                out += &format!("#heading(level: {})[", heading(level));
            }
            Event::End(TagEnd::Heading(_)) => out += "]\n",
            Event::Start(Tag::Strong) => out += "#strong[",
            Event::Start(Tag::Emphasis) => out += "#emph[",
            Event::Start(Tag::Strikethrough) => out += "#strike[",
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough) => out += "]",
            Event::Start(Tag::BlockQuote(_)) => out += "#quote(block: true)[",
            Event::End(TagEnd::BlockQuote(_)) => out += "]\n",
            Event::Start(Tag::CodeBlock(_)) => code = Some(String::new()),
            Event::Start(Tag::List(Some(_))) | Event::Start(Tag::List(None)) => out += "\n",
            Event::End(TagEnd::List(_)) => out += "\n",
            Event::Start(Tag::Item) => out += "#list.item[",
            Event::End(TagEnd::Item) => out += "]\n",
            _ => {}
        }
    }
    out
}

/// A model's heading comes under the section's « heading 2 »: level 3 at least.
fn heading(level: HeadingLevel) -> usize {
    (level as usize).max(3)
}

/// A string as a typst string literal: said as it is, never read as markup.
fn literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out += "\\\\",
            '"' => out += "\\\"",
            c if c.is_control() => out += &format!("\\u{{{:x}}}", c as u32),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What typst reads: the one source, the caller's fonts; no file, no package, no clock.
struct Paper {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl World for Paper {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn file(&self, _: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}
