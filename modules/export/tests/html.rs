//! D108's HTML: `lang` set, real headings, the summary's Markdown rendered and never its raw HTML,
//! the transcript a table with its column headers — read back by an XML reader, the page well formed.

use std::fs;

use kitokey_export::{Include, Labels, Meeting, html};
use quick_xml::Reader;
use quick_xml::events::Event;

const SESSION: &str = r#"{"contract": "session-audio/v3", "started": "2026-11-20T09:30:00+01:00", "tracks": {}}"#;

const MINUTES: &str = "---\ncontract: minutes/v4\nmade: 2026-11-20T10:05:00+01:00\nprovider: \"local\"\nmodel: \"qwen3:8b\"\ncontext: null\ntitle: \"Prix & essai\"\n---\n\n## Points forts\n\nLe prix est **fixé**. <script>alert(1)</script>\n\n## Actions\n\n- Écrire la page — Sam — vendredi [00:04]\n\n## Points clés\n\n- 49 €\n";

const TRANSCRIPT: &str = concat!(
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":4.5,"end":6.0,"text":"a|b <i>c</i> & d"}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":1.0,"end":3.0,"text":"Bonjour à tous."}"#,
    "\n",
);

fn labels() -> Labels {
    Labels {
        transcript: "Transcription".into(),
        me: "Moi".into(),
        others: "Les autres".into(),
        time: "Temps".into(),
        speaker: "Qui".into(),
        text: "Texte".into(),
    }
}

fn page(minutes: Option<&str>, include: Include) -> String {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    if let Some(minutes) = minutes {
        fs::write(dir.path().join("minutes.md"), minutes).unwrap();
    }
    fs::write(dir.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    html(
        &Meeting::read(dir.path()).unwrap(),
        include,
        &labels(),
        "fr",
        "Réunion du 20 nov. 2026",
    )
}

/// Every element as it opens, with its attributes and the text it holds directly; the page must
/// read whole as XML.
#[derive(Debug)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    text: String,
}

