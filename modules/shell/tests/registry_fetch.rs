//! The list read again on the user's gesture (D40, ADR-0010), tried against a server played here
//! on this machine: what it answers is taken only when it is a list this build would ship.

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use kitokey_shell::registry;
use minisign::KeyPair;

/// A pair made for this run of the tests, never the decider's: its public key as the shell holds
/// one, and what signs a list with it.
struct TestKey {
    public: String,
    pair: KeyPair,
}

fn test_key() -> TestKey {
    let pair = KeyPair::generate_unencrypted_keypair().unwrap();
    TestKey {
        public: pair.pk.to_base64(),
        pair,
    }
}

impl TestKey {
    /// The signature file `minisign -S` would write for `list`.
    fn signed(&self, list: &str) -> String {
        minisign::sign(None, &self.pair.sk, list.as_bytes(), None, None)
            .unwrap()
            .to_string()
    }
}

/// What a server played here answers, and the paths it was asked for.
struct Served {
    address: String,
    asked: Arc<Mutex<Vec<String>>>,
}

/// Answers the list with `status`, `headers` — each line ended by CRLF — and `body`, and its
/// signature's address with `signature`, or a page not found.
fn served_with(status: u16, body: String, headers: &'static str, signature: Option<String>) -> Served {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/models.json", listener.local_addr().unwrap());
    let asked = Arc::new(Mutex::new(Vec::new()));
    let noted = asked.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut lines = BufReader::new(stream.try_clone().unwrap()).lines();
            let first = lines.next().unwrap().unwrap();
            let path = first.split(' ').nth(1).unwrap().to_owned();
            for line in lines {
                if line.unwrap().is_empty() {
                    break;
                }
            }
            let (status, headers, body) = match (path.ends_with(".minisig"), &signature) {
                (false, _) => (status, headers, body.as_str()),
                (true, Some(signature)) => (200, "", signature.as_str()),
                (true, None) => (404, "", "Not Found"),
            };
            noted.lock().unwrap().push(path);
            let head = format!(
                "HTTP/1.1 {status} Status\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(body.as_bytes());
        }
    });
    Served { address, asked }
}

/// Answers `list`, with its signature by `key`.
fn signed_by(key: &TestKey, list: String) -> Served {
    let signature = key.signed(&list);
    served_with(200, list, "", Some(signature))
}

/// The list shipped, with one more model after it — a test's, never fetched.
fn with_another() -> String {
    registry::SHIPPED.replacen(
        "\n  ]",
        r#",
    {"address": "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q8_0.bin", "bytes": 1000, "sha256": "0000000000000000000000000000000000000000000000000000000000000000", "good": "faster"}
  ]"#,
        1,
    )
}

#[test]
fn a_list_signed_by_the_key_is_taken_whole() {
    let key = test_key();
    let served = signed_by(&key, with_another());
    let listed = registry::fetch(&served.address, Some(&key.public)).unwrap();
    assert_eq!(listed.len(), registry::shipped().len() + 1);
    assert_eq!(listed[..listed.len() - 1], registry::shipped()[..]);
    assert_eq!(kitokey_shell::model::file(&listed.last().unwrap().published), "ggml-base-q8_0.bin");
    assert_eq!(*served.asked.lock().unwrap(), ["/models.json", "/models.json.minisig"]);
}

#[test]
fn the_list_is_read_again_from_main_on_the_forge() {
    assert_eq!(
        registry::ADDRESS,
        "https://raw.githubusercontent.com/NapkinStack/wholeheard/main/modules/shell/registry/models.json"
    );
    assert_eq!(registry::host(registry::ADDRESS), "raw.githubusercontent.com");
}

#[test]
fn a_list_unsigned_signed_by_another_key_or_changed_after_it_was_signed_is_refused() {
    let key = test_key();
    let list = with_another();
    let refused = [
        ("unsigned", served_with(200, list.clone(), "", None)),
        ("signed by another key", signed_by(&test_key(), list.clone())),
        (
            "changed after it was signed",
            served_with(200, list.replace("\"bytes\": 1000", "\"bytes\": 1001"), "", Some(key.signed(&list))),
        ),
        (
            "a signature that is none",
            served_with(200, list.clone(), "", Some("<html>".into())),
        ),
    ];
    for (what, served) in refused {
        let why = registry::fetch(&served.address, Some(&key.public)).expect_err(what);
        assert!(why.contains("signature"), "{what}: {why}");
    }
}

