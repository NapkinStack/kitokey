//! D132: posting/v1's `parts` — the person chooses what of a summary leaves: its summary, its actions, its
//! key points, its dates; the title always. Each kind receives the chosen parts alone; without `parts`, the
//! send is as before (`bodies.rs`, `notion.rs`). Never the audio, never the transcript, never a date's
//! passage, which is the transcript's words.

use std::fs;

use kitokey_send::notion::blocks;
use kitokey_send::{Chosen, Date, Kind, Meeting, Part, body};
use serde_json::{Value, json};

const V4: &str = "---\ncontract: minutes/v4\ntitle: \"Point hebdo\"\n---\n\n## Résumé\n\nLe lancement est fixé.\n\n\
## Actions\n\n- Écrire la note — Léa — vendredi [12:04]\n- Relancer le client — non précisé — non précisé\n\n\
## Points clés\n\n- Budget : 12 k€\n";

const GENERAL: &str = "---\ncontract: minutes/v5\nstructure: general\nsections: [{\"id\": \"summary\", \"about\": \"a\"}, \
{\"id\": \"actions\", \"about\": \"b\"}, {\"id\": \"key_points\", \"about\": \"c\"}]\nactions: actions\n---\n\n\
## Résumé\n\nFixé.\n\n## Actions\n\n- Écrire — Léa — lundi\n\n## Points clés\n\n- Prix : 10 €\n";

const CLIENT: &str = "---\ncontract: minutes/v5\nstructure: client_call\nsections: [{\"id\": \"needs\", \"about\": \"a\"}, \
{\"id\": \"questions\", \"about\": \"b\"}, {\"id\": \"next_steps\", \"about\": \"c\"}]\nactions: next_steps\n---\n\n\
## Besoins\n\nUn outil.\n\n## Questions\n\n- Le prix ? — 10 €\n\n## Prochaines étapes\n\n- Envoyer l'offre — Marc — lundi\n";

const PASSAGE: &str = "Léa : vendredi, sans faute, dit en réunion.";

fn read(minutes: &str) -> Meeting {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("minutes.md"), minutes).unwrap();
    fs::write(folder.path().join("transcript.jsonl"), format!("{{\"text\":\"{PASSAGE}\"}}\n")).unwrap();
    Meeting::read(folder.path()).unwrap()
}

fn dates() -> Vec<Date> {
    vec![
        Date {
            date: "2026-11-27".into(),
            what: "Envoyer le devis".into(),
            who: Some("Léa".into()),
        },
        Date {
            date: "2026-12-01".into(),
            what: "Relire la page".into(),
            who: None,
        },
    ]
}

fn chosen(parts: &[Part], dates: Option<Vec<Date>>) -> Chosen {
    Chosen {
        parts: parts.to_vec(),
        dates: dates.map(|dates| ("Dates".to_string(), dates)),
    }
}

fn titles(meeting: &Meeting) -> Vec<&str> {
    meeting.sections.iter().map(|(title, _)| title.as_str()).collect()
}

fn webhook(meeting: &Meeting) -> Value {
    serde_json::from_slice(&body(Kind::Webhook, meeting, "Point hebdo", "f").unwrap().bytes).unwrap()
}

fn text(kind: Kind, meeting: &Meeting) -> String {
    String::from_utf8(body(kind, meeting, "Point hebdo", "f").unwrap().bytes).unwrap()
}

