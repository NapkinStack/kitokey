//! A scenario whose Kitokey does not end when it is over (#960): its window's end waits a bounded time
//! for it, then says which scenario left it running — one FAIL — and ends it, so that the next scenario
//! of its shard starts instead of handing over to it and raising « no session in a minute ». `timed`
//! and `Window.__exit__`, read from `scripts/e2e_window.py` without Selenium, are run here on a stand-in
//! process that ignores SIGTERM. Unix only, as the module's other tests that drive its scripts.
#![cfg(unix)]

use std::process::Command;

/// A stand-in « Kitokey » that ignores SIGTERM, ended by a scenario's window as the harness ends one:
/// what the run printed and counted as failed, and whether the stand-in is still there, in JSON.
const PLAY: &str = r#"
import ast, contextlib, io, json, os, pathlib, signal, subprocess, sys, time, traceback
tree = ast.parse(pathlib.Path(sys.argv[1]).read_text())
taken = {node.name: node for node in tree.body if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name in ("timed", "Window")}
leave = [node for node in taken["Window"].body if isinstance(node, ast.FunctionDef) and node.name in ("__exit__", "end_kitokey")]
module = ast.fix_missing_locations(ast.Module(body=[taken["timed"], *leave], type_ignores=[]))
stuck = subprocess.Popen([sys.executable, "-c", "import signal, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)"])
time.sleep(0.3)
alive = lambda: stuck.poll() is None
FAILED = []
def check(what, held, got=""):
    print(f"  {'ok  ' if held else 'FAIL'}  {what}" + ("" if held else f"\n        got: {got}"))
    if not held:
        FAILED.append(what)
class Quiet:
    def quit(self): pass
    def terminate(self): pass
    def wait(self, timeout=None): return 0
scope = {"os": os, "signal": signal, "time": time, "traceback": traceback, "sys": sys, "FAILED": FAILED, "check": check, "PLAYING": None,
         "instances": lambda: [stuck.pid] if alive() else [], "single_instance_held": lambda: alive()}
exec(compile(module, "e2e_window.py", "exec"), scope)
window = type("Window", (), {"__exit__": scope["__exit__"], "end_kitokey": scope.get("end_kitokey")})()
window.browser = window.driver_process = Quiet()
def corrections_gated(now):
    window.__exit__(None, None, None)
out = io.StringIO()
began = time.monotonic()
with contextlib.redirect_stdout(out):
    scope["timed"](corrections_gated)(None)
took = time.monotonic() - began
left = alive()
if left:
    stuck.kill()
print(json.dumps({"out": out.getvalue(), "failed": FAILED, "left": left, "took": round(took, 1)}))
"#;

#[test]
fn a_scenario_that_leaves_its_kitokey_running_is_named_and_the_kitokey_ended() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py");
    let Ok(output) = Command::new("python3").args(["-I", "-c", PLAY, script]).output() else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: a Kitokey left running is not looked for");
        return;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "the stand-in could not be played: {stderr}");
    let said = String::from_utf8_lossy(&output.stdout).to_string();
    let field = |name: &str| said.split(&format!("\"{name}\": ")).nth(1).unwrap_or("").to_string();
    // Ended: never left to hold the next scenario's start.
    assert!(field("left").starts_with("false"), "the stand-in Kitokey still runs: {said}");
    // Said, by the scenario that left it, and counted as failed.
    assert!(
        said.contains("FAIL  corrections_gated left Kitokey"),
        "the scenario not named: {said}"
    );
    assert!(
        field("failed").starts_with("[\"corrections_gated left Kitokey"),
        "not counted as failed: {said}"
    );
    // Within its bound: about ten seconds — not the next start's minute, nor a bound let go.
    let took: f64 = field("took")
        .split([',', '}'])
        .next()
        .unwrap_or("")
        .trim()
        .parse()
        .unwrap_or(f64::MAX);
    assert!((9.0..13.0).contains(&took), "waited {took} s: {said}");
    // The slow end said, in seconds.
    assert!(said.contains("(Kitokey's end after SIGTERM: "), "the slow end not said: {said}");
}
