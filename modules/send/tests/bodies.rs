//! posting/v1's *What it sends*: a meeting read from its folder — confidential first, minutes v1 to v5,
//! never the transcript —, and what each kind receives, within its bounds.

use std::fs;
use std::path::Path;

use kitokey_send::{DISCORD_CONTENT, DISCORD_FILE, Kind, Meeting, SLACK_TEXT, Unsent, body, mrkdwn};
use serde_json::Value;

const V4: &str = "---\ncontract: minutes/v4\nmade: 2026-11-20T10:00:00+01:00\nprovider: local\nmodel: m\n\
context: null\ntitle: \"Point hebdo\"\n---\n\n## Résumé\n\nLe lancement est fixé.\n\n## Actions\n\n\
- Écrire la note — Léa — vendredi [12:04]\n- Relancer le client — non précisé — non précisé\n\n\
## Points clés\n\n- Budget : 12 k€\n";

const V5: &str = "---\ncontract: minutes/v5\nmade: 2026-11-20T10:00:00+01:00\nprovider: local\nmodel: m\n\
context: null\nstructure: client_call\nsections: [{\"id\": \"needs\", \"about\": \"a\"}, {\"id\": \"questions\", \
\"about\": \"b\"}, {\"id\": \"next_steps\", \"about\": \"c\"}]\nactions: next_steps\n---\n\n## Besoins\n\nUn outil.\n\n\
## Questions\n\n- Le prix ? — 10 €\n\n## Prochaines étapes\n\n- Envoyer l'offre — Marc — lundi\n";

fn folder(files: &[(&str, &str)]) -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    for (name, text) in files {
        fs::write(folder.path().join(name), text).unwrap();
    }
    folder
}

fn read(files: &[(&str, &str)]) -> Result<Meeting, Unsent> {
    Meeting::read(folder(files).path())
}

#[test]
fn a_confidential_meeting_is_refused_first_and_a_meeting_without_minutes_has_no_summary() {
    assert_eq!(read(&[("minutes.md", V4), ("confidential", "")]), Err(Unsent::Confidential));
    assert_eq!(read(&[("confidential", "anything")]), Err(Unsent::Confidential));
    assert_eq!(read(&[("transcript.jsonl", "{}\n")]), Err(Unsent::NoSummary));
    assert_eq!(
        read(&[("minutes.md", "---\ncontract: minutes/v4\n---\n\nnothing\n")]),
        Err(Unsent::NoSummary)
    );
    assert_eq!(Meeting::read(Path::new("/no/such/folder")), Err(Unsent::Unreadable));
}

#[test]
fn v4s_actions_are_its_second_section_and_v5s_the_one_it_names() {
    let v4 = read(&[("minutes.md", V4)]).unwrap();
    assert_eq!(v4.title.as_deref(), Some("Point hebdo"));
    assert_eq!(
        v4.action_items(),
        [
            "Écrire la note — Léa — vendredi [12:04]",
            "Relancer le client — non précisé — non précisé"
        ]
    );
    let v5 = read(&[("minutes.md", V5)]).unwrap();
    assert_eq!(v5.title, None);
    assert_eq!(v5.action_items(), ["Envoyer l'offre — Marc — lundi"]);
    // v5 naming no actions has none; naming an id it does not hold, none either.
    let none = V5.replace("actions: next_steps\n", "");
    assert_eq!(read(&[("minutes.md", &none)]).unwrap().action_items(), Vec::<String>::new());
    let wrong = V5.replace("actions: next_steps", "actions: todo");
    assert_eq!(read(&[("minutes.md", &wrong)]).unwrap().action_items(), Vec::<String>::new());
}

fn json_of(kind: Kind, meeting: &Meeting, title: &str) -> Value {
    serde_json::from_slice(&body(kind, meeting, title, "2026-11-20_09-30").unwrap().bytes).unwrap()
}

#[test]
fn a_generic_webhook_receives_the_title_the_sections_and_the_actions_and_nothing_else() {
    let transcript = "{\"type\":\"segment\",\"text\":\"un secret dit en réunion\"}\n";
    let meeting = Meeting::read(folder(&[("minutes.md", V4), ("transcript.jsonl", transcript)]).path()).unwrap();
    let sent = json_of(Kind::Webhook, &meeting, "Point hebdo");
    assert_eq!(
        sent.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["actions", "summary", "title"]
    );
    assert_eq!(sent["title"], "Point hebdo");
    let summary = sent["summary"].as_str().unwrap();
    assert!(
        summary.starts_with("## Résumé\n\nLe lancement est fixé.\n\n## Actions\n"),
        "{summary}"
    );
    assert_eq!(summary.matches("## ").count(), 3);
    for absent in ["un secret dit en réunion", "contract:", "provider", "---"] {
        assert!(!summary.contains(absent), "{absent}");
    }
    assert_eq!(sent["actions"].as_array().unwrap().len(), 2);
}

