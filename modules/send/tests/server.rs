//! The one POST against a server on this computer, reached through the library with a policy of the
//! test's: what each answer becomes, never a redirect or a proxy followed, never sent twice, and the
//! program's own policy refusing this computer however it is named.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpListener};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use kitokey_send::{Body, Outcome, Policy, judged, post};
use url::Url;

/// A server that answers each request with `answer` (none: never answers), and sends what it read.
fn server(answer: Option<&'static str>) -> (Url, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tell, heard) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
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
            tell.send(request).ok();
            match answer {
                Some(answer) => {
                    stream.write_all(answer.as_bytes()).ok();
                }
                None => thread::sleep(Duration::from_secs(5)),
            }
        }
    });
    (Url::parse(&format!("http://127.0.0.1:{port}/api/webhooks/1/token")).unwrap(), heard)
}

/// The test's policy: this computer reachable, plain HTTP, a short bound.
fn here() -> Policy {
    Policy {
        reachable: |_: IpAddr| true,
        https_only: false,
        bound: Duration::from_secs(1),
    }
}

fn json() -> Body {
    Body {
        content_type: "application/json".into(),
        bytes: br#"{"title":"t","summary":"","actions":[]}"#.to_vec(),
    }
}

/// What `answer` becomes, and how many requests the server heard.
fn sent(answer: &'static str) -> (Outcome, usize) {
    let (url, heard) = server(Some(answer));
    let outcome = post(&url, &json(), here());
    thread::sleep(Duration::from_millis(200));
    (outcome, heard.try_iter().count())
}

#[test]
fn each_answer_becomes_its_code_and_is_heard_once() {
    let cases = [
        ("HTTP/1.1 204 No Content\r\n\r\n", Outcome::Sent),
        ("HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\nok", Outcome::Sent),
        ("HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\n\r\n", Outcome::Gone),
        ("HTTP/1.1 403 Forbidden\r\ncontent-length: 0\r\n\r\n", Outcome::Gone),
        ("HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\n\r\n", Outcome::Gone),
        ("HTTP/1.1 410 Gone\r\ncontent-length: 0\r\n\r\n", Outcome::Gone),
        (
            "HTTP/1.1 429 Too Many Requests\r\nretry-after: 7\r\ncontent-length: 0\r\n\r\n",
            Outcome::Busy(Some(7)),
        ),
        ("HTTP/1.1 429 Too Many Requests\r\ncontent-length: 0\r\n\r\n", Outcome::Busy(None)),
        (
            "HTTP/1.1 400 Bad Request\r\ncontent-length: 15\r\n\r\ninvalid_payload",
            Outcome::Declined(400),
        ),
        (
            "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\n\r\n",
            Outcome::Failed(500),
        ),
        (
            "HTTP/1.1 503 Service Unavailable\r\ncontent-length: 0\r\n\r\n",
            Outcome::Failed(503),
        ),
    ];
    for (answer, outcome) in cases {
        assert_eq!(sent(answer), (outcome, 1), "{answer}");
    }
}

#[test]
fn a_redirect_is_never_followed_even_to_this_computer() {
    let (target, followed) = server(Some("HTTP/1.1 204 No Content\r\n\r\n"));
    let redirect = format!("HTTP/1.1 307 Temporary Redirect\r\nlocation: {target}\r\ncontent-length: 0\r\n\r\n");
    let (url, heard) = server(Some(Box::leak(redirect.into_boxed_str())));
    assert_eq!(post(&url, &json(), here()), Outcome::Declined(307));
    thread::sleep(Duration::from_millis(200));
    assert_eq!((heard.try_iter().count(), followed.try_iter().count()), (1, 0));
}

#[test]
fn no_answer_within_the_bound_is_silent_maybe_arrived_and_never_sent_again() {
    let (url, heard) = server(None);
    assert_eq!(post(&url, &json(), here()), Outcome::Silent);
    assert_eq!(heard.try_iter().count(), 1);
}

#[test]
fn nothing_listening_is_unreachable() {
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let url = Url::parse(&format!("http://127.0.0.1:{port}/hook")).unwrap();
    assert_eq!(post(&url, &json(), here()), Outcome::Unreachable);
}

#[test]
fn a_connection_not_made_within_the_bound_is_unreachable_as_nothing_left() {
    // A server that takes the TCP connection and never answers the TLS handshake: the request never leaves,
    // so the send is said « no connection », never « may have arrived » (#1167).
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tell, heard) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            tell.send(()).ok();
            thread::sleep(Duration::from_secs(5));
            drop(stream);
        }
    });
    let url = Url::parse(&format!("https://127.0.0.1:{port}/hook")).unwrap();
    assert_eq!(post(&url, &json(), here()), Outcome::Unreachable);
    assert_eq!(heard.try_iter().count(), 1);
}

#[test]
fn the_body_and_its_type_arrive_as_given() {
    let (url, heard) = server(Some("HTTP/1.1 204 No Content\r\n\r\n"));
    assert_eq!(post(&url, &json(), here()), Outcome::Sent);
    let request = heard.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(request.starts_with("POST /api/webhooks/1/token HTTP/1.1\r\n"), "{request}");
    assert!(
        request.to_ascii_lowercase().contains("content-type: application/json\r\n"),
        "{request}"
    );
    assert!(request.ends_with(r#"{"title":"t","summary":"","actions":[]}"#), "{request}");
}

#[test]
fn no_proxy_of_the_environment_is_followed() {
    // A proxy that would take the request: nothing listens there.
    let dead = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    for name in ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy"] {
        // SAFETY: set before the request, read by no other thread of this test binary that relies on it.
        unsafe { std::env::set_var(name, format!("http://127.0.0.1:{dead}")) };
    }
    // Nothing exempt from it, this computer included.
    for name in ["NO_PROXY", "no_proxy"] {
        // SAFETY: as above.
        unsafe { std::env::remove_var(name) };
    }
    let (url, heard) = server(Some("HTTP/1.1 204 No Content\r\n\r\n"));
    assert_eq!(post(&url, &json(), here()), Outcome::Sent);
    assert_eq!(heard.recv_timeout(Duration::from_secs(1)).map(|_| ()), Ok(()));
}

#[test]
fn the_programs_policy_refuses_this_computer_by_name_and_sends_nothing() {
    let (url, heard) = server(Some("HTTP/1.1 204 No Content\r\n\r\n"));
    let by_name = Url::parse(&format!("http://localhost:{}/hook", url.port().unwrap())).unwrap();
    let strict = Policy {
        https_only: false,
        ..Policy::strict()
    };
    assert_eq!(post(&by_name, &json(), strict), Outcome::Refused);
    // `judged` lets a name through: it is judged on what it resolves to, when sent.
    assert!(judged(&format!("https://localhost:{}/hook", url.port().unwrap())).is_ok());
    thread::sleep(Duration::from_millis(200));
    assert_eq!(heard.try_iter().count(), 0);
}

#[test]
fn a_name_not_resolved_is_unreachable() {
    let url = Url::parse("http://kitokey-test.invalid/hook").unwrap();
    assert_eq!(post(&url, &json(), here()), Outcome::Unreachable);
}
