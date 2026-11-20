//! A window whose session never opens (#991): its start raises, as it did, and the Kitokey it started
//! is ended with it — never left holding the single-instance name, every later start of the shard
//! handing its arguments over to it and raising « no session in a minute » in turn (3 attests in 63).
//! `Window.__enter__`, read from `scripts/e2e_window.py` without Selenium, is played with a stand-in
//! Kitokey that ignores SIGTERM and a driver that fails in each of the two ways a start gives up: its
//! session never answering (« no session in a minute »), or the driver never answering at all, each
//! of its connections refused (« tauri-driver did not answer »). Unix only, as the module's other tests
//! that drive its scripts.
#![cfg(unix)]

use std::process::Command;

/// The window's methods, its driver a stand-in that fails as `sys.argv[2]` says — `session`, its
/// session never answering; `driver`, each connection refused —, the Kitokey it starts a stand-in that
/// ignores SIGTERM: what the start raised, what the run printed and counted as failed,
/// and whether the stand-in Kitokey is still there, in JSON.
const PLAY: &str = r#"
import ast, contextlib, io, json, pathlib, signal, subprocess, sys, time, os
tree = ast.parse(pathlib.Path(sys.argv[1]).read_text())
window = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "Window")
methods = [node for node in window.body if isinstance(node, ast.FunctionDef) and node.name != "__init__"]
started = []
class Popen:
    """tauri-driver: it starts the Kitokey under test, which ignores SIGTERM, then never answers."""
    def __init__(self, *args, **kwargs):
        started.append(subprocess.Popen([sys.executable, "-c", "import signal, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)"]))
        time.sleep(0.3)
    def terminate(self): pass
    def wait(self, timeout=None): return 0
class ReadTimeoutError(Exception): pass
class WebDriverException(Exception): pass
class Remote:
    def __init__(self, *args, **kwargs):
        raise ReadTimeoutError("read timed out") if sys.argv[2] == "session" else WebDriverException("connection refused")
alive = lambda: [p.pid for p in started if p.poll() is None]
FAILED = []
def check(what, held, got=""):
    print(f"  {'ok  ' if held else 'FAIL'}  {what}" + ("" if held else f"\n        got: {got}"))
    if not held:
        FAILED.append(what)
class Options:
    def set_capability(self, *args): pass
scope = {"os": os, "signal": signal, "time": time, "sys": sys, "FAILED": FAILED, "check": check, "PLAYING": "page_one_bar",
         "subprocess": type("subprocess", (), {"Popen": Popen, "DEVNULL": None}), "free_port": lambda: 0,
         "instances": alive, "single_instance_held": lambda: bool(alive()), "ArgOptions": Options,
         "webdriver": type("webdriver", (), {"Remote": Remote}), "ClientConfig": lambda *a, **k: None,
         "ReadTimeoutError": ReadTimeoutError, "WebDriverException": WebDriverException, "HTTPError": type("H", (Exception,), {}),
         "BINARY": pathlib.Path("/kitokey"), "Path": pathlib.Path}
klass = ast.fix_missing_locations(ast.Module(body=[ast.ClassDef(name="Window", bases=[], keywords=[], body=methods, decorator_list=[], type_params=[])], type_ignores=[]))
exec(compile(klass, "e2e_window.py", "exec"), scope)
# The driver's ports are #943's, played in `tests/e2e_ports.rs`: here it is started as it was.
def start_driver(self):
    self.driver_process = Popen()
    return 0
scope["Window"].start_driver = start_driver
w = scope["Window"].__new__(scope["Window"])
w.env, w.args = {}, []
out, raised = io.StringIO(), None
began = time.monotonic()
with contextlib.redirect_stdout(out):
    try:
        w.__enter__()
    except BaseException as why:
        raised = f"{type(why).__name__}: {why}"
took = time.monotonic() - began
left = alive()
for p in started:
    if p.poll() is None:
        p.kill()
print(json.dumps({"raised": raised, "out": out.getvalue(), "failed": FAILED, "left": left, "started": len(started), "took": round(took, 1)}))
"#;

/// `Window.__enter__` played with its driver failing as `how` says: what it printed, in JSON; `None`
/// where python3 is not there, outside CI.
fn played(how: &str) -> Option<String> {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py");
    let Ok(output) = Command::new("python3").args(["-I", "-c", PLAY, script, how]).output() else {
        assert!(std::env::var_os("CI").is_none(), "python3 is not there on this CI runner");
        eprintln!("python3 is not there: a failed start is not played");
        return None;
    };
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "the window could not be played: {stderr}");
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

/// The start raised as it did, `raised` its first words, and the Kitokey it started is gone, said as
/// `__exit__` says one (#962): no single-instance name held for the next start.
fn ended_with_its_start(said: &str, raised: &str) {
    let field = |name: &str| said.split(&format!("\"{name}\": ")).nth(1).unwrap_or("").to_string();
    assert!(field("started").starts_with('1'), "the stand-in Kitokey was not started: {said}");
    assert!(field("raised").starts_with(raised), "the start did not raise as before: {said}");
    assert!(field("left").starts_with("[]"), "the Kitokey it started still runs: {said}");
    assert!(
        said.contains("FAIL  page_one_bar left Kitokey"),
        "the Kitokey left not said: {said}"
    );
}

#[test]
fn a_window_whose_session_never_opens_ends_the_kitokey_it_started() {
    if let Some(said) = played("session") {
        ended_with_its_start(&said, "\"RuntimeError: tauri-driver opened no session in a minute");
    }
}

#[test]
fn a_window_whose_driver_never_answers_ends_the_kitokey_it_started() {
    if let Some(said) = played("driver") {
        ended_with_its_start(&said, "\"SystemExit: tauri-driver did not answer");
    }
}
