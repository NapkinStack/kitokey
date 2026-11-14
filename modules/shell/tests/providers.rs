//! Who can write the minutes: an Ollama made here, answering as Ollama does.

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use kitokey_shell::providers::{
    self, ONLINE, hearing, hearing_listed, host, local, ollama_address, ollama_listed, ollama_models, ollama_pull,
};

/// A server that answers each request with `body`, as many times as asked.
fn answering(body: &'static str, times: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        for stream in listener.incoming().take(times) {
            let mut stream = stream.unwrap();
            let mut asked = [0u8; 2048];
            let _ = stream.read(&mut asked);
            let reply = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    address
}

#[test]
fn the_models_ollama_holds_are_offered_by_name_and_none_when_it_does_not_answer() {
    let ollama = answering(
        r#"{"models":[{"name":"qwen3:14b","size":1},{"name":"mistral:7b"},{"name":"mistral:7b"},{"size":2}]}"#,
        1,
    );
    assert_eq!(ollama_models(&ollama), ["mistral:7b", "qwen3:14b"]);
    // Not running, or not Ollama: no model, no fault.
    let closed = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    assert!(ollama_models(&format!("http://{closed}")).is_empty());
    assert!(ollama_models(&answering("not json", 1)).is_empty());
    assert!(ollama_models(&answering(r#"{"other":1}"#, 1)).is_empty());
    // A name `kitokey-summary` would refuse is not offered.
    let odd = answering(r#"{"models":[{"name":""},{"name":"  "},{"name":"a\u0000b"},{"name":"ok"}]}"#, 1);
    assert_eq!(ollama_models(&odd), ["ok"]);
    assert_eq!(ollama_address(&format!("{}/", providers::OLLAMA)), "http://localhost:11434/v1");
}

#[test]
fn the_model_that_holds_the_minutes_form_is_offered_first() {
    // Qwen3 8B wrote the minutes on this laptop; Mistral 7B and models for code did not (#261).
    let ollama = answering(
        r#"{"models":[{"name":"codellama:latest"},{"name":"deepseek-coder:latest"},{"name":"mistral:latest"},{"name":"qwen3:8b"}]}"#,
        1,
    );
    assert_eq!(
        ollama_models(&ollama),
        ["qwen3:8b", "codellama:latest", "deepseek-coder:latest", "mistral:latest"]
    );
    assert_eq!(providers::RECOMMENDED, "qwen3:8b");
}

#[test]
fn an_ollama_that_takes_the_question_and_never_answers_holds_nothing_up() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        let held: Vec<_> = listener.incoming().take(1).collect();
        thread::sleep(std::time::Duration::from_secs(30));
        drop(held);
    });
    let asked = std::time::Instant::now();
    assert!(ollama_models(&address).is_empty());
    assert!(asked.elapsed() < std::time::Duration::from_secs(5), "{:?}", asked.elapsed());
}

#[test]
fn a_provider_s_host_is_what_its_key_is_filed_under() {
    assert_eq!(host("https://openrouter.ai/api/v1").as_deref(), Some("openrouter.ai"));
    assert_eq!(host("http://LocalHost:11434/v1").as_deref(), Some("localhost:11434"));
    assert_eq!(host("https://user@api.example.org?x#y").as_deref(), Some("api.example.org"));
    assert_eq!(host("https://a.org#x").as_deref(), Some("a.org"));
    // The same server however its address is written: the scheme in any case, its own port or none.
    assert_eq!(host("HTTPS://OpenRouter.ai:443/api/v1").as_deref(), Some("openrouter.ai"));
    assert_eq!(host("http://example.org:80").as_deref(), Some("example.org"));
    assert_eq!(host("https://example.org:8443").as_deref(), Some("example.org:8443"));
    for none in [
        "",
        "openrouter.ai",
        "ftp://x",
        "https://",
        "https:///v1",
        "https://openrouter.ai/api/v1not an address",
        " https://x.org",
    ] {
        assert_eq!(host(none), None, "{none}");
    }
    assert!(ONLINE.iter().all(|online| host(online.address).is_some()));
}

#[test]
fn an_address_is_local_exactly_when_kitokey_summary_reads_it_as_this_machine() {
    // Each address as kitokey-summary reads it — accepted as this machine, accepted as another, or
    // refused before anything is sent —; only the first are local.
    for here in [
        "http://localhost:11434/v1",
        "http://LOCALHOST/v1",
        "HTTP://localhost/v1",
        "http://127.0.0.1:8080/v1",
        "http://127.255.255.254/v1",
        "http://0.0.0.0:11434/v1",
        "http://[::]/v1",
        "http://[::1]:11434/v1",
        "http://[0:0:0:0:0:0:0:1]/v1",
        "http://[::ffff:127.0.0.1]:11434/v1",
    ] {
        assert!(local(here), "{here}");
    }
    for elsewhere in [
        "https://openrouter.ai/api/v1",
        "http://localhost.localdomain/v1",
        "http://192.168.1.10:11434/v1",
    ] {
        assert!(!local(elsewhere), "{elsewhere}");
    }
    for refused in [
        "http://evil.invalid\\@localhost/v1",
        "http://localhost#@evil.invalid/v1",
        "http://evil.invalid@localhost/v1",
        "http://localhost?x/v1",
        "http://localhost:99999/v1",
        "http://localhost:x/v1",
        "http://127.0.0.1:/v1",
        "http://localhost./v1",
        "http://127.1/v1",
        "http://2130706433/v1",
        "http://localhost:9@evil.invalid/v1",
        "http://[localhost]/v1",
        "http://local/v1",
        "ftp://localhost/v1",
        "http:// localhost/v1",
        "not an address",
    ] {
        assert!(!local(refused), "{refused}");
    }
}

/// An Ollama whose pull says `lines`, one per line, then closes (D37).
fn pulling(lines: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        let mut stream = listener.incoming().next().unwrap().unwrap();
        // The whole request read before the answer: Windows resets a connection closed on what was
        // not read, and the answer would be lost.
        let mut asked = Vec::new();
        let mut chunk = [0u8; 4096];
        while let Ok(read @ 1..) = stream.read(&mut chunk) {
            asked.extend_from_slice(&chunk[..read]);
            let text = String::from_utf8_lossy(&asked);
            if let Some(end) = text.find("\r\n\r\n") {
                let length = text[..end]
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse().unwrap_or(0))
                    })
                    .unwrap_or(0);
                if asked.len() >= end + 4 + length {
                    break;
                }
            }
        }
        let reply = format!("HTTP/1.1 200 OK\r\ncontent-type: application/x-ndjson\r\nconnection: close\r\n\r\n{lines}");
        let _ = stream.write_all(reply.as_bytes());
    });
    address
}

