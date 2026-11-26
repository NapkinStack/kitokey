//! D108's PDF: written by typst as a library, tagged PDF/UA-1, its language and title set; each script
//! the spike recorded — Latin with U+00A0 and U+202F, Cyrillic, CJK, emoji — read back by a PDF text
//! extractor (pdf-extract), its structure by a PDF reader (lopdf). The fonts are the test's own: static
//! subsets of Noto, in `tests/fonts/`.

use std::fs;

use kitokey_export::{Include, Labels, Meeting, pdf};
use lopdf::{Document, Object};

const SESSION: &str = r#"{"contract": "session-audio/v3", "started": "2026-11-20T09:30:00+01:00", "tracks": {}}"#;

const MINUTES: &str = "---\ncontract: minutes/v4\nmade: 2026-11-20T10:05:00+01:00\nprovider: \"local\"\nmodel: \"qwen3:8b\"\ncontext: null\ntitle: \"Prix & essai\"\n---\n\n## Points forts\n\nLe prix est **fixé**\u{a0}: 49\u{a0}€, «\u{202f}Kitokey\u{202f}». <script>x</script> [voir](javascript:alert(1)) ![pixel](http://tracker.example/p.png)\n\n# Annexe\n\n- Écrire la page — Sam — vendredi\n\n## Actions\n\n- 会议纪要\n";

const TRANSCRIPT: &str = concat!(
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":4.5,"end":6.0,"text":"Здравствуйте, коллеги."}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":1.0,"end":3.0,"text":"Bonjour à tous."}"#,
    "\n",
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":9.0,"end":11.0,"text":"价格确定，下周发布。"}"#,
    "\n",
    r#"{"type":"segment","speaker":"me","track":"local","voice":null,"start":12.0,"end":13.0,"text":"Parfait 👍 🎉"}"#,
    "\n",
    r#"{"type":"segment","speaker":"others","track":"remote","voice":null,"start":14.0,"end":15.0,"text":"5 $ * 2 = #10 [env.] _x_ \\ @ref <lab> \"q\""}"#,
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