#[test]
fn discord_receives_a_short_message_its_mentions_off_and_the_whole_summary_as_a_file() {
    let meeting = read(&[("minutes.md", V4)]).unwrap();
    let sent = body(Kind::Discord, &meeting, "Point hebdo", "2026-11-20_09-30").unwrap();
    let boundary = sent.content_type.strip_prefix("multipart/form-data; boundary=").unwrap();
    let text = String::from_utf8(sent.bytes).unwrap();
    let parts: Vec<_> = text.split(&format!("--{boundary}")).collect();
    // Before the first boundary nothing, two parts, the closing boundary.
    assert_eq!((parts.len(), parts[0], parts[3]), (4, "", "--\r\n"));
    let payload: Value = serde_json::from_str(parts[1].split("\r\n\r\n").nth(1).unwrap().trim_end()).unwrap();
    assert_eq!(payload["allowed_mentions"], serde_json::json!({"parse": []}));
    assert!(payload["content"].as_str().unwrap().starts_with("**Point hebdo**\n\n## Résumé"));
    assert!(parts[2].contains("name=\"files[0]\"; filename=\"2026-11-20_09-30.md\""));
    assert!(parts[2].contains("\r\n\r\n# Point hebdo\n\n## Résumé"));
    assert!(parts[2].contains("- Budget : 12 k€"));
}

#[test]
fn discords_message_is_cut_at_a_lines_end_within_2000_characters_the_file_whole() {
    let long = format!(
        "---\ncontract: minutes/v4\n---\n\n## A\n\n{}\n## B\n\nfin\n",
        "é ligne\n".repeat(600)
    );
    let meeting = read(&[("minutes.md", &long)]).unwrap();
    let sent = String::from_utf8(body(Kind::Discord, &meeting, "T", "f").unwrap().bytes).unwrap();
    let payload: Value = serde_json::from_str(sent.split("\r\n\r\n").nth(1).unwrap().split("\r\n--").next().unwrap()).unwrap();
    let content = payload["content"].as_str().unwrap();
    assert!(content.chars().count() <= DISCORD_CONTENT && content.chars().count() > DISCORD_CONTENT - 20);
    assert!(content.ends_with("é ligne"), "cut inside a line");
    assert!(sent.contains("\n## B\n\nfin\n"), "the file is whole");
}

#[test]
fn slack_receives_mrkdwn_its_mentions_escaped_within_40000_characters_or_nothing() {
    let meeting = read(&[("minutes.md", V4)]).unwrap();
    let text = json_of(Kind::Slack, &meeting, "Point <!channel> & co")["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(text.starts_with("*Point &lt;!channel&gt; &amp; co*\n\n*Résumé*\n"), "{text}");
    assert_eq!(
        mrkdwn("### Titre\n**gras** et [le site](https://example.com/a) et [x](javascript:alert(1))\n<!here>"),
        "*Titre*\n*gras* et <https://example.com/a|le site> et [x](javascript:alert(1))\n&lt;!here&gt;\n"
    );
    let long = format!("---\ncontract: minutes/v4\n---\n\n## A\n\n{}\n", "x".repeat(SLACK_TEXT));
    assert_eq!(body(Kind::Slack, &read(&[("minutes.md", &long)]).unwrap(), "T", "f"), None);
}

#[test]
fn a_summary_beyond_discords_20_mib_is_not_sent_rather_than_cut() {
    let big = format!("---\ncontract: minutes/v4\n---\n\n## A\n\n{}\n", "x".repeat(DISCORD_FILE));
    let meeting = read(&[("minutes.md", &big)]).unwrap();
    assert_eq!(body(Kind::Discord, &meeting, "T", "f"), None);
    // Just within: sent, whole.
    let within = format!("---\ncontract: minutes/v4\n---\n\n## A\n\n{}\n", "x".repeat(DISCORD_FILE - 100));
    assert!(body(Kind::Discord, &read(&[("minutes.md", &within)]).unwrap(), "T", "f").is_some());
}

#[test]
fn discords_boundary_is_drawn_for_each_send_and_absent_from_the_meetings_text() {
    let meeting = read(&[("minutes.md", V4)]).unwrap();
    let boundary = |sent: &kitokey_send::Body| {
        sent.content_type
            .strip_prefix("multipart/form-data; boundary=")
            .unwrap()
            .to_string()
    };
    let first = body(Kind::Discord, &meeting, "T", "f").unwrap();
    let second = body(Kind::Discord, &meeting, "T", "f").unwrap();
    assert_ne!(boundary(&first), boundary(&second));
    // A summary holding the first boundary, on a line of its own as a lenient parser would read it.
    let forged = format!("---\ncontract: minutes/v4\n---\n\n## A\n\nx\n--{}\ny\n", boundary(&first));
    let sent = body(Kind::Discord, &read(&[("minutes.md", &forged)]).unwrap(), "T", "f").unwrap();
    let text = String::from_utf8(sent.bytes.clone()).unwrap();
    // Its own boundary opens two parts and closes them, and is nowhere in the text.
    assert_eq!(text.matches(&format!("--{}", boundary(&sent))).count(), 3);
}
