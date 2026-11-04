"""Kitokey started as its user starts it, in the e2e's container (D80): with its window, as the
applications' menu starts it, on a display of its own, driven by WebKitWebDriver as the shell's
window e2e drives it. It asks the window's own list of meetings, prints it, and quits Kitokey.

Run as the user, under a display and a session bus: `python3 start-as-user.py <driver>`. Prints
one line of JSON: the meetings' folder Kitokey reads, and the meetings it lists there.
"""

import json
import os
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.request

DRIVER = sys.argv[1]
PORT = 4444
BASE = f"http://127.0.0.1:{PORT}"


def call(method: str, path: str, body: object | None = None, wait: float = 60) -> dict:
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(BASE + path, data=data, method=method, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=wait) as answer:
        return json.load(answer)


driver = subprocess.Popen([DRIVER, f"--port={PORT}"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
try:
    for _ in range(100):
        try:
            call("GET", "/status", wait=2)
            break
        except (urllib.error.URLError, ConnectionError):
            time.sleep(0.1)
    # As tauri-driver asks it of WebKitWebDriver: the program, and its arguments.
    options = {"binary": "/usr/bin/kitokey", "args": ["--window"]}
    session = call("POST", "/session", {"capabilities": {"alwaysMatch": {"webkitgtk:browserOptions": options}}}, wait=120)
    number = session["value"]["sessionId"]
    script = (
        "const done = arguments[arguments.length - 1];"
        "window.__TAURI__.core.invoke('listed')"
        ".then(listing => done({folder: listing.folder, meetings: listing.sessions.map(s => s.folder)}))"
        ".catch(error => done({error: String(error)}));"
    )
    listing = None
    for _ in range(60):
        try:
            listing = call("POST", f"/session/{number}/execute/async", {"script": script, "args": []})["value"]
        except urllib.error.HTTPError:
            listing = None
        if isinstance(listing, dict) and "folder" in listing:
            break
        time.sleep(0.5)
    print(json.dumps(listing))
finally:
    driver.send_signal(signal.SIGTERM)
    driver.wait(timeout=30)
    # No window is no reason for Kitokey to leave: it stays in the tray until quit (D35).
    subprocess.run(["pkill", "-TERM", "-x", "kitokey"], check=False)
    for _ in range(100):
        if subprocess.run(["pgrep", "-x", "kitokey"], stdout=subprocess.DEVNULL).returncode != 0:
            break
        time.sleep(0.1)
    os.sync()
