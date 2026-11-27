//! D92, criterion 2: « environ N min », from the audio's length and the speed last measured on this
//! machine for each step and model — none measured, the step alone and no number —, never moving back.

use kitokey_shell::speeds::{self, Speeds, Step};

const AUDIO: f64 = 600.0;

fn measured() -> Speeds {
    let mut kept = Speeds::default();
    // A transcript made in 60 s for 600 s of audio; its summary in 120 s.
    kept.measured(Step::Transcript, "ggml-small", 60.0, AUDIO);
    kept.measured(Step::Minutes, "qwen3:8b", 120.0, AUDIO);
    kept
}

#[test]
fn nothing_measured_says_no_number() {
    let none = Speeds::default();
    assert_eq!(none.left(&[(Step::Transcript, "ggml-small", Some(0.0))], AUDIO), None);
    // One step measured, the other not: no number for the making, never half of one.
    let mut half = Speeds::default();
    half.measured(Step::Transcript, "ggml-small", 60.0, AUDIO);
    assert_eq!(
        half.left(
            &[(Step::Transcript, "ggml-small", Some(10.0)), (Step::Minutes, "qwen3:8b", None)],
            AUDIO
        ),
        None
    );
    // Another model than the one measured: not this one's speed.
    assert_eq!(half.left(&[(Step::Transcript, "ggml-large", Some(10.0))], AUDIO), None);
}

#[test]
fn what_is_left_is_each_step_at_its_speed_less_what_ran() {
    let kept = measured();
    let left = kept.left(
        &[(Step::Transcript, "ggml-small", Some(20.0)), (Step::Minutes, "qwen3:8b", None)],
        AUDIO,
    );
    assert_eq!(left, Some(40.0 + 120.0));
    // The same audio twice as long: twice the time.
    assert_eq!(kept.left(&[(Step::Minutes, "qwen3:8b", Some(0.0))], 2.0 * AUDIO), Some(240.0));
    // Past its estimate, a step running has nothing left, never less.
    assert_eq!(
        kept.left(
            &[(Step::Transcript, "ggml-small", Some(90.0)), (Step::Minutes, "qwen3:8b", None)],
            AUDIO
        ),
        Some(120.0)
    );
}

#[test]
fn it_never_moves_back_as_the_making_goes_on() {
    let kept = measured();
    let mut seen = Vec::new();
    for second in 0..80 {
        seen.push(
            kept.left(
                &[
                    (Step::Transcript, "ggml-small", Some(f64::from(second))),
                    (Step::Minutes, "qwen3:8b", None),
                ],
                AUDIO,
            )
            .unwrap(),
        );
    }
    // The transcript ended, slower than measured: the summary from its start.
    for second in 0..130 {
        seen.push(kept.left(&[(Step::Minutes, "qwen3:8b", Some(f64::from(second)))], AUDIO).unwrap());
    }
    assert!(seen.windows(2).all(|pair| pair[1] <= pair[0]), "{seen:?}");
}

#[test]
fn the_last_measure_is_the_one_kept() {
    let mut kept = measured();
    kept.measured(Step::Transcript, "ggml-small", 30.0, AUDIO);
    assert_eq!(kept.left(&[(Step::Transcript, "ggml-small", Some(0.0))], AUDIO), Some(30.0));
    // A measure that says nothing — no audio, or not a number — changes nothing.
    kept.measured(Step::Transcript, "ggml-small", 30.0, 0.0);
    kept.measured(Step::Transcript, "ggml-small", f64::NAN, AUDIO);
    kept.measured(Step::Transcript, "ggml-small", -1.0, AUDIO);
    assert_eq!(kept.left(&[(Step::Transcript, "ggml-small", Some(0.0))], AUDIO), Some(30.0));
}

#[test]
fn said_broadly_and_only_past_half_a_minute() {
    // GNOME HIG: a bar with what is left past about 30 s; « environ », never « 1 min 37 ».
    for (seconds, said) in [
        (0.0, None),
        (30.0, None),
        (31.0, Some(1)),
        (89.0, Some(1)),
        (97.0, Some(2)),
        (150.0, Some(3)),
        (600.0, Some(10)),
        (12.0 * 60.0, Some(10)),
        (13.0 * 60.0, Some(15)),
        (47.0 * 60.0, Some(45)),
    ] {
        assert_eq!(speeds::said(seconds), said, "{seconds}");
    }
}

#[test]
fn kept_as_numbers_by_step_and_model_alone_and_an_unreadable_file_is_none() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("speeds.json");
    // None there, or one that does not read: nothing measured, the step said alone.
    assert_eq!(speeds::read(&file), Speeds::default());
    std::fs::write(&file, "not json").unwrap();
    assert_eq!(speeds::read(&file), Speeds::default());
    std::fs::write(&file, r#"{"transcript": {"m": "fast"}}"#).unwrap();
    assert_eq!(speeds::read(&file), Speeds::default());
    let kept = measured();
    speeds::write(&file, &kept).unwrap();
    assert_eq!(speeds::read(&file), kept);
    // Only numbers by step and model: no meeting, no date, no length of audio.
    let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(
        written,
        serde_json::json!({"transcript": {"ggml-small": 0.1}, "minutes": {"qwen3:8b": 0.2}})
    );
}
