//! posting/v1's kind `notion` (D116) against a server on this computer standing for Notion's API, reached
//! through the library with a policy of the test's: the two reads, then one POST, never sent again; each of
//! Notion's answers its code; a read not answered nothing created, the POST not answered maybe created.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use kitokey_send::notion::{VERSION, check, send};
use kitokey_send::{Outcome, Policy};
use serde_json::{Value, json};
use url::Url;

const KEY: &str = "ntn_test-key";
const DATABASE: &str = "0123456789abcdef0123456789abcdef";
const SOURCE: &str = "fedcba98-7654-3210-fedc-ba9876543210";

const ONE_SOURCE: &str = r#"{"object":"database","data_sources":[{"id":"fedcba98-7654-3210-fedc-ba9876543210","name":"Notes"}]}"#;
const DESCRIBED: &str = r#"{"object":"data_source","properties":{"Date":{"id":"a","name":"Date","type":"date"},"Nom":{"id":"title","name":"Nom","type":"title"}}}"#;

/// What the server answers: each of the database's read, the data source's read and the POST, as a full
/// HTTP answer; none, never answered.
#[derive(Clone, Copy)]
struct Answers {
    database: Option<&'static str>,
    source: Option<&'static str>,
    page: Option<&'static str>,
}

fn ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn notion() -> Answers {
    Answers {
        database: Some(leak(ok(ONE_SOURCE))),
        source: Some(leak(ok(DESCRIBED))),
        page: Some(leak(ok(r#"{"object":"page","id":"p"}"#))),
    }
}

/// A server that answers each request as `answers` says, and sends each request it read, whole.
fn server(answers: Answers) -> (Url, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tell, heard) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let tell = tell.clone();
            thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                    request += &line;
                    if line == "\r\n" {
                        break;
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).ok();
                request += &String::from_utf8_lossy(&body);
                let answer = if request.starts_with("GET /v1/databases/") {
                    answers.database
                } else if request.starts_with("GET /v1/data_sources/") {
                    answers.source
                } else if request.starts_with("POST /v1/pages ") {
                    answers.page
                } else {
                    Some("HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n")
                };
                tell.send(request).ok();
                match answer {
                    Some(answer) => {
                        stream.write_all(answer.as_bytes()).ok();
                    }
                    None => thread::sleep(Duration::from_secs(5)),
                }
            });
        }
    });
    (Url::parse(&format!("http://127.0.0.1:{port}/")).unwrap(), heard)
}

/// The test's policy: this computer reachable, plain HTTP, a short bound.
fn here() -> Policy {
    Policy {
        reachable: |_: IpAddr| true,
        https_only: false,
        bound: Duration::from_secs(1),
    }
}

fn children() -> Vec<Value> {
    vec![json!({"object": "block", "type": "paragraph", "paragraph": {"rich_text": [{"type": "text", "text": {"content": "Fixé."}}]}})]
}

/// A send against `answers`: its outcome, and each request's first line, in the order heard.
fn sent(answers: Answers) -> (Outcome, Vec<String>, Vec<String>) {
    let (api, heard) = server(answers);
    let outcome = send(&api, KEY, DATABASE, "Point hebdo", children(), here());
    thread::sleep(Duration::from_millis(200));
    let requests: Vec<String> = heard.try_iter().collect();
    let lines = requests.iter().map(|r| r.lines().next().unwrap_or("").to_string()).collect();
    (outcome, lines, requests)
}

fn answer(status: &str, headers: &str) -> &'static str {
    leak(format!("HTTP/1.1 {status}\r\n{headers}Content-Length: 0\r\n\r\n"))
}

#[test]
fn a_send_reads_the_database_then_its_data_source_then_posts_one_page() {
    let (outcome, lines, requests) = sent(notion());
    assert_eq!(outcome, Outcome::Sent);
    assert_eq!(
        lines,
        [
            format!("GET /v1/databases/{DATABASE} HTTP/1.1"),
            format!("GET /v1/data_sources/{SOURCE} HTTP/1.1"),
            "POST /v1/pages HTTP/1.1".to_string(),
        ]
    );
    for request in &requests {
        let lower = request.to_ascii_lowercase();
        assert!(
            lower.contains(&format!("authorization: bearer {}", KEY.to_ascii_lowercase())),
            "{request}"
        );
        assert!(lower.contains(&format!("notion-version: {VERSION}")), "{request}");
    }
    let body: Value = serde_json::from_str(requests[2].split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["parent"], json!({"type": "data_source_id", "data_source_id": SOURCE}));
    assert_eq!(body["properties"]["Nom"]["title"][0]["text"]["content"], "Point hebdo");
    assert_eq!(body["children"], json!(children()));
}

