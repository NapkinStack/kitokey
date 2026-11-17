//! A window's focus is asked on the main thread alone (D92, #824). Asked from another thread, the
//! toolkit's getter sends its question to the main thread and waits for the answer: a caller holding
//! a lock the main thread then takes — `show`'s, the transcription's line — waits for ever, the
//! window frozen and nothing told. Read from the source: the race it guards is not one a test can
//! make happen at will.

const MAIN: &str = include_str!("../src/main.rs");

/// The body of `fn <name>(`, to its closing brace at the start of a line.
fn body(name: &str) -> &'static str {
    let start = MAIN
        .find(&format!("\nfn {name}("))
        .unwrap_or_else(|| panic!("no fn {name} in main.rs"));
    let end = MAIN[start..].find("\n}\n").expect("the function's end") + start;
    &MAIN[start..end]
}

#[test]
fn the_focus_is_asked_on_the_main_thread_alone() {
    // Only `tell_made` asks it: another caller would ask it from wherever it runs.
    let callers: Vec<&str> = MAIN
        .match_indices("window_focused(")
        .filter(|(at, _)| !MAIN[..*at].ends_with("fn "))
        .map(|(at, _)| {
            let from = MAIN[..at].rfind("\nfn ").expect("inside a function") + 4;
            let name = &MAIN[from..];
            &name[..name.find('(').unwrap()]
        })
        .collect();
    assert_eq!(callers, ["tell_made"], "window_focused is asked by tell_made alone");

    // Within it, inside what is handed to the main thread, never before.
    let told = body("tell_made");
    let handed = told
        .find("run_on_main_thread(")
        .expect("tell_made hands the focus's question to the main thread");
    let asked = told.find("window_focused(").unwrap();
    assert!(asked > handed, "tell_made asks the focus on its caller's thread:\n{told}");
}
