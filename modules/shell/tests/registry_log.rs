//! The list's signature asked again is said in the log (#869): alone in its own test binary, so
//! that no other test reaches the log's call site first, without the subscriber this one sets.

use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use kitokey_shell::registry;
use minisign::KeyPair;

/// Where the log is written, for the test to read.
#[derive(Clone, Default)]
struct Kept(Arc<Mutex<Vec<u8>>>);

impl io::Write for Kept {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Answers the list shipped, signed by a key made for this run; cuts the first connection its
/// signature is asked on before any answer.
fn served(public: &mut String) -> String {
    let pair = KeyPair::generate_unencrypted_keypair().unwrap();
    *public = pair.pk.to_base64();
    let list = registry::SHIPPED.to_owned();
    let signature = minisign::sign(None, &pair.sk, list.as_bytes(), None, None).unwrap().to_string();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/models.json", listener.local_addr().unwrap());
    thread::spawn(move || {
        let mut cut = false;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut lines = BufReader::new(stream.try_clone().unwrap()).lines();
            let path = lines.next().unwrap().unwrap().split(' ').nth(1).unwrap().to_owned();
            for line in lines {
                if line.unwrap().is_empty() {
                    break;
                }
            }
            if path.ends_with(".minisig") && !cut {
                cut = true;
                continue;
            }
            let body = if path.ends_with(".minisig") { &signature } else { &list };
            let head = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(body.as_bytes());
        }
    });
    address
}

#[test]
fn the_signature_asked_again_is_said_in_the_log_once_at_info() {
    let mut public = String::new();
    let address = served(&mut public);
    let kept = Kept::default();
    let writer = kept.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_target(false)
        .with_writer(move || writer.clone())
        .finish();
    let listed = tracing::subscriber::with_default(subscriber, || registry::fetch(&address, Some(&public)));
    assert!(listed.is_ok(), "{listed:?}");
    let text = String::from_utf8(kept.0.lock().unwrap().clone()).unwrap();
    let said: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter(|line| line["message"] == "shell.registry_asked_again")
        .collect();
    assert_eq!(said.len(), 1, "{text}");
    assert_eq!(said[0]["level"], "INFO");
    assert!(said[0]["detail"].as_str().is_some_and(|detail| !detail.is_empty()), "{text}");
}
