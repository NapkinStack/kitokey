//! A screenshot of the other theme than the desktop's puts that theme's roles on the page itself
//! (`shot_themed`, D89): `tokens(theme)` in `scripts/e2e_window.py` reads them from `ui/tokens.css`.
//! The light theme too, with the design system's roles and the window's older names, every name the
//! dark theme sets — or a light screen shot on a dark desktop keeps dark values, or raises (#1298,
//! marc). Played without Selenium, the function alone. Unix only, as the module's other tests that
//! drive its scripts.
#![cfg(unix)]

use std::process::Command;

/// `tokens` taken from the script and called for both themes; each theme's names and the light
/// values of two of them, in JSON.
const PLAY: &str = r#"
import ast, json, pathlib, re, sys
script = pathlib.Path(sys.argv[1])
tree = ast.parse(script.read_text())
taken = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "tokens"]
scope = {"re": re, "Path": pathlib.Path, "__file__": str(script)}
exec(compile(ast.fix_missing_locations(ast.Module(body=taken, type_ignores=[])), "e2e_window.py", "exec"), scope)
said = {}
for theme in ("light", "dark"):
    try:
        block = scope["tokens"](theme)
        said[theme] = {"names": sorted(set(re.findall(r"(--[\w-]+)\s*:", block))),
                       "values": dict(re.findall(r"(--[\w-]+)\s*:\s*([^;]+);", block)),
                       "scheme": "color-scheme" in block}
    except Exception as why:
        said[theme] = {"raised": repr(why)}
print(json.dumps(said))
"#;

#[test]
fn the_light_theme_put_on_a_page_sets_every_name_the_dark_one_sets_with_its_light_value() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py");
    let Ok(output) = Command::new("python3").args(["-I", "-c", PLAY, script]).output() else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: tokens() is not played");
        return;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "tokens() could not be played: {stderr}");
    let said = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(!said.contains("\"raised\""), "tokens() raised: {said}");
    let names = |theme: &str| -> Vec<String> {
        let part = said.split(&format!("\"{theme}\": {{\"names\": [")).nth(1).unwrap_or("");
        part.split(']')
            .next()
            .unwrap_or("")
            .split(", ")
            .map(|n| n.trim_matches('"').to_string())
            .filter(|n| !n.is_empty())
            .collect()
    };
    let (light, dark) = (names("light"), names("dark"));
    assert!(dark.len() > 40, "the dark theme's names were not read: {said}");
    let missing: Vec<&String> = dark.iter().filter(|name| !light.contains(name)).collect();
    assert!(
        missing.is_empty(),
        "names the dark theme sets and the light one does not: {missing:?}"
    );
    // The light values, the design system's and an older name's, as `ui/tokens.css` writes them.
    assert!(
        said.contains("\"--color-canvas\": \"var(--slate-25)\""),
        "--color-canvas is not light: {said}"
    );
    assert!(said.contains("\"--ground\": \"var(--gray-25)\""), "--ground is not light: {said}");
    // The theme's scheme is shot_themed's to say, not the block's.
    assert!(!said.contains("\"scheme\": true"), "a color-scheme is left in a block: {said}");
}
