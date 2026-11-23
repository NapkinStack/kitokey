//! posting/v1's kind `notion` (D116): what Notion receives of a meeting — the database a link names, the
//! summary as blocks, the page that holds them —, within Notion's documented limits, read 2026-11-23
//! (developers.notion.com: *Request limits*, *Block*, *Rich text*, *Create a page*). What does not fit one
//! request is none: `summary.too_large`, nothing created.

use serde_json::{Value, json};

use crate::Meeting;

/// The API's version this program speaks, pinned by `posting/v1`.
pub const VERSION: &str = "2026-03-11";
/// A rich text's `content`, in characters.
pub const TEXT: usize = 2000;
/// Rich texts in one block's `rich_text`.
pub const RICH_TEXTS: usize = 100;
/// Blocks in one request's `children`.
pub const CHILDREN: usize = 100;
/// One request's body, in bytes: Notion's « 500KB », read as 500 000.
pub const PAYLOAD: usize = 500_000;

/// The database a link names: its last 32 hexadecimal characters before any `?`, hyphens left out — a link
/// Notion copies holds the database's title before them; a UUID with its hyphens gives the same 32. None for
/// a link that is not `https:` or holds no such 32.
pub fn database(link: &str) -> Option<String> {
    let url = url::Url::parse(link).ok()?;
    if url.scheme() != "https" {
        return None;
    }
    let path: Vec<char> = url.path().trim_end_matches('/').chars().filter(|c| *c != '-').collect();
    let id: String = path[path.len().saturating_sub(32)..].iter().collect();
    (id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit())).then(|| id.to_ascii_lowercase())
}

/// A run of text and how it is marked.
#[derive(Default, Clone)]
struct Span {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
    link: Option<String>,
}

