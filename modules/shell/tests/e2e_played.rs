//! A scenario of the window's e2e that raises (#949): its run says which one and what it raised, a
//! FAIL with its traceback, and the scenarios after it are played all the same — never a shard cut
//! short, the checks after it hidden as « NOT RUN ». `timed`, the wrapper every scenario is played
//! through, is read from `scripts/e2e_window.py` and run here on two stand-ins, without Selenium.
//! Unix only, as the module's other tests that drive its scripts.
#![cfg(unix)]

use std::process::Command;

/// Two stand-ins played as `main()` plays scenarios, through `timed` as the script defines it: the
/// first raises, the second checks. What the run printed, and what it counted as failed, in JSON.
const PLAY: &str = r#"
import ast, json, sys, time, traceback, io, contextlib
source = open(sys.argv[1]).read()
timed = next(node for node in ast.parse(source).body if isinstance(node, ast.FunctionDef) and node.name == "timed")
scope = {"time": time, "traceback": traceback, "sys": sys, "FAILED": []}
exec(compile(ast.Module(body=[timed], type_ignores=[]), "e2e_window.py", "exec"), scope)
played = []
def raises_first(now):
    played.append("raises_first")
    int("not a number")
def checks_after(now):
    played.append("checks_after")
    print("  ok    the scenario after it")
out = io.StringIO()
escaped = None
with contextlib.redirect_stdout(out):
    try:
        for scenario in (raises_first, checks_after):
            scope["timed"](scenario)(None)
    except Exception as why:
        escaped = repr(why)
print(json.dumps({"out": out.getvalue(), "played": played, "failed": scope["FAILED"], "escaped": escaped}))
"#;

#[test]
fn a_scenario_that_raises_is_a_named_fail_and_the_ones_after_it_are_played() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py");
    let Ok(output) = Command::new("python3").args(["-I", "-c", PLAY, script]).output() else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: a scenario that raises is not played");
        return;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "the stand-ins could not be played: {stderr}");
    let said = String::from_utf8_lossy(&output.stdout).to_string();
    let field = |name: &str| said.split(&format!("\"{name}\": ")).nth(1).unwrap_or("").to_string();
    // Nothing leaves the wrapper: the run goes on to the next scenario.
    assert!(field("escaped").starts_with("null"), "the exception left the scenario: {said}");
    assert!(
        field("played").starts_with("[\"raises_first\", \"checks_after\"]"),
        "the scenario after it was not played: {said}"
    );
    // The one that raised, named with what it raised, counted as failed, its traceback under it.
    assert!(
        said.contains("FAIL  raises_first raised ValueError: invalid literal for int() with base 10: 'not a number'"),
        "not named: {said}"
    );
    assert!(said.contains("Traceback (most recent call last)"), "no traceback: {said}");
    assert!(
        field("failed").starts_with("[\"raises_first raised ValueError"),
        "not counted as failed: {said}"
    );
    // And the scenario after it said its own checks.
    assert!(
        said.contains("ok    the scenario after it"),
        "the scenario after it said nothing: {said}"
    );
}
