//! The recording bar's sizes (D90, the decider's boards of 2026-11-08): 440 CSS px wide, one row
//! of 44, its controls 28 — D89's rule for the bar —, on the 4 px grid.

const STYLE: &str = include_str!("../ui/window.css");
const SHELL: &str = include_str!("../src/main.rs");

/// Each rule of the style, its selector and its body, comments aside.
fn rules(style: &str) -> Vec<(String, String)> {
    let mut bare = String::new();
    let mut rest = style;
    while let Some(at) = rest.find("/*") {
        bare.push_str(&rest[..at]);
        rest = rest[at..].split_once("*/").map_or("", |(_, after)| after);
    }
    bare.push_str(rest);
    bare.split('}')
        .filter_map(|rule| rule.rsplit_once('{'))
        .map(|(selector, body)| (selector.trim().to_string(), body.to_string()))
        .collect()
}

fn px(body: &str, name: &str) -> Option<f64> {
    body.split(';')
        .filter_map(|one| one.split_once(':'))
        .rfind(|(key, _)| key.trim() == name)
        .and_then(|(_, value)| value.trim().strip_suffix("px")?.parse::<f64>().ok())
}

/// The bar's controls: its icon buttons, ● and ■.
const CONTROLS: [&str; 3] = ["#small .bar-row .icon-button", "#small .record", "#small .danger"];

/// Each of the bar's controls not 28 px square, and its row not 44 px high.
fn off_size(style: &str) -> Vec<String> {
    let all = rules(style);
    let mut off = Vec::new();
    for control in CONTROLS {
        match all.iter().find(|(selector, _)| selector == control) {
            Some((_, body)) if px(body, "min-height") == Some(28.0) && px(body, "width") == Some(28.0) => {}
            Some((_, body)) => off.push(format!("{control}: {body}")),
            None => off.push(format!("{control}: no rule")),
        }
    }
    let row = all.iter().find(|(selector, _)| selector == "#small .bar-row");
    if row.and_then(|(_, body)| px(body, "height")) != Some(44.0) {
        off.push(format!("#small .bar-row: {row:?}"));
    }
    // Any other control's height the bar sets, on the grid.
    off.extend(
        all.iter()
            .filter(|(selector, _)| selector.starts_with("#small"))
            .filter_map(|(selector, body)| {
                let set = px(body, "min-height")?;
                (set % 4.0 != 0.0).then(|| format!("{selector}: {set}"))
            }),
    );
    off
}

#[test]
fn the_bar_is_440_by_44_its_controls_28() {
    assert!(
        SHELL.contains("const BAR: (f64, f64) = (440.0, 44.0);"),
        "the bar's window is made 440 × 44"
    );
    // Without the system's title bar (board 8): GNOME counts it inside the size asked.
    let small = SHELL.split_once("fn show_small(").map_or("", |(_, after)| after);
    let small = small.split_once("\n}\n").map_or("", |(body, _)| body);
    assert!(small.contains(".decorations(false)"), "the bar is drawn without a title bar");
    assert!(SHELL.contains("window.start_dragging()"), "the bar is moved by its handle");
    assert_eq!(off_size(STYLE), Vec::<String>::new());
    // A control at the window's 32, a row grown, a height off the grid: each refused.
    for (old, new) in [
        (
            "#small .record { display: inline-grid; place-items: center; width: 28px; min-height: 28px;",
            "#small .record { display: inline-grid; place-items: center; width: 28px; min-height: 32px;",
        ),
        ("height: 44px; padding: 0 8px 0 12px;", "height: 48px; padding: 0 8px 0 12px;"),
        (
            "#small .bar-row .icon-button { width: 28px; min-height: 28px; }",
            "#small .bar-row .icon-button { width: 32px; min-height: 28px; }",
        ),
    ] {
        let mutated = STYLE.replacen(old, new, 1);
        assert_ne!(mutated, STYLE, "{old}");
        assert_eq!(off_size(&mutated).len(), 1, "{new}");
    }
    assert_eq!(off_size(&format!("{STYLE}\n#small .small-said {{ min-height: 30px; }}")).len(), 1);
}