/// One line of Markdown as runs: `` `code` ``, `**bold**`, `*italic*`, `[text](url)` for an `http:` or
/// `https:` url; anything else as written.
fn spans(line: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut rest = line;
    let flush = |plain: &mut String, out: &mut Vec<Span>| {
        if !plain.is_empty() {
            out.push(Span {
                text: std::mem::take(plain),
                ..Span::default()
            });
        }
    };
    while let Some(c) = rest.chars().next() {
        let marked = |open: &str, close: &str| -> Option<(String, usize)> {
            let inner = rest.strip_prefix(open)?;
            let end = inner.find(close).filter(|end| *end > 0 && !inner.starts_with(' '))?;
            Some((inner[..end].to_string(), open.len() + end + close.len()))
        };
        if let Some((text, len)) = marked("`", "`") {
            flush(&mut plain, &mut out);
            out.push(Span {
                text,
                code: true,
                ..Span::default()
            });
            rest = &rest[len..];
        } else if let Some((text, len)) = marked("**", "**") {
            flush(&mut plain, &mut out);
            out.push(Span {
                text,
                bold: true,
                ..Span::default()
            });
            rest = &rest[len..];
        } else if let Some((text, len)) = marked("*", "*") {
            flush(&mut plain, &mut out);
            out.push(Span {
                text,
                italic: true,
                ..Span::default()
            });
            rest = &rest[len..];
        } else if let Some((text, url, len)) = link(rest) {
            flush(&mut plain, &mut out);
            out.push(Span {
                text,
                link: Some(url),
                ..Span::default()
            });
            rest = &rest[len..];
        } else {
            plain.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    flush(&mut plain, &mut out);
    out
}

/// `[text](url)` at the start of `rest`, for an `http:` or `https:` url: its text, its url, its length.
fn link(rest: &str) -> Option<(String, String, usize)> {
    let inner = rest.strip_prefix('[')?;
    let close = inner.find("](")?;
    let after = &inner[close + 2..];
    let end = after.find(')')?;
    let url = &after[..end];
    if close == 0 || !(url.starts_with("http://") || url.starts_with("https://")) || url.contains(' ') {
        return None;
    }
    Some((inner[..close].to_string(), url.to_string(), 1 + close + 2 + end + 1))
}

/// A line as Notion's rich texts, each `content` at most [`TEXT`] characters — a longer run split over
/// several —; none when they would be more than [`RICH_TEXTS`].
pub fn rich_text(line: &str) -> Option<Vec<Value>> {
    let mut out = Vec::new();
    for span in spans(line) {
        let chars: Vec<char> = span.text.chars().collect();
        for chunk in chars.chunks(TEXT) {
            let content: String = chunk.iter().collect();
            let link = span.link.as_ref().map(|url| json!({"url": url}));
            out.push(json!({
                "type": "text",
                "text": {"content": content, "link": link},
                "annotations": {"bold": span.bold, "italic": span.italic, "code": span.code},
            }));
        }
    }
    (out.len() <= RICH_TEXTS).then_some(out)
}

/// A block of `kind` holding `text`'s rich texts, and what else that kind asks.
fn block(kind: &str, text: &str, more: Value) -> Option<Value> {
    let mut inner = json!({"rich_text": rich_text(text)?});
    if let (Some(inner), Some(more)) = (inner.as_object_mut(), more.as_object()) {
        inner.extend(more.clone());
    }
    Some(json!({"object": "block", "type": kind, kind: inner}))
}

/// The summary as blocks, in order: each section's title a `heading_2`, then its body — `### ` a
/// `heading_3`, an item a `bulleted_list_item` (a `to_do`, unticked, in the actions' section), `1. ` a
/// `numbered_list_item`, `> ` a `quote`, a fenced block a `code` in `plain text`, `---` a `divider`, lines
/// together a `paragraph`. None when they are more than [`CHILDREN`], or a block holds too many rich texts.
pub fn blocks(meeting: &Meeting) -> Option<Vec<Value>> {
    let mut out = Vec::new();
    for (at, (title, body)) in meeting.sections.iter().enumerate() {
        out.push(block("heading_2", title, json!({}))?);
        let actions = meeting.actions == Some(at);
        let mut paragraph: Vec<&str> = Vec::new();
        let mut fence: Option<Vec<&str>> = None;
        let close = |paragraph: &mut Vec<&str>, out: &mut Vec<Value>| -> Option<()> {
            if !paragraph.is_empty() {
                out.push(block("paragraph", &paragraph.join("\n"), json!({}))?);
                paragraph.clear();
            }
            Some(())
        };
        for line in body.lines() {
            if let Some(code) = fence.as_mut() {
                if line.trim_start().starts_with("```") {
                    let text: String = code.join("\n");
                    let chars: Vec<char> = text.chars().collect();
                    let rich: Vec<Value> = chars
                        .chunks(TEXT)
                        .map(|chunk| json!({"type": "text", "text": {"content": chunk.iter().collect::<String>()}}))
                        .collect();
                    if rich.len() > RICH_TEXTS {
                        return None;
                    }
                    out.push(json!({"object": "block", "type": "code", "code": {"rich_text": rich, "language": "plain text"}}));
                    fence = None;
                } else {
                    code.push(line);
                }
                continue;
            }
            let trimmed = line.trim();
            let item = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* "));
            let numbered = trimmed
                .split_once(". ")
                .filter(|(number, _)| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
                .map(|(_, text)| text);
            let made = if trimmed.starts_with("```") {
                close(&mut paragraph, &mut out)?;
                fence = Some(Vec::new());
                continue;
            } else if trimmed.is_empty() {
                close(&mut paragraph, &mut out)?;
                continue;
            } else if let Some(heading) = trimmed.strip_prefix("### ").or_else(|| trimmed.strip_prefix("#### ")) {
                block("heading_3", heading, json!({}))
            } else if let Some(text) = item {
                if actions {
                    block("to_do", text, json!({"checked": false}))
                } else {
                    block("bulleted_list_item", text, json!({}))
                }
            } else if let Some(text) = numbered {
                block("numbered_list_item", text, json!({}))
            } else if let Some(text) = trimmed.strip_prefix("> ") {
                block("quote", text, json!({}))
            } else if trimmed == "---" || trimmed == "***" {
                Some(json!({"object": "block", "type": "divider", "divider": {}}))
            } else {
                paragraph.push(trimmed);
                continue;
            };
            close(&mut paragraph, &mut out)?;
            out.push(made?);
        }
        close(&mut paragraph, &mut out)?;
        if let Some(code) = fence {
            // A fence never closed: its lines as written, a paragraph.
            out.push(block("paragraph", &code.join("\n"), json!({}))?);
        }
    }
    (out.len() <= CHILDREN).then_some(out)
}

/// The request that creates the page: its parent the data source, `title_property` the meeting's title, its
/// `children` the summary's blocks; none when the whole is more than [`PAYLOAD`] bytes.
pub fn page(data_source: &str, title_property: &str, title: &str, children: Vec<Value>) -> Option<Vec<u8>> {
    let body = json!({
        "parent": {"type": "data_source_id", "data_source_id": data_source},
        "properties": {title_property: {"title": rich_text(title)?}},
        "children": children,
    })
    .to_string()
    .into_bytes();
    (body.len() <= PAYLOAD).then_some(body)
}
