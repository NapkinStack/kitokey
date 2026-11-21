//! What a package owes the authors of the crates it ships (D102): `THIRD-PARTY-NOTICES.html` names
//! every crate linked into the program on the systems it is built for, each under its licence's
//! text, and none that `Cargo.lock` no longer holds. Read offline: a crate added, removed or moved
//! to another version makes it red, and `about.toml`'s header says how to make it again.

use std::{collections::BTreeSet, process::Command};

/// The systems a package of kitokey-summary is built for: about.toml's `targets`.
const TARGETS: [&str; 3] = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin"];

/// Each crate `Cargo.lock` takes from a registry, as `name@version`: the package itself has no source.
fn locked() -> BTreeSet<String> {
    let lock = include_str!("../Cargo.lock");
    lock.split("[[package]]")
        .skip(1)
        .filter(|package| package.lines().any(|line| line.starts_with("source = ")))
        .map(|package| {
            let field = |key: &str| {
                package
                    .lines()
                    .find_map(|line| line.strip_prefix(key)?.strip_prefix(" = \"")?.strip_suffix('"'))
                    .unwrap_or_else(|| panic!("a package of Cargo.lock without its {key}: {package}"))
            };
            format!("{}@{}", field("name"), field("version"))
        })
        .collect()
}

/// Each crate linked into the program on those systems, as Cargo resolves it, offline: the normal
/// dependencies only, since what builds or tests it ships nothing.
fn shipped() -> BTreeSet<String> {
    let mut tree = Command::new(env!("CARGO"));
    tree.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "tree",
        "--locked",
        "--offline",
        "--edges",
        "normal",
        "--prefix",
        "none",
        "--format",
        "{p}",
    ]);
    for target in TARGETS {
        tree.args(["--target", target]);
    }
    let output = tree.output().unwrap();
    assert!(output.status.success(), "cargo tree: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with(env!("CARGO_PKG_NAME")))
        .map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next().unwrap();
            let version = words.next().unwrap().trim_start_matches('v');
            format!("{name}@{version}")
        })
        .collect()
}

/// Each crate the notices name, as `name@version`, from the `data-crate` about.hbs writes.
fn noticed() -> BTreeSet<String> {
    include_str!("../THIRD-PARTY-NOTICES.html")
        .split("data-crate=\"")
        .skip(1)
        .map(|rest| rest.split('"').next().unwrap().to_string())
        .collect()
}

#[test]
fn the_notices_name_every_crate_the_program_links() {
    let shipped = shipped();
    assert!(shipped.len() > 10, "cargo tree read as {} crates", shipped.len());
    let missing: Vec<_> = shipped.difference(&noticed()).cloned().collect();
    assert!(
        missing.is_empty(),
        "THIRD-PARTY-NOTICES.html misses crates the program links: {missing:?}. Make it again as about.toml's header says"
    );
}

#[test]
fn the_notices_name_no_crate_the_lock_no_longer_holds() {
    let stale: Vec<_> = noticed().difference(&locked()).cloned().collect();
    assert!(
        stale.is_empty(),
        "THIRD-PARTY-NOTICES.html names crates Cargo.lock no longer holds: {stale:?}. Make it again as about.toml's header says"
    );
}

#[test]
fn each_licence_in_the_notices_carries_its_text() {
    let notices = include_str!("../THIRD-PARTY-NOTICES.html");
    let texts: Vec<_> = notices
        .split("<pre class=\"licence-text\">")
        .skip(1)
        .map(|text| text.split("</pre>").next().unwrap())
        .collect();
    assert!(!texts.is_empty(), "no licence text in the notices");
    for text in texts {
        assert!(text.trim().len() > 100, "a licence without its text: {text:?}");
    }
}