#[test]
fn with_no_key_built_in_nothing_is_asked_and_it_is_said_to_wait_on_it() {
    let key = test_key();
    let served = signed_by(&key, with_another());
    let why = registry::fetch(&served.address, None).unwrap_err();
    assert!(why.contains("waits on the decider's key"), "{why}");
    assert!(served.asked.lock().unwrap().is_empty());
}

#[test]
fn a_signed_list_is_still_refused_when_this_build_would_not_ship_it() {
    let key = test_key();
    let first_not_the_default = registry::SHIPPED.replacen("ggml-large-v3-turbo-q8_0.bin", "ggml-other.bin", 1);
    let refused = [
        ("not JSON", "<html>".to_owned()),
        ("a version unknown", registry::SHIPPED.replace("\"version\": 1", "\"version\": 2")),
        ("larger than 64 KB", format!("{}{}", with_another(), " ".repeat(registry::LIMIT))),
        ("its first model not this build's default", first_not_the_default),
    ];
    for (what, list) in refused {
        let served = signed_by(&key, list);
        assert!(
            registry::fetch(&served.address, Some(&key.public)).is_err(),
            "{what} should be refused"
        );
    }
}

#[test]
fn what_does_not_answer_as_a_page_is_refused() {
    let key = test_key();
    let signature = key.signed(&with_another());
    for (what, status, body) in [
        ("a page not found", 404, "Not Found".to_owned()),
        ("a server's error", 500, with_another()),
    ] {
        let served = served_with(status, body, "", Some(signature.clone()));
        assert!(
            registry::fetch(&served.address, Some(&key.public)).is_err(),
            "{what} should be refused"
        );
    }
}

#[test]
fn a_server_that_does_not_answer_is_said_so() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/models.json", listener.local_addr().unwrap());
    drop(listener);
    assert!(registry::fetch(&address, Some(&test_key().public)).is_err());
}

#[test]
fn a_list_sent_on_from_another_address_is_refused() {
    // The host the button names is the one read: a redirect is not followed, wherever it goes.
    let key = test_key();
    let elsewhere = signed_by(&key, with_another());
    let location: &'static str = Box::leak(format!("Location: {}\r\n", elsewhere.address).into_boxed_str());
    let served = served_with(302, String::new(), location, None);
    assert!(registry::fetch(&served.address, Some(&key.public)).is_err());
    assert!(elsewhere.asked.lock().unwrap().is_empty());
}

#[test]
fn the_check_of_the_list_in_the_repository_refuses_a_signature_that_does_not_verify_once_a_key_is_built_in() {
    let key = test_key();
    let list = with_another();
    let good = key.signed(&list);
    assert!(registry::in_the_repository(&list, Some(&good), Some(&key.public)).is_ok());
    assert!(registry::in_the_repository(&list, None, Some(&key.public)).is_err());
    assert!(registry::in_the_repository(&list, Some(&test_key().signed(&list)), Some(&key.public)).is_err());
    assert!(registry::in_the_repository(&format!("{list} "), Some(&good), Some(&key.public)).is_err());
    // No key yet: nothing can be verified, and the check says so rather than saying it passed.
    let said = registry::in_the_repository(&list, None, None).unwrap();
    assert!(said.contains("waits on the decider's key"), "{said}");
    assert!(!said.contains("verifies"), "{said}");
}

#[test]
fn the_list_in_the_repository_verifies_or_waits_on_the_key_and_the_check_says_which() {
    let signature = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/registry/models.json.minisig")).ok();
    let said = registry::in_the_repository(registry::SHIPPED, signature.as_deref(), registry::PUBLIC_KEY)
        .expect("registry/models.json is signed by the key built in: minisign -Sm it, then commit its .minisig");
    // Straight to the error stream: what the test harness captures would not be seen when it passes.
    writeln!(std::io::stderr(), "registry/models.json: {said}").unwrap();
}

// Under the key built in (D56): the decider's, made in their own terminal — its private key in no
// repository and no session. The lists here are signed by a test's key, or are the list on `main`
// with the signature the decider made of it, `registry/models.json.minisig`.

/// The decider's key, as its `.pub` names it: `minisign public key 28AD0E835D7AF31E`.
const DECIDERS_KEY_ID: &str = "28AD0E835D7AF31E";

/// The signature the decider made of the list in the repository.
fn deciders_signature() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/registry/models.json.minisig"))
        .expect("registry/models.json.minisig, the decider's signature of the list")
}