#[test]
fn each_answer_to_the_post_becomes_its_code_and_the_post_is_heard_once() {
    let cases = [
        (answer("401 Unauthorized", ""), Outcome::KeyRefused, "key.refused"),
        (answer("403 Forbidden", ""), Outcome::KeyRestricted, "key.restricted"),
        (answer("404 Not Found", ""), Outcome::Unshared, "database.unshared"),
        (
            answer("429 Too Many Requests", "Retry-After: 7\r\n"),
            Outcome::Busy(Some(7)),
            "destination.busy",
        ),
        (answer("400 Bad Request", ""), Outcome::Declined(400), "destination.refused"),
        (
            answer("302 Found", "Location: http://127.0.0.1:1/\r\n"),
            Outcome::Declined(302),
            "destination.refused",
        ),
        (answer("502 Bad Gateway", ""), Outcome::Failed(502), "destination.failed"),
    ];
    for (page, outcome, code) in cases {
        let (got, lines, _) = sent(Answers {
            page: Some(page),
            ..notion()
        });
        assert_eq!((got, got.code()), (outcome, Some(code)), "{page}");
        assert_eq!(lines.iter().filter(|l| l.starts_with("POST")).count(), 1, "{page}: {lines:?}");
    }
}

#[test]
fn a_read_refused_ends_the_send_and_nothing_is_posted() {
    for (status, outcome) in [
        ("401 Unauthorized", Outcome::KeyRefused),
        ("404 Not Found", Outcome::Unshared),
        ("403 Forbidden", Outcome::KeyRestricted),
        ("503 Service Unavailable", Outcome::Failed(503)),
    ] {
        let (got, lines, _) = sent(Answers {
            database: Some(answer(status, "")),
            ..notion()
        });
        assert_eq!(got, outcome, "{status}");
        assert_eq!(lines.len(), 1, "{status}: {lines:?}");
        let (got, lines, _) = sent(Answers {
            source: Some(answer(status, "")),
            ..notion()
        });
        assert_eq!(got, outcome, "{status}, the data source's read");
        assert_eq!(lines.len(), 2, "{status}: {lines:?}");
    }
}

#[test]
fn a_database_of_several_data_sources_is_several_and_nothing_is_posted() {
    let several = r#"{"data_sources":[{"id":"a1","name":"A"},{"id":"b2","name":"B"}]}"#;
    let (got, lines, _) = sent(Answers {
        database: Some(leak(ok(several))),
        ..notion()
    });
    assert_eq!((got, got.code()), (Outcome::Several, Some("database.several")));
    assert_eq!(lines.len(), 1, "{lines:?}");
}

#[test]
fn a_read_not_answered_is_destination_failed_nothing_created_and_nothing_posted() {
    for answers in [
        Answers {
            database: None,
            ..notion()
        },
        Answers { source: None, ..notion() },
    ] {
        let (got, lines, _) = sent(answers);
        assert_eq!((got, got.code()), (Outcome::Unanswered, Some("destination.failed")));
        assert!(!lines.iter().any(|l| l.starts_with("POST")), "{lines:?}");
    }
}

#[test]
fn the_post_not_answered_is_silent_maybe_created_and_never_sent_again() {
    let (got, lines, _) = sent(Answers { page: None, ..notion() });
    assert_eq!((got, got.code()), (Outcome::Silent, Some("destination.silent")));
    thread::sleep(Duration::from_millis(1500));
    assert_eq!(lines.iter().filter(|l| l.starts_with("POST")).count(), 1, "{lines:?}");
}

#[test]
fn a_page_past_500_kb_once_its_title_property_is_read_is_too_large_and_nothing_is_posted() {
    let (api, heard) = server(notion());
    let big = "x".repeat(2000);
    let block = json!({"object": "block", "type": "paragraph", "paragraph": {"rich_text": vec![json!({"type": "text", "text": {"content": big}}); 3]}});
    let outcome = send(&api, KEY, DATABASE, "Point", vec![block; 90], here());
    thread::sleep(Duration::from_millis(200));
    let lines: Vec<String> = heard.try_iter().map(|r| r.lines().next().unwrap_or("").to_string()).collect();
    assert_eq!((outcome, outcome.code()), (Outcome::TooLarge, Some("summary.too_large")));
    assert_eq!(lines.len(), 2, "{lines:?}");
}

#[test]
fn a_check_makes_the_two_reads_alone() {
    let (api, heard) = server(notion());
    let found = check(&api, KEY, DATABASE, here());
    thread::sleep(Duration::from_millis(200));
    assert_eq!(found, Ok((SOURCE.to_string(), "Nom".to_string())));
    let lines: Vec<String> = heard.try_iter().map(|r| r.lines().next().unwrap_or("").to_string()).collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines.iter().all(|l| l.starts_with("GET")), "{lines:?}");
}

#[test]
fn the_programs_policy_refuses_this_computer_and_reads_nothing() {
    let (mut api, heard) = server(notion());
    api.set_scheme("https").unwrap();
    let found = check(&api, KEY, DATABASE, Policy::strict());
    thread::sleep(Duration::from_millis(200));
    assert_eq!(found, Err(Outcome::Refused));
    assert_eq!(heard.try_iter().count(), 0);
}
