//! `main()`'s own checks in sections (#957): an exception in one of them is a FAIL named by its
//! section, with its traceback, and the sections after it are played all the same — never the main
//! shard cut short, its checks hidden as « NOT RUN », as `timed` does for a scenario (#949). Read
//! from `scripts/e2e_window.py` without Selenium: the shape of `main()`'s body, and `section` played
//! on two stand-ins. Unix only, as the module's other tests that drive its scripts.
#![cfg(unix)]

use std::process::Command;

/// `main()`'s inline part — after the partial run's `if`, before the first scenario it calls — each
/// statement a `with section(...)`; then `section` played on a section that raises and one after it.
/// What was found and what the run printed, played and counted as failed, in JSON.
const PLAY: &str = r#"
import ast, contextlib, io, json, pathlib, sys, time, traceback
tree = ast.parse(pathlib.Path(sys.argv[1]).read_text())
main = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "main")
start = next(i for i, node in enumerate(main.body) if isinstance(node, ast.If) and "KITOKEY_E2E_ONLY" in ast.unparse(node.test)) + 1
end = next(i for i, node in enumerate(main.body) if isinstance(node, ast.Expr) and ast.unparse(node) == "model_offered(now)")
def in_section(node):
    return (isinstance(node, ast.With) and len(node.items) == 1 and isinstance(node.items[0].context_expr, ast.Call)
            and getattr(node.items[0].context_expr.func, "id", "") == "section")
outside = [f"l.{node.lineno}: {ast.unparse(node).splitlines()[0][:80]}" for node in main.body[start:end] if not in_section(node)]
sections = [ast.unparse(node.items[0].context_expr.args[0])[:40] for node in main.body[start:end] if in_section(node)]
taken = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == "section"]
played, out, escaped, failed = [], io.StringIO(), None, []
if taken:
    scope = {"contextlib": contextlib, "time": time, "traceback": traceback, "FAILED": failed}
    exec(compile(ast.fix_missing_locations(ast.Module(body=taken, type_ignores=[])), "e2e_window.py", "exec"), scope)
    with contextlib.redirect_stdout(out):
        try:
            with scope["section"]("H. the first, which raises"):
                played.append("H")
                int("not a number")
            with scope["section"]("J. the one after it"):
                played.append("J")
                print("  ok    the section after it")
        except Exception as why:
            escaped = repr(why)
print(json.dumps({"outside": outside, "sections": len(sections), "section": bool(taken), "out": out.getvalue(),
                  "played": played, "failed": failed, "escaped": escaped}))
"#;

#[test]
fn an_exception_in_main_s_own_checks_is_a_named_fail_and_the_sections_after_it_are_played() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py");
    let Ok(output) = Command::new("python3").args(["-I", "-c", PLAY, script]).output() else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: main()'s sections are not read");
        return;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "main() could not be read: {stderr}");
    let said = String::from_utf8_lossy(&output.stdout).to_string();
    let field = |name: &str| said.split(&format!("\"{name}\": ")).nth(1).unwrap_or("").to_string();
    // Every statement of main()'s own checks inside a section: none left to end the shard.
    assert!(field("outside").starts_with("[]"), "statements of main() outside a section: {said}");
    assert!(!field("sections").starts_with('0'), "no section in main(): {said}");
    assert!(field("section").starts_with("true"), "no `section` in the script: {said}");
    // Nothing leaves the section: main() goes on to the next.
    assert!(field("escaped").starts_with("null"), "the exception left its section: {said}");
    assert!(
        field("played").starts_with("[\"H\", \"J\"]"),
        "the section after it was not played: {said}"
    );
    // The one that raised, named by its section with what it raised, counted, its traceback under it.
    assert!(
        said.contains("FAIL  H. the first, which raises raised ValueError: invalid literal for int() with base 10: 'not a number'"),
        "not named: {said}"
    );
    assert!(said.contains("Traceback (most recent call last)"), "no traceback: {said}");
    assert!(
        field("failed").starts_with("[\"H. the first, which raises raised ValueError"),
        "not counted: {said}"
    );
    // The sections said, as before, and the one after it said its own checks.
    assert!(said.contains("J. the one after it"), "the section's title not said: {said}");
    assert!(
        said.contains("ok    the section after it"),
        "the section after it said nothing: {said}"
    );
}
