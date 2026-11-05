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

/// Any other argument starts the shell itself: here only where the cleared environment leaves it no
/// display, so that it stops at once — on Windows or a Mac it would open, and the run wait for it.
#[cfg(target_os = "linux")]
#[test]
fn version_beside_another_argument_is_no_question_of_version() {
    use std::{
        process::Stdio,
        time::{Duration, Instant},
    };
    for args in [
        &["--version", "extra"][..],
        &["extra", "--version"],
        &["--version", "--version"],
        &["-V"],
        &["--version=x"],
    ] {
        let home = tempfile::tempdir().unwrap();
        let mut run = Command::new(env!("CARGO_BIN_EXE_kitokey"))
            .args(args)
            .env_clear()
            .env("HOME", home.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let asked = Instant::now();
        while run.try_wait().unwrap().is_none() && asked.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = run.kill();
        let output = run.wait_with_output().unwrap();
        assert!(!output.status.success(), "{args:?}: {output:?}");
        assert!(!String::from_utf8_lossy(&output.stdout).contains(VERSION), "{args:?}: {output:?}");
    }
}
