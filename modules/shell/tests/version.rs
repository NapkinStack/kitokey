//! What the package says of the program (D78): `--version` answers the version it was built as,
//! and writes nothing in the user's home.

use std::process::Command;

/// The version a package's build names (`KITOKEY_VERSION`), Cargo's when unset or empty: as the program reads it.
const VERSION: &str = match option_env!("KITOKEY_VERSION") {
    Some(version) if !version.is_empty() => version,
    _ => env!("CARGO_PKG_VERSION"),
};

#[test]
fn it_answers_its_version_and_writes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_kitokey"))
        .arg("--version")
        .env_clear()
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("kitokey {VERSION}\n"));
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0, "it wrote in the home");
}

#[test]
fn version_beside_another_argument_is_no_question_of_version() {
    for args in [
        &["--version", "extra"][..],
        &["extra", "--version"],
        &["--version", "--version"],
        &["-V"],
        &["--version=x"],
    ] {
        let home = tempfile::tempdir().unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_kitokey"))
            .args(args)
            .env_clear()
            .env("HOME", home.path())
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}: {output:?}");
        assert!(!String::from_utf8_lossy(&output.stdout).contains(VERSION), "{args:?}: {output:?}");
    }
}
