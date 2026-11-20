//! D104's count (criterion 2): `scripts/flake_rate.py` on histories built for the test. A check that
//! failed where it passed on another run of the same commit is a flake, its rate over its runs on
//! commits run more than once; a commit run once tells nothing and is said as unclassed; a check
//! failing on every run of its commit is a defect; a run's own temporary path is no part of a name;
//! `logs_read` and the shards' lines are said apart; the share of attests red by flakes alone is said;
//! exactly 2 % fails the count; an attest with no check is set apart, none read at all fails; and
//! `--since <commit>` keeps the attests that descend from it, in a clone.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kitokey-flake-rate-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder for the history");
    dir
}

fn attest(dir: &Path, file: &str, lines: &[&str]) {
    std::fs::write(dir.join(file), lines.join("\n")).unwrap();
}

fn counted(dir: &Path, more: &[&str]) -> (i32, String) {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/flake_rate.py");
    let output = Command::new("python3")
        .arg("-I")
        .arg(script)
        .arg(dir)
        .args(more)
        .output()
        .expect("python3, as the e2e needs it");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "flake_rate.py could not count: {stderr}");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
}

#[test]
fn a_flake_is_counted_over_the_runs_of_commits_run_more_than_once() {
    let dir = folder("rate");
    // aaa1111 run twice: « the bar » fails once and passes once — a flake, which alone made the first
    // attest red; « the page » fails both times — a defect; logs_read and the shard follow. The errors
    // check names each run's own folder: one check, not two.
    attest(
        &dir,
        "20261120T100000-pr1-aaa1111.txt",
        &[
            "  ok    the list",
            "  FAIL  the bar",
            "  ok    test ! -s /c/kitokey-headless/run-111/tmp/tmp.AbC/errors",
            "    FAIL  logs_read",
            "  FAIL  shard 1: e2e",
        ],
    );
    attest(
        &dir,
        "20261120T110000-pr1-aaa1111.txt",
        &[
            "  ok    the list",
            "  ok    the bar",
            "  FAIL  test ! -s /c/kitokey-headless/run-222/tmp/tmp.XyZ/errors",
            "  FAIL  the page",
            "    ok    logs_read",
            "  ok    shard 1: e2e",
        ],
    );
    attest(&dir, "20261120T120000-pr1-aaa1111.txt", &["  ok    the bar", "  FAIL  the page"]);
    // bbb2222 run once: its FAIL is unclassed, and its runs are no part of any rate.
    attest(&dir, "20261120T130000-pr2-bbb2222.txt", &["  FAIL  the list", "  ok    the bar"]);
    // An attest whose e2e printed no check: set apart, named.
    attest(&dir, "20261120T140000-pr3-ccc3333.txt", &["e2e: in 5 shards side by side"]);
    let (code, said) = counted(&dir, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        said.contains("attests read: 4, of 2 commits; run more than once: 1 commits, 3 attests"),
        "{said}"
    );
    assert!(
        said.contains("no check at all, set apart: 20261120T140000-pr3-ccc3333.txt"),
        "{said}"
    );
    assert!(said.contains("FAILs on commits run once, unclassed: 1\n"), "{said}");
    // « the bar »: 1 FAIL over its 3 runs on aaa1111, of its 4 runs in all.
    assert!(said.contains(" 33.3%  1/3 (of 4 runs)  the bar\n"), "the flake's rate: {said}");
    // The errors check, one name for both runs: a flake, not two checks seen once.
    assert!(
        said.contains(" 50.0%  1/2 (of 2 runs)  test ! -s /c/kitokey-headless/run-*/tmp/tmp.*/errors\n"),
        "a run's own path kept in a name: {said}"
    );
    assert!(
        !said.contains("the page") && !said.contains("the list"),
        "a defect, or a commit run once, counted: {said}"
    );
    assert!(said.contains("logs_read (said apart"), "{said}");
    assert!(said.contains("shard 1: e2e (said apart"), "{said}");
    // Of aaa1111's 3 attests, 3 are red (« the page » in two); one by a flake alone.
    assert!(
        said.contains("attests red, on commits run more than once: 3; by flakes alone: 1 of 3 (33.3%)"),
        "{said}"
    );
    assert_eq!(code, 1, "a flake over 2 % did not fail the count: {said}");
}

#[test]
fn exactly_two_percent_fails_the_count() {
    let dir = folder("bound");
    // 25 commits run twice: 50 runs, one FAIL where its commit passed.
    for n in 0..25 {
        let first = if n == 0 { "  FAIL  the bar" } else { "  ok    the bar" };
        attest(&dir, &format!("20261120T10{n:04}-pr1-c{n:06}.txt"), &[first]);
        attest(&dir, &format!("20261120T11{n:04}-pr1-c{n:06}.txt"), &["  ok    the bar"]);
    }
    let (code, said) = counted(&dir, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(said.contains("  2.0%  1/50 (of 50 runs)  the bar\n"), "{said}");
    assert_eq!(code, 1, "2 % did not fail the count: {said}");
}

#[test]
fn nothing_read_fails_the_count() {
    let dir = folder("none");
    let (code, said) = counted(&dir, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(said.contains("nothing to count"), "{said}");
    assert_eq!(code, 2, "an empty history passed: {said}");
}

#[test]
fn the_window_keeps_the_attests_that_descend_from_its_commit() {
    let dir = folder("since");
    let repo = dir.join("clone");
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args([
                "-C",
                repo.to_str().unwrap(),
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    std::fs::create_dir_all(&repo).unwrap();
    git(&["init", "-q", "-b", "main"]);
    git(&["commit", "-q", "--allow-empty", "-m", "before"]);
    let before = git(&["rev-parse", "--short=7", "HEAD"]);
    git(&["commit", "-q", "--allow-empty", "-m", "the fix"]);
    let fix = git(&["rev-parse", "--short=7", "HEAD"]);
    git(&["commit", "-q", "--allow-empty", "-m", "after"]);
    let after = git(&["rev-parse", "--short=7", "HEAD"]);
    // A branch from before the fix, attested later: it has not the fix.
    git(&["checkout", "-q", "-b", "late", &before]);
    git(&["commit", "-q", "--allow-empty", "-m", "a late branch"]);
    let late = git(&["rev-parse", "--short=7", "HEAD"]);
    let history = dir.join("history");
    std::fs::create_dir_all(&history).unwrap();
    for (stamp, commit, bar) in [
        ("T1", &after, "  FAIL  the bar"),
        ("T2", &after, "  ok    the bar"),
        ("T3", &late, "  FAIL  the bar"),
    ] {
        attest(&history, &format!("20261121{stamp}-pr1-{commit}.txt"), &[bar]);
    }
    attest(&history, "20261121T4-pr1-0000000.txt", &["  ok    the bar"]);
    let (_, said) = counted(&history, &["--since", &fix, "--repo", repo.to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        said.contains(&format!("attests read: 2 since {fix}, of 1 commits")),
        "the window not kept: {said}"
    );
    assert!(said.contains("its commit unknown in"), "an unknown commit not said: {said}");
    assert!(said.contains(" 50.0%  1/2 (of 2 runs)  the bar\n"), "{said}");
}
