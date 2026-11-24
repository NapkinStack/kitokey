//! posting/v1's kind `notion` (D116), what Notion receives: the database a link names, the summary as
//! blocks, the page that holds them, each within Notion's documented limits (read 2026-11-23).

use kitokey_send::Meeting;
use kitokey_send::notion::{CHILDREN, PAYLOAD, RICH_TEXTS, TEXT, blocks, database, page, pieces, rich_text};
use serde_json::{Value, json};

const ID: &str = "0123456789abcdef0123456789abcdef";

fn meeting(sections: &[(&str, &str)], actions: Option<usize>) -> Meeting {
    Meeting {
        title: Some("Point hebdo".into()),
        sections: sections.iter().map(|(t, b)| (t.to_string(), b.to_string())).collect(),
        actions,
    }
}

fn kinds(blocks: &[Value]) -> Vec<&str> {
    blocks.iter().map(|b| b["type"].as_str().unwrap()).collect()
}

fn text(block: &Value) -> String {
    let kind = block["type"].as_str().unwrap();
    block[kind]["rich_text"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["text"]["content"].as_str().unwrap())
        .collect()
}

#[test]
fn the_database_is_a_copied_links_last_32_hexadecimal_characters() {
    for link in [
        format!("https://www.notion.so/team/Notes-{ID}?v=1a2b"),
        format!("https://www.notion.so/{ID}"),
        format!("https://www.notion.so/team/{ID}/"),
        "https://www.notion.so/01234567-89ab-cdef-0123-456789abcdef".to_string(),
        format!("https://www.notion.so/team/Cafe-{}", ID.to_uppercase()),
    ] {
        assert_eq!(database(&link).as_deref(), Some(ID), "{link}");
    }
    for link in [
        format!("http://www.notion.so/{ID}"),
        "https://www.notion.so/team/Notes".to_string(),
        "https://www.notion.so/team/Notes-0123456789abcdef0123456789abcdeg".to_string(),
        format!("https://www.notion.so/?p={ID}"),
        "not a link".to_string(),
    ] {
        assert_eq!(database(&link), None, "{link}");
    }
}

#[test]
fn each_section_and_each_line_becomes_its_block_and_each_action_an_unticked_to_do() {
    let m = meeting(
        &[
            (
                "Résumé",
                "Le lancement\nest fixé.\n\n### Détail\n\n> Une citation\n\n1. Premier\n2. Second\n\n---\n\n```\nlet x = 1;\n```",
            ),
            ("Actions", "- Écrire la note — Léa — vendredi\n* Relancer — Marc"),
            ("Points clés", "- Budget : 12 k€"),
        ],
        Some(1),
    );
    let out = blocks(&m).unwrap();
    assert_eq!(
        kinds(&out),
        [
            "heading_2",
            "paragraph",
            "heading_3",
            "quote",
            "numbered_list_item",
            "numbered_list_item",
            "divider",
            "code",
            "heading_2",
            "to_do",
            "to_do",
            "heading_2",
            "bulleted_list_item"
        ]
    );
    assert_eq!(text(&out[1]), "Le lancement\nest fixé.");
    assert_eq!(out[7]["code"]["language"], "plain text");
    assert_eq!(text(&out[7]), "let x = 1;");
    assert_eq!(out[9]["to_do"]["checked"], false);
    assert_eq!(text(&out[9]), "Écrire la note — Léa — vendredi");
    assert_eq!(text(&out[12]), "Budget : 12 k€");
}

#[test]
fn bold_italic_code_and_links_become_annotations_and_a_link() {
    let rich = rich_text("Un **gras**, un *penché*, du `code`, [un lien](https://example.com/a) et [pas un](ftp://x)").unwrap();
    let marked: Vec<(String, bool, bool, bool, Value)> = rich
        .iter()
        .map(|r| {
            (
                r["text"]["content"].as_str().unwrap().to_string(),
                r["annotations"]["bold"].as_bool().unwrap(),
                r["annotations"]["italic"].as_bool().unwrap(),
                r["annotations"]["code"].as_bool().unwrap(),
                r["text"]["link"].clone(),
            )
        })
        .collect();
    assert!(marked.contains(&("gras".into(), true, false, false, Value::Null)));
    assert!(marked.contains(&("penché".into(), false, true, false, Value::Null)));
    assert!(marked.contains(&("code".into(), false, false, true, Value::Null)));
    assert!(marked.contains(&("un lien".into(), false, false, false, json!({"url": "https://example.com/a"}))));
    let all: String = marked.iter().map(|m| m.0.as_str()).collect();
    assert!(all.ends_with(" et [pas un](ftp://x)"), "{all}");
}

