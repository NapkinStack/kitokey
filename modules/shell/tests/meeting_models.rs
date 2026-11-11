//! A meeting's own models (D91): `models.json` of `contracts/session-audio/v3`, written whole on the
//! user's word and read by the shell alone; what it names stands in for Settings' choice when the
//! meeting is made and made again, Settings never changed — a model gone, Settings' choice stands in
//! and it is said; a meeting said confidential never goes to a service, whatever the file names.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use kitokey_shell::{
    confidential,
    meeting_models::{self, Hearing, Models, Step, Writer},
    settings::{Engine, Settings},
    transcription::Hearer,
};

const SERVICE: &str = "https://api.groq.com/openai/v1";
const OLLAMA: &str = "http://localhost:11434/v1";

fn meeting_folder() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    fs::write(
        folder.path().join("session.json"),
        r#"{"contract":"session-audio/v3","started":"2026-11-11T14:05:00+01:00","stopped":"2026-11-11T14:35:00+01:00","tracks":{}}"#,
    )
    .unwrap();
    folder
}

fn names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn whisper(model: &str) -> Hearing {
    Hearing::Whisper { model: model.into() }
}

fn writer(provider: &str, model: &str) -> Writer {
    Writer {
        provider: provider.into(),
        model: model.into(),
    }
}

/// Settings as a user leaves them: the default Whisper model, a service set and agreed, minutes by
/// a service.
fn settings(models: &Path) -> Settings {
    Settings {
        model: models.join("ggml-large-v3-turbo-q8_0.bin"),
        transcription_provider: Some(SERVICE.into()),
        transcription_model: Some("whisper-large-v3".into()),
        transcription_agreed: Some("api.groq.com".into()),
        minutes_provider: Some("https://openrouter.ai/api/v1".into()),
        minutes_model: Some("anthropic/claude-sonnet".into()),
        minutes_agreed: Some("openrouter.ai".into()),
        ..Settings::default()
    }
}

/// A folder of Whisper models, and the list the shell offers from it.
fn installed(names: &[&str]) -> (tempfile::TempDir, Vec<PathBuf>) {
    let folder = tempfile::tempdir().unwrap();
    let listed = names
        .iter()
        .map(|name| {
            let path = folder.path().join(name);
            fs::write(&path, b"model").unwrap();
            path
        })
        .collect();
    (folder, listed)
}

#[test]
fn the_choice_is_written_whole_read_back_and_removed_when_none_is_left() {
    let folder = meeting_folder();
    assert_eq!(meeting_models::read(folder.path()), Models::default());
    let chosen = Models {
        transcription: Some(whisper("ggml-small-q8_0.bin")),
        minutes: Some(writer(OLLAMA, "qwen3:8b")),
    };
    meeting_models::write(folder.path(), &chosen).unwrap();
    assert_eq!(meeting_models::read(folder.path()), chosen);
    assert_eq!(names(folder.path()), ["models.json", "session.json"]);
    // As the contract's page shows it.
    let written: serde_json::Value = serde_json::from_slice(&fs::read(folder.path().join("models.json")).unwrap()).unwrap();
    assert_eq!(
        written,
        serde_json::json!({"transcription": {"engine": "whisper", "model": "ggml-small-q8_0.bin"},
                           "minutes": {"provider": OLLAMA, "model": "qwen3:8b"}})
    );
    // One part taken back: the other stays.
    let one = Models {
        transcription: None,
        minutes: Some(writer(OLLAMA, "qwen3:8b")),
    };
    meeting_models::write(folder.path(), &one).unwrap();
    assert_eq!(meeting_models::read(folder.path()), one);
    // Both: the file goes, nothing left beside it.
    meeting_models::write(folder.path(), &Models::default()).unwrap();
    assert_eq!(names(folder.path()), ["session.json"]);
    meeting_models::write(folder.path(), &Models::default()).unwrap();
}

