//! D118: a confidential meeting is sent only when the shell says the person confirmed it, for this send
//! (`posting/v1`'s `confirmed_confidential`, PDR-0007 point 4 amended 2026-11-24). Against a server on this
//! computer, reached through the library with a policy of the test's: unconfirmed — the field absent or
//! false — nothing is read to be sent and the server hears nothing; confirmed, one request and never a second.
//! A meeting that is not confidential is read the same whatever the field says.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use kitokey_send::{Kind, Meeting, Outcome, Policy, Unsent, body, post};
use url::Url;

const MINUTES: &str = "---\ncontract: minutes/v4\ntitle: \"Point\"\n---\n\n## Résumé\n\nFixé.\n\n## Actions\n\n- a — b — c\n";

/// A server that answers 204 to each request, and tells each request it heard.
fn server() -> (Url, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tell, heard) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let (mut request, mut length) = (String::new(), 0);
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
            let mut rest = vec![0; length];
            reader.read_exact(&mut rest).ok();
            tell.send(request).ok();
            stream.write_all(b"HTTP/1.1 204 No Content\r\n\r\n").ok();
        }
    });
    (Url::parse(&format!("http://127.0.0.1:{port}/hook")).unwrap(), heard)
}

fn here() -> Policy {
    Policy {
        reachable: |_: IpAddr| true,
        https_only: false,
        bound: Duration::from_secs(1),
    }
}

/// A meeting's folder, said confidential or not, sent through the library as the program sends it, the
/// confirmation as the input carries it: what came of it, and how many requests the server heard.
fn sent(confidential: bool, confirmed: bool) -> (Result<Outcome, Unsent>, usize) {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("minutes.md"), MINUTES).unwrap();
    if confidential {
        fs::write(folder.path().join("confidential"), "").unwrap();
    }
    let (url, heard) = server();
    let outcome = Meeting::read_confirmed(folder.path(), confirmed).map(|meeting| {
        let body = body(Kind::Webhook, &meeting, "Point", "2026-11-20_09-30").unwrap();
        post(&url, &body, here())
    });
    thread::sleep(Duration::from_millis(300));
    (outcome, heard.try_iter().count())
}

#[test]
fn a_confidential_meeting_unconfirmed_is_refused_and_nothing_is_heard() {
    assert_eq!(sent(true, false), (Err(Unsent::Confidential), 0));
    // The field absent is `false`: the reading the program does without it.
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("minutes.md"), MINUTES).unwrap();
    fs::write(folder.path().join("confidential"), "").unwrap();
    assert_eq!(Meeting::read(folder.path()).err(), Some(Unsent::Confidential));
}

#[test]
fn a_confidential_meeting_confirmed_is_sent_once_and_never_a_second_time() {
    assert_eq!(sent(true, true), (Ok(Outcome::Sent), 1));
}

#[test]
fn a_meeting_not_confidential_is_sent_as_before_whatever_the_field_says() {
    assert_eq!(sent(false, false), (Ok(Outcome::Sent), 1));
    assert_eq!(sent(false, true), (Ok(Outcome::Sent), 1));
}
