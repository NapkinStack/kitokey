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
fn notion_asks_its_key_and_a_databases_link_and_its_check_takes_no_other_kind() {
    let link = format!("https://www.notion.so/Notes-0123456789abcdef0123456789abcdef?v={SECRET}");
    // No key: the input is not posting/v1's for notion.
    for args in [&["--check", "--kind", "notion"][..], &["f", "--kind", "notion"]] {
        let ran = run(args, &input(&link));
        assert_eq!((ran.code, ran.line.as_str()), (Some(2), ""), "{args:?}");
        assert!(ran.said.contains("« key »"), "{}", ran.said);
    }
    for args in [
        &["--check", "--kind", "slack"][..],
        &["--check", "f", "--kind", "notion"],
        &["--check", "--check"],
    ] {
        let ran = run(args, &input(&link));
        assert_eq!((ran.code, ran.line.as_str()), (Some(2), ""), "{args:?}");
    }
    // A link that names no database is refused before anything is contacted, its key said nowhere.
    let keyed = |address: &str| serde_json::json!({"address": address, "name": "Réunion", "key": format!("ntn_{SECRET}")}).to_string();
    for refused in [
        format!("http://www.notion.so/Notes-0123456789abcdef0123456789abcdef?v={SECRET}"),
        format!("https://www.notion.so/Notes-{SECRET}"),
    ] {
        let ran = run(&["--check", "--kind", "notion"], &keyed(&refused));
        assert_eq!(ran.code, Some(1));
        assert_eq!(ran.line, "{\"code\":\"address.refused\",\"event\":\"failed\"}\n");
    }
    // The meeting before the link: a confidential one is never sent to Notion either.
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("minutes.md"), MINUTES).unwrap();
    fs::write(folder.path().join("confidential"), "").unwrap();
    let ran = run(&[folder.path().to_str().unwrap(), "--kind", "notion"], &keyed(&link));
    assert_eq!(ran.line, "{\"code\":\"meeting.confidential\",\"event\":\"failed\"}\n");
    // A meeting that may leave, to a link that names no database: refused, nothing contacted.
    fs::remove_file(folder.path().join("confidential")).unwrap();
    let ran = run(
        &[folder.path().to_str().unwrap(), "--kind", "notion"],
        &keyed(&format!("https://www.notion.so/Notes-{SECRET}")),
    );
    assert_eq!(ran.line, "{\"code\":\"address.refused\",\"event\":\"failed\"}\n");
}

#[test]
fn it_says_its_version() {
    let ran = run(&["--version"], "");
    assert_eq!(ran.code, Some(0));
    assert!(ran.line.starts_with("kitokey-send "));
}

/// D118: the field on the input, as posting/v1 says it. The meeting is still judged before the address
/// (here a local one, which the program refuses): `meeting.confidential` unconfirmed, the address reached once
/// confirmed; a meeting not confidential reaches the address whatever the field says; a field that is no
/// boolean is not posting/v1's input.
#[test]
fn a_confidential_meeting_goes_on_only_when_its_input_says_confirmed_confidential_true() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("minutes.md"), MINUTES).unwrap();
    let path = folder.path().to_str().unwrap();
    let address = format!("https://127.0.0.1/api/webhooks/1/{SECRET}");
    let with = |confirmed: serde_json::Value| {
        serde_json::json!({"address": address, "name": "Réunion", "confirmed_confidential": confirmed}).to_string()
    };
    let code = |ran: Ran| serde_json::from_str::<serde_json::Value>(&ran.line).unwrap()["code"].clone();
    for input in [input(&address), with(false.into()), with(true.into())] {
        assert_eq!(code(run(&[path, "--kind", "webhook"], &input)), "address.refused", "{input}");
    }
    fs::write(folder.path().join("confidential"), "").unwrap();
    assert_eq!(code(run(&[path, "--kind", "webhook"], &input(&address))), "meeting.confidential");
    assert_eq!(code(run(&[path, "--kind", "slack"], &with(false.into()))), "meeting.confidential");
    assert_eq!(code(run(&[path, "--kind", "discord"], &with(true.into()))), "address.refused");
    for wrong in [with("true".into()), with(1.into()), with(serde_json::Value::Null)] {
        let ran = run(&[path, "--kind", "webhook"], &wrong);
        assert_eq!((ran.code, ran.line.as_str()), (Some(2), ""), "{wrong}");
        assert!(ran.said.contains("confirmed_confidential"), "{}", ran.said);
    }
}
