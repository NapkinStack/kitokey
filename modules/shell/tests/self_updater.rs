//! The Store updates the package; nothing in it updates itself (D102, criterion 2). The trial holds it on
//! Windows' installed package (the install module's trial, #1082); this holds it at every
//! build, on every platform: the shell's `Cargo.lock` resolves no crate that replaces the program.

/// The crates that update or replace a running program, by their name in a lock.
const SELF_UPDATERS: [&str; 4] = ["tauri-plugin-updater", "self_update", "self-replace", "velopack"];

/// The packages of `lock` named in `SELF_UPDATERS`.
fn self_updaters(lock: &str) -> Vec<&str> {
    lock.lines()
        .filter_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
        .filter(|name| SELF_UPDATERS.contains(name))
        .collect()
}

#[test]
fn the_shell_resolves_no_self_updater() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock");
    let lock = std::fs::read_to_string(path).unwrap();
    assert!(
        lock.contains("name = \"tauri\""),
        "{path} read, but no `tauri` in it: not the shell's lock"
    );
    let found = self_updaters(&lock);
    assert!(
        found.is_empty(),
        "{path} resolves {found:?}: the Store updates the package, nothing in it may (D102)"
    );
}

#[test]
fn a_lock_with_a_self_updater_is_refused() {
    let lock = "[[package]]\nname = \"tauri\"\nversion = \"2.12.3\"\n\n\
                [[package]]\nname = \"tauri-plugin-updater\"\nversion = \"2.14.1\"\n";
    assert_eq!(self_updaters(lock), ["tauri-plugin-updater"]);
}