#[test]
fn the_key_built_in_is_the_deciders() {
    let key = registry::PUBLIC_KEY.expect("a key built in");
    let key = minisign::PublicKey::from_base64(key).unwrap();
    // minisign shows a key's number as its eight bytes read little-endian.
    let id: String = key.keynum().iter().rev().map(|byte| format!("{byte:02X}")).collect();
    assert_eq!(id, DECIDERS_KEY_ID);
}

#[test]
fn under_the_key_built_in_a_list_signed_by_another_key_changed_after_it_was_signed_or_unsigned_is_refused() {
    let list = registry::SHIPPED.to_owned();
    let refused = [
        ("signed by another key", signed_by(&test_key(), list.clone())),
        (
            "changed after the decider signed it",
            served_with(
                200,
                list.replacen("\"version\": 1", "\"version\":  1", 1),
                "",
                Some(deciders_signature()),
            ),
        ),
        ("unsigned", served_with(200, list.clone(), "", None)),
    ];
    for (what, served) in refused {
        let why = registry::fetch(&served.address, registry::PUBLIC_KEY).expect_err(what);
        assert!(why.contains("signature") && !why.contains("waits"), "{what}: {why}");
        assert_eq!(*served.asked.lock().unwrap(), ["/models.json", "/models.json.minisig"], "{what}");
    }
}

#[test]
fn under_the_key_built_in_the_list_the_decider_signed_is_taken() {
    let served = served_with(200, registry::SHIPPED.to_owned(), "", Some(deciders_signature()));
    let listed = registry::fetch(&served.address, registry::PUBLIC_KEY).unwrap();
    assert_eq!(listed, registry::shipped());
}

/// A key made for this test, outside it (openssl's Ed25519, the boxes as minisign writes them):
/// `minisign` the crate writes the pre-hashed form only. Its private key was not kept.
const FIXED_KEY: &str = "RWQBI0VniavN7wWzl40CApIQMFl5I82eP9r51HLn0+S12rpml2YUEJMN";
/// What that key signed, both ways.
const FIXED_LIST: &str = "A list signed by a key made for this test.\n";
/// Its signature in minisign's pre-hashed form, `ED`: what `minisign -S` writes.
const PREHASHED: &str = "untrusted comment: signature from minisign secret key\nRUQBI0VniavN7wrzXTMs5fdRG6KVz+5THLiu+a8cBCxN3Z4DuDpIiu2Cuw6V6r3uIeRW5H2FQ8B7Md7iETvI1lBzIvFSaYz98gs=\ntrusted comment: timestamp:1794666686\tfile:models.json\thashed\njSaui6oed+w24Xpc7qoBDxGb4ESZQFH+G1ZlPTqlNFx0oGA4rwdySDORy0ZpX4bGnpIEqTcPo1qHunqdHT7GAg==\n";
/// The same bytes signed in the legacy form, `Ed`: the list itself, not its hash.
const LEGACY: &str = "untrusted comment: signature from minisign secret key\nRWQBI0VniavN77JORcfb0/qk/S+eZJButt7IBOw0QkA5tgFZ6PaeIDWMDMQx9jpkVazHbKuJl/LBc6SjE2HJrM8V5/wcGoAWZgo=\ntrusted comment: timestamp:1794666686\tfile:models.json\thashed\ns6BDg+gP07KNfUzg51q7MJUZRhiw6Jt2I5D/UARqlcYKY/Z/aty8y7kYp9vXkt8Ezy0bjBGe/WWugAD6UjjkBA==\n";

#[test]
fn a_list_signed_in_minisigns_legacy_form_is_refused() {
    // The same key and bytes, pre-hashed: verified — the refusal below is the form's alone.
    assert_eq!(registry::verify(FIXED_LIST, PREHASHED, FIXED_KEY), Ok(()));
    let why = registry::verify(FIXED_LIST, LEGACY, FIXED_KEY).unwrap_err();
    assert!(why.contains("signature"), "{why}");
    assert!(registry::in_the_repository(FIXED_LIST, Some(LEGACY), Some(FIXED_KEY)).is_err());
}

/// How the signature is answered, once its cuts are spent.
enum Signature {
    Signed,
    /// An HTTP status other than 200, with a body.
    Status(u16),
    /// No answer at all for this long, then the connection closed: past the agent's timeout.
    Late(Duration),
}

