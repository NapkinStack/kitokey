//! `kitokey-send` as `posting/v1` starts it: its lines, its exits, the meeting judged before the
//! address, and the address — a secret — in none of its sentences or lines.

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

const SECRET: &str = "Tq9-secret-token";

struct Ran {
    code: Option<i32>,
    line: String,
    said: String,
}

fn run(args: &[&str], input: &str) -> Ran {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kitokey-send"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A program that refuses its arguments ends before reading its input: a pipe closed under the write is
    // that end, judged by its exit and its sentence (as export's, #1068); any other error fails the test.
    if let Err(error) = child.stdin.take().unwrap().write_all(input.as_bytes()) {
        assert_eq!(
            error.kind(),
            std::io::ErrorKind::BrokenPipe,
            "the input could not be written: {error}"
        );
    }
    let out = child.wait_with_output().unwrap();
    let ran = Ran {
        code: out.status.code(),
        line: String::from_utf8(out.stdout).unwrap(),
        said: String::from_utf8(out.stderr).unwrap(),
    };
    for part in [SECRET, "webhooks/1"] {
        assert!(
            !ran.line.contains(part) && !ran.said.contains(part),
            "the address said: {}{}",
            ran.line,
            ran.said
        );
    }
    ran
}

fn input(address: &str) -> String {
    serde_json::json!({"address": address, "name": "Réunion du 20 nov. 2026"}).to_string()
}

const MINUTES: &str = "---\ncontract: minutes/v4\ntitle: \"Point\"\n---\n\n## Résumé\n\nFixé.\n\n## Actions\n\n- a — b — c\n";

#[test]
fn started_bare_or_wrongly_it_says_its_usage_and_exits_2_with_no_line() {
    for args in [
        &[][..],
        &["--kind", "slack"],
        &["f", "--kind", "teams"],
        &["f", "--kind", "slack", "--kind", "slack"],
        &["f", "--to"],
    ] {
        let ran = run(args, &input("https://discord.com/api/webhooks/1/x"));
        assert_eq!((ran.code, ran.line.as_str()), (Some(2), ""), "{args:?}");
        assert!(
            ran.said.starts_with("kitokey-send: ") && ran.said.contains("usage: kitokey-send"),
            "{}",
            ran.said
        );
    }
    let ran = run(&["--check"], "{\"address\": \"https://x.example/h\"}");
    assert_eq!((ran.code, ran.line.as_str()), (Some(2), ""));
}

#[test]
fn check_says_checked_or_address_refused_without_resolving() {
    let ran = run(&["--check"], &input(&format!("https://discord.com/api/webhooks/1/{SECRET}")));
    assert_eq!((ran.code, ran.line.as_str()), (Some(0), "{\"event\":\"checked\"}\n"));
    for refused in [
        format!("http://discord.com/api/webhooks/1/{SECRET}"),
        format!("https://me:{SECRET}@discord.com/api/webhooks/1/x"),
        format!("https://192.168.1.10/api/webhooks/1/{SECRET}"),
        format!("https://[fd00::1]/api/webhooks/1/{SECRET}"),
    ] {
        let ran = run(&["--check"], &input(&refused));
        assert_eq!(ran.code, Some(1));
        assert_eq!(ran.line, "{\"code\":\"address.refused\",\"event\":\"failed\"}\n");
        assert!(ran.said.starts_with("kitokey-send: the address "), "{}", ran.said);
    }
}

#[test]
fn the_meeting_is_judged_before_the_address_a_confidential_one_never_sent() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_str().unwrap();
    let local = input(&format!("https://127.0.0.1/api/webhooks/1/{SECRET}"));
    let code = |ran: Ran| serde_json::from_str::<serde_json::Value>(&ran.line).unwrap()["code"].clone();
    assert_eq!(code(run(&[path, "--kind", "webhook"], &local)), "meeting.no_summary");
    fs::write(folder.path().join("minutes.md"), MINUTES).unwrap();
    assert_eq!(code(run(&[path, "--kind", "webhook"], &local)), "address.refused");
    fs::write(folder.path().join("confidential"), "").unwrap();
    let ran = run(&[path, "--kind", "discord"], &local);
    assert_eq!(ran.code, Some(1));
    assert_eq!(code(ran), "meeting.confidential");
    assert_eq!(code(run(&["/no/such/folder", "--kind", "slack"], &local)), "meeting.unreadable");
}

#[test]
fn a_host_not_found_is_unreachable_and_its_sentence_holds_no_part_of_the_address() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("minutes.md"), MINUTES).unwrap();
    let path = folder.path().to_str().unwrap();
    let ran = run(
        &[path, "--kind", "slack"],
        &input(&format!("https://kitokey-test.invalid/api/webhooks/1/{SECRET}")),
    );
    assert_eq!(ran.code, Some(1));
    assert_eq!(ran.line, "{\"code\":\"network.unreachable\",\"event\":\"failed\"}\n");
    assert!(!ran.said.contains("kitokey-test.invalid"), "{}", ran.said);
}

#[test]
fn it_says_its_version() {
    let ran = run(&["--version"], "");
    assert_eq!(ran.code, Some(0));
    assert!(ran.line.starts_with("kitokey-send "));
}
