//! D69: the page draws its icons, and each theme's text reads on what it is written on, from the files.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use oxc_allocator::Allocator;
use oxc_ast::ast::{AssignmentExpression, AssignmentTarget, CallExpression, Expression, StringLiteral, TemplateElement};
use oxc_ast_visit::{Visit, walk};
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

/// What the page sets as a style's value: `x.style.borderColor = …`, `x.style.cssText = …`,
/// `x.style.setProperty(name, …)` — its texts, where a word is a value and not a class (#739).
#[derive(Default)]
struct StyleValues(Vec<String>);

/// A text written in the code as it is: a string, or a template's parts joined.
fn written(expression: &Expression) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.as_str().unwrap_or_default().to_string()),
        Expression::TemplateLiteral(template) => Some(
            template
                .quasis
                .iter()
                .map(|part| part.value.raw.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        _ => None,
    }
}

/// Whether an expression is some element's `.style`.
fn a_style(expression: &Expression) -> bool {
    matches!(expression, Expression::StaticMemberExpression(member) if member.property.name == "style")
}

impl<'a> Visit<'a> for StyleValues {
    fn visit_assignment_expression(&mut self, assigned: &AssignmentExpression<'a>) {
        if let AssignmentTarget::StaticMemberExpression(member) = &assigned.left
            && a_style(&member.object)
            && let Some(text) = written(&assigned.right)
        {
            // A property set alone gives its value; `cssText` gives its declarations.
            self.0.push(if member.property.name == "cssText" {
                text
            } else {
                format!("{}: {text}", member.property.name)
            });
        }
        walk::walk_assignment_expression(self, assigned);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = &call.callee
            && member.property.name == "setProperty"
            && a_style(&member.object)
            && let Some(text) = call.arguments.get(1).and_then(|value| value.as_expression()).and_then(written)
        {
            let name = call
                .arguments
                .first()
                .and_then(|name| name.as_expression())
                .and_then(written)
                .unwrap_or_default();
            self.0.push(format!("{name}: {text}"));
        }
        walk::walk_call_expression(self, call);
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
const PAIRS: [(&str, &str, f64); 28] = [
    ("var(--ink)", "var(--ground)", 4.5),
    ("var(--ink)", "var(--surface)", 4.5),
    ("var(--ink)", "var(--raised)", 4.5),
    ("var(--soft)", "var(--ground)", 4.5),
    ("var(--soft)", "var(--surface)", 4.5),
    ("var(--soft)", "var(--raised)", 4.5),
    ("var(--ink)", SELECTED, 4.5),
    // A secondary hovered, pressed (#1094): its words still read.
    ("var(--ink)", "var(--hover)", 4.5),
    ("var(--ink)", "var(--pressed)", 4.5),
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
    // The word a search found (#1078): its mark seen where it is, on a row selected too (WCAG 2.2 1.4.11,
    // lina, #1093) — by its edge, the ink's underline in light under a pale yellow, the amber itself in dark —,
    // and its words read on it.
    ("var(--mark-edge)", SELECTED, 3.0),
    ("var(--mark-edge)", "var(--surface)", 3.0),
    ("var(--mark-ink)", "var(--mark)", 4.5),
    // A level's bars, each side's in its colour, on the page's ground as on the surface (#1179, 1.4.11).
    ("var(--ok)", "var(--ground)", 3.0),
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

/// A level meter (#1179, « B · Trace »): its bars rounded, each never less than a dot, drawn at each reading with no
/// transition, the newest at the right and the oldest cut at the left; the bar's trace 14 px high, the page's 24,
/// the panel's 28; each side's bars in its colour, on no tinted track. Less motion: the global rule, and the page
/// draws a held level (tests/meter.rs).
#[test]
fn a_meter_is_a_trace_of_rounded_bars_in_the_style() {
    // A Windows checkout ends its lines with CRLF (#1042): read as LF.
    let style = without_css_comments(STYLE).replace("\r\n", "\n");
    let rule = |selector: &str| {
        style
            .split(selector)
            .nth(1)
            .and_then(|part| part.split('}').next())
            .unwrap_or_else(|| panic!("no {selector}"))
            .to_string()
    };
    let trace = rule("\n.trace {");
    assert!(
        trace.contains("justify-content: flex-end") && trace.contains("overflow: hidden"),
        "{trace}"
    );
    let bar = rule("\n.trace > i {");
    assert!(
        bar.contains("min-height: 4px") && bar.contains("border-radius") && !bar.contains("transition"),
        "{bar}"
    );
    for (track, height) in [
        ("#small .meter .track {", 14),
        (".page-meter > .page-level {", 24),
        (".check-track {", 28),
    ] {
        assert!(rule(track).contains(&format!("height: {height}px")), "{track}{}", rule(track));
    }
    assert!(
        style.contains("#level-others > i, .page-level.others > i, .check-source.far .check-track > i { background: var(--ok); }"),
        "the meeting's bars are green otherwise: say so here"
    );
    // The soft bar is gone, with its fill, its tick and its easing.
    for gone in ["\n.peak {", "\n.falling {", ".check-fill", "color-mix(in srgb, var(--accent) 14%"] {
        assert!(!style.contains(gone), "{gone}");
    }
    assert!(style.contains(
        "prefers-reduced-motion: reduce) {\n  *, *::before, *::after { animation: none !important; transition: none !important; }"
    ));
}

#[test]
fn the_search_mark_draws_its_edge() {
    // The edge is what a selected row sees of the mark (#1093): the rule that draws the mark draws it.
    let drawn = rules(STYLE)
        .into_iter()
        .any(|(selector, body)| selector.trim() == "mark" && body.contains("var(--mark-edge)"));
    assert!(drawn, "the mark's rule does not draw var(--mark-edge)");
}

/// Each rule of the style, its selector and its body, comments aside.
fn rules(style: &str) -> Vec<(String, String)> {
    without_css_comments(style)
        .split('}')
        .filter_map(|rule| rule.rsplit_once('{'))
        .map(|(selector, body)| (selector.trim().to_string(), body.to_string()))
        .collect()
}

/// What draws a recording, the one use of the red (D89): the header's button and the name's dot while it
/// records, a row's live chip, the bar's ● and ■, its handle's dot while it records (D90).
const RECORDING: [&str; 7] = [
    ".brand.live #name-dot",
    ".chip.live",
    ".act .primary.live",
    "#small .handle.live .dot",
    "#small .record",
    "#small .record .record-dot",
    ".danger",
];

/// CSS Color 4's named colours, its table of 148 (https://www.w3.org/TR/css-color-4/#named-colors):
/// a colour written by its name is a value as `#RRGGBB` is, `crimson` a red as `red` is.
const NAMED_COLOURS: &str = "\
aliceblue antiquewhite aqua aquamarine azure beige bisque black blanchedalmond blue blueviolet brown burlywood \
cadetblue chartreuse chocolate coral cornflowerblue cornsilk crimson cyan darkblue darkcyan darkgoldenrod \
darkgray darkgreen darkgrey darkkhaki darkmagenta darkolivegreen darkorange darkorchid darkred darksalmon \
darkseagreen darkslateblue darkslategray darkslategrey darkturquoise darkviolet deeppink deepskyblue dimgray \
dimgrey dodgerblue firebrick floralwhite forestgreen fuchsia gainsboro ghostwhite gold goldenrod gray green \
greenyellow grey honeydew hotpink indianred indigo ivory khaki lavender lavenderblush lawngreen lemonchiffon \
lightblue lightcoral lightcyan lightgoldenrodyellow lightgray lightgreen lightgrey lightpink lightsalmon \
lightseagreen lightskyblue lightslategray lightslategrey lightsteelblue lightyellow lime limegreen linen \
magenta maroon mediumaquamarine mediumblue mediumorchid mediumpurple mediumseagreen mediumslateblue \
mediumspringgreen mediumturquoise mediumvioletred midnightblue mintcream mistyrose moccasin navajowhite navy \
oldlace olive olivedrab orange orangered orchid palegoldenrod palegreen paleturquoise palevioletred papayawhip \
peachpuff peru pink plum powderblue purple rebeccapurple red rosybrown royalblue saddlebrown salmon sandybrown \
seagreen seashell sienna silver skyblue slateblue slategray slategrey snow springgreen steelblue tan teal \
thistle tomato turquoise violet wheat white whitesmoke yellow yellowgreen";

/// CSS Color 4's system colours (https://www.w3.org/TR/css-color-4/#css-system-colors): the system's
/// palette, not the theme's — a value as a name is (#739).
const SYSTEM_COLOURS: [&str; 19] = [
    "accentcolor",
    "accentcolortext",
    "activetext",
    "buttonborder",
    "buttonface",
    "buttontext",
    "canvas",
    "canvastext",
    "field",
    "fieldtext",
    "graytext",
    "highlight",
    "highlighttext",
    "linktext",
    "mark",
    "marktext",
    "selecteditem",
    "selecteditemtext",
    "visitedtext",
];

/// CSS Color 4's functions that write a colour by value; `color-mix` mixes what it is given, tokens here.
const COLOUR_FUNCTIONS: [&str; 10] = [
    "rgb(", "rgba(", "hsl(", "hsla(", "hwb(", "lab(", "lch(", "oklab(", "oklch(", "color(",
];

/// CSS Color 4's deprecated system colours (§6.2), which browsers still draw: a value as the others.
/// Some are words a rule says otherwise (`background`, `menu`): read in a value alone.
const DEPRECATED_SYSTEM_COLOURS: [&str; 23] = [
    "activeborder",
    "activecaption",
    "appworkspace",
    "background",
    "buttonhighlight",
    "buttonshadow",
    "captiontext",
    "inactiveborder",
    "inactivecaption",
    "inactivecaptiontext",
    "infobackground",
    "infotext",
    "menu",
    "menutext",
    "scrollbar",
    "threeddarkshadow",
    "threedface",
    "threedhighlight",
    "threedlightshadow",
    "threedshadow",
    "window",
    "windowframe",
    "windowtext",
];

/// The properties whose value names other properties, never a colour: `transition: background 0.2s`.
const NAMING_PROPERTIES: [&str; 3] = ["transition", "transition-property", "will-change"];

/// Whether a value names one of the system's colours, current or deprecated, as a word of its own.
fn names_a_system_colour(value: &str) -> bool {
    value
        .to_lowercase()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .any(|word| SYSTEM_COLOURS.contains(&word) || DEPRECATED_SYSTEM_COLOURS.contains(&word))
}

/// Whether a rule's declarations — or a text the page sets, `border-color: Highlight` or `Highlight`
/// alone — give a system colour as a value: never in a property's name, nor where a value names
/// properties (#739).
fn sets_a_system_colour(declarations: &str) -> bool {
    declarations.split(';').any(|one| match one.split_once(':') {
        Some((key, value)) => !NAMING_PROPERTIES.contains(&key.trim().to_lowercase().as_str()) && names_a_system_colour(value),
        None => {
            let word = one.trim().to_lowercase();
            SYSTEM_COLOURS.contains(&word.as_str()) || DEPRECATED_SYSTEM_COLOURS.contains(&word.as_str())
        }
    })
}

/// Whether a text writes a colour by one of CSS Color 4's functions: `color(` alone, never `color-mix(`.
fn writes_a_colour(text: &str) -> bool {
    let lower = text.to_lowercase();
    COLOUR_FUNCTIONS.iter().any(|f| {
        lower
            .match_indices(f)
            .any(|(at, _)| !lower[..at].ends_with(|c: char| c.is_ascii_alphanumeric() || c == '-'))
    })
}

/// Whether a text names a colour among its words, a word keeping its hyphens: `white-space` names none.
fn names_a_colour(text: &str) -> bool {
    static NAMED: LazyLock<BTreeSet<&str>> = LazyLock::new(|| NAMED_COLOURS.split_whitespace().collect());
    text.to_lowercase()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .any(|word| NAMED.contains(word))
}

/// The rules that are no recording's and still draw red: by the token, by its value written out, by
/// a colour's name — any of CSS Color 4's — or function. Outside the theme's tokens, a colour is a
/// token: a value written in a rule — `#C2352D` on a border — is refused whatever it is, but a shadow's black.
fn red_outside_a_recording(style: &str) -> Vec<String> {
    rules(style)
        .into_iter()
        .filter(|(selector, _)| !selector.ends_with(":root") && !RECORDING.contains(&selector.as_str()))
        .filter(|(_, body)| {
            let body = body.to_lowercase();
            let literal = names_a_colour(&body)
                || sets_a_system_colour(&body)
                || body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '#')).any(|word| {
                    word.starts_with('#') && [4, 5, 7, 9].contains(&word.len()) && word[1..].chars().all(|c| c.is_ascii_hexdigit())
                });
            // Each function's arguments, but a shadow's black.
            let function = writes_a_colour(&body)
                && COLOUR_FUNCTIONS.iter().any(|f| {
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
            hex || names_a_colour(&lower) || lower.contains("--red") || writes_a_colour(&lower)
        })
        .chain({
            // A system colour, where the page sets a style's value (#739).
            let mut styles = StyleValues::default();
            styles.visit_program(&parsed.program);
            styles.0.into_iter().filter(|set| sets_a_system_colour(set))
        })
        .collect()
}

/// Dark chosen in Settings whatever the system says (D98) is the system's dark itself, token for
/// token: a palette of its own would drift from the one the thresholds above are read on.
#[test]
fn the_dark_chosen_in_settings_is_the_systems_dark() {
    let forced = STYLE
        .split_once("[data-theme=\"dark\"]:root {")
        .unwrap()
        .1
        .split_once('}')
        .unwrap()
        .0;
    let [_, dark] = themes(STYLE);
    assert_eq!(tokens(forced), dark);
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
    for red in [
        "#C2352D",
        "#c2352d",
        "red",
        "crimson",
        "Tomato",
        "darkred",
        "firebrick",
        "rgb(194, 53, 45)",
        "var(--red)",
        // The system's colours and every function of CSS Color 4 (#739).
        "AccentColor",
        "ButtonHighlight",
        "Window",
        "Background",
        "oklch(0.55 0.2 27)",
        "lab(45 60 40)",
        "color(srgb 0.76 0.21 0.18)",
    ] {
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
        "row.style.color = \"crimson\";",
        "row.style.cssText = \"border-color: tomato\";",
        "row.style.setProperty(\"background\", `indianred`);",
        "row.style.setProperty(\"color\", \"var(--red)\");",
        "row.style.borderColor = `oklch(0.55 0.2 27)`;",
        // The system's colours set by the page, current or deprecated (#739).
        "row.style.borderColor = \"Highlight\";",
        "row.style.cssText = \"border-color: ButtonHighlight\";",
        "row.style.setProperty(\"background\", \"Window\");",
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
const SMALLER: [(&str, f64); 6] = [
    (".choice input, .row input[type=\"checkbox\"]", 20.0),
    // A switch's track, inside its 32 px setting.
    (".setting input[type=\"checkbox\"]", 20.0),
    (".remember input", 20.0),
    (".freeable input", 20.0),
    (".line-time", 24.0),
    // The language's list inside its 32 px setting: no minimum of its own, read since #739.
    (".setting select", 0.0),
];

/// Each property that sets a box's height: one of them at another than its control's height contradicts it.
const HEIGHTS: [&str; 6] = [
    "min-height",
    "height",
    "max-height",
    "block-size",
    "min-block-size",
    "max-block-size",
];

/// A height as the rule sets it, in px: a `rem` is 16 px; any other unit or keyword is said as it is
/// written, a control's height being a length the grid can check (#739).
fn height(body: &str, name: &str) -> Option<Result<f64, String>> {
    let (_, value) = body
        .split(';')
        .filter_map(|one| one.split_once(':'))
        .rfind(|(key, _)| key.trim() == name)?;
    let value = value.trim();
    let read = |unit: &str, scale: f64| value.strip_suffix(unit)?.parse::<f64>().ok().map(|n| n * scale);
    let zero = (value == "0").then_some(0.0);
    Some(
        zero.or_else(|| read("px", 1.0))
            .or_else(|| read("rem", 16.0))
            .ok_or_else(|| value.to_string()),
    )
}

fn px(body: &str, name: &str) -> Option<f64> {
    height(body, name)?.ok()
}

/// Whether a rule is the small window's (D90's), every selector of it in the bar: `#small` and what
/// it holds, its `.small-` parts, the page drawn as the bar — not `#small-link`, the frame's button (#739).
fn of_the_bar(selector: &str) -> bool {
    selector.split(',').all(|one| {
        let one = one.trim();
        one == "#small" || one.starts_with("#small ") || one.starts_with(".small-") || one.starts_with("body.small-window")
    })
}

/// The ids the page's frame gives to what is no control. Every other id is taken for a control's — the
/// frame's buttons and fields, and each id the script gives —, so that a control named by its id
/// (`#record`) is measured too.
fn ids_of_no_control(html: &str) -> Vec<String> {
    html.split('<')
        .filter_map(|tag| {
            let name = tag.split([' ', '>', '\n']).next()?;
            let id = tag.split('>').next()?.split(" id=\"").nth(1)?.split('"').next()?;
            (!["button", "input", "select", "textarea", "a"].contains(&name)).then(|| format!("#{id}"))
        })
        .collect()
}

/// Whether one compound of a selector (`.lines .correcting:hover`'s last) names a control's part.
fn names_a_control(selector: &str) -> bool {
    static NO_CONTROL: LazyLock<Vec<String>> = LazyLock::new(|| ids_of_no_control(HTML));
    selector.split(',').any(|one| {
        let last = one.trim().rsplit([' ', '>']).next().unwrap_or_default();
        let id = last.split_once('#').map(|(_, id)| {
            let end = id
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                .unwrap_or(id.len());
            format!("#{}", &id[..end])
        });
        if id.is_some_and(|id| !NO_CONTROL.contains(&id)) {
            return true;
        }
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
        .filter(|(selector, _)| !of_the_bar(selector) && names_a_control(selector))
        .filter_map(|(selector, body)| {
            let allowed = SMALLER.iter().find(|(name, _)| *name == selector).map_or(32.0, |(_, px)| *px);
            let set: Vec<String> = HEIGHTS
                .iter()
                .filter_map(|name| height(&body, name))
                .filter(|set| *set != Ok(allowed))
                .map(|set| set.map_or_else(|written| written, |px| px.to_string()))
                .collect();
            (!set.is_empty()).then(|| format!("{selector}: {}", set.join(", ")))
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
        "#record { min-height: 40px; }",
        ".sidebar #find:focus { height: 28px; }",
        // The frame's button, whose id holds « small » and is no part of the bar; a height in rem (#739).
        "#small-link { min-height: 40px; }",
        ".menu-item { height: 2.5rem; }",
        ".menu-item { height: 3em; }",
    ] {
        assert_eq!(controls_not_32(&format!("{STYLE}\n{wrong}")).len(), 1, "{wrong}");
    }
    // A height that contradicts the 32 px its control keeps, in the control's own rule.
    for (property, set) in [
        ("height", "40px"),
        ("block-size", "40px"),
        ("max-height", "24px"),
        ("max-block-size", "24px"),
    ] {
        let mutated = STYLE.replacen(
            ".primary { min-height: 32px;",
            &format!(".primary {{ min-height: 32px; {property}: {set};"),
            1,
        );
        assert_ne!(mutated, STYLE);
        assert_eq!(
            controls_not_32(&mutated),
            vec![format!(".primary: {}", set.trim_end_matches("px"))],
            "{property}"
        );
    }
    assert!(
        names_a_control(".step select")
            && names_a_control("#small .link:hover")
            && !names_a_control(".linked")
            && !names_a_control(".open-row span")
            && names_a_control("#record")
            && names_a_control(".more #more-actions:hover")
            && !names_a_control("#welcome")
            && !names_a_control("#list")
    );
    assert!(ids_of_no_control(HTML).contains(&"#groups".to_string()) && !ids_of_no_control(HTML).contains(&"#find".to_string()));
    // Every height the window sets is on the grid; the small window's are D90's.
    let off: Vec<String> = all
        .iter()
        .filter(|(selector, _)| !of_the_bar(selector))
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
        // #1094: a field may rest on the line when its bottom edge holds the 3:1 (Fluent 2's fields).
        let edged_below = body.contains("border-bottom-color: var(--edge)");
        assert!(
            !(bordered_by_line && a_control && !edged_below),
            "{selector} borders a control with --line alone"
        );
    }
    // A field is edged: all round, or at its bottom (#1094); a secondary button is not framed at all.
    for control in [".search", "select", ".field input", ".context-text", ".setting"] {
        let edged = rules(STYLE)
            .into_iter()
            .any(|(selector, body)| selector.split(',').any(|one| one.trim() == control) && body.contains("var(--edge)"));
        assert!(edged, "{control}");
    }
}

/// The value the last rule naming `selector` alone in its list gives `name`, comments aside.
fn declared(style: &str, selector: &str, name: &str) -> Option<String> {
    rules(style)
        .into_iter()
        .filter(|(selectors, _)| selectors.split(',').any(|one| one.trim() == selector))
        .filter_map(|(_, body)| {
            body.split(';')
                .filter_map(|one| one.split_once(':'))
                .rfind(|(key, _)| key.trim() == name)
                .map(|(_, value)| value.trim().to_string())
        })
        .next_back()
}

/// #1094: the window's controls in a hierarchy, after Fluent 2's appearances and GNOME's HIG — one
/// primary filled; a secondary on the raised ground, on the line with its bottom edged as Windows 11's
/// controls are (a signifier at rest: ines' challenge of #1096, NN/g's weak signifiers), answering the
/// pointer and the press; an icon button bare until hovered, pressed too.
#[test]
fn a_secondary_rests_on_the_line_its_bottom_edged_and_answers_the_pointer_and_the_press() {
    let missing: Vec<String> = [
        (".open", "border-color", "var(--line)"),
        (".open", "border-bottom-color", "var(--edge)"),
        (".open", "background", "var(--raised)"),
        (".open:hover", "background", "var(--hover)"),
        (".open:active", "background", "var(--pressed)"),
        (".icon-button:hover", "background", "var(--raised)"),
        (".icon-button:active", "background", "var(--pressed)"),
    ]
    .into_iter()
    .filter(|(selector, name, value)| declared(STYLE, selector, name).as_deref() != Some(*value))
    .map(|(selector, name, value)| format!("{selector} {name}: {:?}, not {value}", declared(STYLE, selector, name)))
    .collect();
    assert_eq!(missing, Vec::<String>::new());
}

/// #1094: a field rests on the divider's line, its bottom edge at 3:1 (WCAG 2.2 1.4.11) as Fluent 2's
/// fields do; hovered or focused, it is edged all round.
#[test]
fn a_field_rests_on_the_line_its_bottom_edged() {
    let mut missing = Vec::new();
    for field in ["select", ".search", ".field input", ".picks input", ".setting"] {
        for (name, value) in [("border-color", "var(--line)"), ("border-bottom-color", "var(--edge)")] {
            if declared(STYLE, field, name).as_deref() != Some(value) {
                missing.push(format!("{field} {name}: {:?}, not {value}", declared(STYLE, field, name)));
            }
        }
    }
    for state in ["select:hover", "select:focus", ".search:hover", ".search:focus", ".setting:hover"] {
        if declared(STYLE, state, "border-color").as_deref() != Some("var(--edge)") {
            missing.push(format!(
                "{state} border-color: {:?}, not var(--edge)",
                declared(STYLE, state, "border-color")
            ));
        }
    }
    assert_eq!(missing, Vec::<String>::new());
}

/// #1094: a step's model on its line is a quiet list — no frame, no ground until hovered, a chevron
/// after it (the language's setting's, board 4) — and the line reads in the text's face, not as a log.
#[test]
fn a_makers_line_reads_in_the_text_face_its_model_a_quiet_list() {
    let maker = declared(STYLE, ".maker", "font").unwrap_or_default();
    assert!(!maker.contains("monospace"), ".maker is set in {maker}");
    for (name, value) in [("border-color", "transparent"), ("background", "transparent")] {
        assert_eq!(
            declared(STYLE, ".maker .page-model", name).as_deref(),
            Some(value),
            ".maker .page-model {name}"
        );
    }
    assert_eq!(
        declared(STYLE, ".maker .page-model:hover", "background").as_deref(),
        Some("var(--raised)")
    );
    assert!(
        PAGE.contains("lineWith") && PAGE.contains("pick.tagName === \"SELECT\""),
        "the list's chevron is not drawn"
    );
}

/// #1094: the focus ring is the one D69 drew, unchanged: 3 px of the accent, 2 px off.
#[test]
fn the_focus_ring_is_unchanged() {
    assert_eq!(
        declared(STYLE, "button:focus-visible", "outline").as_deref(),
        Some("3px solid var(--accent)")
    );
    assert_eq!(declared(STYLE, "button:focus-visible", "outline-offset").as_deref(), Some("2px"));
}

/// D120: `ui/tokens.css` alone holds a value. What the rest of the style writes out — a colour, or a
/// size, a radius, a shadow or a duration not read from a `var(--…)` — is counted, each place said.
const OUTSIDE: &str = include_str!("../ui/window.css");
/// The count is a ratchet: it may only go down, to zero at cycle 12's closure (D120).
const RATCHET: &str = include_str!("look-outside-tokens.txt");
/// The properties whose value is a size, a radius, a shadow or a duration (D120).
const SIZED: [&str; 23] = [
    "font", "font-size", "border-radius", "border-top-left-radius", "border-top-right-radius",
    "border-bottom-left-radius", "border-bottom-right-radius", "box-shadow", "transition",
    "transition-duration", "transition-delay", "animation", "animation-duration", "animation-delay",
    "margin", "margin-top", "margin-right", "margin-bottom", "margin-left", "padding", "gap", "row-gap",
    "column-gap",
];
/// The literals that are no choice of a design: nothing, a hairline, a focus offset, a whole, a half.
const PLAIN: [&str; 5] = ["0", "1px", "2px", "100%", "50%"];

fn outside_the_tokens(style: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (selector, body) in rules(style) {
        for (property, value) in body.split(';').filter_map(|one| one.split_once(':')) {
            let (property, value) = (property.trim(), value.trim().to_lowercase());
            if property.starts_with("--") {
                found.push(format!("{selector}: {property} is a token outside tokens.css"));
                continue;
            }
            let words: Vec<&str> =
                value.split(|c: char| c.is_whitespace() || ",()/".contains(c)).filter(|w| !w.is_empty()).collect();
            for word in &words {
                let hex = word.starts_with('#') && [4, 5, 7, 9].contains(&word.len()) && word[1..].chars().all(|c| c.is_ascii_hexdigit());
                if hex || ["rgb", "rgba", "hsl", "hsla", "color-mix"].contains(word) {
                    found.push(format!("{selector}: {property}: {word}"));
                }
            }
            if !SIZED.contains(&property) {
                continue;
            }
            // What a var() reads is a token; its fallback is not.
            let mut bare = value.clone();
            while let Some(at) = bare.find("var(") {
                let end = bare[at..].find(')').map_or(bare.len(), |end| at + end + 1);
                bare.replace_range(at..end, " ");
            }
            for word in bare.split(|c: char| c.is_whitespace() || ",()/".contains(c)).filter(|w| !w.is_empty()) {
                let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic() || c == '%');
                let (number, unit) = (digits.trim_start_matches('-'), &word[digits.len()..]);
                if number.parse::<f64>().is_err() || (unit.is_empty() && property != "font-size") {
                    continue;
                }
                if !PLAIN.contains(&word.trim_start_matches('-')) && number.parse::<f64>() != Ok(0.0) {
                    found.push(format!("{selector}: {property}: {word}"));
                }
            }
        }
    }
    found
}

#[test]
fn only_the_tokens_hold_a_value() {
    let found = outside_the_tokens(OUTSIDE);
    let ratchet: usize = RATCHET.trim().parse().expect("look-outside-tokens.txt holds a count");
    for one in &found {
        println!("outside tokens.css: {one}");
    }
    assert!(found.len() <= ratchet, "{} values outside tokens.css, the ratchet allows {ratchet}: read them with --nocapture", found.len());
    assert!(found.len() >= ratchet, "{} values outside tokens.css: lower tests/look-outside-tokens.txt to it", found.len());
}

#[test]
fn a_value_outside_the_tokens_is_counted() {
    let style = ".a { color: #FFF; margin: 0 4px; transition: opacity 0.2s; }\n\
                 .b { color: var(--ink); padding: var(--space-2); border-radius: 50%; border: 1px solid var(--line); }\n\
                 .c { background: color-mix(in srgb, var(--accent) 13%, var(--surface)); --x: #000000; font: 15px/1.45 system-ui; }";
    let found = outside_the_tokens(style);
    for expected in [".a: color: #fff", ".a: margin: 4px", ".a: transition: 0.2s", ".c: background: color-mix", ".c: --x", ".c: font: 15px"] {
        assert!(found.iter().any(|one| one.starts_with(expected)), "{expected}: {found:?}");
    }
    assert!(!found.iter().any(|one| one.starts_with(".b")), "{found:?}");
    assert_eq!(found.len(), 6, "{found:?}");
}
