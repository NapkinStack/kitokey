//! D120, criterion 5: `scripts/design-export.py` writes `design-system/` from the code, the code staying the
//! source. `design-system/tokens.css` and `project/components/bundle.css` are `ui/`'s own, byte for byte: changed
//! without the export made again, they differ, and this test says so. `design-system/tokens.json` is the W3C
//! Design Tokens format; `design-system/project/` is what Claude Design's « Design System » reads as it is
//! (`editing: code`): its `tokens.json` in lists, each token's CSS variable `--<name>`, which `bundle.css` reads.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A file of the module, its line ends as Unix writes them; one the export should have written is said missing.
fn read(path: &str) -> String {
    std::fs::read_to_string(here().join(path))
        .unwrap_or_else(|_| panic!("{path} is missing: run `python3 scripts/design-export.py` in modules/shell"))
        .replace("\r\n", "\n")
}

fn json(path: &str) -> Value {
    serde_json::from_str(&read(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

const STALE: &str = "the export is not the code's: run `python3 scripts/design-export.py` in modules/shell";

#[test]
fn the_export_is_the_codes_own() {
    assert!(
        read("design-system/tokens.css") == read("ui/tokens.css"),
        "design-system/tokens.css: {STALE}"
    );
    assert!(
        read("design-system/project/components/bundle.css") == read("ui/components.css"),
        "project/components/bundle.css: {STALE}"
    );
    for generated in ["tokens.css", "manifest.json", "api"] {
        assert!(
            !here().join("design-system/project").join(generated).exists(),
            "project/{generated} is the page's to write"
        );
    }
}

/// The CSS's declarations, light then dark: `:root`'s, then Settings' dark laid over them.
fn css_themes(css: &str) -> [BTreeMap<String, String>; 2] {
    let mut bare = String::new();
    let mut rest = css;
    while let Some((before, after)) = rest.split_once("/*") {
        bare.push_str(before);
        rest = after.split_once("*/").map_or("", |(_, tail)| tail);
    }
    bare.push_str(rest);
    let mut light = BTreeMap::new();
    let mut dark = BTreeMap::new();
    for rule in bare.split('}') {
        let Some((selector, body)) = rule.rsplit_once('{') else { continue };
        let into = match selector.trim() {
            ":root" => &mut light,
            ":root[data-theme=\"dark\"]" => &mut dark,
            _ => continue,
        };
        for (name, value) in body.split(';').filter_map(|one| one.split_once(':')) {
            if let Some(name) = name.trim().strip_prefix("--") {
                into.insert(name.to_string(), value.trim().to_string());
            }
        }
    }
    let dark = light.clone().into_iter().chain(dark).collect();
    [light, dark]
}

/// A value of the CSS as hex: `#RRGGBB`, `var(--…)` followed, `rgb(r g b / a)` as `#RRGGBBAA`.
fn css_hex(text: &str, theme: &BTreeMap<String, String>) -> Option<String> {
    let text = text.trim();
    if let Some(name) = text.strip_prefix("var(--").and_then(|rest| rest.strip_suffix(')')) {
        return css_hex(theme.get(name)?, theme);
    }
    if text.starts_with('#') {
        return Some(text.to_uppercase());
    }
    let inside = text.strip_prefix("rgb(")?.strip_suffix(')')?;
    let (channels, alpha) = inside.split_once('/')?;
    let channels: Vec<u8> = channels.split_whitespace().filter_map(|c| c.parse().ok()).collect();
    let alpha = (alpha.trim().parse::<f64>().ok()? * 255.0).round() as u8;
    (channels.len() == 3).then(|| format!("#{:02X}{:02X}{:02X}{alpha:02X}", channels[0], channels[1], channels[2]))
}

/// A token's value in a theme, its aliases `{name}` followed through `tokens`.
fn token_hex(value: &Value, theme: &str, tokens: &BTreeMap<String, Value>) -> Option<String> {
    let value = match value {
        Value::Object(themes) => themes.get(theme)?,
        other => other,
    };
    let text = value.as_str()?;
    match text.strip_prefix('{').and_then(|rest| rest.strip_suffix('}')) {
        Some(name) => token_hex(tokens.get(name)?, theme, tokens),
        None => Some(text.to_uppercase()),
    }
}

/// Each family's tokens of `project/tokens.json`, by name.
fn project_tokens(project: &Value) -> BTreeMap<String, (String, Value)> {
    let mut all = BTreeMap::new();
    let families = project.as_object().expect("project/tokens.json is an object");
    for (family, content) in families {
        let Some(tokens) = content.get("tokens").and_then(Value::as_array) else {
            continue;
        };
        for token in tokens {
            let name = token["name"]
                .as_str()
                .unwrap_or_else(|| panic!("{family}: a token without its name"))
                .to_string();
            let earlier = all.insert(name.clone(), (family.clone(), token["value"].clone()));
            assert!(earlier.is_none(), "{name} is named twice");
        }
    }
    all
}

#[test]
fn each_variable_a_component_reads_is_a_token_of_the_design_system() {
    let project = json("design-system/project/tokens.json");
    let tokens = project_tokens(&project);
    let mut made: BTreeSet<String> = tokens.keys().cloned().collect();
    if let Some(families) = project["type"]["families"].as_object() {
        made.extend(families.keys().map(|key| format!("font-{key}")));
    }
    for name in &made {
        let first = name.chars().next().unwrap_or(' ');
        assert!(
            first.is_ascii_alphanumeric() && name.len() <= 64 && name.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c)),
            "{name} is no name Claude Design takes"
        );
    }
    let read: BTreeSet<String> = read("ui/components.css")
        .split("var(--")
        .skip(1)
        .filter_map(|rest| rest.split([')', ',']).next())
        .map(|name| name.trim().to_string())
        .collect();
    let missing: Vec<&String> = read.difference(&made).collect();
    assert!(
        missing.is_empty(),
        "components.css reads what no token of project/tokens.json makes: {missing:?}"
    );
}

#[test]
fn no_value_the_page_cannot_read() {
    let project = json("design-system/project/tokens.json");
    let mut wrong = Vec::new();
    for (name, (_, value)) in project_tokens(&project) {
        let texts: Vec<String> = match &value {
            Value::Object(themes) => themes.values().filter_map(|v| v.as_str().map(String::from)).collect(),
            other => other.as_str().map(String::from).into_iter().collect(),
        };
        for text in texts {
            if ["var(", "url(", "color-mix(", "transparent"]
                .iter()
                .any(|refused| text.contains(refused))
                || text.is_empty()
            {
                wrong.push(format!("{name}: {text}"));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// The two formats say what `ui/tokens.css` says, colour by colour, in both themes.
#[test]
fn the_two_formats_say_the_colours_of_the_code() {
    let css = read("ui/tokens.css");
    let themes = css_themes(&css);
    let project = json("design-system/project/tokens.json");
    let tokens: BTreeMap<String, Value> = project_tokens(&project)
        .into_iter()
        .filter(|(_, (family, _))| family == "color")
        .map(|(name, (_, value))| (name, value))
        .collect();
    let dtcg = json("design-system/tokens.json");
    let dtcg_tokens = dtcg["color"].as_object().expect("tokens.json (W3C): its « color » group").clone();
    let mut wrong = Vec::new();
    // The primitives (a hex) and the roles (`--color-*`); the window's older names are no token of the export.
    let colours: Vec<&String> = themes[0]
        .iter()
        .filter(|(name, value)| name.starts_with("color-") || value.starts_with('#'))
        .map(|(name, _)| name)
        .collect();
    assert!(colours.len() > 40, "{} colours read from tokens.css", colours.len());
    for name in colours {
        for (index, theme) in ["light", "dark"].into_iter().enumerate() {
            let code = css_hex(&themes[index][name], &themes[index]);
            let page = tokens.get(name.as_str()).and_then(|value| token_hex(value, theme, &tokens));
            let w3c = w3c_hex(name, theme, &dtcg_tokens);
            if code.is_none() || code != page || code != w3c {
                wrong.push(format!("{name} ({theme}): code {code:?}, project {page:?}, W3C {w3c:?}"));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// A W3C token's value in a theme — its `$value`, or in the dark the `com.kitokey` extension's when it has one —,
/// its aliases `{color.<name>}` followed.
fn w3c_hex(name: &str, theme: &str, tokens: &serde_json::Map<String, Value>) -> Option<String> {
    let token = tokens.get(name)?;
    assert_eq!(token["$type"], "color", "{name}: its $type");
    let dark = &token["$extensions"]["com.kitokey"]["dark"];
    let value = if theme == "dark" && !dark.is_null() {
        dark
    } else {
        &token["$value"]
    };
    let text = value.as_str()?;
    match text.strip_prefix("{color.").and_then(|rest| rest.strip_suffix('}')) {
        Some(other) => w3c_hex(other, theme, tokens),
        None => Some(text.to_uppercase()),
    }
}

#[test]
fn the_index_its_book_and_its_components() {
    let index = json("design-system/project/design-system.json");
    assert_eq!(index["v"], 3);
    assert_eq!(index["editing"], "code", "the page is the code's, read only");
    assert_eq!(index["docs"]["readme"], "project/README.md");
    // Made by CI, it says the run and who started it (`GITHUB_ACTOR`); made on a workstation, the commit alone and
    // nobody: a published file names no person (D120 2b).
    let via = index["lastChange"]["via"].as_str().unwrap_or_default();
    let by = &index["lastChange"]["by"];
    let commit = match via.strip_prefix("CI · NapkinStack/kitokey@") {
        Some(commit) => {
            assert!(by.as_str().is_some_and(|by| !by.is_empty()), "lastChange.by: {by}");
            commit
        }
        None => {
            let commit = via.strip_prefix("local · ").unwrap_or_else(|| panic!("lastChange.via: {via}"));
            assert!(by.is_null(), "made on a workstation, lastChange names {by}");
            commit
        }
    };
    assert!(
        commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()),
        "lastChange.via's commit: {commit}"
    );
    let book = read("design-system/project/README.md");
    assert!(book.contains(commit), "project/README.md names the commit the export was made from");
    let project = json("design-system/project/tokens.json");
    for font in project["type"]["fonts"].as_array().expect("type.fonts") {
        let file = font["file"].as_str().unwrap_or_default();
        assert!(here().join("design-system/project").join(file).is_file(), "type.fonts: {file}");
    }
    for component in [
        "Button", "Banner", "Dialog", "EngineCard", "Chip", "OptionCard", "Progress", "Toast", "Cover",
    ] {
        let folder = Path::new("design-system/project/components").join(component);
        let preview = read(folder.join("preview.html").to_str().unwrap());
        let first = preview.lines().next().unwrap_or_default();
        assert!(
            first.starts_with("<!-- @dsCard group=\"") && first.contains(" height="),
            "{component}: {first}"
        );
        assert!(!preview.contains("http"), "{component}'s preview reaches the network");
        // A banner carries its icon, as the gallery draws it.
        if component == "Banner" {
            assert_eq!(preview.matches("<svg").count(), 3, "each banner's icon");
        }
        if component != "Cover" {
            let about = read(folder.join("README.md").to_str().unwrap());
            assert!(
                about.trim().split_once(". ").is_some_and(|(summary, _)| !summary.is_empty()),
                "{component}: its summary"
            );
        }
    }
}
