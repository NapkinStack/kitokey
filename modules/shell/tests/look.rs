//! D69: the page draws its icons, and each theme's text reads on what it is written on, from the files.

use std::collections::BTreeMap;

use oxc_allocator::Allocator;
use oxc_ast::ast::{StringLiteral, TemplateElement};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

const PAGE: &str = include_str!("../ui/window.js");
const STYLE: &str = include_str!("../ui/window.css");
const HTML: &str = include_str!("../ui/index.html");
/// The glyphs a font would draw as an icon (UX review §8), and the back arrow beside them; the
/// sidebar's fold arrows and a line's pencil, Lucide's since D89.
const GLYPHS: [char; 10] = ['›', '‹', '⤢', '✓', '✔', '→', '×', '⇤', '⇥', '✎'];
/// The same, written otherwise.
const ESCAPES: [&str; 12] = [
    "\\203A", "\\2039", "\\2922", "\\2713", "\\2714", "\\2192", "\\D7", "&times;", "&rarr;", "&rsaquo;", "&lsaquo;", "&#",
];

#[derive(Default)]
struct Texts(Vec<String>);

impl<'a> Visit<'a> for Texts {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.0.push(literal.value.as_str().unwrap_or_default().to_string());
    }

    fn visit_template_element(&mut self, element: &TemplateElement<'a>) {
        self.0.push(element.value.raw.as_str().to_string());
    }
}

fn without_css_comments(mut css: &str) -> String {
    let mut kept = String::new();
    while let Some((before, rest)) = css.split_once("/*") {
        kept.push_str(before);
        css = rest.split_once("*/").map_or("", |(_, after)| after);
    }
    kept + css
}

fn glyphs_in_page(page: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, page, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "the page does not parse");
    let mut texts = Texts::default();
    texts.visit_program(&parsed.program);
    texts.0.into_iter().filter(|text| text.contains(GLYPHS)).collect()
}

