//! A model is said in the shell as the page says it (D91): `words::model_said` is the page's
//! `modelSaid`, read here from `ui/window.js` and run by Node on the same ids — a divergence on any
//! of them fails, unicode and edge cases included (the verifier's 89 ids, and more).

use std::{fs, process::Command};

use kitokey_shell::words;

#[test]
fn every_id_is_said_by_the_shell_exactly_as_the_page_says_it() {
    let here = env!("CARGO_MANIFEST_DIR");
    let ids: Vec<String> = serde_json::from_str(&fs::read_to_string(format!("{here}/tests/said/model_ids.json")).unwrap()).unwrap();
    assert!(ids.len() >= 89, "{} ids", ids.len());
    let page = r#"
const fs = require("fs");
const src = fs.readFileSync(process.argv[1], "utf8");
const start = src.indexOf("function modelSaid(model) {");
if (start < 0) throw new Error("no modelSaid in window.js");
let depth = 0, i = src.indexOf("{", start), end;
for (; i < src.length; i++) { if (src[i] === "{") depth++; else if (src[i] === "}") { depth--; if (depth === 0) { end = i + 1; break; } } }
const modelSaid = new Function(src.slice(start, end) + "; return modelSaid;")();
console.log(JSON.stringify(JSON.parse(fs.readFileSync(process.argv[2], "utf8")).map(modelSaid)));
"#;
    // Where Node is not there, the page's side cannot be read: said, and the shell's side still never panics.
    let run = Command::new("node")
        .args([
            "-e",
            page,
            &format!("{here}/ui/window.js"),
            &format!("{here}/tests/said/model_ids.json"),
        ])
        .output();
    let Ok(output) = run else {
        // On a CI runner the comparison is owed: GitHub's Ubuntu and Windows images, where the shell's
        // tests run, carry Node. Elsewhere — a workstation without it — said, and not compared.
        assert!(
            std::env::var_os("CI").is_none(),
            "node is not there on this CI runner: the page's modelSaid cannot be compared"
        );
        eprintln!("node is not there: the page's modelSaid is not compared");
        for id in &ids {
            words::model_said(id);
        }
        return;
    };
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let said: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
    let differ: Vec<String> = ids
        .iter()
        .zip(&said)
        .filter(|(id, page)| words::model_said(id) != **page)
        .map(|(id, page)| format!("{id:?}: page {page:?}, shell {:?}", words::model_said(id)))
        .collect();
    assert!(
        differ.is_empty(),
        "{} of {} differ:\n{}",
        differ.len(),
        ids.len(),
        differ.join("\n")
    );
}