#[test]
fn each_part_is_its_sections_v1_to_v4_by_their_place_and_v5_by_their_id() {
    use Part::*;
    let v4 = read(V4);
    assert_eq!(titles(&v4.only(&chosen(&[Summary], None))), ["Résumé"]);
    assert_eq!(titles(&v4.only(&chosen(&[Actions], None))), ["Actions"]);
    assert_eq!(titles(&v4.only(&chosen(&[KeyPoints], None))), ["Points clés"]);
    // In the meeting's order whatever the order chosen; the actions' section found again where it went.
    let two = v4.only(&chosen(&[KeyPoints, Actions], None));
    assert_eq!(titles(&two), ["Actions", "Points clés"]);
    assert_eq!(two.action_items().len(), 2);
    assert_eq!(v4.only(&chosen(&[KeyPoints], None)).action_items(), Vec::<String>::new());
    let general = read(GENERAL);
    assert_eq!(titles(&general.only(&chosen(&[KeyPoints], None))), ["Points clés"]);
    assert_eq!(titles(&general.only(&chosen(&[Summary], None))), ["Résumé"]);
    // A structure without key points: its summary is every section but the actions, its key points none.
    let client = read(CLIENT);
    assert_eq!(titles(&client.only(&chosen(&[Summary], None))), ["Besoins", "Questions"]);
    assert_eq!(titles(&client.only(&chosen(&[KeyPoints], None))), Vec::<&str>::new());
    assert_eq!(
        client.only(&chosen(&[Actions], None)).action_items(),
        ["Envoyer l'offre — Marc — lundi"]
    );
}

#[test]
fn the_dates_are_a_section_of_their_title_each_date_what_who_and_asked_but_absent_nothing() {
    let v4 = read(V4);
    let only = v4.only(&chosen(&[Part::Dates], Some(dates())));
    assert_eq!(
        only.sections,
        [(
            "Dates".to_string(),
            "- 2026-11-27 — Envoyer le devis — Léa\n- 2026-12-01 — Relire la page".to_string()
        )]
    );
    assert_eq!(titles(&v4.only(&chosen(&[Part::Dates], None))), Vec::<&str>::new());
    // Given but not chosen: not sent.
    assert_eq!(titles(&v4.only(&chosen(&[Part::Summary], Some(dates())))), ["Résumé"]);
}

#[test]
fn a_generic_webhook_receives_the_chosen_parts_its_actions_and_summary_empty_when_not_chosen() {
    use Part::*;
    let v4 = read(V4);
    let sent = webhook(&v4.only(&chosen(&[Actions, KeyPoints], None)));
    assert_eq!(
        sent.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["actions", "key_points", "summary", "title"]
    );
    assert_eq!(sent["title"], "Point hebdo");
    assert_eq!(sent["actions"].as_array().unwrap().len(), 2);
    assert_eq!(sent["key_points"], json!(["Budget : 12 k€"]));
    assert!(!sent["summary"].as_str().unwrap().contains("Le lancement"), "{sent}");
    let summary = webhook(&v4.only(&chosen(&[Summary], None)));
    assert_eq!(summary["actions"], json!([]));
    assert_eq!(summary["summary"], "## Résumé\n\nLe lancement est fixé.\n");
    assert!(summary.get("key_points").is_none() && summary.get("dates").is_none());
    let dated = webhook(&v4.only(&chosen(&[Dates], Some(dates()))));
    assert_eq!(
        dated["dates"],
        json!([
            {"date": "2026-11-27", "what": "Envoyer le devis", "who": "Léa"},
            {"date": "2026-12-01", "what": "Relire la page", "who": null}
        ])
    );
    assert_eq!(dated["actions"], json!([]));
    assert_eq!(webhook(&v4.only(&chosen(&[Dates], None)))["dates"], json!([]));
}

#[test]
fn discord_and_slack_receive_the_title_and_the_chosen_parts_alone() {
    let only = read(V4).only(&chosen(&[Part::KeyPoints], None));
    for kind in [Kind::Discord, Kind::Slack] {
        let sent = text(kind, &only);
        assert!(sent.contains("Point hebdo") && sent.contains("Budget : 12 k€"), "{kind:?}: {sent}");
        for absent in ["Résumé", "Le lancement", "Écrire la note", "Actions"] {
            assert!(!sent.contains(absent), "{kind:?} sent {absent}: {sent}");
        }
    }
}

#[test]
fn notion_receives_the_chosen_parts_as_blocks_the_dates_a_list() {
    let only = read(V4).only(&chosen(&[Part::Dates, Part::Actions], Some(dates())));
    let made = blocks(&only).unwrap();
    let kinds: Vec<_> = made.iter().map(|block| block["type"].as_str().unwrap()).collect();
    assert_eq!(
        kinds,
        [
            "heading_2",
            "to_do",
            "to_do",
            "heading_2",
            "bulleted_list_item",
            "bulleted_list_item"
        ]
    );
    let all = serde_json::to_string(&made).unwrap();
    assert!(
        all.contains("2026-11-27 — Envoyer le devis — Léa") && !all.contains("Résumé"),
        "{all}"
    );
}