/// Answers `list` signed by `key`, but cuts the first `list_cuts` connections the list is asked on and
/// the first `cuts` its signature is asked on before any answer — as a network or a proxy cuts a
/// connection (#869) —, then answers as `signature` says.
fn served_cutting(key: &TestKey, list: String, list_cuts: usize, cuts: usize, signature: Signature) -> Served {
    let signed = key.signed(&list);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/models.json", listener.local_addr().unwrap());
    let asked = Arc::new(Mutex::new(Vec::new()));
    let noted = asked.clone();
    thread::spawn(move || {
        let (mut list_cut, mut cut) = (0, 0);
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut lines = BufReader::new(stream.try_clone().unwrap()).lines();
            let first = lines.next().unwrap().unwrap();
            let path = first.split(' ').nth(1).unwrap().to_owned();
            for line in lines {
                if line.unwrap().is_empty() {
                    break;
                }
            }
            noted.lock().unwrap().push(path.clone());
            let of_signature = path.ends_with(".minisig");
            if of_signature && cut < cuts {
                cut += 1;
                continue;
            }
            if !of_signature && list_cut < list_cuts {
                list_cut += 1;
                continue;
            }
            let (status, body) = match (of_signature, &signature) {
                (false, _) | (true, Signature::Signed) => (200, if of_signature { signed.clone() } else { list.clone() }),
                (true, Signature::Status(status)) => (*status, "Unavailable".to_owned()),
                (true, Signature::Late(late)) => {
                    thread::sleep(*late);
                    continue;
                }
            };
            let head = format!(
                "HTTP/1.1 {status} Status\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(body.as_bytes());
        }
    });
    Served { address, asked }
}

/// Answers `list` signed by `key`, but cuts the first `cuts` connections its signature is asked on.
fn signature_cut(key: &TestKey, list: String, cuts: usize) -> Served {
    served_cutting(key, list, 0, cuts, Signature::Signed)
}

#[test]
fn the_signature_cut_before_any_answer_is_asked_once_more_and_the_list_taken() {
    let key = test_key();
    let served = signature_cut(&key, with_another(), 1);
    let listed = registry::fetch(&served.address, Some(&key.public)).unwrap();
    assert_eq!(listed.len(), registry::shipped().len() + 1);
    assert_eq!(
        *served.asked.lock().unwrap(),
        ["/models.json", "/models.json.minisig", "/models.json.minisig"]
    );
}

#[test]
fn the_signature_cut_twice_is_refused_and_said_why() {
    let key = test_key();
    let served = signature_cut(&key, with_another(), 2);
    let why = registry::fetch(&served.address, Some(&key.public)).unwrap_err();
    assert!(why.starts_with("the list's signature could not be read"), "{why}");
    assert_eq!(
        *served.asked.lock().unwrap(),
        ["/models.json", "/models.json.minisig", "/models.json.minisig"]
    );
}

#[test]
fn a_signature_answered_with_an_error_is_never_asked_again() {
    let key = test_key();
    let served = served_with(200, with_another(), "", None);
    let why = registry::fetch(&served.address, Some(&key.public)).unwrap_err();
    assert!(why.contains("HTTP 404"), "{why}");
    assert_eq!(*served.asked.lock().unwrap(), ["/models.json", "/models.json.minisig"]);
}

#[test]
fn the_list_cut_before_any_answer_is_never_asked_again() {
    let key = test_key();
    let served = served_cutting(&key, with_another(), 1, 0, Signature::Signed);
    let why = registry::fetch(&served.address, Some(&key.public)).unwrap_err();
    assert!(why.starts_with("the list could not be read"), "{why}");
    assert_eq!(*served.asked.lock().unwrap(), ["/models.json"]);
}

#[test]
fn a_signature_answered_by_a_server_error_is_never_asked_again() {
    let key = test_key();
    let served = served_cutting(&key, with_another(), 0, 0, Signature::Status(503));
    let why = registry::fetch(&served.address, Some(&key.public)).unwrap_err();
    assert!(why.contains("HTTP 503"), "{why}");
    assert_eq!(*served.asked.lock().unwrap(), ["/models.json", "/models.json.minisig"]);
}

/// The agent's answer timeout is 20 s: this test waits for it once, and twice would say it was asked again.
#[test]
fn a_signature_that_answers_past_its_timeout_is_never_asked_again() {
    let key = test_key();
    let served = served_cutting(&key, with_another(), 0, 0, Signature::Late(Duration::from_secs(25)));
    let why = registry::fetch(&served.address, Some(&key.public)).unwrap_err();
    assert!(why.starts_with("the list's signature could not be read"), "{why}");
    assert_eq!(*served.asked.lock().unwrap(), ["/models.json", "/models.json.minisig"]);
}