fn fonts() -> Vec<Vec<u8>> {
    [
        "NotoSans-Regular-subset.ttf",
        "NotoSans-Bold-subset.ttf",
        "NotoSansSC-Regular-subset.otf",
        "NotoSansSC-Bold-subset.otf",
        "NotoColorEmoji-subset.ttf",
    ]
    .iter()
    .map(|name| fs::read(format!("{}/tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap())
    .collect()
}

fn written(include: Include) -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    fs::write(dir.path().join("minutes.md"), MINUTES).unwrap();
    fs::write(dir.path().join("transcript.jsonl"), TRANSCRIPT).unwrap();
    let meeting = Meeting::read(dir.path()).unwrap();
    pdf(&meeting, include, &labels(), "fr", "Réunion du 20 nov. 2026", &fonts()).expect("a PDF")
}

/// The text as an extractor reads it, its runs of white space — U+00A0 and U+202F among them — as one.
fn read(bytes: &[u8]) -> String {
    let text = pdf_extract::extract_text_from_mem(bytes).expect("a PDF an extractor reads");
    text.split(|c: char| c.is_whitespace() || c == '\u{a0}' || c == '\u{202f}')
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn each_script_the_spike_recorded_is_written_and_read_back() {
    let text = read(&written(Include::all()));
    for said in [
        "Prix & essai",
        "Points forts",
        "Le prix est fixé : 49 €, « Kitokey ».",
        "Annexe",
        "Écrire la page — Sam — vendredi",
        "会议纪要",
        "Transcription",
        "Bonjour à tous.",
        "Здравствуйте, коллеги.",
        "价格确定，下周发布。",
        "Parfait 👍 🎉",
        "5 $ * 2 = #10 [env.] _x_ \\ @ref <lab> \"q\"",
    ] {
        assert!(text.contains(said), "« {said} » not in: {text}");
    }
    let order: Vec<_> = ["Bonjour", "Здравствуйте", "价格确定", "Parfait"]
        .iter()
        .map(|s| text.find(s).unwrap())
        .collect();
    assert!(order.is_sorted(), "the transcript in the meeting's order: {text}");
}

#[test]
fn the_pdf_is_tagged_pdf_ua_its_language_and_its_title_set() {
    let bytes = written(Include::all());
    let document = Document::load_mem(&bytes).unwrap();
    let catalog = document.catalog().unwrap();
    assert!(catalog.has(b"StructTreeRoot"), "a structure tree: {catalog:?}");
    assert_eq!(catalog.get(b"Lang").unwrap().as_str().unwrap(), b"fr");
    let metadata = document
        .get_object(catalog.get(b"Metadata").unwrap().as_reference().unwrap())
        .unwrap();
    let Object::Stream(stream) = metadata else {
        panic!("a metadata stream")
    };
    let xmp = String::from_utf8_lossy(&stream.content);
    assert!(xmp.contains("<pdfuaid:part>1</pdfuaid:part>"), "PDF/UA-1 declared: {xmp}");
    assert!(xmp.contains("Prix &amp; essai"), "the title: {xmp}");

    let tags: Vec<String> = structure(&document);
    let headings: Vec<_> = tags.iter().filter(|tag| tag.starts_with('H') && tag.len() == 2).collect();
    assert_eq!(
        headings,
        ["H1", "H2", "H3", "H2", "H2"],
        "the model's « # » under its section: {tags:?}"
    );
    assert_eq!(tags.iter().filter(|tag| *tag == "TH").count(), 3, "the transcript's head: {tags:?}");
    assert!(tags.contains(&"THead".to_string()), "{tags:?}");
}

/// The structure tree's element types, in the order their objects were written.
fn structure(document: &Document) -> Vec<String> {
    let mut ids: Vec<_> = document.objects.keys().copied().collect();
    ids.sort();
    ids.iter()
        .filter_map(|id| document.objects[id].as_dict().ok())
        .filter(|dict| dict.has(b"S") && dict.has(b"P"))
        .filter_map(|dict| {
            dict.get(b"S")
                .ok()?
                .as_name()
                .ok()
                .map(|name| String::from_utf8_lossy(name).into_owned())
        })
        .collect()
}

#[test]
fn a_models_raw_html_links_and_images_are_said_as_their_words() {
    let bytes = written(Include::all());
    let text = read(&bytes);
    assert!(text.contains("<script>x</script> voir pixel"), "{text}");
    let document = Document::load_mem(&bytes).unwrap();
    for object in document.objects.values() {
        if let Ok(dict) = object.as_dict() {
            assert!(!dict.has(b"URI"), "no link followed: {dict:?}");
        }
    }
    assert!(!String::from_utf8_lossy(&bytes).contains("tracker.example"));
}

#[test]
fn what_is_included_is_the_callers_choice() {
    let mut include = Include::all();
    include.speakers = false;
    include.times = false;
    let text = read(&written(include));
    assert!(
        text.contains("Bonjour à tous.") && !text.contains("Moi") && !text.contains("00:01"),
        "{text}"
    );

    include.transcript = false;
    let text = read(&written(include));
    assert!(!text.contains("Bonjour") && !text.contains("Transcription"), "{text}");
    assert!(text.contains("Points forts"), "{text}");

    let mut include = Include::all();
    include.summary = false;
    let text = read(&written(include));
    assert!(
        !text.contains("Points forts") && text.contains("Moi") && text.contains("00:01"),
        "{text}"
    );
}

/// A tab or another control character no font draws — `transcript/v1` allows them in a line — never stops the PDF:
/// PDF/UA-1 refuses a character without its glyph, so a tab is written as a space and the others are left out,
/// in the title, the minutes and the transcript alike (sam's survivor on #1212).
#[test]
fn a_tab_or_a_control_character_is_written_as_a_space_or_left_out_never_refused() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    fs::write(
        dir.path().join("minutes.md"),
        "---\ncontract: minutes/v4\nmade: 2026-11-20T10:05:00+01:00\nprovider: \"local\"\nmodel: \"qwen3:8b\"\ncontext: null\ntitle: \"Prix\\tet\\u0007 essai\"\n---\n\n## Points\n\nLe\tprix\u{1b} fixé.\n\n```\nun\tdeux\n```\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("transcript.jsonl"),
        "{\"type\":\"segment\",\"speaker\":\"me\",\"track\":\"local\",\"voice\":null,\"start\":1.0,\"end\":2.0,\"text\":\"Bon\\tjour\\u0007 à\\u001b[31m tous.\"}\n",
    )
    .unwrap();
    let meeting = Meeting::read(dir.path()).unwrap();
    let bytes = pdf(&meeting, Include::all(), &labels(), "fr", "Réunion\tdu\u{7} 20", &fonts())
        .expect("a PDF, whatever control characters the meeting holds");
    let text = read(&bytes);
    for said in ["Réunion du 20", "Le prix fixé.", "un deux", "Bon jour à[31m tous."] {
        assert!(text.contains(said), "« {said} » not in: {text}");
    }
}