#[test]
fn no_kind_receives_the_transcript_nor_a_dates_passage() {
    // The meeting's folder holds its transcript, and the date was given with its passage: neither leaves.
    let date = json!({"id": "a1", "date": "2026-11-27", "what": "Envoyer le devis", "passage": PASSAGE});
    let input = json!({"parts": ["summary", "actions", "key_points", "dates"], "dates": [date], "dates_title": "Dates"});
    let only = read(V4).only(&Chosen::read(&input).unwrap().unwrap());
    let sent = [
        text(Kind::Webhook, &only),
        text(Kind::Discord, &only),
        text(Kind::Slack, &only),
        serde_json::to_string(&blocks(&only).unwrap()).unwrap(),
    ];
    for each in sent {
        assert!(!each.contains("dit en réunion"), "{each}");
    }
}

#[test]
fn the_input_names_known_parts_once_each_and_the_dates_a_title_when_they_are_sent() {
    let read = |object: Value| Chosen::read(&object);
    let date = json!({"id": "a1", "date": "2026-11-27", "what": "Envoyer le devis", "who": "Léa", "passage": PASSAGE});
    assert_eq!(read(json!({})), Ok(None));
    assert_eq!(read(json!({"dates": [date]})), Ok(None), "dates given, no parts: sent as before");
    let got = read(json!({"parts": ["dates", "summary"], "dates": [date], "dates_title": "Dates"}))
        .unwrap()
        .unwrap();
    assert_eq!(got.parts, [Part::Dates, Part::Summary]);
    assert_eq!(got.dates, Some(("Dates".to_string(), dates()[..1].to_vec())));
    // Asked, absent: the part is empty, no title needed.
    assert_eq!(read(json!({"parts": ["dates"]})).unwrap().unwrap().dates, None);
    for (wrong, why) in [
        (json!({"parts": []}), "no part"),
        (json!({"parts": "summary"}), "no array"),
        (json!({"parts": ["title"]}), "the title is always sent, never a part"),
        (json!({"parts": ["summary", "summary"]}), "a part twice"),
        (json!({"parts": ["transcript"]}), "unknown"),
        (json!({"parts": ["dates"], "dates": [date]}), "dates without their title"),
        (json!({"parts": ["dates"], "dates": [date], "dates_title": ""}), "an empty title"),
        (json!({"parts": ["dates"], "dates": [date], "dates_title": "a\nb"}), "two lines"),
        (json!({"parts": ["dates"], "dates": []}), "no date"),
        (
            json!({"dates": [{"id": "a1", "date": "27/11/2026", "what": "x"}]}),
            "no calendar day",
        ),
        (
            json!({"dates": [{"id": "A 1", "date": "2026-11-27", "what": "x"}]}),
            "an id out of its shape",
        ),
        // One past input.schema.json's bounds, the id's 64 characters and the day's digits (sam's P6 on #1322).
        (
            json!({"dates": [{"id": "a".repeat(65), "date": "2026-11-27", "what": "x"}]}),
            "an id of 65 characters",
        ),
        (
            json!({"dates": [{"id": "a1", "date": "2026-ab-27", "what": "x"}]}),
            "a day of letters",
        ),
        (json!({"dates": [{"date": "2026-11-27", "what": "x"}]}), "no id"),
        (json!({"dates": [{"id": "a1", "date": "2026-11-27", "what": ""}]}), "no what"),
        (
            json!({"dates": [{"id": "a1", "date": "2026-11-27", "what": "x", "who": "a\rb"}]}),
            "who on two lines",
        ),
        (
            json!({"dates": [{"id": "a1", "date": "2026-11-27", "what": "x", "passage": ""}]}),
            "an empty passage",
        ),
    ] {
        assert!(read(wrong.clone()).is_err(), "{why}: {wrong}");
    }
}
