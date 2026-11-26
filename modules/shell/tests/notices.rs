//! What a package owes the authors of the crates it ships (D102): `THIRD-PARTY-NOTICES.html` names
//! every crate linked into the program on the systems it is built for, each under its licence's
//! text, and none that `Cargo.lock` no longer holds. A crate added, removed or moved to another
//! version makes it red, and `about.toml`'s header says how to make it again.

use std::{collections::BTreeSet, process::Command};

/// The systems a package of kitokey-shell is built for: about.toml's `targets`.
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

/// Each crate linked into the program on those systems, as Cargo resolves it: the normal
/// dependencies only, neither a procedural macro nor what only it uses: what builds, tests or
/// runs at compile time ships nothing. Not offline: a build fetches
/// only its own system's crates, and Cargo reads the others' to resolve them, fetching each once
/// at the version the lock holds (`--locked`).
fn shipped() -> BTreeSet<String> {
    let mut tree = Command::new(env!("CARGO"));
    tree.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "tree",
        "--locked",
        "--edges",
        "normal,no-proc-macro",
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

/// What the window draws that is no crate: Lucide's icons and Montserrat's outlines. Their licences
/// are written into about.hbs from `ui/LUCIDE-LICENSE` and `ui/MONTSERRAT-OFL.txt`, and must stay
/// those files, word for word — line ends aside: a Windows checkout writes them as \r\n.
#[test]
fn the_notices_carry_lucides_and_montserrats_licences_as_their_files_say() {
    let notices = include_str!("../THIRD-PARTY-NOTICES.html").replace("\r\n", "\n");
    for (file, text) in [
        ("ui/LUCIDE-LICENSE", include_str!("../ui/LUCIDE-LICENSE")),
        ("ui/MONTSERRAT-OFL.txt", include_str!("../ui/MONTSERRAT-OFL.txt")),
        ("ui/BRICOLAGE-OFL.txt", include_str!("../ui/BRICOLAGE-OFL.txt")),
    ] {
        let escaped = text
            .replace("\r\n", "\n")
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        assert!(
            notices.contains(&format!("<pre class=\"licence-text\">{escaped}</pre>")),
            "{file} is not in the notices as it is written: copy it into about.hbs, then make the page again"
        );
    }
}

/// No GNU licence's text on the page — GPL, LGPL, AGPL, FDL: about.toml accepts none, and a crate's
/// sources may carry one for parts it does not build (libflac-sys, for FLAC's programs). Each opens
/// with the sentence below; MPL-2.0, which names the GPL as a Secondary License, does not.
#[test]
fn the_notices_hold_no_gnu_licence() {
    let notices: String = include_str!("../THIRD-PARTY-NOTICES.html")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let gnu = "Everyone is permitted to copy and distribute verbatim copies of this license document";
    assert!(
        !notices.contains(gnu),
        "the notices hold a GNU licence's text: about.toml's clarify, then make the page again"
    );
}

/// No licence text is source code: cargo-about, when a crate's notice is in no licence file or a
/// clarify's checksum no longer matches, falls back on reading whole sources (ring, libflac-sys).
#[test]
fn the_notices_hold_no_source_code() {
    let notices = include_str!("../THIRD-PARTY-NOTICES.html");
    for text in notices.split("<pre class=\"licence-text\">").skip(1) {
        let text = text.split("</pre>").next().unwrap();
        let code = text.lines().map(str::trim_start).find(|line| {
            [
                "#include", "#define", "#ifdef", "#ifndef", "pub fn ", "fn ", "impl ", "use ", "static ", "extern ",
            ]
            .iter()
            .any(|start| line.starts_with(start))
        });
        assert!(
            code.is_none(),
            "the notices hold source code, « {} »: a clarify in about.toml, then make the page again",
            code.unwrap_or_default()
        );
    }
}
