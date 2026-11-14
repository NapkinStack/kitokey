//! A transcript as two runs write it alike: its `complete` line says when each made it whole
//! (transcript/v1, D93), and is compared without that `made`, which it must carry.

pub fn unmade(text: &str) -> String {
    text.lines()
        .map(|line| match serde_json::from_str::<serde_json::Value>(line) {
            Ok(mut complete) if complete["type"] == "complete" => {
                assert!(complete["made"].is_string(), "{line}");
                complete.as_object_mut().unwrap().remove("made");
                format!("{complete}\n")
            }
            _ => format!("{line}\n"),
        })
        .collect()
}
