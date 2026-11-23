//! D108's DOCX: Word's heading styles, the summary's Markdown as paragraphs and runs — its raw HTML,
//! links and images said as their words —, the transcript a table; read back by docx-rs's reader.

use std::fs;

use kitokey_export::{Include, Labels, Meeting, docx};
use serde_json::Value;

const SESSION: &str = r#"{"contract": "session-audio/v3", "started": "2026-11-20T09:30:00+01:00", "tracks": {}}"#;

const MINUTES: &str = "---\ncontract: minutes/v4\nmade: 2026-11-20T10:05:00+01:00\nprovider: \"local\"\nmodel: \"qwen3:8b\"\ncontext: null\ntitle: \"Prix & essai\"\n---\n\n## Points forts\n\nLe prix est **fixé**. <script>x</script> [voir](javascript:alert(1)) ![pixel](http://tracker.example/p.png)\n\n## Actions\n\n- Écrire la page — Sam — vendredi [00:04]\n\n## Points clés\n\n- 49 €\n\n# Annexe\n\ntexte\n";

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

/// The document as docx-rs reads it back, as JSON.
fn read(minutes: Option<&str>, include: Include) -> Value {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    if let Some(minutes) = minutes {
        fs::write(dir.path().join("minutes.md"), minutes).unwrap();
    }
    fs::write(dir.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    let bytes = docx(&Meeting::read(dir.path()).unwrap(), include, &labels(), "Réunion du 20 nov. 2026").unwrap();
    serde_json::from_str(&docx_rs::read_docx(&bytes).expect("a document Word's format reads").json()).unwrap()
}

fn runs(paragraph: &Value) -> Vec<(String, bool)> {
    paragraph["data"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["type"] == "run")
        .map(|run| {
            let text: String = run["data"]["children"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|t| t["data"]["text"].as_str())
                .collect();
            (text, run["data"]["runProperty"]["bold"] == true)
        })
        .collect()
}

fn text(paragraph: &Value) -> String {
    runs(paragraph).into_iter().map(|(t, _)| t).collect()
}

/// The paragraphs at the document's level, each with its style.
fn paragraphs(doc: &Value) -> Vec<(String, String)> {
    doc["document"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["type"] == "paragraph")
        .map(|p| (p["data"]["property"]["style"].as_str().unwrap_or("").to_string(), text(p)))
        .collect()
}

fn styled(doc: &Value, style: &str) -> Vec<String> {
    paragraphs(doc).into_iter().filter(|(s, _)| s == style).map(|(_, t)| t).collect()
}

/// The transcript's table, its cells' text row by row.
fn table(doc: &Value) -> Vec<Vec<String>> {
    let Some(table) = doc["document"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["type"] == "table")
    else {
        return vec![];
    };
    table["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row["data"]["cells"]
                .as_array()
                .unwrap()
                .iter()
                .map(|cell| cell["data"]["children"].as_array().unwrap().iter().map(text).collect())
                .collect()
        })
        .collect()
}

#[test]
fn a_document_holds_words_heading_styles_its_title_its_sections_and_its_transcript() {
    let doc = read(Some(MINUTES), Include::all());
    assert_eq!(styled(&doc, "Heading1"), ["Prix & essai"]);
    assert_eq!(
        styled(&doc, "Heading2"),
        ["Points forts", "Actions", "Points clés", "Transcription"]
    );
    assert_eq!(
        styled(&doc, "Heading3"),
        ["Annexe"],
        "a model's `#` under the section's own heading"
    );
    // Word knows them by their names: « heading 1 », « heading 2 », « heading 3 ».
    let styles = doc["styles"].to_string();
    for name in ["heading 1", "heading 2", "heading 3"] {
        assert!(styles.contains(&format!("\"{name}\"")), "{name} not defined");
    }
}

