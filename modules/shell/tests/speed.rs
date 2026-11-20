//! Kitokey's speed (D105) is read from /proc and the page: what `scripts/speed.py` makes of what it
//! reads is checked here, on inputs built for the test, without a session or a window. A start read
//! from a process's `stat` whose name holds spaces and a `)`, the boot's `btime`, a proportional set
//! size, the medians of an even and an odd number of runs (one slow run in each, which a mean would
//! follow), and the meeting of an hour made up.
//! Linux only: the numbers are /proc's.
#![cfg(target_os = "linux")]

use std::process::Command;

/// `speed.py`'s functions, read without running it, each played on what the test gives: their
/// answers, one per line.
const PLAY: &str = r#"
import ast, json, os, pathlib, platform, statistics, subprocess, sys, tempfile, time
tree = ast.parse(pathlib.Path(sys.argv[1]).read_text())
wanted = ("started_at", "booted", "pss_kib", "summary", "hour")
kept = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in wanted]
kept += [node for node in tree.body if isinstance(node, ast.Assign) and "HOUR" in ast.unparse(node.targets[0])]
scope = {"json": json, "statistics": statistics, "Path": pathlib.Path}
exec(compile(ast.fix_missing_locations(ast.Module(body=sorted(kept, key=lambda n: n.lineno), type_ignores=[])), "speed.py", "exec"), scope)
# A process named « kitokey (x) y »: its start is field 22, 250 ticks after the boot.
stat = "4242 (kitokey (x) y) S 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 3 0 250 1000000 500 18446744073709551615"
print(scope["started_at"](stat, 1700000000.0, 100))
print(scope["booted"]("cpu  1 2 3\nbtime 1700000000\nprocesses 9\n"))
print(scope["pss_kib"]("00400000-ffffffff ---p 00000000 00:00 0 [rollup]\nRss: 9000 kB\nPss: 7321 kB\nPss_Anon: 5000 kB\n"))
print(json.dumps(scope["summary"]({"even": [10.0, 1.0, 3.0, 2.0], "odd": [50.0, 1.0, 3.0]}), sort_keys=True))
meeting = pathlib.Path(tempfile.mkdtemp())
lines = scope["hour"](meeting)
read = [json.loads(line) for line in (meeting / "transcript.jsonl").read_text().splitlines()]
segments = [line for line in read if line["type"] == "segment"]
print(lines, len(segments), read[0]["type"], read[-1], segments[-1]["end"] <= 3600, {s["track"] for s in segments} == {"local", "remote"})
"#;

#[test]
fn speed_reads_a_start_a_boot_a_memory_and_medians_as_proc_and_the_runs_give_them() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/speed.py");
    let output = Command::new("python3")
        .args(["-I", "-c", PLAY, script])
        .output()
        .expect("python3, as the e2e needs it");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "speed.py could not be played: {stderr}");
    let said = String::from_utf8_lossy(&output.stdout).to_string();
    let lines: Vec<&str> = said.lines().collect();
    // 250 ticks of 1/100 s after a boot at 1700000000: 2.5 s after it.
    assert_eq!(lines[0], "1700000002.5", "the start read from stat: {said}");
    assert_eq!(lines[1], "1700000000.0", "the boot read from /proc/stat: {said}");
    assert_eq!(lines[2], "7321", "the PSS read from smaps_rollup: {said}");
    assert_eq!(lines[3], r#"{"even": 2.5, "odd": 3.0}"#, "the medians: {said}");
    // An hour in lines of 6 s: 600, both tracks, the complete line last, nothing past the hour.
    assert_eq!(
        lines[4], "600 600 header {'type': 'complete', 'duration': 3600.0} True True",
        "the meeting of an hour: {said}"
    );
}
