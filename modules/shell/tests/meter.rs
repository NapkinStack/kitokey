//! A level meter's drawing from its readings (#1179, « B · Trace »): its last readings as bars, the newest at the
//! right, moving left a bar a reading; for who asks for less motion, a level held a second, in three steps, the same
//! on every bar. The page's `levelShown`, read from `ui/window.js` and run by Node over a sequence of readings.

use std::process::Command;

use serde_json::{Value, json};

/// Runs `levelShown` over `readings` — [reading, milliseconds, less motion] —, each from the one before, on a track
/// of `count` bars.
fn shown(count: usize, readings: &[(f64, f64, bool)]) -> Option<Vec<Value>> {
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
const count = Number(process.argv[3]);
console.log(JSON.stringify(JSON.parse(process.argv[2]).map(([reading, now, still]) => (was = levelShown(was, reading, now, still, count)))));
"#;
    let asked = serde_json::to_string(&readings.iter().map(|(r, t, s)| json!([r, t, s])).collect::<Vec<_>>()).unwrap();
    let Ok(output) = Command::new("node")
        .args(["-e", page, &format!("{here}/ui/window.js"), &asked, &count.to_string()])
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

fn bars(drawn: &Value) -> Vec<i64> {
    drawn["bars"].as_array().unwrap().iter().map(|b| b.as_i64().unwrap()).collect()
}

#[test]
fn a_trace_draws_its_last_readings_the_newest_at_the_right() {
    let Some(drawn) = shown(4, &[(0.1, 0.0, false), (0.8, 100.0, false), (0.3, 200.0, false)]) else {
        return;
    };
    // From an empty track, each reading a bar at the right, the ones before it moved left.
    assert_eq!(bars(&drawn[0]), [0, 0, 0, 10], "{drawn:?}");
    assert_eq!(bars(&drawn[1]), [0, 0, 10, 80], "{drawn:?}");
    assert_eq!(bars(&drawn[2]), [0, 10, 80, 30], "{drawn:?}");
}

#[test]
fn a_trace_keeps_as_many_bars_as_its_track_the_oldest_gone() {
    let readings: Vec<(f64, f64, bool)> = (1..=6).map(|n| (f64::from(n) / 10.0, f64::from(n) * 100.0, false)).collect();
    let Some(drawn) = shown(4, &readings) else {
        return;
    };
    assert_eq!(bars(drawn.last().unwrap()), [30, 40, 50, 60], "{drawn:?}");
}

#[test]
fn a_reading_out_of_range_is_drawn_inside_it() {
    let Some(drawn) = shown(2, &[(1.4, 0.0, false), (-0.2, 100.0, false)]) else {
        return;
    };
    assert_eq!(bars(&drawn[1]), [100, 0], "{drawn:?}");
}

#[test]
fn less_motion_holds_a_level_a_second_in_three_steps_on_every_bar() {
    let Some(drawn) = shown(
        3,
        &[
            (0.9, 0.0, true),
            (0.1, 500.0, true),
            (0.1, 999.0, true),
            (0.1, 1000.0, true),
            (0.5, 2000.0, true),
            (0.3, 3000.0, true),
            (0.74, 4000.0, true),
            (0.76, 5000.0, true),
            (0.19, 6000.0, true),
        ],
    ) else {
        return;
    };
    let held: Vec<Vec<i64>> = drawn.iter().map(bars).collect();
    let steps: Vec<i64> = held.iter().map(|b| b[0]).collect();
    assert_eq!(steps, [90, 90, 90, 0, 50, 50, 50, 90, 0], "{drawn:?}");
    // Nothing scrolls: every bar the level held.
    assert!(held.iter().all(|b| b.iter().all(|h| *h == b[0])), "{held:?}");
}

#[test]
fn less_motion_asked_while_drawing_holds_the_level_and_given_back_starts_a_trace() {
    let Some(drawn) = shown(3, &[(0.4, 0.0, false), (0.8, 100.0, true), (0.3, 200.0, false)]) else {
        return;
    };
    assert_eq!(bars(&drawn[1]), [90, 90, 90], "{drawn:?}");
    // The held level is no reading: the trace starts again from empty.
    assert_eq!(bars(&drawn[2]), [0, 0, 30], "{drawn:?}");
}