#[test]
fn ollama_s_pull_is_whole_only_when_it_says_so_and_how_far_is_told() {
    let mut told = Vec::new();
    let whole =
        pulling("{\"status\":\"pulling manifest\"}\n{\"status\":\"pulling a\",\"total\":10,\"completed\":4}\n{\"status\":\"success\"}\n");
    assert_eq!(ollama_pull(&whole, "qwen3:8b", &mut |held, all| told.push((held, all))), Ok(()));
    assert_eq!(told, [(4, 10)]);
    // Refused with its words, cut before success, or not answering: said, never whole.
    let refused = pulling("{\"error\":\"pull model manifest: file does not exist\"}\n");
    assert!(
        ollama_pull(&refused, "qwen3:8b", &mut |_, _| ())
            .unwrap_err()
            .contains("file does not exist")
    );
    let cut = pulling("{\"status\":\"pulling a\",\"total\":10,\"completed\":4}\n");
    assert!(ollama_pull(&cut, "qwen3:8b", &mut |_, _| ()).is_err());
    let closed = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    assert!(ollama_pull(&format!("http://{closed}"), "qwen3:8b", &mut |_, _| ()).is_err());
}

#[test]
fn ollama_answering_with_no_model_is_not_ollama_missing() {
    assert_eq!(ollama_listed(&answering(r#"{"models":[]}"#, 1)), Some(Vec::new()));
    let closed = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    assert_eq!(ollama_listed(&format!("http://{closed}")), None);
}

#[test]
fn the_services_that_hear_are_the_minutes_own_offered_by_name_with_their_published_models() {
    let offered = hearing();
    let names: Vec<_> = offered.iter().map(|service| service.name).collect();
    assert_eq!(names, ["OpenRouter", "OpenAI"]);
    assert_eq!(offered[0].address, ONLINE[0].address);
    assert_eq!(offered[1].address, ONLINE[1].address);
    // OpenRouter lists its own; OpenAI's with segments are named — `gpt-4o-transcribe` gives none.
    assert!(offered[0].listed && !offered[1].listed);
    assert_eq!(offered[1].models, ["whisper-1"]);
}

#[test]
fn the_transcription_models_a_service_lists_are_offered_by_name_and_none_when_it_does_not_list() {
    let listing = answering(
        r#"{"data":[{"id":"openai/whisper-large-v3"},{"id":"vendor/scribe","name":"Scribe"},{"name":"no id"},{"id":""},{"id":"a\u0000b"}]}"#,
        1,
    );
    assert_eq!(hearing_listed(&listing).unwrap(), ["openai/whisper-large-v3", "vendor/scribe"]);
    // Not answering, not a list, an empty one: none — the models published are offered then.
    let closed = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    assert_eq!(hearing_listed(&format!("http://{closed}")), None);
    assert_eq!(hearing_listed(&answering("not json", 1)), None);
    assert_eq!(hearing_listed(&answering(r#"{"data":[]}"#, 1)), None);
}

/// A service that answers each path with its status and body, and keeps what it was asked:
/// the path and the `authorization` header, as many times as asked (D95).
fn serving(routes: &'static [(&'static str, u16, &'static str)], times: usize) -> (String, std::sync::mpsc::Receiver<(String, String)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/api/v1", listener.local_addr().unwrap());
    let (sent, asked) = std::sync::mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming().take(times) {
            let mut stream = stream.unwrap();
            let mut request = [0u8; 4096];
            let read = stream.read(&mut request).unwrap_or(0);
            let request = String::from_utf8_lossy(&request[..read]).to_string();
            let path = request.split_whitespace().nth(1).unwrap_or_default().to_string();
            let authorization = request
                .lines()
                .find_map(|line| line.to_ascii_lowercase().starts_with("authorization:").then(|| line[14..].trim().to_string()))
                .unwrap_or_default();
            let _ = sent.send((path.clone(), authorization));
            let (status, body) = routes
                .iter()
                .find(|(route, _, _)| path == *route)
                .map_or((404, "{}"), |(_, status, body)| (*status, *body));
            let reply = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    (address, asked)
}

#[test]
fn a_service_s_models_are_listed_with_the_key_and_a_refusal_of_it_is_the_refused_key() {
    use kitokey_shell::{providers::service_models, summarising::Cause};
    // OpenAI's form and Mistral's example, a bare list: the ids alone, none the step would refuse.
    let (listing, asked) = serving(&[("/api/v1/models", 200, r#"{"object":"list","data":[{"id":"gpt-4o-mini","owned_by":"x"},{"id":""},{"name":"no id"}]}"#)], 1);
    assert_eq!(service_models(&listing, false, Some("sk-good")).unwrap(), ["gpt-4o-mini"]);
    assert_eq!(asked.recv().unwrap(), ("/api/v1/models".to_string(), "Bearer sk-good".to_string()));
    let (bare, _) = serving(&[("/api/v1/models", 200, r#"[{"id":"mistral-small-latest"},{"id":"a\u0000b"}]"#)], 1);
    assert_eq!(service_models(&bare, false, Some("k")).unwrap(), ["mistral-small-latest"]);
    // The key refused, 401 or 403: the refused key, whatever the service says with it.
    for status in [401, 403] {
        let routes: &'static [(&str, u16, &str)] = if status == 401 {
            &[("/api/v1/models", 401, r#"{"error":"Incorrect API key provided: sk-f***"}"#)]
        } else {
            &[("/api/v1/models", 403, "{}")]
        };
        let (refusing, _) = serving(routes, 1);
        assert_eq!(service_models(&refusing, false, Some("sk-false")), Err(Cause::RefusedKey));
    }
    // Each other refusal its own cause; none read as a list.
    for (status, cause) in [(402, Cause::NoCredit), (429, Cause::TooMany), (503, Cause::Down), (404, Cause::ProviderFailed)] {
        let routes: &'static [(&str, u16, &str)] = match status {
            402 => &[("/api/v1/models", 402, "{}")],
            429 => &[("/api/v1/models", 429, "{}")],
            503 => &[("/api/v1/models", 503, "{}")],
            _ => &[],
        };
        let (failing, _) = serving(routes, 1);
        assert_eq!(service_models(&failing, false, Some("k")), Err(cause), "HTTP {status}");
    }
    let (not_a_list, _) = serving(&[("/api/v1/models", 200, "not json")], 1);
    assert_eq!(service_models(&not_a_list, false, Some("k")), Err(Cause::ProviderFailed));
    let closed = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    assert_eq!(service_models(&format!("http://{closed}/v1"), false, Some("k")), Err(Cause::Unreachable));
}

#[test]
fn a_service_that_lists_its_models_to_anyone_has_its_key_asked_first() {
    use kitokey_shell::{providers::service_models, summarising::Cause};
    // OpenRouter lists its models to anyone: its own route says whether the key is one (`GET /key`).
    assert_eq!(ONLINE.map(|service| service.key_checked), [true, false, false]);
    let (refusing, asked) = serving(&[("/api/v1/key", 401, "{}"), ("/api/v1/models", 200, r#"{"data":[{"id":"a/b"}]}"#)], 2);
    assert_eq!(service_models(&refusing, true, Some("sk-false")), Err(Cause::RefusedKey));
    assert_eq!(asked.recv().unwrap(), ("/api/v1/key".to_string(), "Bearer sk-false".to_string()));
    // Refused there, its models are not asked.
    assert!(asked.recv_timeout(std::time::Duration::from_millis(300)).is_err());
    let (good, asked) = serving(&[("/api/v1/key", 200, r#"{"data":{"label":"x"}}"#), ("/api/v1/models", 200, r#"{"data":[{"id":"anthropic/claude-sonnet-4-5"}]}"#)], 2);
    assert_eq!(service_models(&good, true, Some("sk-good")).unwrap(), ["anthropic/claude-sonnet-4-5"]);
    assert_eq!(asked.recv().unwrap().0, "/api/v1/key");
    assert_eq!(asked.recv().unwrap(), ("/api/v1/models".to_string(), "Bearer sk-good".to_string()));
}