#[test]
fn the_summary_is_paragraphs_and_runs_its_raw_html_links_and_images_said_as_words() {
    let doc = read(Some(MINUTES), Include::all());
    let body: Vec<String> = paragraphs(&doc)
        .into_iter()
        .filter(|(s, _)| s.is_empty() || s == "ListBullet")
        .map(|(_, t)| t)
        .collect();
    assert!(
        body.iter().any(|t| t == "Le prix est fixé. <script>x</script> voir pixel"),
        "{body:?}"
    );
    assert!(body.iter().any(|t| t == "• Écrire la page — Sam — vendredi [00:04]"), "{body:?}");
    let all = doc.to_string();
    assert!(
        !all.contains("javascript:") && !all.contains("tracker.example") && !all.contains(r#""type":"hyperlink""#),
        "a target kept"
    );
    assert_eq!(
        doc["documentRels"]["hyperlinks"],
        serde_json::json!([]),
        "a link's relationship written"
    );
    let strong = doc["document"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["type"] == "paragraph")
        .flat_map(runs)
        .filter(|(_, bold)| *bold)
        .map(|(t, _)| t)
        .collect::<Vec<_>>();
    assert!(strong.contains(&"fixé".to_string()), "{strong:?}");
}

#[test]
fn the_transcript_is_a_table_its_head_bold_its_rows_in_the_meetings_order() {
    let doc = read(Some(MINUTES), Include::all());
    assert_eq!(
        table(&doc),
        [
            vec!["Temps", "Qui", "Texte"],
            vec!["00:01", "Moi", "Bonjour à tous."],
            vec!["00:04", "Les autres", "a|b <i>c</i> & d"]
        ]
    );
    // The head's runs bold, the rows' not: what tells a reader the head from the rows.
    let rows = doc["document"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["type"] == "table")
        .unwrap()["data"]["rows"]
        .clone();
    let bold: Vec<Vec<bool>> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row["data"]["cells"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|cell| {
                    cell["data"]["children"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .flat_map(runs)
                        .map(|(_, bold)| bold)
                })
                .collect()
        })
        .collect();
    assert_eq!(bold, [vec![true; 3], vec![false; 3], vec![false; 3]]);
}

#[test]
fn what_is_included_is_the_callers_choice_and_a_meeting_without_a_title_takes_the_callers_name() {
    let doc = read(
        None,
        Include {
            summary: false,
            transcript: true,
            speakers: false,
            times: false,
        },
    );
    assert_eq!(styled(&doc, "Heading1"), ["Réunion du 20 nov. 2026"]);
    assert_eq!(styled(&doc, "Heading2"), ["Transcription"]);
    assert_eq!(table(&doc), [vec!["Texte"], vec!["Bonjour à tous."], vec!["a|b <i>c</i> & d"]]);
    let doc = read(
        Some(MINUTES),
        Include {
            summary: true,
            transcript: false,
            speakers: true,
            times: true,
        },
    );
    assert!(table(&doc).is_empty(), "no transcript asked, a table written");
}

/// A part of the package as Word reads it: docx-rs's reader keeps neither a row's header mark nor a
/// style's outline level, so the test reads the XML itself.
fn part(minutes: Option<&str>, include: Include, name: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    if let Some(minutes) = minutes {
        fs::write(dir.path().join("minutes.md"), minutes).unwrap();
    }
    fs::write(dir.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    let bytes = docx(&Meeting::read(dir.path()).unwrap(), include, &labels(), "Réunion du 20 nov. 2026").unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    std::io::read_to_string(archive.by_name(name).unwrap()).unwrap()
}

#[test]
fn the_tables_head_is_declared_its_header_row_for_a_screen_reader_and_word_s_checker() {
    let document = part(Some(MINUTES), Include::all(), "word/document.xml");
    let rows: Vec<&str> = document.split("<w:tr>").skip(1).collect();
    assert_eq!(rows.len(), 3, "the head and two rows");
    assert!(
        rows[0].starts_with("<w:trPr><w:tblHeader />"),
        "the head's row is not declared a header: {}",
        &rows[0][..80.min(rows[0].len())]
    );
    assert!(
        rows[1..].iter().all(|row| !row.contains("w:tblHeader")),
        "only the head is a header row"
    );
}

#[test]
fn the_heading_styles_carry_their_outline_level_for_the_navigation_pane() {
    let styles = part(Some(MINUTES), Include::all(), "word/styles.xml");
    for (id, level) in [("Heading1", 0), ("Heading2", 1), ("Heading3", 2)] {
        let style = styles
            .split("<w:style ")
            .find(|s| s.contains(&format!("w:styleId=\"{id}\"")))
            .expect(id);
        let style = &style[..style.find("</w:style>").unwrap()];
        assert!(
            style.contains(&format!("<w:outlineLvl w:val=\"{level}\" />")),
            "{id} has no outline level {level}: {style}"
        );
    }
}
