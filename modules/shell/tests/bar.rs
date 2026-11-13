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

/// Every property that sizes a box: each one a control's rule sets must say 28 (#742).
const SIZES: [&str; 6] = ["height", "min-height", "max-height", "width", "min-width", "max-width"];

/// Whether a selector reaches one of the bar's controls: under `#small`, its last part a button,
/// an icon button, ● or ■ — the text field's own clear button aside, 24 by design.
fn reaches_a_control(selector: &str) -> bool {
    let last = selector.rsplit(' ').next().unwrap_or("");
    let mut parts = last.split('.');
    let element = parts.next().unwrap_or("");
    selector.starts_with("#small")
        && !selector.contains(".next-context")
        && (element == "button" || parts.any(|class| ["icon-button", "record", "danger"].contains(&class)))
}

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
    // Any rule reaching a control, in any order: a later one wins, and one size beside another too.
    // A rule said off above is said once.
    for (selectors, body) in &all {
        let said = format!("{selectors}: ");
        if !off.iter().any(|one| one.starts_with(&said))
            && selectors.split(',').map(str::trim).any(reaches_a_control)
            && SIZES.iter().any(|size| px(body, size).is_some_and(|set| set != 28.0))
        {
            off.push(format!("{selectors}: {body}"));
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

/// A function's body in the shell, its comments removed: what is compiled, not what is said.
fn body(shell: &str, name: &str) -> String {
    // Checked out with CRLF on Windows: the function's end is read on `\n` alone.
    let shell = shell.replace("\r\n", "\n");
    let after = shell.split_once(&format!("fn {name}(")).map_or("", |(_, after)| after);
    let mut text = after.split_once("\n}\n").map_or("", |(body, _)| body).to_string();
    while let Some(at) = text.find("/*") {
        let end = text[at..].find("*/").map_or(text.len(), |end| at + end + 2);
        text.replace_range(at..end, "");
    }
    text.lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n")
}

/// What the shell does to the bar's window that is not what D90 drew.
fn shell_off(shell: &str) -> Vec<String> {
    let mut off = Vec::new();
    if !shell.contains("const BAR: (f64, f64) = (440.0, 44.0);") {
        off.push("the bar's window is not made 440 × 44".to_string());
    }
    let small = body(shell, "show_small");
    // Its size the constant's, never a number beside it.
    if !small.contains(".inner_size(BAR.0, BAR.1)") {
        off.push("show_small: not made at BAR".to_string());
    }
    // Without the system's title bar (board 8): GNOME counts it inside the size asked.
    if !small.contains(".decorations(false)") || small.contains(".decorations(true)") {
        off.push("show_small: drawn with a title bar".to_string());
    }
    // As tall as it holds, between its row and BAR_TALLEST.
    if !body(shell, "bar_fit").contains("height.clamp(BAR.1, BAR_TALLEST)") {
        off.push("bar_fit: not bounded by its row and BAR_TALLEST".to_string());
    }
    if !body(shell, "bar_drag").contains("window.start_dragging()") {
        off.push("bar_drag: the bar is not moved by its handle".to_string());
    }
    off
}

#[test]
fn the_bar_is_440_by_44_its_controls_28() {
    assert_eq!(shell_off(SHELL), Vec::<String>::new());
    // The title bar commented or turned on, a width beside BAR, the bar not bounded, not moved:
    // each refused (#742).
    for (old, new) in [
        (".decorations(false)", "// .decorations(false)"),
        (".decorations(false)", ".decorations(true) // was .decorations(false)"),
        (".inner_size(BAR.0, BAR.1)", ".inner_size(448.0, BAR.1)"),
        ("height.clamp(BAR.1, BAR_TALLEST)", "height"),
        ("height.clamp(BAR.1, BAR_TALLEST)", "height.clamp(BAR.1, BAR.1)"),
        ("let _ = window.start_dragging();", "// let _ = window.start_dragging();"),
    ] {
        let mutated = SHELL.replacen(old, new, 1);
        assert_ne!(mutated, SHELL, "{old}");
        assert_eq!(shell_off(&mutated).len(), 1, "{new}");
    }
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
    // ● given a height beside its min-height, a later rule growing the icon buttons (#742).
    let mutated = STYLE.replacen(
        "width: 28px; min-height: 28px; padding: 0; border: 2px",
        "width: 28px; min-height: 28px; height: 40px; padding: 0; border: 2px",
        1,
    );
    assert_ne!(mutated, STYLE);
    assert_eq!(off_size(&mutated).len(), 1, "height: 40px on ●");
    assert_eq!(
        off_size(&format!("{STYLE}\n#small .bar-row button.icon-button {{ min-height: 32px }}")).len(),
        1
    );
}
