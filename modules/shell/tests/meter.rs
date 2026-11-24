//! A level meter's drawing from its readings (#1179): up at once, down eased, its peak held 0.8 s; for
//! who asks for less motion, a level held a second, in three steps, and no peak. The page's
//! `levelShown`, read from `ui/window.js` and run by Node over a sequence of readings.

use std::process::Command;

use serde_json::{Value, json};

/// Runs `levelShown` over `readings` — [reading, milliseconds, less motion] —, each from the one before.
fn shown(readings: &[(f64, f64, bool)]) -> Option<Vec<Value>> {
    let here = env!("CARGO_MANIFEST_DIR");
    let page = r#"
const fs = require("fs");
const src = fs.readFileSync(process.argv[1], "utf8");
const start = src.indexOf("function levelShown(");
if (start < 0) throw new Error("no levelShown in window.js");
let depth = 0, i = src.indexOf("{", start), end;
for (; i < src.length; i++) { if (src[i] === "{") depth++; else if (src[i] === "}") { depth--; if (depth === 0) { end = i + 1; break; } } }
const levelShown = new Function(src.slice(start, end) + "; return levelShown;")();
let was;
console.log(JSON.stringify(JSON.parse(process.argv[2]).map(([reading, now, still]) => (was = levelShown(was, reading, now, still)))));
"#;
    let asked = serde_json::to_string(&readings.iter().map(|(r, t, s)| json!([r, t, s])).collect::<Vec<_>>()).unwrap();
    let Ok(output) = Command::new("node")
        .args(["-e", page, &format!("{here}/ui/window.js"), &asked])
        .output()
    else {
        // As tests/model_said.rs: owed on a CI runner, which carries Node; said elsewhere.
        assert!(
            std::env::var_os("CI").is_none(),
            "node is not there on this CI runner: levelShown cannot be run"
        );
        eprintln!("node is not there: levelShown is not run");
        return None;
    };
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    Some(serde_json::from_slice(&output.stdout).unwrap())
}

fn widths(drawn: &[Value]) -> Vec<i64> {
    drawn.iter().map(|d| d["width"].as_i64().unwrap()).collect()
}

#[test]
fn a_level_rises_at_once_and_falls_eased() {
    let Some(drawn) = shown(&[(0.1, 0.0, false), (0.8, 100.0, false), (0.3, 200.0, false), (0.3, 300.0, false)]) else {
        return;
    };
    assert_eq!(widths(&drawn), [10, 80, 30, 30], "{drawn:?}");
    // The drawing is the reading; only a fall is eased, by the style's `.falling`.
    let falling: Vec<bool> = drawn.iter().map(|d| d["falling"].as_bool().unwrap()).collect();
    assert_eq!(falling, [false, false, true, false], "{drawn:?}");
}

#[test]
fn a_peak_stays_eight_tenths_of_a_second_then_follows() {
    let Some(drawn) = shown(&[
        (0.8, 0.0, false),
        (0.3, 500.0, false),
        (0.2, 790.0, false),
        (0.2, 900.0, false),
        (0.5, 1000.0, false),
    ]) else {
        return;
    };
    let peaks: Vec<i64> = drawn.iter().map(|d| d["peak"].as_i64().unwrap()).collect();
    assert_eq!(peaks, [80, 80, 80, 20, 50], "{drawn:?}");
}

#[test]
fn a_reading_out_of_range_is_drawn_inside_it() {
    let Some(drawn) = shown(&[(1.4, 0.0, false), (-0.2, 100.0, false)]) else {
        return;
    };
    assert_eq!(widths(&drawn), [100, 0], "{drawn:?}");
}

#[test]
fn less_motion_holds_a_level_a_second_in_three_steps_and_no_peak() {
    let Some(drawn) = shown(&[
        (0.9, 0.0, true),
        (0.1, 500.0, true),
        (0.1, 999.0, true),
        (0.1, 1000.0, true),
        (0.5, 2000.0, true),
        (0.3, 3000.0, true),
        (0.74, 4000.0, true),
        (0.76, 5000.0, true),
        (0.19, 6000.0, true),
    ]) else {
        return;
    };
    assert_eq!(widths(&drawn), [90, 90, 90, 0, 50, 50, 50, 90, 0], "{drawn:?}");
    assert!(drawn.iter().all(|d| d["peak"].is_null() && d["falling"] == false), "{drawn:?}");
}
