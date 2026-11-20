//! tauri-driver's two ports are picked free, then released before it binds them: another run on the
//! machine can take one in between (#943). tauri-driver says so, never by its exit code: its port
//! taken, it ends with code 0 and « can not listen to address »; its native port taken, it stays up
//! while its WebKitWebDriver ends with « FATAL: Unable to listen » (measured, 2026-11-20). Either
//! way the session was asked of another run's driver. `Window.__enter__`, read from
//! `scripts/e2e_window.py` without Selenium, is played with a stand-in tauri-driver that behaves as
//! measured, and a port held by « another run »: the session is asked of this run's own driver, each
//! start again said with what the driver said, and a port still taken after three starts fails the
//! start, naming it. Linux only: the e2e's WebKitGTK and the processes' sockets read in `/proc`.
#![cfg(target_os = "linux")]

use std::process::Command;

/// The stand-in tauri-driver: its WebKitWebDriver started first, as the real one (it outlives a
/// driver whose own port was taken); every connection to its port relayed to its native port, as a
/// session is. Whoever listens there says who they are: « ours », or « other ».
const DRIVER: &str = r#"
import socket, subprocess, sys
port, native = int(sys.argv[sys.argv.index("--port") + 1]), int(sys.argv[sys.argv.index("--native-port") + 1])
WEBKIT = r'''
import socket, sys
native = int(sys.argv[1])
s = socket.socket()
try:
    s.bind(("127.0.0.1", native))
except OSError:
    print(f"FATAL: Unable to listen for HTTP server at host 127.0.0.1 and port {native}.", flush=True)
    sys.exit(1)
s.listen()
while True:
    c, _ = s.accept(); c.sendall(b"ours\n"); c.close()
'''
print(f"webkit {subprocess.Popen([sys.executable, '-c', WEBKIT, str(native)]).pid}", flush=True)
s = socket.socket()
try:
    s.bind(("127.0.0.1", port))
except OSError:
    print(f"can not listen to address: 127.0.0.1:{port}", flush=True)
    sys.exit(0)
s.listen()
while True:
    c, _ = s.accept()
    try:
        with socket.create_connection(("127.0.0.1", native), timeout=2) as n:
            c.sendall(n.recv(64))
    except OSError:
        pass
    c.close()
"#;

/// The window's methods and the helpers they use, played with that stand-in in place of tauri-driver
/// and `free_port` handing out the held port where `sys.argv[3]` says: `port` (first pick), `native`
/// (second pick), `always` (every first pick of a pair). What the start raised, printed, and whom the
/// session was asked of, in JSON.
const PLAY: &str = r#"
import ast, contextlib, io, json, os, pathlib, signal, socket, subprocess, sys, tempfile, threading, time
source, driver, how = pathlib.Path(sys.argv[1]).read_text(), sys.argv[2], sys.argv[3]
tree = ast.parse(source)
window = next(node for node in tree.body if isinstance(node, ast.ClassDef) and node.name == "Window")
methods = [node for node in window.body if isinstance(node, ast.FunctionDef) and node.name != "__init__"]
helpers = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in ("descendants", "listening")]
# Another run's server, on the held port: it answers « other ».
held = socket.socket(); held.bind(("127.0.0.1", 0)); held.listen(); HELD = held.getsockname()[1]
def other():
    while True:
        c, _ = held.accept(); c.sendall(b"other\n"); c.close()
threading.Thread(target=other, daemon=True).start()
picks = []
def free_port():
    n = len(picks); picks.append(n)
    if (how == "port" and n == 0) or (how == "native" and n == 1) or (how == "always" and n % 2 == 0):
        return HELD
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0)); return probe.getsockname()[1]
started = []
class Popen(subprocess.Popen):
    def __init__(self, args, **kwargs):
        args = [sys.executable, driver] + list(args[1:]) if args and args[0] == "tauri-driver" else args
        kwargs.setdefault("start_new_session", True)
        super().__init__(args, **kwargs)
        started.append(self.pid)
class ReadTimeoutError(Exception): pass
class WebDriverException(Exception): pass
class Browser:
    def __init__(self, who): self.who = who
def Remote(address, **kwargs):
    try:
        with socket.create_connection(("127.0.0.1", int(address.rsplit(":", 1)[1])), timeout=2) as c:
            who = c.recv(64).decode().strip()
    except OSError as why:
        raise WebDriverException(str(why))
    if not who:
        raise WebDriverException("no answer")
    return Browser(who)
class Options:
    def set_capability(self, *args): pass
FAILED = []
def check(what, holds, got=""):
    print(f"  {'ok  ' if holds else 'FAIL'}  {what}")
    if not holds: FAILED.append(what)