#[test]
fn no_glyph_stands_for_an_icon_in_the_page() {
    assert_eq!(glyphs_in_page(PAGE), Vec::<String>::new());
    let style = without_css_comments(STYLE);
    assert!(!style.contains(GLYPHS), "a glyph in the style");
    assert!(!HTML.contains(GLYPHS), "a glyph in the page's frame");
    for escape in ESCAPES {
        assert!(!style.to_uppercase().contains(&escape.to_uppercase()), "{escape} in the style");
        assert!(!HTML.contains(escape), "{escape} in the page's frame");
    }
    assert_eq!(
        glyphs_in_page(r#"const a = `✓ ${b}`; const c = "→"; const d = `✓`; // ✓ in a comment"#).len(),
        3
    );
}

#[test]
fn nothing_is_fetched_for_an_icon() {
    let style = without_css_comments(STYLE);
    assert!(
        !style.contains("url("),
        "the page's policy refuses it, and a test of a value would not see it"
    );
    assert!(!HTML.contains("src=\"http") && !HTML.contains("href=\"http"));
    assert!(include_str!("../ui/LUCIDE-LICENSE").contains("ISC License"));
}

/// The one rule whose opacity stays: a line's pencil, no text, faint until its line is hovered or
/// focused (D62), each state then at full strength.
const FADED_ON_PURPOSE: &str = ".lines .edit";

type Rgb = [f64; 3];

fn hex(text: &str) -> Rgb {
    let digit = |at: usize| f64::from(u8::from_str_radix(&text[at..at + 2], 16).unwrap());
    [digit(1), digit(3), digit(5)]
}

/// The tokens in one block: one this test cannot read is refused, or it would say nothing of all that use it.
fn tokens(block: &str) -> BTreeMap<String, Rgb> {
    block
        .split(';')
        .filter_map(|one| one.split_once(':'))
        .filter(|(name, _)| name.trim().starts_with("--"))
        .map(|(name, value)| {
            let value = value.trim();
            assert!(value.len() == 7 && value.starts_with('#'), "{name}: {value} is no #RRGGBB");
            (name.trim().to_string(), hex(value))
        })
        .collect()
}

/// Each theme's tokens, read from the file.
fn themes(style: &str) -> [BTreeMap<String, Rgb>; 2] {
    let light = style.split_once(":root {").unwrap().1.split_once('}').unwrap().0;
    let dark = style
        .split_once("prefers-color-scheme: dark")
        .unwrap()
        .1
        .split_once(":root {")
        .unwrap()
        .1
        .split_once('}')
        .unwrap()
        .0;
    [tokens(light), tokens(dark)]
}

/// A value of the style in a theme: `#RRGGBB`, a token, or `color-mix` of two as the style writes it.
fn value(text: &str, theme: &BTreeMap<String, Rgb>) -> Option<Rgb> {
    let text = text.trim().trim_end_matches("!important").trim();
    if text.len() == 7 && text.starts_with('#') {
        return Some(hex(text));
    }
    if let Some(name) = text.strip_prefix("var(").and_then(|rest| rest.strip_suffix(')')) {
        return theme.get(name).copied();
    }
    let mix = text.strip_prefix("color-mix(in srgb,")?.strip_suffix(')')?;
    let (first, second) = mix.split_once(',')?;
    let (first, share) = first.trim().rsplit_once(' ')?;
    let share = share.strip_suffix('%')?.parse::<f64>().ok()? / 100.0;
    let (a, b) = (value(first, theme)?, value(second, theme)?);
    Some([0, 1, 2].map(|at| a[at] * share + b[at] * (1.0 - share)))
}

fn luminance(colour: Rgb) -> f64 {
    let [r, g, b] = colour.map(|channel| {
        let c = channel / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Each rule's text on its background; a rule naming no background is read on the three the window
/// is built of. 4.5:1; 3:1 from 24 px. What it cannot read in a rule that names a text's colour is
/// reported too, never passed over. `inherit` and `currentColor` name none, `transparent` and `none`
/// no background. A background alone says nothing of the text inherited over it: a bar holds none.
fn too_faint(style: &str) -> Vec<String> {
    let style = without_css_comments(style);
    let mut faint = Vec::new();
    for (index, theme) in themes(&style).iter().enumerate() {
        for rule in style.split('}') {
            let Some((selector, body)) = rule.rsplit_once('{') else { continue };
            let selector = selector.trim();
            // The last one is what holds.
            let property = |name: &str| {
                body.split(';')
                    .filter_map(|one| one.split_once(':'))
                    .rfind(|(key, _)| key.trim() == name)
                    .map(|(_, found)| found.trim().to_string())
            };
            let colour = property("-webkit-text-fill-color").or_else(|| property("color"));
            let Some(colour) = colour else { continue };
            if ["inherit", "currentColor", "transparent"].contains(&colour.as_str()) {
                continue;
            }
            let Some(colour) = value(&colour, theme) else {
                faint.push(format!("{selector}: its colour {colour} cannot be read"));
                continue;
            };
            if selector != FADED_ON_PURPOSE
                && property("opacity").is_some_and(|opacity| opacity.parse::<f64>().is_ok_and(|opacity| opacity < 1.0))
            {
                faint.push(format!("{selector}: text with an opacity cannot be read"));
                continue;
            }
            let large = property("font-size")
                .and_then(|size| size.strip_suffix("px").and_then(|px| px.parse::<f64>().ok()))
                .is_some_and(|px| px >= 24.0);
            let need = if large { 3.0 } else { 4.5 };
            let ground = property("background-color")
                .or_else(|| property("background"))
                .filter(|text| !["transparent", "none", "inherit"].contains(&text.as_str()));
            let grounds: Vec<Rgb> = match ground {
                Some(own) => match value(own.split_whitespace().next().unwrap_or_default(), theme).or_else(|| value(&own, theme)) {
                    Some(own) => vec![own],
                    None => {
                        faint.push(format!("{selector}: its background {own} cannot be read"));
                        continue;
                    }
                },
                None => ["--ground", "--surface", "--raised"].map(|name| theme[name]).to_vec(),
            };
            for ground in grounds {
                let ratio = contrast(colour, ground);
                if ratio < need {
                    faint.push(format!("theme {index}, {selector}: {ratio:.2}"));
                }
            }
        }
    }
    faint
}

#[test]
fn each_themes_text_reaches_aa_on_what_it_is_written_on() {
    assert_eq!(too_faint(STYLE), Vec::<String>::new());
}

#[test]
fn a_pair_below_aa_is_refused() {
    let style = ":root { --ground: #FFFFFF; --surface: #FFFFFF; --raised: #FFFFFF; --ink: #000000; --pale: #BBBBBB; }\n\
                 @media (prefers-color-scheme: dark) { :root { --ground: #000000; --surface: #000000; --raised: #000000; --ink: #FFFFFF; --pale: #444444; } }\n\
                 .good { color: var(--ink); }\n.bad { color: var(--pale); }\n.big { color: var(--pale); font-size: 30px; }";
    let faint = too_faint(style);
    assert!(faint.iter().any(|line| line.contains(".bad")), "{faint:?}");
    assert!(!faint.iter().any(|line| line.contains(".good")), "{faint:?}");
    assert!((contrast([0.0; 3], [255.0; 3]) - 21.0).abs() < 0.01);
    let unread = ":root { --ground: #FFFFFF; --surface: #FFFFFF; --raised: #FFFFFF; --ink: #000000; }\n\
                  @media (prefers-color-scheme: dark) { :root { --ground: #000000; --surface: #000000; --raised: #000000; --ink: #FFFFFF; } }\n\
                  .a { color: rgb(1, 2, 3); }\n.b { color: var(--ink); background: linear-gradient(red, blue); }\n.c { color: var(--ink); opacity: 0.2; }\n\
                  .d { color: var(--ink); -webkit-text-fill-color: #888888; background-color: #888888; }\n.e { color: var(--ink); color: #FEFEFE; background: #FFFFFF; }";
    let refused = too_faint(unread);
    for selector in [".a", ".b", ".c", ".d", ".e"] {
        assert!(refused.iter().any(|line| line.contains(selector)), "{selector}: {refused:?}");
    }
}

#[test]
fn what_moves_moves_shortly_and_not_for_who_asked_it_not_to() {
    let style = without_css_comments(STYLE);
    for selector in [".small-rest, .small-live {", ".turned {"] {
        let rule = style
            .split(selector)
            .skip(1)
            .map(|part| part.split('}').next().unwrap())
            .find(|rule| rule.contains("animation"))
            .unwrap_or_else(|| panic!("{selector}"));
        let animation = rule
            .trim()
            .strip_prefix("animation:")
            .unwrap_or_else(|| panic!("{selector}: {rule}"))
            .trim()
            .trim_end_matches(';');
        // One name, one length, one ease: nothing repeating, nothing waiting.
        let parts: Vec<&str> = animation.split_whitespace().collect();
        assert_eq!(parts.len(), 3, "{selector}: {animation}");
        let ms: u32 = parts[1]
            .strip_suffix("ms")
            .unwrap_or_else(|| panic!("{animation}"))
            .parse()
            .unwrap();
        assert!(ms <= 200, "{selector} {ms} ms");
    }
    assert!(style.contains("prefers-reduced-motion: reduce) { .small-rest, .small-live, .turned { animation: none; } }"));
}

/// Each pair of colours the theme uses (D89's first round of drawings, *Contrast, both themes*), read
/// from the file in both themes: text 4.5:1 (WCAG 2.2, 1.4.3), a control's edge or a meter 3:1 (1.4.11).
/// `--line` is no pair: it divides, and borders no control (the test below).
const PAIRS: [(&str, &str, f64); 22] = [
    ("var(--ink)", "var(--ground)", 4.5),
    ("var(--ink)", "var(--surface)", 4.5),
    ("var(--ink)", "var(--raised)", 4.5),
    ("var(--soft)", "var(--ground)", 4.5),
    ("var(--soft)", "var(--surface)", 4.5),
    ("var(--soft)", "var(--raised)", 4.5),
    ("var(--ink)", SELECTED, 4.5),
    ("var(--soft)", SELECTED, 4.5),
    ("var(--accent)", "var(--surface)", 4.5),
    ("var(--accent)", "var(--ground)", 4.5),
    ("var(--accent)", SELECTED, 4.5),
    // The primary's words, on the accent; Arrêter's, on the red.
    ("var(--surface)", "var(--accent)", 4.5),
    ("var(--surface)", "var(--red)", 4.5),
    ("var(--red)", "var(--surface)", 4.5),
    ("var(--ok)", "var(--surface)", 4.5),
    ("var(--accent)", "var(--raised)", 3.0),
    ("var(--ok)", "var(--raised)", 3.0),
    ("var(--edge)", "var(--surface)", 3.0),
    ("var(--edge)", "var(--ground)", 3.0),
    ("var(--edge)", "var(--raised)", 3.0),
    // The mark, in the accent, while recording its dot red.
    ("var(--accent)", "var(--surface)", 3.0),
    ("var(--red)", "var(--raised)", 3.0),
];
/// The selected row's ground, as the sidebar mixes it.
const SELECTED: &str = "color-mix(in srgb, var(--accent) 13%, var(--surface))";

fn below_their_threshold(style: &str) -> Vec<String> {
    let style = without_css_comments(style);
    let mut below = Vec::new();
    for (index, theme) in themes(&style).iter().enumerate() {
        for (front, ground, need) in PAIRS {
            let (Some(a), Some(b)) = (value(front, theme), value(ground, theme)) else {
                below.push(format!("theme {index}: {front} on {ground} cannot be read"));
                continue;
            };
            let ratio = contrast(a, b);
            if ratio < need {
                below.push(format!("theme {index}: {front} on {ground} {ratio:.2} < {need}"));
            }
        }
    }
    below
}

#[test]
fn each_pair_of_the_theme_reaches_its_threshold_in_both_themes() {
    assert!(STYLE.contains(SELECTED), "the selected row is mixed otherwise: say so here");
    assert_eq!(below_their_threshold(STYLE), Vec::<String>::new());
    // An edge as faint as a divider is refused, in the dark theme as in the light one.
    let faint = STYLE.replacen("--edge: #737B8C", "--edge: #343946", 1);
    assert!(
        below_their_threshold(&faint)
            .iter()
            .any(|line| line.starts_with("theme 1: var(--edge)")),
        "a faint edge passed"
    );
}

/// Each rule of the style, its selector and its body, comments aside.
fn rules(style: &str) -> Vec<(String, String)> {
    without_css_comments(style)
        .split('}')
        .filter_map(|rule| rule.rsplit_once('{'))
        .map(|(selector, body)| (selector.trim().to_string(), body.to_string()))
        .collect()
}

/// What draws a recording, the one use of the red (D89): the header's button and the mark while it
/// records, a row's live chip, the small window's dot and Arrêter.
const RECORDING: [&str; 5] = [".brand.live #mark-dot", ".chip.live", ".act .primary.live", ".live-dot", ".danger"];

/// The rules that are no recording's and still draw red: by the token, by its value written out, by
/// a colour's name or function. Outside the theme's tokens, a colour is a token: a value written in a
/// rule — `#C2352D` on a border — is refused whatever it is, but a shadow's black.
fn red_outside_a_recording(style: &str) -> Vec<String> {
    rules(style)
        .into_iter()
        .filter(|(selector, _)| !selector.ends_with(":root") && !RECORDING.contains(&selector.as_str()))
        .filter(|(_, body)| {
            let body = body.to_lowercase();
            let literal = body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '#')).any(|word| {
                word == "red"
                    || (word.starts_with('#') && [4, 5, 7, 9].contains(&word.len()) && word[1..].chars().all(|c| c.is_ascii_hexdigit()))
            });
            let function = ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|f| {
                body.replace(f, "\u{0}")
                    .split('\u{0}')
                    .skip(1)
                    .any(|args| !args.trim_start().starts_with("0 0 0"))
            });
            body.contains("var(--red)") || literal || function
        })
        .map(|(selector, _)| selector)
        .collect()
}

/// A colour the page would set by value, or the red token it would name: the page draws a state by a
/// class, never by a colour (D89).
fn colours_in_page(page: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, page, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "the page does not parse");
    let mut texts = Texts::default();
    texts.visit_program(&parsed.program);
    texts
        .0
        .into_iter()
        .filter(|text| {
            let lower = text.to_lowercase();
            let hex = lower
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '#'))
                .any(|word| word.starts_with('#') && [4, 7, 9].contains(&word.len()) && word[1..].chars().all(|c| c.is_ascii_hexdigit()));
            hex || lower.trim() == "red" || lower.contains("--red") || ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|f| lower.contains(f))
        })
        .collect()
}