fn elements(page: &str) -> Vec<Element> {
    let body = page.strip_prefix("<!DOCTYPE html>\n").expect("the page opens on its doctype");
    let mut reader = Reader::from_str(body);
    let (mut out, mut open): (Vec<Element>, Vec<usize>) = (vec![], vec![]);
    loop {
        match reader
            .read_event()
            .unwrap_or_else(|error| panic!("not well formed: {error}\n{page}"))
        {
            Event::Start(start) | Event::Empty(start) => {
                let name = start.name().as_ref().to_string();
                let attributes = start
                    .attributes()
                    .map(|a| {
                        let a = a.unwrap();
                        (
                            a.key.as_ref().to_string(),
                            a.normalized_value(quick_xml::XmlVersion::Implicit1_0).unwrap().into_owned(),
                        )
                    })
                    .collect();
                out.push(Element {
                    name,
                    attributes,
                    text: String::new(),
                });
                open.push(out.len() - 1);
            }
            Event::End(_) => {
                open.pop();
            }
            Event::Text(text) => {
                if let Some(&at) = open.last() {
                    out[at].text.push_str(&text.html_content());
                }
            }
            // `&amp;`, `&lt;`, `&#160;`: an entity comes as an event of its own.
            Event::GeneralRef(entity) => {
                if let Some(&at) = open.last() {
                    let said = quick_xml::escape::unescape(&format!("&{};", entity.html_content()))
                        .unwrap()
                        .into_owned();
                    out[at].text.push_str(&said);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    // Void elements written as `<meta … />` open and close at once: nothing stays open but them.
    out
}

fn texts<'a>(all: &'a [Element], name: &str) -> Vec<&'a str> {
    all.iter().filter(|e| e.name == name).map(|e| e.text.as_str()).collect()
}

#[test]
fn a_page_says_its_language_and_its_title_and_its_headings_are_real() {
    let all = elements(&page(Some(MINUTES), Include::all()));
    assert_eq!(all[0].name, "html");
    assert_eq!(all[0].attributes, [("lang".to_string(), "fr".to_string())]);
    assert!(
        all.iter()
            .any(|e| e.name == "meta" && e.attributes == [("charset".to_string(), "utf-8".to_string())])
    );
    assert_eq!(texts(&all, "title"), ["Prix & essai"]);
    assert_eq!(texts(&all, "h1"), ["Prix & essai"]);
    assert_eq!(texts(&all, "h2"), ["Points forts", "Actions", "Points clés", "Transcription"]);
}

#[test]
fn the_summary_is_rendered_and_its_raw_html_is_said_never_run() {
    let all = elements(&page(Some(MINUTES), Include::all()));
    assert_eq!(texts(&all, "strong"), ["fixé"]);
    assert_eq!(texts(&all, "li"), ["Écrire la page — Sam — vendredi [00:04]", "49 €"]);
    assert!(!all.iter().any(|e| e.name == "script"), "a model's raw HTML became an element");
    assert!(
        all.iter().any(|e| e.text.contains("<script>alert(1)</script>")),
        "a model's raw HTML not said as text"
    );
}

#[test]
fn the_transcript_is_a_table_its_columns_headed_its_rows_in_the_meetings_order() {
    let all = elements(&page(Some(MINUTES), Include::all()));
    let heads: Vec<&Element> = all.iter().filter(|e| e.name == "th").collect();
    assert_eq!(heads.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), ["Temps", "Qui", "Texte"]);
    assert!(heads.iter().all(|e| e.attributes == [("scope".to_string(), "col".to_string())]));
    assert_eq!(
        texts(&all, "td"),
        ["00:01", "Moi", "Bonjour à tous.", "00:04", "Les autres", "a|b <i>c</i> & d"]
    );
    assert!(!all.iter().any(|e| e.name == "i"), "a transcript's text became markup");
}

#[test]
fn what_is_included_is_the_callers_choice_and_a_meeting_without_a_title_takes_the_callers_name() {
    let only = Include {
        summary: false,
        transcript: true,
        speakers: false,
        times: false,
    };
    let all = elements(&page(None, only));
    assert_eq!(texts(&all, "title"), ["Réunion du 20 nov. 2026"]);
    assert_eq!(texts(&all, "h1"), ["Réunion du 20 nov. 2026"]);
    assert_eq!(texts(&all, "h2"), ["Transcription"]);
    assert_eq!(texts(&all, "th"), ["Texte"]);
    let none = Include {
        summary: true,
        transcript: false,
        speakers: true,
        times: true,
    };
    let all = elements(&page(Some(MINUTES), none));
    assert!(!all.iter().any(|e| e.name == "table"), "no transcript asked, a table written");
}

#[test]
fn a_models_links_and_images_are_said_as_their_words_never_followed_nor_fetched() {
    // lina, verifying #1021: a `javascript:` link runs at a click and could send the meeting the page
    // shows; a remote image is fetched as the file opens. The shell already says both as their words.
    let minutes = MINUTES.replace(
        "Le prix est **fixé**.",
        "Le prix est **fixé**. [voir](javascript:alert(1)) ![pixel](http://tracker.example/p.png) <https://a.example>",
    );
    let page = page(Some(&minutes), Include::all());
    let all = elements(&page);
    assert!(
        !all.iter().any(|e| e.name == "a" || e.name == "img"),
        "a link or an image written:\n{page}"
    );
    assert!(
        !page.contains("javascript:") && !page.contains("tracker.example"),
        "a target kept:\n{page}"
    );
    let said: String = all.iter().filter(|e| e.name == "p").map(|e| e.text.as_str()).collect();
    assert!(
        said.contains("voir") && said.contains("pixel") && said.contains("https://a.example"),
        "their words lost: {said}"
    );
}

#[test]
fn a_heading_inside_a_section_stays_under_the_sections_own() {
    // The page's outline: h1 the meeting, h2 each section; a model's `#` comes below them. A `## ` line
    // opens a section, as `minutes` says; a `###` stays where it is.
    let minutes = MINUTES.replace("- 49 €\n", "- 49 €\n\n# Annexe\n\n### Détail\n\ntexte\n");
    let all = elements(&page(Some(&minutes), Include::all()));
    assert_eq!(texts(&all, "h1"), ["Prix & essai"]);
    assert_eq!(texts(&all, "h2"), ["Points forts", "Actions", "Points clés", "Transcription"]);
    assert_eq!(texts(&all, "h3"), ["Annexe", "Détail"]);
}
