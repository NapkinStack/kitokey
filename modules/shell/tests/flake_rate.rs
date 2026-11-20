//! D104's count (criterion 2): `scripts/flake_rate.py` on a history built for the test. A check that
//! failed once where it passed on another run of the same commit is counted, at its rate over all its
//! runs; one that fails on every run of its commit is a defect, not counted; `logs_read` and the
//! shards' own lines are said apart and never fail the count; `--since` keeps D104's window.

use std::process::Command;

fn history(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("kitokey-flake-rate-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder for the history");
    let attest = |file: &str, lines: &[&str]| std::fs::write(dir.join(file), lines.join("\n")).unwrap();
    // Commit aaa1111, run twice: « the bar » fails once and passes once — a flake; « the page » fails
    // both times — a defect; logs_read follows the flake.
    attest(
        "20261120T100000-pr1-aaa1111.txt",
        &[
            "  ok    the list",
            "  FAIL  the bar",
            "  FAIL  the page",
            "    FAIL  logs_read",
            "  FAIL  shard 1: e2e",
        ],
    );
    attest(
        "20261120T110000-pr1-aaa1111.txt",
        &[
            "  ok    the list",
            "  ok    the bar",
            "  FAIL  the page",
            "    ok    logs_read",
            "  ok    shard 1: e2e",
        ],
    );
    // Commit bbb2222, once, before the window: « the list » fails, never passed on that commit.
    attest("20261119T090000-pr2-bbb2222.txt", &["  FAIL  the list", "  ok    the bar"]);
    dir
}

fn counted(dir: &std::path::Path, since: &[&str]) -> (bool, String) {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/flake_rate.py");
    let output = Command::new("python3")
        .arg("-I")
        .arg(script)
        .arg(dir)
        .args(since)
        .output()
        .expect("python3, as the e2e needs it");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "flake_rate.py could not count: {stderr}");
    (output.status.success(), String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn a_check_failing_where_it_passed_on_the_same_commit_is_counted_over_its_runs() {
    let dir = history("all");
    let (passed, said) = counted(&dir, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(said.contains("attests read: 3; checks: 5"), "{said}");
    // « the bar »: 1 FAIL where it passed, over its 3 runs.
    assert!(said.contains(" 33.3%  1/3  the bar\n"), "the flake not counted at its rate: {said}");
    // « the page » fails on both runs of its commit, « the list » never passed on bbb2222.
    assert!(
        !said.contains("the page") && !said.contains("the list"),
        "a defect counted as a flake: {said}"
    );
    assert!(
        said.contains("logs_read (said apart: it follows another FAIL)"),
        "logs_read not said apart: {said}"
    );
    assert!(said.contains("shard 1: e2e (said apart"), "a shard's line not said apart: {said}");
    assert!(said.contains("checks at or over 2%, derived ones apart: 1"), "{said}");
    assert!(!passed, "a flake over 2 % did not fail the count: {said}");
}

#[test]
fn the_window_keeps_the_attests_from_its_stamp_on() {
    let dir = history("since");
    let (_, said) = counted(&dir, &["--since", "20261120T"]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(said.contains("attests read: 2 since 20261120T"), "the window not kept: {said}");
    assert!(
        said.contains(" 50.0%  1/2  the bar\n"),
        "the rate not over the window's runs: {said}"
    );
}