#[test]
fn red_says_a_recording_and_nothing_else() {
    assert_eq!(red_outside_a_recording(STYLE), Vec::<String>::new());
    assert_eq!(colours_in_page(PAGE), Vec::<String>::new());
    assert_eq!(
        PAGE.matches("\"danger\"").count(),
        1,
        "the red button is the small window's Arrêter alone"
    );
    // The red written out on a failure's border, named, or by its function, is refused as the token is.
    for red in ["#C2352D", "#c2352d", "red", "rgb(194, 53, 45)", "var(--red)"] {
        let mutated = STYLE.replacen(
            ".making-state.failed { background: transparent; border: 1px solid var(--edge); }",
            &format!(".making-state.failed {{ background: transparent; border: 1px solid {red}; }}"),
            1,
        );
        assert_ne!(mutated, STYLE);
        assert_eq!(red_outside_a_recording(&mutated), vec![".making-state.failed".to_string()], "{red}");
    }
    // A shadow's black stays a shadow.
    assert!(red_outside_a_recording(".menu { box-shadow: 0 8px 24px rgb(0 0 0 / 0.25); }").is_empty());
    // Set by the page by value, or by the token.
    for set in [
        "mark.style.fill = \"#C2352D\";",
        "row.style.borderColor = \"#f00\";",
        "row.style.color = `rgb(194 53 45)`;",
        "row.style.color = \"red\";",
        "row.style.setProperty(\"color\", \"var(--red)\");",
    ] {
        assert_eq!(colours_in_page(&format!("{PAGE}\n{set}")).len(), 1, "{set}");
    }
}