#[test]
fn nothing_is_written_before_the_meeting_holds_its_session() {
    let folder = tempfile::tempdir().unwrap();
    let chosen = Models {
        transcription: Some(Hearing::SpeechAnalyzer),
        minutes: None,
    };
    let refused = meeting_models::write(folder.path(), &chosen).unwrap_err();
    assert_eq!(refused.kind(), ErrorKind::NotFound);
    assert!(names(folder.path()).is_empty());
}

#[test]
fn a_part_the_contract_refuses_reads_as_absent_and_the_rest_still_reads() {
    let folder = meeting_folder();
    let file = folder.path().join("models.json");
    let cases = [
        r#"{"transcription": {"engine": "whisper", "model": "../x.bin"}, "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b"}}"#,
        r#"{"transcription": {"engine": "whisper", "model": "C:\\m\\x.bin"}, "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b"}}"#,
        r#"{"transcription": {"engine": "whisper", "model": ".."}, "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b"}}"#,
        r#"{"transcription": {"engine": "whisper"}, "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b"}}"#,
        r#"{"transcription": {"engine": "another", "model": "m"}, "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b"}}"#,
        r#"{"transcription": {"engine": "service", "provider": "", "model": "m"}, "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b"}}"#,
        r#"{"transcription": "ggml-small.bin", "minutes": {"provider": "http://localhost:11434/v1", "model": "qwen3:8b", "added_later": 1}}"#,
    ];
    for text in cases {
        fs::write(&file, text).unwrap();
        let read = meeting_models::read(folder.path());
        assert_eq!(read.transcription, None, "{text}");
        assert_eq!(read.minutes, Some(writer(OLLAMA, "qwen3:8b")), "{text}");
    }
    // A field it does not name is left aside: `model` beside `speechanalyzer`.
    fs::write(&file, r#"{"transcription": {"engine": "speechanalyzer", "model": "x"}}"#).unwrap();
    assert_eq!(meeting_models::read(folder.path()).transcription, Some(Hearing::SpeechAnalyzer));
    // No JSON object: no file.
    for text in ["", "[]", "not json", "{\"minutes\": 3}"] {
        fs::write(&file, text).unwrap();
        assert_eq!(meeting_models::read(folder.path()), Models::default(), "{text:?}");
    }
}

#[test]
fn what_the_meeting_names_stands_in_for_settings_choice_and_settings_are_unchanged() {
    let (models, listed) = installed(&["ggml-large-v3-turbo-q8_0.bin", "ggml-small-q8_0.bin"]);
    let settings = settings(models.path());
    let before = settings.clone();
    let chosen = Models {
        transcription: Some(whisper("ggml-small-q8_0.bin")),
        minutes: Some(writer(OLLAMA, "qwen3:8b")),
    };
    let applied = meeting_models::applied(&settings, &chosen, &listed);
    assert_eq!(applied.gone, Vec::<Step>::new());
    assert_eq!(applied.settings.engine, Engine::Whisper);
    assert_eq!(applied.settings.model, models.path().join("ggml-small-q8_0.bin"));
    assert_eq!(applied.settings.minutes_provider.as_deref(), Some(OLLAMA));
    assert_eq!(applied.settings.minutes_model.as_deref(), Some("qwen3:8b"));
    assert_eq!(
        Hearer::of(&applied.settings, "fr", || false),
        Hearer::Model(models.path().join("ggml-small-q8_0.bin"))
    );
    assert_eq!(settings, before);
    // None named: Settings' choice, whole.
    let none = meeting_models::applied(&settings, &Models::default(), &listed);
    assert_eq!((none.settings, none.gone), (settings.clone(), vec![]));
    // The service Settings set, with another of its models.
    let service = Models {
        transcription: Some(Hearing::Service {
            provider: SERVICE.into(),
            model: "whisper-large-v3-turbo".into(),
        }),
        minutes: None,
    };
    let applied = meeting_models::applied(&settings, &service, &listed);
    assert_eq!(
        Hearer::of(&applied.settings, "fr", || false),
        Hearer::Service {
            provider: SERVICE.into(),
            model: "whisper-large-v3-turbo".into()
        }
    );
    assert_eq!(applied.settings.minutes_model, settings.minutes_model);
}

#[test]
fn a_whisper_model_is_found_by_its_name_among_those_listed_never_as_a_path() {
    let (models, listed) = installed(&["ggml-large-v3-turbo-q8_0.bin", "ggml-small-q8_0.bin"]);
    let settings = settings(models.path());
    // A file beside the folder, reachable by a path the name makes: never taken.
    fs::write(models.path().join("x.bin"), b"model").unwrap();
    for name in ["C:x", "../x.bin", "x.bin", "NUL.bin", "CON", "...", " ..", "ggml-medium.bin"] {
        let chosen = Models {
            transcription: Some(whisper(name)),
            minutes: None,
        };
        let applied = meeting_models::applied(&settings, &chosen, &listed);
        assert_eq!(applied.settings, settings, "{name}");
        assert_eq!(applied.gone, vec![Step::Transcription], "{name}");
    }
}

#[test]
fn a_model_chosen_then_removed_gives_way_to_settings_choice_and_is_said() {
    let (models, mut listed) = installed(&["ggml-large-v3-turbo-q8_0.bin", "ggml-small-q8_0.bin"]);
    let settings = settings(models.path());
    let chosen = Models {
        transcription: Some(whisper("ggml-small-q8_0.bin")),
        minutes: None,
    };
    fs::remove_file(models.path().join("ggml-small-q8_0.bin")).unwrap();
    listed.retain(|path| path.is_file());
    let applied = meeting_models::applied(&settings, &chosen, &listed);
    assert_eq!(applied.settings, settings);
    assert_eq!(applied.gone, vec![Step::Transcription]);
    // A service no longer set in Settings is gone as well.
    let unset = Settings {
        transcription_provider: None,
        ..settings.clone()
    };
    let service = Models {
        transcription: Some(Hearing::Service {
            provider: SERVICE.into(),
            model: "whisper-large-v3".into(),
        }),
        minutes: None,
    };
    let applied = meeting_models::applied(&unset, &service, &listed);
    assert_eq!((applied.settings, applied.gone), (unset, vec![Step::Transcription]));
}

#[test]
fn a_confidential_meeting_never_goes_to_a_service_whatever_the_file_names() {
    let (models, listed) = installed(&["ggml-large-v3-turbo-q8_0.bin", "ggml-small-q8_0.bin"]);
    let settings = Settings {
        engine: Engine::Whisper,
        ..settings(models.path())
    };
    let chosen = Models {
        transcription: Some(Hearing::Service {
            provider: SERVICE.into(),
            model: "whisper-large-v3".into(),
        }),
        minutes: Some(writer("https://openrouter.ai/api/v1", "anthropic/claude-sonnet")),
    };
    let applied = meeting_models::applied(&settings, &chosen, &listed);
    // Not confidential, the service the meeting named hears it.
    assert!(matches!(Hearer::of(&applied.settings, "fr", || false), Hearer::Service { .. }));
    // Confidential: Whisper on this computer, and the minutes by Ollama here, never the service.
    let hearer = confidential::hearer(&applied.settings, "fr", || false);
    assert_eq!(hearer, Hearer::Model(models.path().join("ggml-large-v3-turbo-q8_0.bin")));
    let named = applied
        .settings
        .minutes_provider
        .clone()
        .zip(applied.settings.minutes_model.clone());
    let written = confidential::writer(named, "http://localhost:11434", || Some(vec!["qwen3:8b".into()])).unwrap();
    assert_eq!(written, ("http://localhost:11434/v1".to_string(), "qwen3:8b".to_string()));
    // A Whisper model the meeting chose is the one heard, confidential too.
    let small = Models {
        transcription: Some(whisper("ggml-small-q8_0.bin")),
        minutes: None,
    };
    let applied = meeting_models::applied(&settings, &small, &listed);
    assert_eq!(
        confidential::hearer(&applied.settings, "fr", || false),
        Hearer::Model(models.path().join("ggml-small-q8_0.bin"))
    );
}