#[test]
fn a_text_longer_than_2000_characters_is_split_and_too_many_rich_texts_is_none() {
    let long = "é".repeat(TEXT * 2 + 5);
    let rich = rich_text(&long).unwrap();
    let lengths: Vec<usize> = rich
        .iter()
        .map(|r| r["text"]["content"].as_str().unwrap().chars().count())
        .collect();
    assert_eq!(lengths, [TEXT, TEXT, 5]);
    assert!(rich_text(&"é".repeat(TEXT * RICH_TEXTS)).is_some());
    assert!(rich_text(&"é".repeat(TEXT * RICH_TEXTS + 1)).is_none());
}

#[test]
fn at_most_100_blocks_or_none() {
    let fits = "- x\n".repeat(CHILDREN - 1);
    assert_eq!(blocks(&meeting(&[("A", &fits)], None)).unwrap().len(), CHILDREN);
    let over = "- x\n".repeat(CHILDREN);
    assert!(blocks(&meeting(&[("A", &over)], None)).is_none());
}

#[test]
fn the_page_has_the_data_source_for_parent_the_title_in_its_title_property_and_at_most_500_kb() {
    let m = meeting(&[("Résumé", "Fixé.")], None);
    let body: Value = serde_json::from_slice(&page("ds-1", "Nom", "Point hebdo", blocks(&m).unwrap()).unwrap()).unwrap();
    assert_eq!(body["parent"], json!({"type": "data_source_id", "data_source_id": "ds-1"}));
    assert_eq!(body["properties"]["Nom"]["title"][0]["text"]["content"], "Point hebdo");
    assert_eq!(kinds(body["children"].as_array().unwrap()), ["heading_2", "paragraph"]);
    // Bodies near the bound: 60 paragraphs of 9 000 characters each fit the blocks, not the bytes.
    let big = vec![
        json!({"object": "block", "type": "paragraph", "paragraph": {"rich_text": [{"type": "text", "text": {"content": "x".repeat(9_000)}}]}});
        60
    ];
    assert!(page("ds-1", "Nom", "t", big).is_none());
    let small = vec![
        json!({"object": "block", "type": "paragraph", "paragraph": {"rich_text": [{"type": "text", "text": {"content": "x".repeat(PAYLOAD / 70)}}]}});
        60
    ];
    assert!(page("ds-1", "Nom", "t", small).is_some());
}

#[test]
fn a_piece_is_within_2000_utf16_units_so_within_2000_characters_too() {
    // One unit first, so that the bound falls inside an emoji's two units.
    let emoji = format!("a{}", "😀".repeat(TEXT));
    let parts = pieces(&emoji);
    assert!(parts.iter().all(|p| p.encode_utf16().count() <= TEXT), "a piece past 2000 units");
    assert_eq!(parts.len(), 3);
    assert_eq!(parts.concat(), emoji);
}

#[test]
fn a_numbered_action_is_a_to_do_too() {
    let out = blocks(&meeting(
        &[("Résumé", "Fixé."), ("Actions", "1. Écrire — Léa\n2. Relancer — Marc")],
        Some(1),
    ))
    .unwrap();
    assert_eq!(kinds(&out), ["heading_2", "paragraph", "heading_2", "to_do", "to_do"]);
}

#[test]
fn a_fence_never_closed_keeps_its_lines_as_a_paragraph() {
    let out = blocks(&meeting(&[("Résumé", "Avant.\n\n```\nune ligne\nune autre")], None)).unwrap();
    assert_eq!(kinds(&out), ["heading_2", "paragraph", "paragraph"]);
    assert_eq!(text(&out[2]), "une ligne\nune autre");
}

#[test]
fn a_code_block_past_100_pieces_is_none() {
    let fits = format!("```\n{}\n```", "x".repeat(TEXT * RICH_TEXTS));
    assert!(blocks(&meeting(&[("A", &fits)], None)).is_some());
    let over = format!("```\n{}\n```", "x".repeat(TEXT * RICH_TEXTS + 1));
    assert!(blocks(&meeting(&[("A", &over)], None)).is_none());
}
