//! A partial run of the window's e2e (pytest's `-k`): `KITOKEY_E2E_ONLY` names scenarios by their
//! function's name, and `scripts/e2e_only.py` says which a run of `scripts/e2e_window.py` plays —
//! those alone, in the order `main()` calls them; without it, every one `main()` calls. A red check
//! is replayed alone before a pull request is blamed for it (the coordinator, 2026-11-14).

use std::{fs, process::Command};

/// What `e2e_only.py` says a run plays, with `KITOKEY_E2E_ONLY` as given — `None` when Python is not
/// there; the exit's success and what it printed otherwise.
fn played(only: Option<&str>) -> Option<(bool, Vec<String>, String)> {
    let here = env!("CARGO_MANIFEST_DIR");
    let mut command = Command::new("python3");
    command.args([
        "-I",
        &format!("{here}/scripts/e2e_only.py"),
        &format!("{here}/scripts/e2e_window.py"),
    ]);
    command.env_remove("KITOKEY_E2E_ONLY");
    if let Some(only) = only {
        command.env("KITOKEY_E2E_ONLY", only);
    }
    let output = command.output().ok()?;
    let out = String::from_utf8_lossy(&output.stdout);
    Some((
        output.status.success(),
        out.lines().map(str::to_string).collect(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

/// The scenarios `main()` calls, read here from the script's text: each a line `    name(now)`.
fn called() -> Vec<String> {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py")).unwrap();
    let main = &source[source.find("\ndef main() -> int:\n").unwrap()..];
    let main = &main[..main[1..].find("\n\n\n").map_or(main.len(), |end| end + 1)];
    main.lines()
        .filter_map(|line| line.strip_prefix("    ")?.strip_suffix("(now)"))
        .filter(|name| name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
        .map(str::to_string)
        .collect()
}

#[test]
fn a_run_without_the_filter_plays_every_scenario_and_with_it_those_named_alone() {
    let Some((ok, all, _)) = played(None) else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: the partial run is not read");
        return;
    };
    // Without it: every scenario main() calls, in its order.
    let called = called();
    assert!(called.len() > 30, "{} scenarios read in main()", called.len());
    assert!(ok);
    assert_eq!(all, called);
    // With it: the one named alone.
    let (ok, one, _) = played(Some("models_advanced")).unwrap();
    assert!(ok);
    assert_eq!(one, ["models_advanced"]);
    // Several, as main() orders them, whatever order they are named in; spaces around a name ignored.
    let first = called.iter().position(|name| name == "two_panes").unwrap();
    let second = called.iter().position(|name| name == "app_language").unwrap();
    let (expected_first, expected_second) = if first < second {
        ("two_panes", "app_language")
    } else {
        ("app_language", "two_panes")
    };
    let (ok, two, _) = played(Some(" app_language , two_panes ")).unwrap();
    assert!(ok);
    assert_eq!(two, [expected_first, expected_second]);
    // A name main() does not call: refused, nothing played, the name said.
    let (ok, none, said) = played(Some("models_advanced,no_such_scenario")).unwrap();
    assert!(!ok);
    assert!(none.is_empty(), "{none:?}");
    assert!(said.contains("no_such_scenario"), "{said}");
    // Set and empty: no filter.
    let (ok, empty, _) = played(Some("")).unwrap();
    assert!(ok);
    assert_eq!(empty, called);
}

/// What `e2e_only.py --asked` gives `e2e.sh` before anything is built: the names asked, checked and
/// joined — empty for none —, or a refusal naming the unknown one.
fn asked(only: &str) -> Option<(bool, String, String)> {
    let here = env!("CARGO_MANIFEST_DIR");
    let output = Command::new("python3")
        .args([
            "-I",
            &format!("{here}/scripts/e2e_only.py"),
            "--asked",
            &format!("{here}/scripts/e2e_window.py"),
        ])
        .env("KITOKEY_E2E_ONLY", only)
        .output()
        .ok()?;
    Some((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

#[test]
fn the_names_are_checked_before_the_run_and_a_blank_value_is_a_full_run() {
    let Some((ok, names, _)) = asked("  ") else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: the names asked are not read");
        return;
    };
    // Blank: no name, a full run — as e2e_window.py and e2e_logs.py read it.
    assert!(ok);
    assert_eq!(names, "");
    // Named: checked, spaces gone, in main()'s order, joined as e2e.sh passes them on.
    let called = called();
    let first = called.iter().position(|name| name == "two_panes").unwrap();
    let second = called.iter().position(|name| name == "app_language").unwrap();
    let (ok, names, _) = asked(" app_language , two_panes ").unwrap();
    assert!(ok);
    assert_eq!(
        names,
        if first < second {
            "two_panes,app_language"
        } else {
            "app_language,two_panes"
        }
    );
    // A typo: refused before anything is built, the name said.
    let (ok, names, said) = asked("models_advnced").unwrap();
    assert!(!ok);
    assert_eq!(names, "");
    assert!(said.contains("models_advnced"), "{said}");
}
