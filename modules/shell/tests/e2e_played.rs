//! A scenario of the window's e2e that raises (#949): its run says which one and what it raised, a
//! FAIL with its traceback, and the scenarios after it are played all the same — never a shard cut
//! short, the checks after it hidden as « NOT RUN ». The script's own `main()` and `timed`, read from
//! `scripts/e2e_window.py` without Selenium, play two stand-ins as a partial run plays its scenarios:
//! `main()` calls them, `KITOKEY_E2E_ONLY` names them, and `main()` wraps them itself.
//! Unix only, as the module's other tests that drive its scripts.
#![cfg(unix)]

use std::process::Command;

/// The script's `main()`, with two more scenarios at the end of its body — the first raises, the
/// second checks —, run as a partial run naming them. What the run printed, what it played and counted
/// as failed, and what escaped it, in JSON.
const PLAY: &str = r#"
import ast, contextlib, datetime as dt, io, json, os, pathlib, sys, tempfile, time, traceback
script = pathlib.Path(sys.argv[1])
sys.path.insert(0, str(script.parent))
import e2e_only
tree = ast.parse(script.read_text())
taken = {node.name: node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in ("main", "timed")}
main = taken["main"]
for name in ("raises_first", "checks_after"):
    main.body.insert(len(main.body) - 1, ast.parse(f"{name}(now)").body[0])
module = ast.fix_missing_locations(ast.Module(body=[taken["timed"], main], type_ignores=[]))
source = pathlib.Path(tempfile.mkdtemp()) / "e2e_window.py"
source.write_text(ast.unparse(module))
played = []
def raises_first(now):
    played.append("raises_first")
    int("not a number")
def checks_after(now):
    played.append("checks_after")
    print("  ok    the scenario after it")
scope = {"__file__": str(source), "dt": dt, "os": os, "Path": pathlib.Path, "time": time, "traceback": traceback, "sys": sys, "FAILED": []}
scope.update({name: (lambda now: None) for name in e2e_only.scenarios(source.read_text())})
scope.update(raises_first=raises_first, checks_after=checks_after)
exec(compile(module, str(source), "exec"), scope)
os.environ.pop("KITOKEY_E2E_PART", None)
os.environ["KITOKEY_E2E_ONLY"] = "raises_first,checks_after"
out = io.StringIO()
escaped = None
returned = None
with contextlib.redirect_stdout(out):
    try:
        returned = scope["main"]()
    except Exception as why:
        escaped = repr(why)
print(json.dumps({"out": out.getvalue(), "played": played, "failed": scope["FAILED"], "escaped": escaped, "returned": returned}))
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
    // Played by main() itself, as a partial run: each wrapped by it — its time said —, the run red.
    assert!(
        said.contains("partial run: raises_first, checks_after"),
        "not main()'s partial run: {said}"
    );
    assert!(
        said.contains("(raises_first: ") && said.contains("(checks_after: "),
        "not timed by main(): {said}"
    );
    assert!(field("returned").starts_with('1'), "the run not red: {said}");
}
