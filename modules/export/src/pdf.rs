//! The meeting as a tagged PDF (PDF/UA-1), written by typst as a library (D108, its spike on #129).
//!
//! Every word the meeting holds reaches typst as a string literal, never as markup: a summary or a
//! transcript can say `#`, `*` or `]` and stay what it says. The fonts are the caller's, in the order
//! it prefers them; this module reads none from the system.

use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

use crate::{Include, Labels, Meeting, Speaker, time};

/// The meeting as a PDF/UA-1 document: its title and language set, « heading 1 » the meeting, « heading
/// 2 » each section and the transcript, a model's own headings under them; the transcript a table its
/// head marked as such. As in the page, a model's raw HTML, links and images are said as their words.
pub fn pdf(meeting: &Meeting, include: Include, labels: &Labels, lang: &str, name: &str, fonts: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    let fonts: Vec<Font> = fonts.iter().flat_map(|font| Font::iter(Bytes::new(font.clone()))).collect();
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