module = type("subprocess", (), {"Popen": Popen, "DEVNULL": subprocess.DEVNULL, "STDOUT": subprocess.STDOUT, "PIPE": subprocess.PIPE})
scope = {"os": os, "signal": signal, "time": time, "sys": sys, "socket": socket, "contextlib": contextlib, "Path": pathlib.Path, "FAILED": FAILED, "check": check,
         "PLAYING": "a_scenario", "subprocess": module, "free_port": free_port, "instances": lambda: [], "single_instance_held": lambda: False,
         "ArgOptions": Options, "webdriver": type("webdriver", (), {"Remote": Remote}), "ClientConfig": lambda *a, **k: None,
         "ReadTimeoutError": ReadTimeoutError, "WebDriverException": WebDriverException, "HTTPError": type("H", (Exception,), {}),
         "BINARY": pathlib.Path("/kitokey"), "WORK": pathlib.Path(tempfile.mkdtemp()), "wait_for": lambda *a, **k: True,
         "By": type("By", (), {"ID": "id"})}
body = helpers + [ast.ClassDef(name="Window", bases=[], keywords=[], body=methods, decorator_list=[], type_params=[])]
exec(compile(ast.fix_missing_locations(ast.Module(body=body, type_ignores=[])), "e2e_window.py", "exec"), scope)
w = scope["Window"].__new__(scope["Window"])
w.env, w.args = dict(os.environ), []
out, raised, who = io.StringIO(), None, None
with contextlib.redirect_stdout(out):
    try:
        who = w.__enter__().who
    except BaseException as why:
        raised = f"{type(why).__name__}: {why}"
# The WebKitWebDrivers of the starts given up: none still there, an orphan included.
said = [f.read_text() for f in sorted(scope["WORK"].glob("tauri-driver-*.txt"), key=lambda f: f.stat().st_mtime)]
given_up = [int(line.split()[1]) for text in (said[:-1] if who else said) for line in text.splitlines() if line.startswith("webkit ")]
def alive(pid):
    with contextlib.suppress(OSError):
        return pathlib.Path(f"/proc/{pid}/stat").read_text().split(")")[1].split()[0] != "Z"
    return False
until = time.monotonic() + 2
while any(alive(pid) for pid in given_up) and time.monotonic() < until:
    time.sleep(0.05)
stale = [pid for pid in given_up if alive(pid)]
for pid in started + stale:
    with contextlib.suppress(ProcessLookupError, PermissionError):
        os.killpg(os.getpgid(pid), signal.SIGKILL) if pid in stale else os.killpg(pid, signal.SIGKILL)
print(json.dumps({"who": who, "raised": raised, "out": out.getvalue(), "picks": len(picks), "stale": stale}))
"#;

/// `Window.__enter__` played with the held port where `how` says: what it printed, in JSON.
fn played(how: &str) -> String {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/e2e_window.py");
    let dir = std::env::temp_dir().join(format!("kitokey-e2e-ports-{}-{how}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a folder for the stand-in driver");
    let driver = dir.join("tauri-driver.py");
    std::fs::write(&driver, DRIVER).expect("the stand-in driver written");
    let output = Command::new("python3")
        .args(["-I", "-c", PLAY, script, driver.to_str().unwrap(), how])
        .output()
        .expect("python3, as the e2e needs it");
    let _ = std::fs::remove_dir_all(&dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "the window could not be played: {stderr}");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn field(said: &str, name: &str) -> String {
    said.split(&format!("\"{name}\": ")).nth(1).unwrap_or("").to_string()
}

#[test]
fn its_port_taken_the_driver_is_started_again_and_the_session_is_asked_of_its_own() {
    let said = played("port");
    assert!(
        field(&said, "who").starts_with("\"ours\""),
        "the session was asked of another run's driver: {said}"
    );
    assert!(
        said.contains("tauri-driver's ports taken") && said.contains("can not listen to address"),
        "the start again not said with what the driver said: {said}"
    );
    assert!(
        !said.contains("neither refused nor listening"),
        "a start fell back on its 10 s: {said}"
    );
    assert!(
        field(&said, "stale").starts_with("[]"),
        "a given-up start's WebKitWebDriver still runs: {said}"
    );
}

#[test]
fn its_native_port_taken_the_driver_is_started_again_and_the_session_is_asked_of_its_own() {
    let said = played("native");
    assert!(
        field(&said, "who").starts_with("\"ours\""),
        "the session was asked of another run's driver: {said}"
    );
    assert!(
        said.contains("tauri-driver's ports taken") && said.contains("Unable to listen"),
        "the start again not said with what the driver said: {said}"
    );
    assert!(
        !said.contains("neither refused nor listening"),
        "a start fell back on its 10 s: {said}"
    );
    assert!(
        field(&said, "stale").starts_with("[]"),
        "a given-up start's WebKitWebDriver still runs: {said}"
    );
}

#[test]
fn a_port_still_taken_after_three_starts_fails_the_start_naming_it() {
    let said = played("always");
    assert!(
        field(&said, "raised").starts_with("\"RuntimeError: tauri-driver's ports taken three times")
            && field(&said, "raised").contains("can not listen to address"),
        "the start did not fail naming the taken port: {said}"
    );
    assert!(
        !said.contains("neither refused nor listening"),
        "a start fell back on its 10 s: {said}"
    );
    assert!(field(&said, "picks").starts_with('6'), "not three starts, two ports each: {said}");
}