/// The controls of the window and Settings (D89): 32 px high; the small window's are D90's. A
/// meeting's settings row and its « Plus d’actions » menu, board 4's. Each must be found, so that
/// none leaves the scan below by a renamed rule.
const CONTROLS: [&str; 12] = [
    ".open, .search",
    "select",
    ".picks input",
    ".field input",
    ".primary",
    ".danger",
    ".act .primary",
    ".sidebar .act .primary",
    ".icon-button",
    ".folder input",
    ".setting",
    ".menu-item",
];

/// What makes a rule a control's: an element a user presses or types in, or a class or id the page
/// gives one. A rule whose selector ends on one of them and sets a height sets 32 px.
const CONTROL_PARTS: [&str; 21] = [
    "input",
    "select",
    "textarea",
    "button",
    ".open",
    ".search",
    ".primary",
    ".danger",
    ".icon-button",
    ".edit",
    ".correcting",
    ".rename",
    ".naming",
    ".link",
    ".path",
    ".line-time",
    ".disclose",
    "#free-link",
    ".choice",
    ".setting",
    ".menu-item",
];

/// The heights a control keeps under 32 px, each with its reason: a box to tick, a switch's track
/// inside its setting, a time in a line of text that is a link and no button.
const SMALLER: [(&str, f64); 5] = [
    (".choice input, .row input[type=\"checkbox\"]", 20.0),
    // A switch's track, inside its 32 px setting.
    (".setting input[type=\"checkbox\"]", 20.0),
    (".remember input", 20.0),
    (".freeable input", 20.0),
    (".line-time", 24.0),
];

