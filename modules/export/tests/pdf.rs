//! D108's PDF: written by typst as a library, tagged PDF/UA-1, its language and title set; each script
//! the spike recorded — Latin with U+00A0 and U+202F, Cyrillic, CJK, emoji — read back by a PDF text
//! extractor (pdf-extract), its structure by a PDF reader (lopdf). The fonts are the test's own: static
//! subsets of Noto, in `tests/fonts/`.

use std::fs;
use std::sync::Arc;

use kitokey_export::{Face, FontInfo, Include, Labels, Meeting, chosen, pdf};
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

/// A test font, as one face.
fn face(name: &str) -> Face {
    Face {
        data: Arc::new(fs::read(format!("{}/tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()),
        index: 0,
    }
}

fn fonts() -> Vec<Face> {
    [
        "NotoSans-Regular-subset.ttf",
        "NotoSans-Bold-subset.ttf",
        "NotoSansSC-Regular-subset.otf",
        "NotoSansSC-Bold-subset.otf",
        "NotoColorEmoji-subset.ttf",
    ]
    .into_iter()
    .map(face)
    .collect()
}

/// The meeting of these tests, or one of a summary alone.
fn meeting(minutes: &str, transcript: &str) -> Meeting {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("session.json"), SESSION).unwrap();
    fs::write(dir.path().join("minutes.md"), minutes).unwrap();
    fs::write(dir.path().join("transcript.jsonl"), transcript).unwrap();
    Meeting::read(dir.path()).unwrap()
}

fn written(include: Include) -> Vec<u8> {
    pdf(
        &meeting(MINUTES, TRANSCRIPT),
        include,
        &labels(),
        "fr",
        "Réunion du 20 nov. 2026",
        &fonts(),
    )
    .expect("a PDF")
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

const VARIABLE: &str = "NotoSansSC-VF-subset.otf";

fn choose(meeting: &Meeting, names: &[&str], sans: Option<&str>) -> Result<Vec<String>, char> {
    let faces: Vec<FontInfo> = names
        .iter()
        .map(|name| FontInfo::new(&fs::read(format!("{}/tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap(), 0).unwrap())
        .collect();
    chosen(meeting, Include::all(), &labels(), "Réunion du 20 nov. 2026", &faces, sans)
        .map(|taken| taken.into_iter().map(|i| names[i].to_string()).collect())
}

/// Two families with as many of the missing letters: the one given first is taken, and the other, left with
/// nothing to add, is not (`chosen()`'s tie, lina's B2 on #1279). Given in either order, the first wins; the
/// meeting's CJK and emoji are their own faces'.
#[test]
fn of_two_families_with_as_many_letters_the_first_given_is_taken() {
    let read = |name: &str| FontInfo::new(&fs::read(format!("{}/tests/fonts/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap(), 0).unwrap();
    let mut twin = read("NotoSans-Regular-subset.ttf");
    twin.family = "Noto Sans Twin".into();
    let regular = read("NotoSans-Regular-subset.ttf");
    let others = [read("NotoSansSC-Regular-subset.otf"), read("NotoColorEmoji-subset.ttf")];
    let meeting = meeting(MINUTES, TRANSCRIPT);
    let taken = |first: &FontInfo, second: &FontInfo| {
        let faces = [first.clone(), second.clone(), others[0].clone(), others[1].clone()];
        chosen(&meeting, Include::all(), &labels(), "Réunion du 20 nov. 2026", &faces, None).unwrap()
    };
    for (first, second) in [(&twin, &regular), (&regular, &twin)] {
        let taken = taken(first, second);
        assert!(
            taken.contains(&0) && !taken.contains(&1),
            "{} given first, {} taken: {taken:?}",
            first.family,
            second.family
        );
    }
}

#[test]
fn static_faces_are_chosen_before_a_variable_one_of_the_same_letters() {
    let names = [
        VARIABLE,
        "NotoSans-Regular-subset.ttf",
        "NotoSans-Bold-subset.ttf",
        "NotoSansSC-Regular-subset.otf",
        "NotoSansSC-Bold-subset.otf",
        "NotoColorEmoji-subset.ttf",
    ];
    assert_eq!(
        choose(&meeting(MINUTES, TRANSCRIPT), &names, Some("Noto Sans")).unwrap(),
        &names[1..],
        "the sans-serif first, then the family that has the most of what is missing; the variable face left out"
    );
}

#[test]
fn a_letter_only_a_variable_face_has_is_refused_never_written_in_its_default_instance() {
    // The summary's CJK alone, which the variable subset has and no static face here: refused by its first
    // letter, never written Thin (the decider, 2026-11-25).
    let names = ["NotoSans-Regular-subset.ttf", "NotoSans-Bold-subset.ttf", VARIABLE];
    let only_latin_and_these = meeting(MINUTES, "");
    let refused = choose(&only_latin_and_these, &names, Some("Noto Sans")).unwrap_err();
    assert!(('\u{4e00}'..='\u{9fff}').contains(&refused), "a CJK letter: {refused:?}");
}

#[test]
fn a_letter_no_face_has_is_refused_by_that_letter() {
    let names = [
        "NotoSans-Regular-subset.ttf",
        "NotoSans-Bold-subset.ttf",
        VARIABLE,
        "NotoColorEmoji-subset.ttf",
    ];
    // 下 is the first, by its code, of the transcript's CJK that no face here has.
    assert_eq!(choose(&meeting(MINUTES, TRANSCRIPT), &names, None), Err('下'));
}

#[test]
fn a_symbol_no_face_has_is_no_refusal_and_a_family_nothing_needs_is_not_chosen() {
    // No emoji face: 👍 and 🎉 are no letters, the PDF is written. A summary in Latin alone: no CJK face.
    let names = [
        "NotoSans-Regular-subset.ttf",
        "NotoSans-Bold-subset.ttf",
        "NotoSansSC-Regular-subset.otf",
    ];
    assert_eq!(choose(&meeting(MINUTES, TRANSCRIPT), &names, None).unwrap(), names);
    let latin = MINUTES.replace("- 会议纪要\n", "- Fini\n");
    assert_eq!(choose(&meeting(&latin, ""), &names, None).unwrap(), &names[..2],);
}

#[test]
fn the_systems_sans_serif_comes_first_when_it_has_any_of_the_letters() {
    let names = [
        "NotoSans-Regular-subset.ttf",
        "NotoSans-Bold-subset.ttf",
        "NotoSansSC-Regular-subset.otf",
        "NotoColorEmoji-subset.ttf",
    ];
    assert_eq!(
        choose(&meeting(MINUTES, TRANSCRIPT), &names, Some("Noto Color Emoji")).unwrap(),
        [
            "NotoColorEmoji-subset.ttf",
            "NotoSans-Regular-subset.ttf",
            "NotoSans-Bold-subset.ttf",
            "NotoSansSC-Regular-subset.otf"
        ]
    );
    // A sans-serif the system names but has not, or which has none of the letters: the rule of the most.
    assert_eq!(
        choose(&meeting(MINUTES, TRANSCRIPT), &names, Some("Cantarell")).unwrap()[0],
        names[0]
    );
}