fn px(body: &str, name: &str) -> Option<f64> {
    body.split(';')
        .filter_map(|one| one.split_once(':'))
        .rfind(|(key, _)| key.trim() == name)
        .and_then(|(_, value)| value.trim().strip_suffix("px")?.parse::<f64>().ok())
}

/// Whether one compound of a selector (`.lines .correcting:hover`'s last) names a control's part.
fn names_a_control(selector: &str) -> bool {
    selector.split(',').any(|one| {
        let last = one.trim().rsplit([' ', '>']).next().unwrap_or_default();
        CONTROL_PARTS.iter().any(|part| {
            let at = if part.starts_with(['.', '#']) {
                last.match_indices(part).map(|(at, _)| at).collect::<Vec<_>>()
            } else if last.starts_with(part) {
                vec![0]
            } else {
                vec![]
            };
            at.into_iter()
                .any(|at| !last[at + part.len()..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        })
    })
}

/// Each rule of the window that names a control and sets its height, at another than 32 px.
fn controls_not_32(style: &str) -> Vec<String> {
    rules(style)
        .into_iter()
        .filter(|(selector, _)| !selector.contains("small") && names_a_control(selector))
        .filter_map(|(selector, body)| {
            let set = px(&body, "min-height").or_else(|| px(&body, "height"))?;
            let allowed = SMALLER.iter().find(|(name, _)| *name == selector).map_or(32.0, |(_, px)| *px);
            (set != allowed).then(|| format!("{selector}: {set}"))
        })
        .collect()
}

#[test]
fn controls_are_32_px_high_on_a_4_px_grid() {
    let all = rules(STYLE);
    for control in CONTROLS {
        let found = all
            .iter()
            .find(|(selector, _)| selector == control)
            .unwrap_or_else(|| panic!("{control}"));
        assert_eq!(px(&found.1, "min-height"), Some(32.0), "{control}");
    }
    for (selector, _) in SMALLER {
        assert!(
            all.iter().any(|(found, _)| found == selector),
            "{selector} is no longer in the style: take it out"
        );
    }
    assert_eq!(controls_not_32(STYLE), Vec::<String>::new());
    // A control the list above does not name is measured all the same.
    for wrong in [
        ".lines .correcting { min-height: 36px; }",
        ".foot nav #free-link { min-height: 28px; }",
        ".lines .edit { min-height: 40px; }",
    ] {
        assert_eq!(controls_not_32(&format!("{STYLE}\n{wrong}")).len(), 1, "{wrong}");
    }
    assert!(
        names_a_control(".step select")
            && names_a_control("#small .link:hover")
            && !names_a_control(".linked")
            && !names_a_control(".open-row span")
    );
    // Every height the window sets is on the grid; the small window's are D90's.
    let off: Vec<String> = all
        .iter()
        .filter(|(selector, _)| !selector.contains("small"))
        .filter_map(|(selector, body)| {
            px(body, "min-height")
                .filter(|px| px % 4.0 != 0.0)
                .map(|px| format!("{selector}: {px}"))
        })
        .collect();
    assert_eq!(off, Vec::<String>::new());
}

/// What a field, a list's choice or a switch is drawn with: never `--line`, which a ground hides
/// (1.36:1); `--edge` instead.
const FIELDS: [&str; 8] = [
    "input",
    "select",
    "textarea",
    ".open",
    ".search",
    ".choice",
    ".context-text",
    ".setting",
];

#[test]
fn a_control_is_edged_and_a_line_only_divides() {
    for (selector, body) in rules(STYLE) {
        let bordered_by_line = body
            .split(';')
            .filter_map(|one| one.split_once(':'))
            .any(|(key, value)| key.trim().starts_with("border") && value.contains("var(--line)"));
        let a_control = selector
            .split(',')
            .any(|one| FIELDS.iter().any(|field| one.trim().split(' ').any(|part| part.starts_with(field))));
        assert!(!(bordered_by_line && a_control), "{selector} borders a control with --line");
    }
    for control in [".open, .search", "select", ".field input", ".context-text", ".setting"] {
        let found = rules(STYLE).into_iter().find(|(selector, _)| selector == control).unwrap();
        assert!(found.1.contains("var(--edge)"), "{control}");
    }
}
