//! What the user is told: the sentences, read here as a user reads them, in both languages.

use std::path::PathBuf;

use kitokey_shell::{
    recording::{Said, Source},
    settings::Refused,
    words::{self, Failure, Language, Notice, Refusal},
};

const BOTH: [Language; 2] = [Language::English, Language::French];

fn told(said: &Said, reminder: bool, language: Language) -> (String, String) {
    let Notice { title, body } = words::of(said, reminder, language).expect("nothing was said");
    (title, body)
}

#[test]
fn the_interface_follows_the_systems_language_and_any_other_is_english() {
    for french in ["fr", "fr-FR", "fr_CA.UTF-8", "FR", "fr@euro"] {
        assert_eq!(Language::of(french), Language::French, "{french}");
    }
    for other in ["en-GB", "en_US.UTF-8", "de-DE", "C", "POSIX", "", "fra", "french", "frisian-NL"] {
        assert_eq!(Language::of(other), Language::English, "{other}");
    }
}

#[test]
fn the_reminder_is_a_setting_and_a_source_that_comes_back_calls_for_no_word() {
    let started = Said::Started(PathBuf::from("/s"), None, None);
    for language in BOTH {
        assert_eq!(words::of(&started, false, language), None);
        assert_eq!(words::of(&Said::Heard(Source::Me), true, language), None);
        assert_eq!(words::of(&Said::Heard(Source::Others), true, language), None);
    }
    assert_eq!(
        told(&started, true, Language::English),
        (
            "Recording".into(),
            "Remember to tell participants this meeting is being recorded.".into()
        )
    );
    assert_eq!(
        told(&started, true, Language::French),
        (
            "Enregistrement en cours".into(),
            "Pensez à prévenir les participants que la réunion est enregistrée.".into()
        )
    );
}

#[test]
fn the_recording_names_the_microphone_it_records() {
    let started = Said::Started(PathBuf::from("/s"), Some("Headset Mic".into()), None);
    assert_eq!(
        told(&started, true, Language::English).1,
        "Microphone: Headset Mic. Remember to tell participants this meeting is being recorded."
    );
    assert_eq!(
        told(&started, true, Language::French).1,
        "Micro\u{a0}: Headset Mic. Pensez à prévenir les participants que la réunion est enregistrée."
    );
    for language in BOTH {
        assert_eq!(words::of(&started, false, language), None);
    }
}

#[test]
fn a_source_that_is_silent_or_refused_is_named_and_the_recording_is_said_to_go_on() {
    for (source, english, french) in [(Source::Me, "microphone", "micro"), (Source::Others, "meeting", "réunion")] {
        let (title, body) = told(&Said::Silent(source), false, Language::English);
        assert!(
            title.contains(english) && body.ends_with("Recording continues."),
            "{title} — {body}"
        );
        let (title, body) = told(&Said::Silent(source), false, Language::French);
        assert!(
            title.contains(french) && body.ends_with("L’enregistrement continue."),
            "{title} — {body}"
        );
        // A source refused is named in the title, and the body says the other is still recorded.
        for language in BOTH {
            let (title, body) = told(&Said::Unavailable(source, String::new()), false, language);
            let (named, other) = if source == Source::Me {
                (french, "réunion")
            } else {
                (french, "micro")
            };
            let named = if language == Language::English { english } else { named };
            let other = match (language, source) {
                (Language::English, Source::Me) => "meeting",
                (Language::English, Source::Others) => "microphone",
                _ => other,
            };
            assert!(
                title.to_lowercase().contains(named) && body.contains(other) && !body.contains('('),
                "{title} — {body}"
            );
        }
    }
    // What the program said of a refusal is the log's, never the user's (D29).
    let refused = Said::Unavailable(Source::Me, "ALSA: the system names no default device".into());
    assert_eq!(
        told(&refused, false, Language::English),
        (
            "Your microphone isn’t being recorded".into(),
            "The meeting audio is still being recorded.".into()
        )
    );
    assert_eq!(
        told(&refused, false, Language::French).1,
        "Le son de la réunion, lui, est bien enregistré."
    );
}

#[test]
fn a_microphone_that_sends_nothing_is_told_from_one_that_hears_no_sound_and_named() {
    let mut sides = words::Sides::default();
    sides.told(&Said::Started("/s".into(), Some("Earbuds".into()), None), false, Language::French);
    let nothing = sides.told(&Said::Nothing, false, Language::French).unwrap();
    assert_eq!(nothing.title, "Votre micro ne transmet aucun signal");
    assert!(
        nothing.body.contains("du micro Earbuds")
            && nothing.body.contains("pas même un bruit de fond. Vérifiez")
            && !nothing.body.contains('\'')
            && nothing.body.ends_with("L’enregistrement continue."),
        "{}",
        nothing.body
    );
    assert_ne!(nothing.title, told(&Said::Silent(Source::Me), false, Language::French).0);
    // Moved since: the device recorded now is the one named.
    sides.told(&Said::Moved(Source::Me, Some("Built-in Mic".into())), false, Language::English);
    let nothing = sides.told(&Said::Nothing, false, Language::English).unwrap();
    assert!(
        nothing.title == "Your microphone isn’t sending any signal" && nothing.body.contains("Built-in Mic"),
        "{} — {}",
        nothing.title,
        nothing.body
    );
    // No microphone left, said so: the zeros kept in its place are no news.
    sides.told(&Said::Lost(Source::Me), false, Language::French);
    assert_eq!(sides.told(&Said::Nothing, false, Language::French), None);
    // Named by nothing: the microphone, in so many words.
    let (title, body) = told(&Said::Nothing, false, Language::English);
    assert!(
        title == "Your microphone isn’t sending any signal" && body.contains("your microphone"),
        "{body}"
    );
}

#[test]
fn french_is_written_as_french_is_printed() {
    // A space that does not break before a colon and a question mark: a notification never
    // starts a line on one.
    let silent = told(&Said::Silent(Source::Others), false, Language::French).1;
    assert!(
        silent.contains("secondes\u{a0}: l’appel") && silent.contains("ordinateur\u{202f}?"),
        "{silent}"
    );
    assert_eq!(
        told(&Said::Stopped("/s".into(), 1912.4), false, Language::French).1,
        "32 min enregistrées."
    );
    // No straight apostrophe in a sentence of ours, in either language.
    let all = [
        Said::Started("/s".into(), None, None),
        Said::Started("/s".into(), Some("Mic".into()), None),
        Said::Unavailable(Source::Me, String::new()),
        Said::Unavailable(Source::Others, String::new()),
        Said::Silent(Source::Me),
        Said::Silent(Source::Others),
        Said::Nothing,
        Said::Stopped("/s".into(), 60.0),
        Said::Moved(Source::Me, Some("Mic".into())),
        Said::Moved(Source::Me, None),
        Said::Moved(Source::Others, Some("Speaker".into())),
        Said::Moved(Source::Others, None),
        Said::Lost(Source::Me),
        Said::Lost(Source::Others),
        Said::Failed(String::new()),
        Said::Failed("x".into()),
    ];
    for (said, language) in all.iter().flat_map(|said| BOTH.map(|language| (said, language))) {
        let (title, body) = told(said, true, language);
        assert!(!title.contains('\'') && !body.contains('\''), "{title} — {body}");
    }
}

#[test]
fn a_length_reads_as_a_person_says_it_each_to_the_nearest() {
    for (seconds, said) in [
        (0.0, "0 s"),
        (0.4, "0 s"),
        (0.5, "1 s"),
        (59.4, "59 s"),
        (59.5, "1 min"),
        (89.4, "1 min"),
        (90.0, "2 min"),
        (119.4, "2 min"),
        (1912.4, "32 min"),
        (3569.0, "59 min"),
        (3570.0, "1 h 00"),
        (5400.0, "1 h 30"),
        (86400.0, "24 h 00"),
        (-3.0, "0 s"),
        (f64::NAN, "0 s"),
        (f64::INFINITY, "0 s"),
    ] {
        assert_eq!(words::length(seconds), said, "{seconds}");
    }
    assert_eq!(
        told(&Said::Stopped("/s".into(), 42.4), false, Language::English).1,
        "42 s recorded."
    );
}

#[test]
fn a_failure_never_says_what_the_program_said_of_it() {
    let said = |sentence: &str, language| told(&Said::Failed(sentence.into()), false, language).1;
    // The sentence is the user's; what the program said is the log's, and the developer mode's (D29).
    for sentence in [
        "kitokey-capture: no source could be opened: nothing to record",
        "Timeout: no sound for 30 s",
        "(os error 28)",
        &"é".repeat(5000),
    ] {
        for language in BOTH {
            let body = said(sentence, language);
            assert!(
                !body.contains('(') && (body.ends_with("meeting’s folder.") || body.ends_with("dossier de la réunion.")),
                "{sentence:?}: {body}"
            );
        }
    }
    // A program that said nothing — or its name and nothing — is said as an unexpected error, never its words.
    for nothing in ["", "   ", "kitokey-capture:", " kitokey-capture: \t", "()"] {
        for language in BOTH {
            let body = said(nothing, language);
            assert!(
                !body.contains('(') && (body.contains("Something went wrong") || body.contains("Une erreur inattendue")),
                "{nothing:?}: {body}"
            );
        }
    }
}

#[test]
fn no_notice_of_a_failure_carries_what_a_program_or_the_system_said() {
    use kitokey_shell::{
        transcription::Made,
        words::{Minutes, Trouble},
    };
    const SAID: &str = "kitokey-summary: CANARY (os error 2)";
    for language in BOTH {
        let mut notices = vec![
            words::trouble(&Trouble::NoRecorder(SAID.into()), language),
            words::trouble(&Trouble::Shortcut(SAID.into()), language),
            words::trouble(&Trouble::NoTranscriber(SAID.into()), language),
            words::made(&Made::Failed(SAID.into()), language),
            words::minutes(&Minutes::Failed(None), language),
            words::minutes(&Minutes::RedoFailed(None), language),
            words::fetched(&Err(SAID.into()), true, language),
        ];
        for said in [
            Said::Failed(SAID.into()),
            Said::Unavailable(Source::Me, SAID.into()),
            Said::Unavailable(Source::Others, SAID.into()),
        ] {
            notices.extend(words::of(&said, false, language));
        }
        for Notice { title, body } in notices {
            assert!(
                !title.contains("CANARY") && !body.contains("CANARY") && !body.contains('('),
                "{title} — {body}"
            );
        }
    }
}

#[test]
fn the_menu_and_the_shells_own_troubles_are_said_in_both_languages() {
    use words::Trouble;
    let (english, french) = (words::menu(Language::English), words::menu(Language::French));
    assert_eq!(
        (english.start, english.stop, english.quit),
        ("Start recording", "Stop recording", "Quit Kitokey")
    );
    assert_eq!(
        (french.start, french.stop),
        ("Démarrer un enregistrement", "Arrêter l’enregistrement")
    );
    // No entry is the same in the two languages, and none is empty.
    for (one, other) in [
        (english.start, french.start),
        (english.stop, french.stop),
        (english.stopping, french.stopping),
        (english.quit, french.quit),
        (english.recording, french.recording),
        (english.transcribe, french.transcribe),
        (english.transcribing, french.transcribing),
    ] {
        assert!(
            !one.is_empty() && !other.is_empty() && one != other && !one.contains('\'') && !other.contains('\''),
            "{one} / {other}"
        );
    }
    // A trouble says what it is and what still works. Where the user has something to put
    // right — the settings file's line, the place the model goes —, it says where (D29).
    for language in BOTH {
        for trouble in [
            Trouble::NoRecorder(String::new()),
            Trouble::Settings(Refused::Unparsed {
                line: 3,
                key: Some("reminder"),
            }),
            Trouble::Shortcut(String::new()),
            Trouble::NoModel("<data>/kitokey/models/ggml.bin".into()),
            Trouble::NoTranscriber(String::new()),
        ] {
            let Notice { title, body } = words::trouble(&trouble, language);
            assert!(
                !title.is_empty() && !title.contains('\'') && !body.contains('('),
                "{title} — {body}"
            );
        }
        let unparsed = Refused::Unparsed {
            line: 3,
            key: Some("reminder"),
        };
        let settings = words::trouble(&Trouble::Settings(unparsed), language).body;
        assert!(settings.ends_with(&words::refused(&unparsed, language)), "{settings}");
        let model = words::trouble(&Trouble::NoModel("<data>/kitokey/models/ggml.bin".into()), language).body;
        assert!(model.contains("<data>/kitokey/models/ggml.bin"), "{model}");
    }
    // Each says what still works, or what to do — not the trouble a second time.
    let body = |trouble, language| words::trouble(&trouble, language).body;
    assert!(body(Trouble::NoRecorder(String::new()), Language::English).ends_with("Reinstall Kitokey to fix it."));
    assert!(body(Trouble::NoRecorder(String::new()), Language::French).ends_with("Réinstallez Kitokey pour le réparer."));
    assert!(body(Trouble::Settings(Refused::Unopened), Language::English).contains("using default settings"));
    assert!(body(Trouble::Shortcut(String::new()), Language::French).contains("pouvez toujours démarrer"));
}

#[test]
fn a_transcript_is_said_ready_or_not_with_how_to_have_it_again() {
    use kitokey_shell::{
        transcription::Made,
        words::{Minutes, Trouble},
    };
    let ready = |language| words::made(&Made::Written("/s/2026-10-16_09-30".into()), language);
    assert_eq!(ready(Language::English).title, "Transcript ready");
    // D44: on Linux a click opens the meeting in Kitokey, and says so; elsewhere no click is
    // promised — the notification is said, and the window holds the meeting.
    if cfg!(target_os = "linux") {
        assert_eq!(ready(Language::French).body, "Cliquez pour l’ouvrir.");
        assert_eq!(ready(Language::English).body, "Click to open it.");
        assert_eq!(words::minutes(&Minutes::Ready, Language::French).body, "Cliquez pour l’ouvrir.");
    } else {
        assert!(!ready(Language::French).body.contains("Cliquez") && !ready(Language::English).body.contains("Click"));
        assert!(!words::minutes(&Minutes::Ready, Language::English).body.contains("Click"));
    }
    assert!(!ready(Language::French).body.contains("dossier"));
    // Failed: the recording safe, the menu's entry by its name — never what the program said.
    let failed = words::made(
        &Made::Failed("kitokey-transcription: the model cannot be loaded".into()),
        Language::English,
    );
    assert!(
        failed.body.contains("safe") && failed.body.contains(words::menu(Language::English).transcribe),
        "{}",
        failed.body
    );
    assert!(!failed.body.contains("cannot be loaded"), "{}", failed.body);
    let french = words::made(&Made::Failed(String::new()), Language::French);
    assert!(french.body.contains("Transcrire le dernier enregistrement") && !french.body.contains('('));
    // The entry named in the sentences is the menu's own.
    for language in BOTH {
        let entry = words::menu(language).transcribe;
        assert!(words::made(&Made::Failed(String::new()), language).body.contains(entry), "{entry}");
        assert!(
            words::trouble(&Trouble::NoModel(String::new()), language).body.contains(entry),
            "{entry}"
        );
        let none = words::trouble(&Trouble::NoSession, language);
        assert!(!none.title.is_empty() && !none.body.contains('('), "{}", none.body);
    }
    assert_ne!(words::follows(Language::English), words::follows(Language::French));
}

#[test]
fn the_model_is_offered_with_its_size_and_host_and_its_fetch_is_said() {
    for language in BOTH {
        let entry = words::download(language, "874 MB", "huggingface.co");
        assert!(entry.contains("874 MB") && entry.contains("huggingface.co"), "{entry}");
        // The notice names the very entry to choose, and keeps the recording.
        let needed = words::model_needed(language, "874 MB", "huggingface.co");
        assert!(needed.body.contains(&entry), "{}", needed.body);
        let kept = if language == Language::French {
            "L’enregistrement est conservé"
        } else {
            "The recording is saved"
        };
        assert!(needed.body.contains(kept), "{}", needed.body);
        assert!(words::downloading(language, 37).contains("37"));
        // While it comes, its share; every byte there, the check — never "100 %" for a while.
        assert_eq!(words::fetching(language, 0, 874), words::downloading(language, 0));
        assert_eq!(words::fetching(language, 437, 874), words::downloading(language, 50));
        assert_eq!(words::fetching(language, 873, 874), words::downloading(language, 99));
        let checking = words::fetching(language, 874, 874);
        assert!(
            !checking.contains('%') && checking != words::downloading(language, 100),
            "{checking}"
        );
        let refused = words::fetched(&Err("what came is not the published model".into()), true, language);
        assert!(!refused.body.contains("published model"), "{}", refused.body);
        assert_ne!(words::fetched(&Ok(()), true, language).title, refused.title);
        // Another than the one chosen: no transcript said begun, the choice left to the user (D37).
        assert_ne!(
            words::fetched(&Ok(()), false, language).body,
            words::fetched(&Ok(()), true, language).body
        );
    }
    assert_eq!(words::fetched(&Ok(()), true, Language::French).title, "Modèle prêt");
}

#[test]
fn audio_sent_to_the_trash_is_said_with_its_counts_and_a_refusal_never_claims_too_much() {
    let said = |language, outcome: Result<(usize, u32), Refusal>| {
        let Notice { title, body } = words::audio_trashed(language, &outcome);
        format!("{title} | {body}")
    };
    assert_eq!(
        said(Language::English, Ok((1, 1))),
        "Audio moved to trash | The audio of 1 meeting older than 1 day is in the trash; its transcript and summary remain."
    );
    assert_eq!(
        said(Language::English, Ok((3, 30))),
        "Audio moved to trash | The audio of 3 meetings older than 30 days is in the trash; \
         their transcripts and summaries remain."
    );
    assert_eq!(
        said(Language::French, Ok((1, 1))),
        "Audio déplacé dans la corbeille | L’audio d’une réunion de plus d’un jour est dans la corbeille\u{202f}; \
         sa transcription et son compte rendu restent."
    );
    assert_eq!(
        said(Language::French, Ok((3, 30))),
        "Audio déplacé dans la corbeille | L’audio de 3 réunions de plus de 30 jours est dans la \
         corbeille\u{202f}; leurs transcriptions et comptes rendus restent."
    );
    // The trash may have taken some before it refused: what is said holds either way, in the
    // user's language, without the system's own wording.
    assert_eq!(
        said(Language::English, Err(Refusal::NotAllowed)),
        "Some audio couldn’t be moved to the trash | The trash refused access. The rest is untouched — \
         nothing was deleted."
    );
    assert_eq!(
        said(Language::French, Err(Refusal::NoSpace)),
        "Audio pas entièrement mis à la corbeille | La corbeille est pleine. Le reste est intact\u{a0}: \
         rien n’a été supprimé."
    );
    assert_eq!(
        said(Language::French, Err(Refusal::Other)),
        "Audio pas entièrement mis à la corbeille | La corbeille du système a refusé ces fichiers. Le reste \
         est intact\u{a0}: rien n’a été supprimé."
    );
    assert!(said(Language::English, Err(Refusal::NoSpace)).contains("The trash is full."));
    assert!(said(Language::English, Err(Refusal::Other)).contains("The system trash refused these files."));
    assert!(said(Language::French, Err(Refusal::NotAllowed)).contains("La corbeille a refusé l’accès."));
}

#[test]
fn the_minutes_are_said_ready_waiting_or_not_made_with_what_to_do() {
    use kitokey_shell::words::Minutes;
    for language in BOTH {
        let entry = words::menu(language).minutes;
        let ready = words::minutes(&Minutes::Ready, language);
        assert!(!ready.title.is_empty());
        // Failed: the transcript untouched, the entry that makes them again — never the program's sentence.
        let failed = words::minutes(&Minutes::Failed(None), language);
        assert!(
            failed.body.contains(entry) && !failed.body.contains("refused the key"),
            "{}",
            failed.body
        );
        // Waiting: nothing sent to that host before the user agrees, in the window.
        let waiting = words::minutes(&Minutes::Waiting("openrouter.ai".into()), language);
        assert!(waiting.body.contains("openrouter.ai"), "{}", waiting.body);
        for other in [Minutes::Unchosen, Minutes::NoTranscript] {
            let said = words::minutes(&other, language);
            assert!(
                !said.title.is_empty() && !said.body.is_empty() && !said.body.contains('('),
                "{}",
                said.body
            );
        }
        assert_ne!(words::menu(language).minuting, entry);
    }
    assert_eq!(words::minutes(&Minutes::Ready, Language::French).title, "Compte rendu prêt");
    // The minutes replaced, kept beside when the trash refused them: said, with where.
    for language in BOTH {
        assert!(
            words::minutes(&Minutes::ReadyKeptBeside("minutes-before-1760751234.md".into()), language)
                .body
                .contains("minutes-before-1760751234.md")
        );
        assert!(
            words::transcript_kept_beside(language, "transcript-before-1760751234.jsonl", false)
                .body
                .contains("transcript-before-1760751234.jsonl")
        );
        // A transcript made again that failed, and the one there not put back: said as such,
        // never as a trash that refused.
        let back = words::transcript_kept_beside(language, "transcript-before-1760751234.jsonl", true);
        let trashed = words::transcript_kept_beside(language, "transcript-before-1760751234.jsonl", false);
        assert!(
            back.body.contains("transcript-before-1760751234.jsonl") && back.title != trashed.title && back.body != trashed.body,
            "{}",
            back.body
        );
        // Minutes made again that failed: the ones there stayed, and the menu's entry is not offered.
        let redo = words::minutes(&Minutes::failed(None, true), language);
        assert_eq!(
            words::minutes(&Minutes::failed(None, false), language).title,
            words::minutes(&Minutes::Failed(None), language).title
        );
        let failed = words::minutes(&Minutes::Failed(None), language);
        assert!(
            redo.title != failed.title && !redo.body.contains(words::menu(language).minutes) && !redo.body.contains("not minutes"),
            "{}",
            redo.body
        );
    }
    assert_eq!(
        words::minutes(&Minutes::RedoFailed(None), Language::French).title,
        "Le compte rendu n’a pas pu être refait"
    );
    assert_eq!(
        words::minutes(&Minutes::Waiting("openrouter.ai".into()), Language::French).body,
        "Rien n’est envoyé à openrouter.ai sans votre accord. Donnez-le dans la fenêtre de Kitokey."
    );
}

#[test]
fn a_failure_the_user_can_put_right_says_where_and_the_gesture() {
    use kitokey_shell::words::{Minutes, Trouble};
    for language in BOTH {
        // The meetings' folder cannot be made: the settings choose another, reinstalling does not.
        let folder = words::trouble(&Trouble::NoFolder("/media/gone/Kitokey".into()), language).body;
        assert!(
            folder.contains("/media/gone/Kitokey") && !folder.contains("nstall") && !folder.contains("éinstall"),
            "{folder}"
        );
        // Who writes the minutes is no provider's address: the settings, not a try again.
        let provider = words::minutes(&Minutes::NoProvider("ollama:11434".into()), language).body;
        assert!(
            provider.contains("ollama:11434") && !provider.contains(words::menu(language).minutes),
            "{provider}"
        );
    }
}

#[test]
fn the_page_has_what_a_program_said_only_in_developer_mode() {
    assert_eq!(words::shown_detail("kitokey-summary: the key was refused", false), "");
    assert_eq!(
        words::shown_detail("kitokey-summary: the key was refused", true),
        "kitokey-summary: the key was refused"
    );
}

#[test]
fn minutes_not_made_say_their_cause_and_its_gesture_when_the_program_named_one() {
    use kitokey_shell::{summarising::Cause, words::Minutes};
    const ALL: [Cause; 16] = [
        Cause::TooMany,
        Cause::Down,
        Cause::NoTimes,
        Cause::Unreachable,
        Cause::TooSlow,
        Cause::RefusedKey,
        Cause::NoCredit,
        Cause::UnknownModel,
        Cause::Redirected,
        Cause::ProviderFailed,
        Cause::CutShort,
        Cause::NotMinutes,
        Cause::TranscriptUnusable,
        Cause::NoSpeech,
        Cause::Settings,
        Cause::Unwritable,
    ];
    // Where the gesture is a setting, the sentence says the settings, not the menu's entry.
    const SETTINGS: [Cause; 8] = [
        Cause::RefusedKey,
        Cause::UnknownModel,
        Cause::Redirected,
        Cause::Settings,
        Cause::CutShort,
        Cause::TooSlow,
        Cause::NotMinutes,
        Cause::NoTimes,
    ];
    // What each sentence is about, in each language: a sentence said for another cause fails.
    let about = |cause: Cause| match cause {
        Cause::Unreachable => ("Can’t reach", "Impossible de joindre"),
        Cause::TooSlow => ("too long to respond", "pas répondu à temps"),
        Cause::RefusedKey => ("rejected the API key", "refusé la clé"),
        Cause::NoCredit => ("credit", "Crédit"),
        Cause::UnknownModel => ("doesn’t recognize", "ne reconnaît pas"),
        Cause::Redirected => ("another address", "autre adresse"),
        Cause::ProviderFailed => ("couldn’t respond", "pas pu répondre"),
        Cause::CutShort => ("before finishing", "avant la fin"),
        Cause::NotMinutes => ("didn’t produce a summary", "pas produit de compte rendu"),
        Cause::TranscriptUnusable => ("can’t be read", "illisible"),
        Cause::NoSpeech => ("No speech", "Aucune parole"),
        Cause::Settings => ("isn’t valid", "ne sont pas valides"),
        Cause::Unwritable => ("Couldn’t save", "Impossible d’enregistrer"),
        Cause::TooMany => ("limiting requests", "limite le nombre de demandes"),
        Cause::Down => ("having problems", "rencontre un problème"),
        Cause::NoTimes => ("timestamps", "horodatage"),
    };
    for language in BOTH {
        let mut said = std::collections::HashSet::new();
        for cause in ALL {
            let sentence = words::cause(cause, language);
            assert!(
                !sentence.is_empty() && !sentence.contains('\'') && said.insert(sentence),
                "{cause:?}: {sentence}"
            );
            let (english, french) = about(cause);
            let topic = if language == Language::French { french } else { english };
            assert!(sentence.contains(topic), "{cause:?}: {sentence}");
            if language == Language::French {
                // Every colon after a space that does not break, as French is printed.
                assert!(
                    sentence.match_indices(':').all(|(at, _)| sentence[..at].ends_with('\u{a0}')),
                    "{sentence}"
                );
            }
            for failed in [Minutes::failed(Some(cause), false), Minutes::failed(Some(cause), true)] {
                let body = words::minutes(&failed, language).body;
                assert!(body.contains(sentence), "{cause:?}: {body}");
                if SETTINGS.contains(&cause) {
                    let settings = if language == Language::French { "réglages" } else { "settings" };
                    assert!(
                        body.to_lowercase().contains(settings) && !body.contains(words::menu(language).minutes),
                        "{cause:?}: {body}"
                    );
                }
            }
        }
        // No cause named, or one this version does not know: the step's sentence, as before.
        assert!(
            words::minutes(&Minutes::Failed(None), language)
                .body
                .contains(words::menu(language).minutes)
        );
    }
}

#[test]
fn the_minutes_sections_are_named_as_the_contract_and_d11_name_them() {
    // minutes/v1's own names in English; D11's second board's in French.
    assert_eq!(
        words::window(Language::English).minutes_sections,
        ["Highlights", "Actions", "Key points"]
    );
    assert_eq!(
        words::window(Language::French).minutes_sections,
        ["Points saillants", "Actions", "Points clés"]
    );
}

#[test]
fn a_device_changed_says_what_is_recorded_now_one_gone_says_that_side_and_one_back_says_so() {
    // D31, each sentence as the user reads it.
    use Language::{English, French};
    let mic = Some("Headset Mic".to_string());
    for (said, language, title, body) in [
        (
            Said::Moved(Source::Me, mic.clone()),
            English,
            "Microphone changed",
            "Now recording from Headset Mic.",
        ),
        (
            Said::Moved(Source::Me, mic.clone()),
            French,
            "Micro changé",
            "Enregistré maintenant\u{a0}: Headset Mic.",
        ),
        (
            Said::Moved(Source::Others, Some("Speaker".into())),
            English,
            "Sound output changed",
            "The meeting’s sound is now recorded from Speaker.",
        ),
        (
            Said::Moved(Source::Others, Some("Speaker".into())),
            French,
            "Sortie audio changée",
            "Le son de la réunion est maintenant enregistré depuis Speaker.",
        ),
        (
            Said::Moved(Source::Me, None),
            English,
            "Microphone changed",
            "Now recording from your computer’s current microphone.",
        ),
        (
            Said::Moved(Source::Me, None),
            French,
            "Micro changé",
            "Kitokey enregistre désormais le micro utilisé par votre ordinateur.",
        ),
        (
            Said::Moved(Source::Others, None),
            English,
            "Sound output changed",
            "Meeting audio is now recorded from your computer’s current output.",
        ),
        (
            Said::Moved(Source::Others, None),
            French,
            "Sortie audio changée",
            "Le son de la réunion est désormais enregistré depuis la sortie actuelle de votre ordinateur.",
        ),
        // None left: that side is not recorded for now — nothing said of the other, which may
        // have none either. The microphone: never why, a call may simply have let go (D66).
        (
            Said::Lost(Source::Me),
            English,
            "Your microphone isn’t being recorded",
            "Recording continues.",
        ),
        (
            Said::Lost(Source::Me),
            French,
            "Votre micro n’est pas enregistré",
            "L’enregistrement continue.",
        ),
        (
            Said::Lost(Source::Others),
            English,
            "Meeting audio is no longer being recorded",
            "Your computer has no audio output. Plug one in to resume; recording continues.",
        ),
        (
            Said::Lost(Source::Others),
            French,
            "Le son de la réunion n’est plus enregistré",
            "Votre ordinateur n’a plus de sortie audio. Branchez-en une pour reprendre\u{202f}; l’enregistrement continue.",
        ),
    ] {
        assert_eq!(
            told(&said, false, language),
            (title.to_string(), body.to_string()),
            "{said:?} {language:?}"
        );
    }
    // The meeting's sound back after none was left: recorded again, and said so.
    for (name, language, title, body) in [
        (Some("Speaker"), English, "Meeting audio is being recorded again", "From Speaker."),
        (
            Some("Speaker"),
            French,
            "Le son de la réunion est de nouveau enregistré",
            "Depuis Speaker.",
        ),
        (
            None,
            French,
            "Le son de la réunion est de nouveau enregistré",
            "Depuis la sortie audio actuelle.",
        ),
    ] {
        let Notice { title: got, body: said } = words::back(name, language);
        assert_eq!((got.as_str(), said.as_str()), (title, body), "{language:?}");
    }
}

#[test]
fn what_the_devices_are_decides_what_is_said_of_one_gone_and_one_back() {
    use Language::{English, French};
    use words::Sides;
    let (lost, back) = (Said::Lost(Source::Others), Said::Moved(Source::Others, Some("Casque".into())));
    let body = |sides: &mut Sides, said: &Said, language| sides.told(said, false, language).map(|notice| notice.body);

    // The microphone still recorded: the output gone says so — the criterion's "the recording
    // goes on with the other".
    let mut sides = Sides::default();
    assert_eq!(
        body(&mut sides, &lost, English).as_deref(),
        Some("Your computer has no audio output. Your microphone is still recorded; plug in an output to resume.")
    );
    // Back: recorded again; then a move like any other.
    assert_eq!(
        sides.told(&back, false, English).unwrap().title,
        "Meeting audio is being recorded again"
    );
    assert_eq!(sides.told(&back, false, English).unwrap().title, "Sound output changed");

    // Both gone: neither is said still recorded.
    let mut sides = Sides::default();
    sides.told(&Said::Lost(Source::Me), false, French);
    assert_eq!(
        body(&mut sides, &lost, French).as_deref(),
        Some("Votre ordinateur n’a plus de sortie audio. Branchez-en une pour reprendre\u{202f}; l’enregistrement continue.")
    );
    // The microphone not opened at the start: the same.
    let mut sides = Sides::default();
    sides.told(&Said::Unavailable(Source::Me, String::new()), false, English);
    assert_eq!(
        body(&mut sides, &lost, English).as_deref(),
        Some("Your computer has no audio output. Plug one in to resume; recording continues.")
    );
    // And the other way round, in French, the meeting's sound still recorded.
    let mut sides = Sides::default();
    assert_eq!(
        body(&mut sides, &Said::Lost(Source::Me), French).as_deref(),
        Some("Le son de la réunion, lui, est bien enregistré.")
    );
}

#[test]
fn a_microphone_not_recorded_says_nothing_of_why_and_no_gesture() {
    use Language::{English, French};
    use words::Sides;
    for (said, language, title, body) in [
        (
            Said::Unavailable(Source::Me, "the call holds it".into()),
            English,
            "Your microphone isn’t being recorded",
            "The meeting audio is still being recorded.",
        ),
        (
            Said::Unavailable(Source::Me, "the call holds it".into()),
            French,
            "Votre micro n’est pas enregistré",
            "Le son de la réunion, lui, est bien enregistré.",
        ),
        (
            Said::Lost(Source::Me),
            English,
            "Your microphone isn’t being recorded",
            "The meeting audio is still being recorded.",
        ),
        (
            Said::Lost(Source::Me),
            French,
            "Votre micro n’est pas enregistré",
            "Le son de la réunion, lui, est bien enregistré.",
        ),
    ] {
        let notice = Sides::default().told(&said, false, language).unwrap();
        assert_eq!((notice.title.as_str(), notice.body.as_str()), (title, body), "{said:?}");
    }
}

#[test]
fn a_microphone_going_is_said_once_a_recording_and_its_return_never() {
    use Language::{English, French};
    use words::Sides;
    let (lost, taken) = (Said::Lost(Source::Me), Said::Moved(Source::Me, Some("AirPods".into())));
    let waiting = Said::Unavailable(Source::Me, String::new());
    let title = |sides: &mut Sides, said: &Said| sides.told(said, false, French).map(|notice| notice.title);
    let not_recorded = Some("Votre micro n’est pas enregistré");

    // It waited from the start: said once, however often a call comes.
    let mut sides = Sides::default();
    assert_eq!(title(&mut sides, &waiting).as_deref(), not_recorded);
    assert_eq!(title(&mut sides, &taken), None);
    assert_eq!(title(&mut sides, &lost), None);
    assert_eq!(title(&mut sides, &taken), None);
    assert_eq!(title(&mut sides, &lost), None);

    // In the contract's order, `started` after what waits: still said once.
    let mut sides = Sides::default();
    assert_eq!(title(&mut sides, &waiting).as_deref(), not_recorded);
    sides.told(&Said::Started("/s".into(), None, None), false, French);
    assert_eq!(title(&mut sides, &taken), None);
    assert_eq!(title(&mut sides, &lost), None);

    // It was recorded: the same sentence, in both languages, once.
    let mut sides = Sides::default();
    sides.told(&Said::Started("/s".into(), Some("AirPods".into()), None), false, French);
    assert_eq!(title(&mut sides, &lost).as_deref(), not_recorded);
    assert_eq!(title(&mut sides, &taken), None);
    assert_eq!(title(&mut sides, &lost), None);
    let notice = Sides::default().told(&lost, false, English).unwrap();
    assert_eq!(notice.title, "Your microphone isn’t being recorded");
    assert_eq!(notice.body, "The meeting audio is still being recorded.");

    // Back, it is recorded: a microphone that sends nothing is told again.
    assert_eq!(title(&mut sides, &taken), None);
    assert_eq!(
        title(&mut sides, &Said::Nothing).as_deref(),
        Some("Votre micro ne transmet aucun signal")
    );

    // Neither side left: the meeting's sound is not said recorded, however the two went.
    for (others, me) in [
        (Said::Lost(Source::Others), Said::Lost(Source::Me)),
        (Said::Unavailable(Source::Others, String::new()), waiting.clone()),
    ] {
        let mut sides = Sides::default();
        sides.told(&others, false, French);
        let notice = sides.told(&me, false, French).unwrap();
        assert_eq!(notice.title, "Votre micro n’est pas enregistré");
        assert_eq!(notice.body, "L’enregistrement continue.");
    }

    // The meeting's side is not a call's to take: each said, as before (D31).
    let (gone, back) = (Said::Lost(Source::Others), Said::Moved(Source::Others, Some("Casque".into())));
    let mut sides = Sides::default();
    for _ in 0..2 {
        assert!(title(&mut sides, &gone).is_some());
        assert!(title(&mut sides, &back).is_some());
    }
}

#[test]
fn a_microphone_is_said_changed_only_from_the_last_name_said() {
    use Language::French;
    use words::Sides;
    let (a, b) = (
        Said::Moved(Source::Me, Some("Micro du portable".into())),
        Said::Moved(Source::Me, Some("AirPods".into())),
    );
    let (lost, waiting) = (Said::Lost(Source::Me), Said::Unavailable(Source::Me, String::new()));
    let said = |sides: &mut Sides, said: &Said| sides.told(said, false, French).map(|notice| notice.body);
    let named = |name: &str| Some(format!("Enregistré maintenant\u{a0}: {name}."));
    let from_a = || {
        let mut sides = Sides::default();
        sides.told(&Said::Started("/s".into(), Some("Micro du portable".into()), None), false, French);
        sides
    };

    // A, B, A: each said, so the last word stays the true one.
    let mut sides = from_a();
    assert_eq!(said(&mut sides, &a), None);
    assert_eq!(said(&mut sides, &b), named("AirPods"));
    assert_eq!(said(&mut sides, &b), None);
    assert_eq!(said(&mut sides, &a), named("Micro du portable"));

    // Back after a gap with no other between: silent. Another: said, and it is the last now.
    let mut sides = from_a();
    sides.told(&lost, false, French);
    assert_eq!(said(&mut sides, &a), None);
    sides.told(&lost, false, French);
    assert_eq!(said(&mut sides, &b), named("AirPods"));
    sides.told(&lost, false, French);
    assert_eq!(said(&mut sides, &b), None);
    assert_eq!(said(&mut sides, &a), named("Micro du portable"));

    // A call's first take is silent, and is the last name; only then does a change count.
    let mut sides = Sides::default();
    sides.told(&waiting, false, French);
    sides.told(&Said::Started("/s".into(), None, None), false, French);
    assert_eq!(said(&mut sides, &b), None);
    assert_eq!(said(&mut sides, &b), None);
    assert_eq!(said(&mut sides, &a), named("Micro du portable"));
    sides.told(&lost, false, French);
    assert_eq!(said(&mut sides, &a), None);

    // Gone before the first take: still the first take.
    let mut sides = Sides::default();
    sides.told(&waiting, false, French);
    sides.told(&lost, false, French);
    assert_eq!(said(&mut sides, &b), None);
    sides.told(&lost, false, French);
    assert_eq!(said(&mut sides, &b), None);
    assert_eq!(said(&mut sides, &a), named("Micro du portable"));
}

#[test]
fn the_first_run_s_french_is_typeset_as_french_and_both_languages_say_it() {
    let french = words::window(Language::French);
    let english = words::window(Language::English);
    let said = [
        french.welcome_question,
        french.welcome_writer,
        french.welcome_here[1],
        french.welcome_connected[1],
        french.welcome_ollama_silent,
        french.welcome_ollama_lacks,
        french.welcome_again[1],
    ];
    for sentence in said {
        // A space before ? ! ; is a narrow one, before : a no-break one — never a plain space.
        for mark in ['?', '!', ';', ':'] {
            assert!(!sentence.contains(&format!(" {mark}")), "{sentence}");
        }
        assert!(!sentence.contains('\''), "{sentence}");
    }
    assert!(french.welcome_question.ends_with("confidentiels\u{202f}?"));
    assert!(french.welcome_ollama_silent.contains("ordinateur\u{a0}:") && french.welcome_ollama_silent.contains("ollama.com"));
    assert_eq!(french.welcome_private[0], "Mode privé");
    assert_eq!(english.welcome_private[0], "Private mode");
    assert_eq!(english.welcome_connected[0], "Connected mode");
    assert!(english.welcome_ollama_silent.contains("ollama.com"));
}

#[test]
fn the_context_s_words_are_said_in_both_languages_french_typeset_as_french() {
    let french = words::window(Language::French);
    let english = words::window(Language::English);
    // The field's name, as D59's criterion says it.
    assert_eq!(french.context_label, "De quoi parle la réunion\u{202f}? Qui y participe\u{202f}?");
    assert_eq!(english.context_label, "What’s the meeting about? Who’s attending?");
    // Its button names what it starts (D113, the decider, 2026-11-18): the transcript, and the summary
    // after it where one follows — as `context_changed` says what was made.
    assert_eq!(
        french.context_redo,
        ["Refaire la transcription", "Refaire la transcription et le compte rendu"]
    );
    assert_eq!(
        english.context_redo,
        ["Write the transcript again", "Write the transcript and the summary again"]
    );
    // So does the summary's, made again with a transcript's corrections (D113).
    assert_eq!(french.redo_corrections, "Refaire le compte rendu avec les corrections");
    assert_eq!(english.redo_corrections, "Write the summary again with your edits");
    // The transcript's redo panel says the summary there is kept (the decider, 2026-11-18).
    assert_eq!(
        french.redo_keeps_minutes,
        "Le compte rendu actuel est gardé\u{202f}; refaites-le ensuite si besoin."
    );
    assert_eq!(
        english.redo_keeps_minutes,
        "The current summary is kept; write it again afterwards if needed."
    );
    for sentence in [
        french.redo_keeps_minutes,
        french.context_label,
        french.context_example,
        french.context_why,
        french.context_script,
        french.context_making,
        french.context_unfinished,
        french.context_changed[0],
        french.context_changed[1],
    ] {
        for mark in ['?', '!', ';', ':'] {
            assert!(!sentence.contains(&format!(" {mark}")), "{sentence}");
        }
        assert!(!sentence.contains('\''), "{sentence}");
    }
    // The count's two halves, each with its number.
    for (most, left) in [
        (french.context_most, french.context_left),
        (english.context_most, english.context_left),
    ] {
        assert_eq!(most.matches("{}").count(), 1);
        assert!(left.iter().all(|left| left.matches("{}").count() == 1));
    }
    // What a script Whisper does not take whole may do to the context, said without the engine's name (R2).
    assert!(french.context_script.contains("tronqué") && english.context_script.contains("cut short"));
}

#[test]
fn a_confidential_meeting_s_words_are_said_in_both_languages_french_typeset_as_french() {
    let french = words::window(Language::French);
    let english = words::window(Language::English);
    // The switch's name and words, as D60's criterion says them.
    assert_eq!(french.confidential_name, "Réunion confidentielle");
    assert_eq!(french.confidential_says, "Rien ne quitte cet ordinateur");
    assert_eq!(english.confidential_name, "Confidential meeting");
    for sentence in [
        french.confidential_left,
        french.confidential_not_saved,
        french.confidential_ollama_empty,
    ] {
        for mark in ['?', '!', ';', ':'] {
            assert!(!sentence.contains(&format!(" {mark}")), "{sentence}");
        }
        assert!(!sentence.contains('\''), "{sentence}");
    }
    for (said, holes) in [
        (french.confidential_left, 1),
        (english.confidential_left, 1),
        (french.confidential_left_sound, 1),
        (english.confidential_left_sound, 1),
        (french.confidential_left_transcript, 1),
        (english.confidential_left_transcript, 1),
        (french.minutes_fetch_then, 1),
        (english.minutes_fetch_then, 1),
    ] {
        assert_eq!(said.matches("{}").count(), holes, "{said}");
    }
    // Waiting for Ollama, said in a notification too, the model it would fetch named.
    for language in [Language::French, Language::English] {
        for silent in [true, false] {
            let said = words::minutes(&words::Minutes::OllamaWaits { silent }, language);
            assert!(said.body.contains("Ollama"), "{}", said.body);
            assert_eq!(said.body.contains(kitokey_shell::providers::RECOMMENDED), !silent, "{}", said.body);
        }
    }
    let said = words::minutes(&words::Minutes::OllamaWaits { silent: false }, Language::French);
    assert!(!said.body.contains(" :"), "{}", said.body);
}

#[test]
fn every_french_word_of_the_window_is_typeset_as_french() {
    fn strings(value: &serde_json::Value, found: &mut Vec<String>) {
        match value {
            serde_json::Value::String(text) => found.push(text.clone()),
            serde_json::Value::Array(values) => values.iter().for_each(|value| strings(value, found)),
            serde_json::Value::Object(fields) => fields.values().for_each(|value| strings(value, found)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    strings(&serde_json::to_value(words::window(Language::French)).unwrap(), &mut found);
    assert!(found.len() > 100, "{}", found.len());
    // Before : and », after «, a space that does not break; before ? ! ;, a narrow one — never a
    // plain space, which a narrow window would break the line on.
    for sentence in &found {
        for mark in ['?', '!', ';', ':', '»'] {
            assert!(!sentence.contains(&format!(" {mark}")), "{sentence}");
        }
        assert!(!sentence.contains("« "), "{sentence}");
    }
}

#[test]
fn the_announcement_is_in_the_meetings_language_when_it_says_french_or_english_else_the_windows() {
    use kitokey_shell::words::{Language, announcement};
    let french = "J’enregistre cette réunion pour en faire le compte rendu. Dites-le-moi si vous préférez que je ne le fasse pas.";
    let english = "I’m recording this meeting to write a summary. Let me know if you’d rather I didn’t.";
    assert_eq!(announcement(Some("fr"), Language::English), french);
    assert_eq!(announcement(Some("en"), Language::French), english);
    assert_eq!(announcement(Some("de"), Language::French), french);
    assert_eq!(announcement(None, Language::English), english);
}

/// The French sentences of `src/words.rs`, read from its structure: each string literal after the
/// word `French` and before the next `English`, as every arm of the file is written. Its escapes
/// (`\u{a0}`, `\u{202f}`, `\"`) are read as the compiler reads them. Also the literals after
/// `English`, for the test to hold that the structure was read right.
fn literals_by_language() -> (Vec<String>, Vec<String>) {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/words.rs")).unwrap();
    let (mut french, mut english) = (Vec::new(), Vec::new());
    let mut in_french = false;
    let mut word = String::new();
    let mut chars = source.chars().peekable();
    let mut previous = ' ';
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            chars.by_ref().find(|&d| d == '\n');
            previous = '\n';
            continue;
        }
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            previous = c;
            continue;
        }
        match word.as_str() {
            "French" => in_french = true,
            "English" => in_french = false,
            _ => {}
        }
        word.clear();
        if c == '"' && previous != '\'' {
            let mut text = String::new();
            while let Some(d) = chars.next() {
                match d {
                    '\\' => match chars.next() {
                        Some('u') => {
                            let code: String = chars.by_ref().skip(1).take_while(|&e| e != '}').collect();
                            text.push(char::from_u32(u32::from_str_radix(&code, 16).unwrap()).unwrap());
                        }
                        Some('n') => text.push('\n'),
                        Some(other) => text.push(other),
                        None => break,
                    },
                    '"' => break,
                    other => text.push(other),
                }
            }
            if in_french { &mut french } else { &mut english }.push(text);
            previous = '"';
            continue;
        }
        previous = c;
    }
    (french, english)
}

#[test]
fn the_literals_of_each_language_are_read_as_the_file_arranges_them() {
    let (french, english) = literals_by_language();
    assert!(french.len() > 300 && english.len() > 300, "{} {}", french.len(), english.len());
    let accented: Vec<_> = english
        .iter()
        .filter(|text| text.contains(|c: char| "éèêàâçùûîôœÉÈÀÇ".contains(c)))
        .collect();
    // Inès: a name in an English example.
    assert_eq!(
        accented,
        ["E.g. Q4 budget with Inès Durand and the purchasing team"],
        "French read as English"
    );
    // An `expect`'s words, for the developer: no sentence the user reads.
    let plain: Vec<_> = french.iter().filter(|text| format!(" {text} ").contains(" the ")).collect();
    assert_eq!(plain, ["the minutes' end is always said"], "English read as French");
}

#[test]
fn every_french_sentence_has_its_spaces_before_the_tall_signs() {
    // D47: U+202F before « ? ! ; », U+00A0 before « : » — times (12:21) and addresses (https:)
    // aside — in every French sentence the window and the notifications say.
    let mut wrong = Vec::new();
    for text in literals_by_language().0 {
        // A program's name, as its own messages open: no sentence. What is filled in neither.
        if matches!(text.as_str(), "kitokey-capture:" | "kitokey-transcription:") {
            continue;
        }
        let chars: Vec<char> = text.chars().collect();
        let mut filled = false;
        for (at, &c) in chars.iter().enumerate() {
            filled = (filled || c == '{') && c != '}';
            if filled {
                continue;
            }
            let before = if at == 0 { None } else { Some(chars[at - 1]) };
            let bad = match c {
                '?' | '!' | ';' => before != Some('\u{202f}'),
                ':' => {
                    // 12:21: two figures after; https:// : its own word before, two slashes after.
                    let time = before.is_some_and(|b| b.is_ascii_digit())
                        && chars[at + 1..].iter().take_while(|d| d.is_ascii_digit()).count() == 2;
                    let word: String = chars[..at].iter().rev().take_while(|d| d.is_alphabetic()).collect();
                    let address = matches!(word.as_str(), "sptth" | "ptth") && chars[at + 1..].starts_with(&['/', '/']);
                    !time && !address && before != Some('\u{a0}')
                }
                _ => false,
            };
            if bad {
                wrong.push(text.clone());
                break;
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} French sentences without their space:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

// The voice guide (`docs/voice.md`, D94): what every sentence holds, read from `src/words.rs`.

/// What the rewrite (D94, one pull request an area) still has to put right, sentence by sentence:
/// `tests/words_still_wrong.txt`, one `rule key` a line, sorted. A sentence that breaks a rule and
/// is not listed fails; one listed that no longer breaks it fails too, until it is removed — the
/// list only shrinks, and a sentence put right never covers another written wrong.
const STILL_WRONG: &str = include_str!("words_still_wrong.txt");

/// The sentences still without their comment, counted: 602 keys would be three review budgets.
/// The number only goes down, as the list does.
const WITHOUT_COMMENT: usize = 12;

/// A sentence of `src/words.rs`, as the file arranges it.
#[derive(Debug)]
struct Sentence {
    /// The function that says it.
    function: String,
    /// In `window` and `menu`, the field it fills and its place there: an array holds several.
    field: Option<(String, usize)>,
    /// Said in French, in English, or before either is named.
    french: Option<bool>,
    text: String,
    /// The comment that covers it, when it gives the guide's four words: its field's, or the last
    /// one above it in its function. Numbered, so that what one comment covers is read together.
    comment: Option<(usize, String)>,
}

impl Sentence {
    fn name(&self) -> String {
        match &self.field {
            Some((field, place)) => format!("{}.{field}[{place}] {:?}", self.function, self.text),
            None => format!("{} {:?}", self.function, self.text),
        }
    }
}

/// Functions whose literals are no sentence: a language's code, a model's name made fit to show,
/// what a program said made fit to show.
const NOT_SAID: [&str; 9] = [
    "named",
    "code",
    "detail",
    "quantised",
    "dated",
    "without_tail",
    "js_space",
    "model_said",
    "shown_detail",
];

/// A comment that gives the guide's four words, in their order.
fn context(comment: &str) -> bool {
    let at: Vec<_> = ["Shows:", "When:", "At most:", "Placeholders:"]
        .iter()
        .map(|word| comment.find(word))
        .collect();
    at.iter().all(Option::is_some) && at.windows(2).all(|pair| pair[0] < pair[1])
}

/// Every sentence of `src/words.rs`: each string literal of a function, its escapes read as the
/// compiler reads them, with the comment that covers it.
fn sentences() -> Vec<Sentence> {
    sentences_in(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/words.rs")).unwrap())
}

/// The sentences of a source written as `words.rs` is.
fn sentences_in(source: &str) -> Vec<Sentence> {
    let source: Vec<char> = source.chars().collect();
    let mut found = Vec::new();
    let mut fields = std::collections::HashMap::new();
    let (mut function, mut structure, mut keyword, mut word_before) = (String::new(), String::new(), String::new(), String::new());
    let mut french = None;
    let (mut block, mut doc) = (String::new(), None::<String>);
    let (mut comment, mut comments) = (None::<(usize, String)>, 0);
    let (mut field, mut place) = (String::new(), 0);
    let mut at = 0;
    while at < source.len() {
        let c = source[at];
        if c == '/' && source.get(at + 1) == Some(&'/') {
            let end = source[at..].iter().position(|&d| d == '\n').map_or(source.len(), |n| at + n);
            block.extend(&source[at..end]);
            block.push('\n');
            at = end;
            continue;
        }
        if c.is_whitespace() {
            at += 1;
            continue;
        }
        // A token: the comment above it ends here.
        if !block.is_empty() {
            let closed = std::mem::take(&mut block);
            if keyword == "fn" && context(&closed) {
                comments += 1;
                comment = Some((comments, closed.clone()));
            }
            doc = Some(closed);
        }
        if c.is_alphanumeric() || c == '_' {
            let start = at;
            while at < source.len() && (source[at].is_alphanumeric() || source[at] == '_') {
                at += 1;
            }
            let word: String = source[start..at].iter().collect();
            match word_before.as_str() {
                "fn" => {
                    (function, keyword, french, comment) = (word.clone(), "fn".into(), None, None);
                }
                "struct" => (structure, keyword) = (word.clone(), "struct".into()),
                _ => {}
            }
            match word.as_str() {
                "French" => french = Some(true),
                "English" => french = Some(false),
                _ => {}
            }
            let next = source[at..]
                .iter()
                .position(|d| !d.is_whitespace())
                .map_or(source.len(), |n| at + n);
            let named = source.get(next) == Some(&':') && source.get(next + 1) != Some(&':');
            if named
                && keyword == "struct"
                && word_before == "pub"
                && let Some(doc) = doc.take()
            {
                fields.insert((structure.clone(), word.clone()), doc);
            }
            if named && keyword == "fn" {
                (field, place) = (word.clone(), 0);
            }
            if word != "pub" {
                doc = None;
            }
            word_before = word;
            continue;
        }
        doc = None;
        if c == '\'' {
            // A character, escaped or not, or a lifetime.
            at = match (source.get(at + 1), source.get(at + 2)) {
                (Some('\\'), _) => at + 3 + source[at + 3..].iter().position(|&d| d == '\'').unwrap() + 1,
                (_, Some('\'')) => at + 3,
                _ => at + 1,
            };
            continue;
        }
        if c != '"' {
            at += 1;
            continue;
        }
        let mut text = String::new();
        at += 1;
        while at < source.len() {
            let d = source[at];
            at += 1;
            match d {
                '\\' => {
                    let escaped = source[at];
                    at += 1;
                    match escaped {
                        'u' => {
                            let close = at + source[at..].iter().position(|&e| e == '}').unwrap();
                            let code: String = source[at + 1..close].iter().collect();
                            text.push(char::from_u32(u32::from_str_radix(&code, 16).unwrap()).unwrap());
                            at = close + 1;
                        }
                        'n' => text.push('\n'),
                        other => text.push(other),
                    }
                }
                '"' => break,
                other => text.push(other),
            }
        }
        let shown = matches!(function.as_str(), "window" | "menu");
        let said = keyword == "fn"
            && !NOT_SAID.contains(&function.as_str())
            // An `expect`'s words, a locale compared, a target named: for the code, no sentence.
            && !matches!(word_before.as_str(), "expect" | "eq_ignore_ascii_case" | "target_os")
            && !(shown && field == "locale")
            && strip_placeholders(&text).chars().any(char::is_alphabetic);
        if said {
            let owner = if function == "window" { "Window" } else { "Menu" };
            let own = fields.get(&(owner.to_string(), field.clone())).filter(|doc| context(doc));
            found.push(Sentence {
                function: function.clone(),
                field: shown.then(|| (field.clone(), place)),
                french,
                text,
                comment: if shown { own.map(|doc| (0, doc.clone())) } else { comment.clone() },
            });
            place += 1;
        }
        // Only the literal right after a word of the code is the code's: the next one is read.
        word_before.clear();
    }
    found
}

/// The placeholders of a sentence, `{}` or `{name}`, a format's options left out: `{minutes:02}`
/// is `{minutes}`. `{{` is no placeholder.
fn placeholders(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        if let Some(escaped) = after.strip_prefix('{') {
            rest = escaped;
            continue;
        }
        let Some(close) = after.find('}') else { break };
        let name = after[..close].split(':').next().unwrap();
        if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            found.push(format!("{{{name}}}"));
        }
        rest = &after[close + 1..];
    }
    found
}

fn strip_placeholders(text: &str) -> String {
    placeholders(text)
        .iter()
        .fold(text.to_string(), |text, placeholder| text.replacen(placeholder.as_str(), "", 1))
}

/// A sentence's length as the guide counts it: each placeholder as 6 characters, the size or the
/// time that usually fills it.
fn measured(text: &str) -> usize {
    strip_placeholders(text).chars().count() + 6 * placeholders(text).len()
}

/// The limit a comment gives, in characters: none when it says `no limit`.
fn limit(comment: &str) -> Option<usize> {
    let after = &comment[comment.find("At most:")? + "At most:".len()..];
    after.trim_start().split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
}

/// What `rule` finds today, against what `STILL_WRONG` lists of it, key by key.
fn still_wrong(rule: &str, found: Vec<String>) {
    let lines: Vec<_> = STILL_WRONG.lines().filter(|line| !line.is_empty()).collect();
    assert!(lines.is_sorted(), "tests/words_still_wrong.txt: keep it sorted");
    let mut listed: Vec<_> = lines
        .iter()
        .filter_map(|line| line.strip_prefix(&format!("{rule} ")))
        .map(str::to_string)
        .collect();
    let mut found = found;
    listed.sort();
    found.sort();
    let new: Vec<_> = found
        .iter()
        .filter(|key| !listed.contains(key))
        .map(|key| format!("{rule} {key}"))
        .collect();
    let gone: Vec<_> = listed
        .iter()
        .filter(|key| !found.contains(key))
        .map(|key| format!("{rule} {key}"))
        .collect();
    assert!(
        new.is_empty(),
        "written against the guide (`{rule}`), put it right:\n{}",
        new.join("\n")
    );
    assert!(
        gone.is_empty(),
        "put right (`{rule}`): remove it from the list:\n{}",
        gone.join("\n")
    );
    assert_eq!(
        found, listed,
        "`{rule}`: a key listed once more or once less than it breaks the rule"
    );
}

/// Only the literal right after `expect`, `eq_ignore_ascii_case` or `target_os` is the code's: a
/// sentence after it is still read (the verifier's probes on #812).
#[test]
fn a_literal_of_the_code_leaves_the_sentence_after_it_read() {
    let read = |source: &str| sentences_in(source).into_iter().map(|sentence| sentence.text).collect::<Vec<_>>();
    assert_eq!(
        read("fn opened() -> &'static str {\n    if cfg!(target_os = \"linux\") { \"Can't open it on Linux\" } else { \"Open it\" }\n}\n"),
        ["Can't open it on Linux", "Open it"]
    );
    assert_eq!(
        read(
            "fn said(code: &str) -> &'static str {\n    if code.eq_ignore_ascii_case(\"fr\") { \"Le fournisseur\" } else { \"The provider\" }\n}\n"
        ),
        ["Le fournisseur", "The provider"]
    );
    assert_eq!(
        read("fn told(x: Option<u8>) -> &'static str {\n    x.expect(\"always there\");\n    \"A sentence\"\n}\n"),
        ["A sentence"]
    );
}

#[test]
fn the_sentences_are_read_as_the_file_arranges_them() {
    let sentences = sentences();
    let (french, english): (Vec<_>, Vec<_>) = sentences.iter().partition(|sentence| sentence.french == Some(true));
    assert!(french.len() > 300 && english.len() > 300, "{} {}", french.len(), english.len());
    fn strings(value: &serde_json::Value, found: &mut Vec<String>) {
        match value {
            serde_json::Value::String(text) => found.push(text.clone()),
            serde_json::Value::Array(values) => values.iter().for_each(|value| strings(value, found)),
            serde_json::Value::Object(fields) => fields.values().for_each(|value| strings(value, found)),
            _ => {}
        }
    }
    let shown = |language: Language| {
        let mut texts = Vec::new();
        strings(&serde_json::to_value(words::window(language)).unwrap(), &mut texts);
        texts.retain(|text| text != words::window(language).locale && strip_placeholders(text).chars().any(char::is_alphabetic));
        texts.sort();
        texts
    };
    for (language, french) in [(Language::English, false), (Language::French, true)] {
        let mut read: Vec<_> = sentences
            .iter()
            .filter(|sentence| sentence.function == "window" && sentence.french == Some(french))
            .map(|sentence| sentence.text.clone())
            .collect();
        read.sort();
        assert_eq!(read, shown(language), "{language:?}: the window's sentences, each read once");
    }
}

/// Each sentence carries a comment that says where it shows, when, its limit and its placeholders:
/// one for both its languages, so counted on its English.
#[test]
fn every_sentence_carries_its_comment() {
    let without: Vec<_> = sentences()
        .into_iter()
        .filter(|sentence| sentence.french != Some(true) && sentence.comment.is_none())
        .map(|s| s.name())
        .collect();
    // A baseline, as ESLint's or Psalm's: it fails only when a sentence is written without its comment,
    // never when one gains it or goes — two pull requests merged side by side each lowered it, and
    // main went red twice (#857). Lower, it says by how much it can come down.
    assert!(
        without.len() <= WITHOUT_COMMENT,
        "{} sentences without their comment, WITHOUT_COMMENT says {WITHOUT_COMMENT}: a sentence was written without its comment\n{}",
        without.len(),
        without.join("\n")
    );
    if without.len() < WITHOUT_COMMENT {
        eprintln!("WITHOUT_COMMENT can be lowered to {}", without.len());
    }
}

/// The same placeholders in French and in English — a field's, or those one comment covers —, and
/// those its comment names.
#[test]
fn a_sentence_has_the_same_placeholders_in_both_languages_and_its_comment_names_them() {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, [Vec<String>; 2]> = BTreeMap::new();
    // What one comment covers — a whole field, each place of an array, or the arms after it — and
    // the placeholders it says, against those its sentences hold.
    let mut covered: BTreeMap<String, (std::collections::BTreeSet<String>, String)> = BTreeMap::new();
    for sentence in sentences() {
        let Some(french) = sentence.french else { continue };
        let group = match (&sentence.field, &sentence.comment) {
            (Some((field, place)), _) => format!("{}.{field}[{place}]", sentence.function),
            (None, Some((number, _))) => format!("{} comment {number}", sentence.function),
            (None, None) => sentence.function.clone(),
        };
        groups.entry(group).or_default()[french as usize].extend(placeholders(&sentence.text));
        if let Some((number, comment)) = &sentence.comment {
            let key = match &sentence.field {
                Some((field, _)) => format!("{}.{field}", sentence.function),
                None => format!("{} comment {number}", sentence.function),
            };
            let entry = covered.entry(key).or_default();
            entry.0.extend(placeholders(&sentence.text));
            entry.1 = comment[comment.find("Placeholders:").unwrap()..].to_string();
        }
    }
    let mut wrong = Vec::new();
    for (group, [mut english, mut french]) in groups {
        english.sort();
        french.sort();
        if english != french {
            wrong.push(format!("{group}: in both languages"));
        }
    }
    for (key, (said, named)) in covered {
        let listed: std::collections::BTreeSet<_> = placeholders(&named).into_iter().collect();
        if said != listed {
            wrong.push(format!("{key}: in its comment"));
        }
    }
    still_wrong("placeholders", wrong);
}

#[test]
fn no_sentence_has_a_straight_apostrophe() {
    let wrong = sentences()
        .into_iter()
        .filter(|sentence| sentence.text.contains('\''))
        .map(|s| s.name())
        .collect();
    still_wrong("apostrophe", wrong);
}

/// A list's label at most 18 characters, whatever its comment says, and every sentence at most what
/// its comment says. A notification's title is read whole below.
#[test]
fn a_sentence_fits_its_limit_and_a_list_label_holds_18_characters() {
    let script = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/window.js")).unwrap();
    // A list's labels: what the window puts in a chip, by name or by the `chip_` they all begin with.
    let mut labels = std::collections::BTreeSet::new();
    for (opening, _) in script.match_indices("words.") {
        let name: String = script[opening + 6..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if name.starts_with("chip_") || script[..opening].ends_with("chip(") {
            labels.insert(name);
        }
    }
    assert!(labels.contains("chip_making") && labels.contains("recording"), "{labels:?}");
    let mut wrong = Vec::new();
    for sentence in sentences() {
        let comment = sentence.comment.as_ref().map(|(_, comment)| comment.as_str()).unwrap_or_default();
        let label = sentence
            .field
            .as_ref()
            .is_some_and(|(field, _)| sentence.function == "window" && labels.contains(field))
            || comment.contains("list label");
        let at_most = match limit(comment) {
            Some(at_most) if label => Some(at_most.min(18)),
            None if label => Some(18),
            at_most => at_most,
        };
        if at_most.is_some_and(|at_most| measured(&sentence.text) > at_most) {
            wrong.push(sentence.name());
        }
    }
    still_wrong("limit", wrong);
}

/// The lexicon's refused words that a test can tell apart: « fournisseur », « segment », « tu »;
/// "provider", "keyring", and "minutes" — the summary's, not a length's: after a number or a
/// placeholder, it is a length.
#[test]
fn no_sentence_says_a_word_the_lexicon_refuses() {
    let mut wrong = Vec::new();
    for sentence in sentences() {
        const FRENCH: [&str; 5] = ["fournisseur", "fournisseurs", "segment", "segments", "tu"];
        const ENGLISH: [&str; 4] = ["provider", "providers", "keyring", "minutes"];
        // A sentence the reader cannot place in either language — a helper such as `french_days` —
        // is held to both lexicons.
        let both = [FRENCH.as_slice(), ENGLISH.as_slice()].concat();
        let refused: &[&str] = match sentence.french {
            Some(true) => &FRENCH,
            Some(false) => &ENGLISH,
            None => &both,
        };
        let words: Vec<String> = sentence
            .text
            .split(|c: char| !(c.is_alphanumeric() || c == '{' || c == '}' || c == '_'))
            .filter(|word| !word.is_empty())
            .map(str::to_lowercase)
            .collect();
        let length = |at: usize| at > 0 && (words[at - 1].starts_with('{') || words[at - 1].chars().all(|c| c.is_ascii_digit()));
        if let Some(at) = words
            .iter()
            .enumerate()
            .position(|(at, word)| refused.contains(&word.as_str()) && !(word == "minutes" && length(at)))
        {
            wrong.push(format!("{} « {} »", sentence.name(), words[at]));
        }
    }
    still_wrong("lexicon", wrong);
}

/// What D124's rules can tell apart in a sentence (`docs/voice.md`, V1, V3, V6): each rule a
/// sentence breaks, by its name. A test reads the literal only: what fills a placeholder is the
/// review's.
fn broken(text: &str) -> Vec<&'static str> {
    let mut rules = Vec::new();
    let lower = text.to_lowercase();
    // Each word as written, quotes and the punctuation around it left out.
    let tokens: Vec<&str> = text
        .split(|c: char| c.is_whitespace())
        .map(|token| token.trim_matches(|c: char| "«»“”‘’\"'(),;!?.:".contains(c)))
        .filter(|token| !token.is_empty())
        .collect();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    if text.contains('\u{2014}') {
        rules.push("dash");
    }
    let arrow = |c: char| matches!(c, '\u{2190}'..='\u{21ff}' | '\u{27f0}'..='\u{27ff}' | '\u{2900}'..='\u{297f}');
    if text.chars().any(arrow) || text.contains("->") || text.contains("=>") {
        rules.push("arrow");
    }
    if text.contains('\u{2026}') || text.contains("...") {
        rules.push("ellipsis");
    }
    if words.iter().any(|word| ["modèle", "modèles", "model", "models"].contains(word)) {
        rules.push("model");
    }
    const DOMAINS: [&str; 9] = ["com", "ai", "io", "co", "org", "net", "fr", "dev", "app"];
    let domain = |token: &str| {
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() > 1
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
            && DOMAINS.contains(&parts[parts.len() - 1].to_ascii_lowercase().as_str())
    };
    let address = |token: &str| {
        token.split('.').count() == 4
            && token
                .split('.')
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
    };
    if text.contains("://")
        || words.contains(&"localhost")
        || tokens
            .iter()
            .any(|token| domain(token) || address(token.split(':').next().unwrap()))
    {
        rules.push("host");
    }
    // A port: a colon after a letter or a digit, then three digits or more (a time has two); a colon
    // after a space, then digits (« port :8080 »); or the word « port » followed by digits, with or
    // without its colon (« le port 8080 », "port: 8080").
    let digits_after = |rest: &str| rest.chars().take_while(char::is_ascii_digit).count();
    let port = text.char_indices().any(|(at, c)| {
        let before = text[..at].chars().next_back();
        c == ':'
            && (before.is_some_and(char::is_alphanumeric) && digits_after(&text[at + 1..]) >= 3
                || before.is_some_and(char::is_whitespace) && digits_after(&text[at + 1..]) >= 2)
    }) || words
        .windows(2)
        .any(|pair| ["port", "ports"].contains(&pair[0]) && pair[1].chars().all(|c| c.is_ascii_digit()));
    if port {
        rules.push("port");
    }
    let path = |token: &&str| {
        !token.contains("://")
            && (token.starts_with('/')
                || token.starts_with("~/")
                || token.contains('\\')
                || token.split('/').filter(|part| !part.is_empty()).count() > 1)
    };
    if tokens.iter().any(path) {
        rules.push("path");
    }
    // An engine's own id: its family's name, or a word where letters and digits meet across a
    // « - », a « . » or a « : » (`whisper-large-v3`, `llama3.2:3b`).
    let id = |token: &&str| {
        token.chars().any(|c| c.is_ascii_alphabetic())
            && token.chars().any(|c| c.is_ascii_digit())
            && token.contains(['-', '.', ':'])
            && token.chars().all(|c| c.is_ascii_alphanumeric() || "-.:_".contains(c))
    };
    if words.iter().any(|word| ["whisper", "ggml", "gguf"].contains(word)) || tokens.iter().any(id) {
        rules.push("engine_id");
    }
    const UNDER_A_MEGABYTE: [&str; 10] = ["o", "B", "ko", "kB", "KB", "Ko", "KiB", "Kio", "octets", "bytes"];
    if tokens
        .iter()
        .any(|token| UNDER_A_MEGABYTE.contains(token) || ["octet", "byte"].contains(token))
    {
        rules.push("bytes");
    }
    rules
}

/// The sentences right to keep against a rule of D124, each by its key, with its reason: an action
/// in progress or an entry that opens a dialog keeps its « … » (V3, GNOME HIG and Apple HIG); a raw
/// value shown under « Avancé », « Détails » or the developer mode keeps it (V1). Never a pattern.
const KEPT: [(&str, &str, &str); 28] = [
    (
        "ellipsis",
        r#"menu.minuting[0] "Rédaction du compte rendu…""#,
        "an action in progress, in the tray's menu",
    ),
    (
        "ellipsis",
        r#"menu.minuting[0] "Writing summary…""#,
        "an action in progress, in the tray's menu",
    ),
    (
        "ellipsis",
        r#"menu.stopping[0] "Arrêt en cours…""#,
        "an action in progress, in the tray's menu",
    ),
    (
        "ellipsis",
        r#"menu.stopping[0] "Stopping…""#,
        "an action in progress, in the tray's menu",
    ),
    (
        "ellipsis",
        r#"menu.transcribing[0] "Transcribing…""#,
        "an action in progress, in the tray's menu",
    ),
    (
        "ellipsis",
        r#"menu.transcribing[0] "Transcription en cours…""#,
        "an action in progress, in the tray's menu",
    ),
    (
        "ellipsis",
        r#"window.chip_making[0] "Compte rendu…""#,
        "a meeting's label while its summary is written",
    ),
    (
        "ellipsis",
        r#"window.chip_making[0] "Summary…""#,
        "a meeting's label while its summary is written",
    ),
    (
        "ellipsis",
        r#"window.chip_transcribing[0] "Transcript…""#,
        "a meeting's label while it is transcribed",
    ),
    (
        "ellipsis",
        r#"window.chip_transcribing[0] "Transcription…""#,
        "a meeting's label while it is transcribed",
    ),
    ("ellipsis", r#"window.choose[0] "Choisir…""#, "opens the system's folder chooser"),
    ("ellipsis", r#"window.choose[0] "Choose…""#, "opens the system's folder chooser"),
    (
        "ellipsis",
        r#"window.keep_other[0] "Autre durée…""#,
        "an entry that opens the field of a duration",
    ),
    (
        "ellipsis",
        r#"window.keep_other[0] "Other duration…""#,
        "an entry that opens the field of a duration",
    ),
    (
        "ellipsis",
        r#"window.record_starting[0] "Démarrage…""#,
        "the record button while the recording starts",
    ),
    (
        "ellipsis",
        r#"window.record_starting[0] "Starting…""#,
        "the record button while the recording starts",
    ),
    (
        "ellipsis",
        r#"window.record_stopping[0] "Arrêt…""#,
        "the record button while the recording stops",
    ),
    (
        "ellipsis",
        r#"window.record_stopping[0] "Stopping…""#,
        "the record button while the recording stops",
    ),
    ("ellipsis", r#"window.redo[0] "Redo…""#, "opens the dialog of a redo"),
    ("ellipsis", r#"window.redo[0] "Refaire…""#, "opens the dialog of a redo"),
    ("ellipsis", r#"window.send_open[0] "Envoyer…""#, "opens the choice of a channel"),
    ("ellipsis", r#"window.send_open[0] "Send…""#, "opens the choice of a channel"),
    (
        "ellipsis",
        r#"window.send_sending[0] "Envoi vers «\u{a0}{}\u{a0}»…""#,
        "a sending in progress",
    ),
    ("ellipsis", r#"window.send_sending[0] "Sending to “{}”…""#, "a sending in progress"),
    (
        "ellipsis",
        r#"window.speakers_open[0] "Name the speakers…""#,
        "opens the naming of the speakers",
    ),
    (
        "ellipsis",
        r#"window.speakers_open[0] "Nommer les intervenants…""#,
        "opens the naming of the speakers",
    ),
    ("ellipsis", r#"window.testing[0] "Test en cours…""#, "a test in progress"),
    ("ellipsis", r#"window.testing[0] "Testing…""#, "a test in progress"),
];

/// D124: no em dash, no arrow, no decorative « … », no « modèle » nor "model", no host, port,
/// path, engine's raw id or size under 1 Mo, in any sentence of either language (`docs/voice.md`,
/// V1, V3, V6; PDR-0008). What a screen's pull request has still to rewrite is listed, the list
/// only shrinking; what is right to keep is `KEPT`, one sentence at a time.
#[test]
fn no_sentence_holds_ai_typography_the_word_model_or_raw_data() {
    let mut wrong: std::collections::BTreeMap<&str, Vec<String>> = std::collections::BTreeMap::new();
    let mut kept_used = Vec::new();
    for sentence in sentences() {
        let name = sentence.name();
        for rule in broken(&sentence.text) {
            match KEPT.iter().position(|(kept, key, _)| *kept == rule && *key == name) {
                Some(at) => kept_used.push(at),
                None => wrong.entry(rule).or_default().push(name.clone()),
            }
        }
    }
    let stale: Vec<_> = (0..KEPT.len()).filter(|at| !kept_used.contains(at)).map(|at| KEPT[at].1).collect();
    assert!(
        stale.is_empty(),
        "`KEPT` names a sentence that no longer breaks its rule: remove it\n{stale:#?}"
    );
    for (_, _, reason) in KEPT {
        assert!(!reason.is_empty());
    }
    for rule in ["dash", "arrow", "ellipsis", "model", "host", "port", "path", "engine_id", "bytes"] {
        still_wrong(rule, wrong.remove(rule).unwrap_or_default());
    }
}

/// A port alone, with no host before it, is told in both languages: after a space and a colon, or
/// after the word « port » (sam's survivor on #1277, V3: « port :8080 » passed).
#[test]
fn a_port_is_told_after_a_space_or_the_word_port_in_both_languages() {
    for english in [
        "Kitokey listens on port :8080",
        "Kitokey listens on port 8080",
        "Ollama isn’t responding on :11434",
    ] {
        assert_eq!(broken(english), ["port"], "{english}");
    }
    for french in [
        "Kitokey écoute sur le port\u{a0}:8080",
        "Kitokey écoute sur le port 8080",
        "Ollama ne répond pas sur\u{a0}:11434",
    ] {
        assert_eq!(broken(french), ["port"], "{french}");
    }
}

/// Each rule of D124 tells its case from the sentences it must leave alone: a time is no port, a
/// menu's path no file's, Mistral no engine's id, a size in Mo none under 1 Mo.
#[test]
fn each_rule_of_d124_tells_its_case_apart() {
    let breaks = |text: &str| broken(text);
    assert_eq!(breaks("Votre micro : rien depuis 2 min — enregistrement en cours."), ["dash"]);
    assert_eq!(breaks("Réglages → Avancé"), ["arrow"]);
    assert_eq!(breaks("Settings -> Advanced"), ["arrow"]);
    assert_eq!(breaks("Clients, RH, santé…"), ["ellipsis"]);
    assert_eq!(breaks("Topic, participants..."), ["ellipsis"]);
    assert_eq!(breaks("Télécharger le modèle"), ["model"]);
    assert_eq!(breaks("Choose a model"), ["model"]);
    assert_eq!(breaks("Ollama est gratuit, sur ollama.com."), ["host"]);
    assert_eq!(breaks("starting with https://"), ["host"]);
    assert_eq!(breaks("Kitokey écoute sur 127.0.0.1:43567"), ["host", "port"]);
    assert_eq!(breaks("Le dossier C:\\Réunions est plein"), ["path"]);
    assert_eq!(breaks("Placez le fichier dans /srv/reunions"), ["path"]);
    assert_eq!(breaks("par exemple whisper-large-v3"), ["engine_id"]);
    assert_eq!(breaks("Téléchargez llama3.2:3b"), ["engine_id"]);
    assert_eq!(breaks("Télécharger (300 ko)"), ["bytes"]);
    assert_eq!(breaks("Download (300 kB)"), ["bytes"]);
    for right in [
        "Réunion de 10:30",
        "Rendez-vous à 10\u{a0}h\u{a0}30, port de Brest",
        "Export: 3 files",
        "Seul le texte part vers Mistral. Le son reste ici.",
        "Refaire\u{a0}› Avancé",
        "Transcrire (300 Mo, une seule fois)",
        "Download (1.5 GB)",
        "Compte rendu prêt.",
        "Point hebdo · 42\u{202f}%",
    ] {
        assert_eq!(breaks(right), Vec::<&str>::new(), "{right}");
    }
}

#[test]
fn the_bar_says_a_silence_as_the_notifications_do() {
    // D49: the same gesture as the notification's, in each language; the side's own. D90: a
    // silence of two minutes said as such, the recording going on — never as an error.
    let cases = [
        (
            Language::French,
            "Votre micro",
            "Le son de la réunion",
            "n’est pas coupé sur l’appareil et qu’il est allumé",
            "rien depuis 2\u{a0}min — enregistrement en cours",
        ),
        (
            Language::English,
            "Your microphone",
            "Meeting audio",
            "isn’t muted on the device and that it’s switched on",
            "silent for 2 min — still recording",
        ),
    ];
    for (language, me, others, gesture, silent) in cases {
        let page = words::window(language);
        assert_eq!(page.small_mic, me);
        assert_eq!(page.small_meeting, others);
        let told = words::of(&Said::Nothing, true, language).unwrap();
        assert!(told.body.contains(gesture), "{}", told.body);
        assert!(page.small_nothing.contains(gesture), "{}", page.small_nothing);
        assert!(page.bar_silent_me.contains(silent) && page.bar_silent_others.contains(silent));
        for said in [
            page.bar_silent_me,
            page.bar_silent_others,
            page.small_nothing,
            page.small_waiting_me,
            page.small_waiting_others,
        ] {
            assert!(
                !said.is_empty() && said.chars().count() <= 110,
                "too long for two lines of the bar: {said}"
            );
        }
    }
}
/// The wording table of the UX review, §10 (D47): its sentences, as they are read. A row left out
/// here is one the table's sentence would make false: said in the pull request.
#[test]
fn the_review_s_wording_is_what_the_window_says_in_french() {
    let french = words::window(Language::French);
    for (said, wanted) in [
        (french.transcript, "Transcription"),
        (french.transcript_title, "Transcription"),
        (french.chip_transcribing, "Transcription…"),
        (french.heard_by, "Transcrit par {}, sur cet ordinateur"),
        (french.remote_described, "Les autres — le son de l’appel"),
        (french.no_match, "Aucun résultat."),
        (french.search, "Rechercher dans le texte"),
        (french.interrupted, "Interrompu"),
        (french.undated, "Sans date"),
        (french.chip_awaiting, "Accord requis"),
        (french.minutes_waiting, "En attente · {} avant"),
        (french.minutes_unsaved, "Non enregistré\u{a0}: {} reste utilisé."),
        (french.test_sends, "Le test envoie une courte phrase à {}, rien de vos réunions."),
        (french.test_answered, "Ça fonctionne\u{a0}: le modèle répond."),
        (
            french.developer[1],
            "Affiche le détail technique sous chaque erreur, pour aider au diagnostic.",
        ),
        (french.log_level[1], "S’applique au prochain lancement de Kitokey."),
        (french.playback, "Position de lecture"),
        (french.not_playable, "Cet audio ne peut pas être lu dans Kitokey."),
        (
            french.not_played,
            "Lecture impossible ici. Ouvrez le dossier pour l’écouter avec un autre lecteur.",
        ),
        (
            french.no_player,
            "L’audio a été supprimé pour faire de la place. La transcription et le compte rendu restent.",
        ),
        (french.time_listen, "Écouter à partir de {}"),
        (french.time_passage, "Voir le passage à {}"),
        (french.bar_more, "Plus d’options"),
        (french.shortcut_none, "Aucun raccourci défini"),
        (
            french.shortcut_where,
            "Vous pouvez le modifier dans les réglages clavier du système.",
        ),
        (
            french.ollama_silent,
            "Ollama ne répond pas ou n’a aucun modèle. Lancez Ollama, puis téléchargez un modèle.",
        ),
        (french.key, "Clé d’API"),
        (french.agree_local_write, "Rédiger le compte rendu ici"),
        // #933: a button that starts a making names it, as D113's Refaire… does.
        (french.agree_local, "Rédiger plutôt le compte rendu sur cet ordinateur"),
    ] {
        assert_eq!(said, wanted);
    }
    assert!(
        french.ollama_lacks.ends_with("ou téléchargez-le avec Ollama."),
        "{}",
        french.ollama_lacks
    );
    let menu = words::menu(Language::French);
    assert_eq!(menu.start, french.record_start);
    assert_eq!(menu.minutes, "Rédiger le compte rendu du dernier enregistrement");
    assert_eq!(
        words::minutes(&words::Minutes::Unchosen, Language::French).title,
        "Aucun modèle choisi pour les comptes rendus"
    );
}

#[test]
fn the_review_s_english_sentences_are_rewritten() {
    let english = words::window(Language::English);
    assert_eq!(english.interrupted, "Interrupted");
    assert_eq!(english.keep_not_days, "“{}” isn’t a valid number of days (1 to 36,500)");
}

/// *Transcript* is the English word: French says *Transcription*, feminine, everywhere (D47).
#[test]
fn french_never_says_transcript() {
    use words::Minutes;
    let window = serde_json::to_value(words::window(Language::French)).unwrap();
    let mut strings = vec![window];
    while let Some(value) = strings.pop() {
        match value {
            serde_json::Value::Array(items) => strings.extend(items),
            serde_json::Value::Object(fields) => strings.extend(fields.into_iter().map(|(_, value)| value)),
            serde_json::Value::String(said) => {
                assert!(
                    !said.to_lowercase().contains("transcript") || said.to_lowercase().contains("transcription"),
                    "{said}"
                );
                assert!(!said.contains("Le transcription") && !said.contains("le transcription"), "{said}");
            }
            _ => {}
        }
    }
    for notice in [
        words::minutes(&Minutes::Ready, Language::French),
        words::minutes(&Minutes::NoTranscript, Language::French),
        words::minutes(&Minutes::Failed(None), Language::French),
        words::transcript_kept_beside(Language::French, "x", false),
        words::transcript_kept_beside(Language::French, "x", true),
        words::fetched(&Ok(()), false, Language::French),
        words::audio_trashed(Language::French, &Ok((2, 30))),
    ] {
        let said = format!("{} {}", notice.title, notice.body);
        assert!(
            !said.contains("transcript ") && !said.contains("transcripts") && !said.contains("transcript."),
            "{said}"
        );
    }
}

#[test]
fn a_stop_says_how_long_was_recorded_agreed_in_french_and_no_session_in_a_failure() {
    let stopped = |seconds| told(&Said::Stopped("/s".into(), seconds), false, Language::French).1;
    assert_eq!(stopped(1.0), "1 s enregistrée.");
    assert_eq!(stopped(60.0), "1 min enregistrée.");
    assert_eq!(stopped(960.0), "16 min enregistrées.");
    assert_eq!(stopped(5400.0), "1 h 30 enregistrées.");
    for why in ["", "gone"] {
        let failed = told(&Said::Failed(why.into()), false, Language::French).1;
        assert!(failed.ends_with("est conservé dans le dossier de la réunion."), "{failed}");
    }
}

fn main_rs() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .unwrap()
        // A checkout on Windows ends its lines otherwise.
        .replace("\r\n", "\n")
}

/// Each `#[tauri::command…]` of the shell's `main.rs`, whatever its arguments: its signature, then
/// its body.
fn commands() -> Vec<(String, String)> {
    main_rs()
        .split("#[tauri::command")
        .skip(1)
        .map(|command| {
            let command = &command[command.find("]\n").expect("an attribute ends") + 2..];
            let command = &command[..command.find("\n}\n").expect("a command ends")];
            let (signature, body) = command.split_once(" {\n").expect("a command has a body");
            (signature.to_string(), body.to_string())
        })
        .collect()
}

/// The structs of `main.rs` the page is handed: each with its fields, name and type.
fn payloads() -> Vec<(String, Vec<(String, String)>)> {
    let source = main_rs();
    let mut found = Vec::new();
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        if !(line.starts_with("#[derive(") && line.contains("Serialize")) {
            continue;
        }
        while lines.peek().is_some_and(|next| next.starts_with("#[")) {
            lines.next();
        }
        let Some(name) = lines.peek().and_then(|next| next.strip_prefix("struct ")) else {
            continue;
        };
        let name = name.split([' ', '<']).next().unwrap().to_string();
        let mut fields = Vec::new();
        for field in lines.by_ref().skip(1).take_while(|field| *field != "}") {
            if let Some((field, kind)) = field.trim().split_once(": ")
                && !field.starts_with("//")
            {
                fields.push((field.to_string(), kind.trim_end_matches(',').to_string()));
            }
        }
        found.push((name, fields));
    }
    found
}

#[test]
fn a_command_hands_the_page_no_text_of_its_own_only_a_sentence_of_these_words() {
    let commands = commands();
    assert!(commands.len() > 30, "{} commands read", commands.len());
    let structs: Vec<String> = payloads().into_iter().map(|(name, _)| name).collect();
    assert!(structs.len() > 10, "{structs:?}");
    for (signature, body) in &commands {
        if let Some((_, answer)) = signature.rsplit_once("-> ") {
            // Every name of its answer is known here: an alias could hide a text.
            for name in answer
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|name| !name.is_empty())
            {
                assert!(
                    ["Result", "Option", "Vec", "Told", "PathBuf", "String", "speech", "Status"].contains(&name)
                        || structs.iter().any(|known| known == name),
                    "{signature}: {name} is no type this test knows"
                );
            }
            // A text alone is none of the page's answers, a list of names and the logs asked for aside.
            assert!(
                !answer.replace("Vec<String>", "").contains("String") || signature.starts_with("async fn diagnostic("),
                "a command answers with a text: {signature}"
            );
            assert!(!answer.contains("Result") || answer.starts_with("Result<"), "{signature}");
            if let Some(answer) = answer.strip_prefix("Result<") {
                let error = answer.rsplit(", ").next().unwrap().trim();
                assert!(
                    error == "Told>" || error == "()>",
                    "a command fails with a sentence of words.rs (Told), or says nothing: {signature}"
                );
            }
        }
        for text in ["Err(\"", "Err(format!", ".into())", "ok_or(\"", ".ok_or_else(|| format!"] {
            assert!(!body.contains(text), "{signature} hands the page a text: {text}");
        }
        // An error kept as an answer's field: read by what `Ok` is made of, as far as a name tells.
        for (at, _) in body.match_indices(".err()") {
            assert!(
                body[at + ".err()".len()..].trim_start().starts_with(".map(|why| shell.told("),
                "{signature} keeps an error for its answer"
            );
        }
    }
}

#[test]
fn an_answer_holds_a_failure_as_a_sentence_or_for_developer_mode_never_as_raw_text() {
    // By the field's name: a raw error under another name, or within another type, is not seen.
    const NAMES: [&str; 8] = ["unreadable", "error", "why", "failed", "refused", "detail", "reason", "problem"];
    // Those of `makings`, the page's reading of the makings: what a program said, through
    // `shown_detail`, and its cause's sentence.
    const THROUGH_THE_WORDS: [&str; 3] = ["failed", "transcripts_failed", "failed_said"];
    let payloads = payloads();
    let mut read = 0;
    for (name, fields) in &payloads {
        for (field, kind) in fields {
            if !NAMES.iter().any(|said| field.contains(said)) {
                continue;
            }
            read += 1;
            assert!(
                !kind.contains("String") || (name == "Makings" && THROUGH_THE_WORDS.contains(&field.as_str())),
                "{name}.{field}: {kind} hands the page raw text"
            );
        }
    }
    assert!(read >= 4, "{read} fields read: the structs are no longer read as written");
    let source = main_rs();
    assert!(source.contains("let said = |why: &String| words::shown_detail(why, developer);"));
    assert_eq!(
        source.matches("(named(folder), said(why))").count(),
        2,
        "failed, transcripts_failed"
    );
}

#[test]
fn the_settings_files_path_stays_out_of_what_a_command_keeps_of_a_failure() {
    let source = main_rs();
    let reread = &source[source.find("fn reread(&self)").unwrap()..];
    let reread = &reread[..reread.find("\n    }\n").unwrap()];
    // Every route starts here, the transcript's and the minutes' that keep it for developer mode.
    assert!(reread.contains("settings::unnamed("), "{reread}");
    // One place logs a command's failure: `log::command_failed`, which `tests/log.rs` reads.
    assert!(!source.contains("\"shell.command_failed\""));
    assert_eq!(source.matches("log::command_failed(").count(), 1);
}

#[test]
fn a_settings_file_that_cannot_be_read_is_said_whole_its_raw_text_under_it_only_in_developer_mode() {
    let raw = "settings.toml, line 1: keep_audio is \"always\" or a number of days";
    let locked = Failure::SettingsLocked(Refused::KeepAudio { line: 1 });
    let page = |language, developer| serde_json::to_value(words::told(locked, raw, language, developer)).unwrap();
    assert_eq!(
        page(Language::French, false),
        "Le fichier de réglages est illisible\u{a0}: Kitokey n’y touchera pas tant qu’il n’est pas corrigé. \
         Ligne 1\u{a0}: «\u{a0}keep_audio\u{a0}» vaut «\u{a0}always\u{a0}» ou un nombre de jours, de 1 à 36\u{202f}500. Corrigez-le."
    );
    assert_eq!(
        page(Language::English, false),
        "The settings file can’t be read. Kitokey won’t change it until it’s fixed. \
         Line 1: “keep_audio” is “always” or a number of days, from 1 to 36,500. Correct it."
    );
    assert_eq!(
        page(Language::English, true),
        format!(
            "The settings file can’t be read. Kitokey won’t change it until it’s fixed. \
             Line 1: “keep_audio” is “always” or a number of days, from 1 to 36,500. Correct it.\n{raw}"
        )
        .as_str()
    );
}

#[test]
fn a_command_that_failed_says_its_sentence_and_the_raw_text_only_in_developer_mode() {
    let raw = "os error 28";
    let page = |language, developer| serde_json::to_string(&words::told(Failure::MeetingFile, raw, language, developer)).unwrap();
    assert_eq!(
        page(Language::French, false),
        "\"impossible d’écrire dans son dossier. Vérifiez l’espace disque et vos droits sur ce dossier, puis réessayez.\""
    );
    assert!(page(Language::English, false).starts_with("\"can’t write to its folder"));
    assert!(!page(Language::English, false).contains(raw));
    assert!(
        page(Language::French, true).ends_with("réessayez.\\nos error 28\""),
        "{}",
        page(Language::French, true)
    );
    // A trash's refusal is the sentence its expiry says.
    assert_eq!(
        words::failure(Failure::Trash(Refusal::NoSpace), Language::French),
        words::audio_trashed(Language::French, &Err(Refusal::NoSpace)).body
    );
}

#[test]
fn a_test_with_a_cause_says_it_alone_the_raw_text_only_in_developer_mode() {
    use kitokey_shell::{summarising::Cause, words};
    for (cause, advice) in [
        (Cause::TooMany, "Wait, or check your plan"),
        (
            Cause::Down,
            "Try again later. If the model runs on this computer, it may be too large for its memory — choose a smaller one",
        ),
        (Cause::NoTimes, "Choose one that does in Settings"),
    ] {
        assert!(words::cause(cause, Language::English).contains(advice), "{cause:?}");
    }
    let plain = serde_json::to_string(&words::test_caused(Cause::Down, "raw", Language::French, false)).unwrap();
    assert!(
        plain.contains("rencontre un problème") && !plain.contains("raw") && !plain.contains("ne répond pas"),
        "{plain}"
    );
    let developer = serde_json::to_string(&words::test_caused(Cause::Down, "raw", Language::French, true)).unwrap();
    assert!(developer.contains("raw"), "{developer}");
}

#[test]
fn a_microphone_gone_is_told_once_and_what_stands_in_is_no_change_of_its_own() {
    for language in [Language::English, Language::French] {
        let mut sides = words::Sides::default();
        let folder = PathBuf::from("f");
        sides.told(
            &Said::Started(folder, Some("Chosen".into()), Some("Speaker".into())),
            false,
            language,
        );
        let told = sides.told(&Said::MicrophoneGone, false, language).unwrap();
        assert!(
            !told.title.is_empty() && !told.body.is_empty() && !told.body.contains('\''),
            "{told:?}"
        );
        assert_eq!(sides.told(&Said::Moved(Source::Me, Some("Default".into())), false, language), None);
        // Its return is said by what follows: the device named, or the side with none.
        assert_eq!(sides.told(&Said::MicrophoneBack, false, language), None);
    }
}

#[test]
fn what_is_left_of_the_trial_is_said_in_days_and_in_meetings_each_in_its_number() {
    use kitokey_shell::trial::Left;
    let said = |days, meetings, language| words::trial_left(Left { days, meetings }, language);
    for (days, meetings, english, french) in [
        (
            14,
            10,
            "14 days and 10 meetings left. The trial ends when both run out.",
            "Il reste 14 jours et 10 réunions\u{202f}; l’essai se termine quand les deux sont épuisés.",
        ),
        (
            1,
            1,
            "1 day and 1 meeting left. The trial ends when both run out.",
            "Il reste 1 jour et 1 réunion\u{202f}; l’essai se termine quand les deux sont épuisés.",
        ),
        (
            2,
            2,
            "2 days and 2 meetings left. The trial ends when both run out.",
            "Il reste 2 jours et 2 réunions\u{202f}; l’essai se termine quand les deux sont épuisés.",
        ),
        // Ten meetings made, days left: the days alone.
        (4, 0, "4 days left.", "Il reste 4 jours."),
        (1, 0, "1 day left.", "Il reste 1 jour."),
        // The days past, meetings left: the meetings alone.
        (0, 7, "7 meetings left.", "Il reste 7 réunions."),
        (0, 1, "1 meeting left.", "Il reste 1 réunion."),
    ] {
        assert_eq!(said(days, meetings, Language::English), english);
        assert_eq!(said(days, meetings, Language::French), french);
    }
    for language in BOTH {
        assert!(!words::window(language).trial.is_empty());
    }
    assert_eq!(words::window(Language::French).trial, "Essai gratuit");
    assert_eq!(words::window(Language::English).trial, "Free trial");
}

#[test]
fn minutes_failed_by_the_meetings_own_model_name_it_and_send_to_refaire_advanced_not_settings() {
    use kitokey_shell::summarising::Cause;
    // Where the gesture is another model: the meeting's own named, and where it was chosen.
    for cause in [Cause::TooSlow, Cause::UnknownModel, Cause::Down, Cause::CutShort, Cause::NotMinutes] {
        for (language, place, settings) in [
            (Language::English, "Redo…\u{a0}› Advanced", "settings"),
            (Language::French, "Refaire…\u{a0}› Avancé", "réglages"),
        ] {
            let said = words::cause_own(cause, language).unwrap_or_else(|| panic!("{cause:?} has no sentence of its own model"));
            assert!(
                said.matches("{}").count() == 1 && said.contains(place) && !said.contains(settings) && !said.contains('\''),
                "{cause:?}: {said}"
            );
        }
    }
    // Where the gesture is the provider's — a key, credit, an address —, Settings' sentence stays.
    for cause in [
        Cause::RefusedKey,
        Cause::NoCredit,
        Cause::Redirected,
        Cause::Settings,
        Cause::Unreachable,
        Cause::NoSpeech,
    ] {
        assert_eq!(words::cause_own(cause, Language::French), None, "{cause:?}");
    }
}

#[test]
fn a_model_is_said_as_the_page_says_it() {
    // Each as the page's modelSaid says it (node, on window.js's function, 2026-11-12).
    for (model, said) in [
        ("mistral:latest", "Mistral"),
        ("mistral:7b", "Mistral 7b"),
        ("qwen3:8b", "Qwen3 8b"),
        ("anthropic/claude-sonnet-4-5", "Claude Sonnet 4.5"),
        ("gpt-4.1-mini", "GPT 4.1 Mini"),
        ("llama3.1:8b-instruct-q4_K_M", "Llama3.1 8b Instruct"),
        ("deepseek-coder:6.7b", "Deepseek Coder 6.7b"),
        ("gemma3:27b-it-qat", "Gemma3 27b It"),
        ("mistral-small3.2:24b-instruct-2506-q4_K_M", "Mistral Small3.2 24b Instruct 2506"),
        ("openai/gpt-4o-2024-08-06", "GPT 4o"),
        ("Qwen/Qwen2.5-7B-Instruct-GGUF", "Qwen2.5 7B Instruct GGUF"),
        ("model.Q5_K_M.gguf", "Model"),
        ("phi4:14b-fp16", "Phi4 14b"),
        ("q4_K_M", "q4_K_M"),
        ("llama-3-8b-int8", "Llama 3 8b"),
        ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku"),
        ("x/", "X"),
        ("a b", "A B"),
    ] {
        assert_eq!(words::model_said(model), said, "{model}");
    }
}

#[test]
fn a_notice_of_minutes_failed_by_the_meetings_own_model_says_what_its_page_says() {
    use kitokey_shell::{summarising::Cause, words::Minutes};
    for (language, place, settings) in [
        (Language::English, "Redo…\u{a0}› Advanced", "settings"),
        (Language::French, "Refaire…\u{a0}› Avancé", "réglages"),
    ] {
        for again in [false, true] {
            let own = words::minutes(
                &Minutes::failed_by(Some(Cause::NotMinutes), again, Some("mistral:latest"), language),
                language,
            );
            let page = words::cause_said(Cause::NotMinutes, Some("mistral:latest"), language);
            assert!(
                own.body.ends_with(&page) && page.starts_with("Mistral, ") && page.contains(place) && !own.body.contains(settings),
                "{}",
                own.body
            );
            // The title as for any failure of the same kind.
            assert_eq!(
                own.title,
                words::minutes(&Minutes::failed(Some(Cause::NotMinutes), again), language).title
            );
            // Settings' model, or a cause that is the provider's: today's notice, word for word.
            for (cause, by) in [(Cause::NotMinutes, None), (Cause::RefusedKey, Some("mistral:latest"))] {
                let notice = words::minutes(&Minutes::failed_by(Some(cause), again, by, language), language);
                let before = words::minutes(&Minutes::failed(Some(cause), again), language);
                assert_eq!((notice.title, notice.body), (before.title, before.body));
            }
        }
    }
}

/// A meeting's day and hour, as the window's `Intl` says them (CLDR's abbreviated months).
fn at(day: &str, time: &str) -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::parse_from_str(&format!("{day} {time}"), "%Y-%m-%d %H:%M").unwrap()
}

#[test]
fn a_day_and_an_hour_are_said_as_the_window_says_them() {
    let french = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc.",
    ];
    let english = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    for (month, (fr, en)) in french.iter().zip(english).enumerate() {
        let day = chrono::NaiveDate::from_ymd_opt(2026, month as u32 + 1, 18).unwrap();
        assert_eq!(words::day(day, Language::French), format!("18 {fr}"));
        assert_eq!(words::day(day, Language::English), format!("{en} 18"));
    }
    let first = chrono::NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
    assert_eq!(
        (words::day(first, Language::French), words::day(first, Language::English)),
        ("1 nov.".into(), "Nov 1".into())
    );
    for (time, fr, en) in [
        ("12:00", "12:00", "12:00 PM"),
        ("09:42", "09:42", "09:42 AM"),
        ("00:05", "00:05", "12:05 AM"),
        ("17:31", "17:31", "05:31 PM"),
    ] {
        let when = at("2026-11-13", time);
        assert_eq!(words::hour(when.time(), Language::French), fr);
        assert_eq!(words::hour(when.time(), Language::English), en);
    }
}

#[test]
fn a_meeting_is_named_by_its_title_else_its_hour_that_day_else_its_day_and_hour() {
    let started = Some(at("2026-10-18", "12:00"));
    let (that_day, later) = (at("2026-10-18", "23:59").date(), at("2026-11-14", "01:00").date());
    let named = |title, when, today, language| words::meeting(title, when, today, language);
    assert_eq!(
        named(Some("Point hebdo"), started, later, Language::French),
        "«\u{a0}Point hebdo\u{a0}»"
    );
    assert_eq!(named(Some("Weekly sync"), started, later, Language::English), "“Weekly sync”");
    assert_eq!(named(None, started, that_day, Language::French), "la réunion de 12:00");
    assert_eq!(named(None, started, that_day, Language::English), "the 12:00 PM meeting");
    assert_eq!(named(None, started, later, Language::French), "la réunion du 18 oct. à 12:00");
    assert_eq!(named(None, started, later, Language::English), "the meeting of Oct 18 at 12:00 PM");
    // A title of blanks is none; a meeting whose start cannot be read is the meeting.
    assert_eq!(
        named(Some(" \t"), started, later, Language::French),
        "la réunion du 18 oct. à 12:00"
    );
    assert_eq!(named(None, None, later, Language::French), "la réunion");
    assert_eq!(named(None, None, later, Language::English), "the meeting");
}

#[test]
fn the_notification_that_ends_a_meeting_names_it_first_and_keeps_its_title() {
    for language in BOTH {
        let notice = words::minutes(&words::Minutes::Ready, language);
        let meeting = words::meeting(None, Some(at("2026-10-18", "12:00")), at("2026-11-14", "01:00").date(), language);
        let told = words::for_meeting(notice.clone(), &meeting, language);
        assert_eq!(told.title, notice.title);
        assert!(told.body.ends_with(&notice.body), "{told:?}");
        assert!(told.body.contains(&meeting), "{told:?}");
        assert!(told.title.chars().count() <= 50, "{told:?}");
    }
    let told = words::for_meeting(
        words::minutes(&words::Minutes::Ready, Language::French),
        "«\u{a0}Point hebdo\u{a0}»",
        Language::French,
    );
    assert!(told.body.starts_with("Pour «\u{a0}Point hebdo\u{a0}». "), "{told:?}");
    let told = words::for_meeting(
        words::minutes(&words::Minutes::Ready, Language::English),
        "the 12:00 PM meeting",
        Language::English,
    );
    assert!(told.body.starts_with("For the 12:00 PM meeting. "), "{told:?}");
}

/// A settings file refused is said in the user's language: which setting, on which line, and what
/// to do — never `settings.rs`'s own sentence, which stays for the log and behind « Développeur »
/// (#783, D94). The window says it under each command it refuses, and the notification at a start.
#[test]
fn a_settings_file_refused_says_which_setting_and_what_to_do_in_both_languages() {
    use words::Trouble;
    let said = [
        (
            Refused::Unopened,
            "Kitokey cannot open its settings file: check that you may read it.",
            "Kitokey ne peut pas ouvrir son fichier de réglages\u{a0}: vérifiez que vous pouvez le lire.",
        ),
        (
            Refused::Unparsed {
                line: 4,
                key: Some("after"),
            },
            "Line 4: Kitokey does not understand the value of “after”. Correct it, or remove the line.",
            "Ligne 4\u{a0}: Kitokey ne comprend pas la valeur de «\u{a0}after\u{a0}». Corrigez-la, ou supprimez la ligne.",
        ),
        (
            Refused::Unparsed { line: 4, key: None },
            "Line 4 is not written as a setting. Correct it, or remove it.",
            "La ligne 4 n’est pas écrite comme un réglage. Corrigez-la, ou supprimez-la.",
        ),
        (
            Refused::SessionsNotWhole { line: 4 },
            "Line 4: “sessions” gives a folder’s whole path, from / or from ~. Correct it.",
            "Ligne 4\u{a0}: «\u{a0}sessions\u{a0}» donne le chemin entier d’un dossier, depuis / ou depuis ~. Corrigez-le.",
        ),
        (
            Refused::ModelNotWhole { line: 4 },
            "Line 4: “model” gives a file’s whole path, from / or from ~. Correct it.",
            "Ligne 4\u{a0}: «\u{a0}model\u{a0}» donne le chemin entier d’un fichier, depuis / ou depuis ~. Corrigez-le.",
        ),
        (
            Refused::KeepAudio { line: 4 },
            "Line 4: “keep_audio” is “always” or a number of days, from 1 to 36,500. Correct it.",
            "Ligne 4\u{a0}: «\u{a0}keep_audio\u{a0}» vaut «\u{a0}always\u{a0}» ou un nombre de jours, de 1 à 36\u{202f}500. Corrigez-le.",
        ),
    ];
    // Each refusal `settings.rs` makes is said: one added fails to compile here until it is named.
    for (refused, ..) in &said {
        match refused {
            Refused::Unopened
            | Refused::Unparsed { .. }
            | Refused::SessionsNotWhole { .. }
            | Refused::ModelNotWhole { .. }
            | Refused::KeepAudio { .. } => {}
        }
    }
    let raw = "settings.toml, line 4: keep_audio is \"always\" or a number of days";
    for (refused, english, french) in said {
        for (language, sentence) in [(Language::English, english), (Language::French, french)] {
            assert_eq!(words::refused(&refused, language), sentence);
            // The notification at a start: what still works, then the setting and the gesture.
            let Notice { title, body } = words::trouble(&Trouble::Settings(refused), language);
            assert!(title.chars().count() <= 50 && body.ends_with(sentence), "{title} — {body}");
            assert!(
                body.starts_with(["Kitokey is using default settings", "Kitokey utilise les réglages par défaut"][language as usize]),
                "{body}"
            );
            // Under a command the window refused: its sentence, the refusal after it, the raw text
            // only in developer mode.
            for failure in [Failure::SettingsUnread(refused), Failure::SettingsLocked(refused)] {
                let shown = |developer| serde_json::to_value(words::told(failure, raw, language, developer)).unwrap();
                let shown = (shown(false), shown(true));
                assert!(shown.0.as_str().unwrap().ends_with(sentence), "{shown:?}");
                assert!(!shown.0.as_str().unwrap().contains(raw), "{shown:?}");
                assert_eq!(shown.1.as_str().unwrap(), format!("{}\n{raw}", shown.0.as_str().unwrap()));
            }
        }
    }
}

// The voice guide (`docs/voice.md`, D94): a notification's title, read whole as it is sent.

/// The titles over 50 characters: none since the notifications' sentences were rewritten (D94),
/// and a list that only shrinks.
const TITLES_TOO_LONG: [&str; 0] = [];

/// The functions of `words.rs` that return a notification, each read below: one added there fails
/// the title's test until it is named here — and read in `every_notice`, which the review checks.
const NOTIFYING: [&str; 18] = [
    "for_meeting",
    "of",
    "notice",
    "trouble",
    "made",
    "minutes",
    "transcript_kept_beside",
    "told",
    "sends_nothing",
    "lost_beside",
    "back",
    "offer",
    "audio_trashed",
    "carried_over",
    "carry_failed",
    "model_needed",
    "pulled",
    "fetched",
];

/// Every notification of the shell in one language, each variant of what it is sent for read. Each
/// enum is matched here without a wildcard: a variant added fails to compile until it is named in
/// that match — which does not give it a value in the list; the review of the change does.
fn every_notice(language: Language) -> Vec<Notice> {
    use kitokey_shell::{summarising::Cause, transcription::Made};
    use words::{Minutes, Sides, Trouble};
    let named = || Some("Jabra Evolve2 65".to_string());
    let causes = [
        Cause::Unreachable,
        Cause::TooSlow,
        Cause::RefusedKey,
        Cause::NoCredit,
        Cause::UnknownModel,
        Cause::TooMany,
        Cause::Down,
        Cause::NoTimes,
        Cause::Redirected,
        Cause::ProviderFailed,
        Cause::CutShort,
        Cause::NotMinutes,
        Cause::TranscriptUnusable,
        Cause::NoSpeech,
        Cause::Settings,
        Cause::Unwritable,
    ];
    for cause in causes {
        match cause {
            Cause::Unreachable
            | Cause::TooSlow
            | Cause::RefusedKey
            | Cause::NoCredit
            | Cause::UnknownModel
            | Cause::TooMany
            | Cause::Down
            | Cause::NoTimes
            | Cause::Redirected
            | Cause::ProviderFailed
            | Cause::CutShort
            | Cause::NotMinutes
            | Cause::TranscriptUnusable
            | Cause::NoSpeech
            | Cause::Settings
            | Cause::Unwritable => {}
        }
    }
    let mut notices = Vec::new();
    for source in [Source::Me, Source::Others] {
        for said in [
            Said::Unavailable(source, "x".into()),
            Said::Started(PathBuf::from("/s"), named(), named()),
            Said::Silent(source),
            Said::Nothing,
            Said::Heard(source),
            Said::Level(source, 0.5),
            Said::Moved(source, named()),
            Said::Moved(source, None),
            Said::Lost(source),
            Said::MicrophoneGone,
            Said::MicrophoneBack,
            Said::Stopped(PathBuf::from("/s"), 3725.0),
            Said::Failed("x".into()),
        ] {
            match said {
                Said::Unavailable(..)
                | Said::Started(..)
                | Said::Silent(_)
                | Said::Nothing
                | Said::Heard(_)
                | Said::Level(..)
                | Said::Moved(..)
                | Said::Lost(_)
                | Said::MicrophoneGone
                | Said::MicrophoneBack
                | Said::Stopped(..)
                | Said::Failed(_) => {}
            }
            notices.extend(words::of(&said, true, language));
            let mut sides = Sides::default();
            sides.told(&Said::Started(PathBuf::from("/s"), named(), named()), false, language);
            notices.extend(sides.told(&said, false, language));
        }
        notices.push(words::back(named().as_deref(), language));
    }
    for trouble in [
        Trouble::NoRecorder("x".into()),
        Trouble::Settings(Refused::Unopened),
        Trouble::Shortcut("x".into()),
        Trouble::NoModel("x".into()),
        Trouble::NoTranscriber("x".into()),
        Trouble::NoFolder("x".into()),
        Trouble::NoSession,
    ] {
        match trouble {
            Trouble::NoRecorder(_)
            | Trouble::Settings(_)
            | Trouble::Shortcut(_)
            | Trouble::NoModel(_)
            | Trouble::NoTranscriber(_)
            | Trouble::NoFolder(_)
            | Trouble::NoSession => {}
        }
        notices.push(words::trouble(&trouble, language));
    }
    for made in [Made::Written(PathBuf::from("/s/transcript.jsonl")), Made::Failed("x".into())] {
        match made {
            Made::Written(_) | Made::Failed(_) => {}
        }
        notices.push(words::made(&made, language));
    }
    let mut minutes = vec![
        Minutes::Ready,
        Minutes::ReadyKeptBeside("x".into()),
        Minutes::Failed(None),
        Minutes::RedoFailed(None),
        Minutes::Waiting("x".into()),
        Minutes::Unchosen,
        Minutes::NoProvider("x".into()),
        Minutes::NoTranscript,
        Minutes::OllamaWaits { silent: false },
        Minutes::OllamaWaits { silent: true },
    ];
    for cause in causes {
        minutes.push(Minutes::Failed(Some(cause)));
        minutes.push(Minutes::RedoFailed(Some(cause)));
        for again in [false, true] {
            minutes.push(Minutes::FailedOwn {
                cause,
                model: "qwen3:8b".into(),
                again,
            });
        }
    }
    for said in minutes {
        match said {
            Minutes::Ready
            | Minutes::ReadyKeptBeside(_)
            | Minutes::Failed(_)
            | Minutes::RedoFailed(_)
            | Minutes::FailedOwn { .. }
            | Minutes::Waiting(_)
            | Minutes::Unchosen
            | Minutes::NoProvider(_)
            | Minutes::NoTranscript
            | Minutes::OllamaWaits { .. } => {}
        }
        notices.push(words::minutes(&said, language));
    }
    for not_put_back in [false, true] {
        notices.push(words::transcript_kept_beside(language, "x", not_put_back));
    }
    notices.push(words::offer("Microsoft Teams", language).0);
    for outcome in [
        Ok((1, 30)),
        Ok((3, 30)),
        Err(Refusal::NotAllowed),
        Err(Refusal::NoSpace),
        Err(Refusal::Other),
    ] {
        if let Err(refusal) = outcome {
            match refusal {
                Refusal::NotAllowed | Refusal::NoSpace | Refusal::Other => {}
            }
        }
        notices.push(words::audio_trashed(language, &outcome));
    }
    notices.push(words::carried_over(language));
    notices.push(words::carry_failed(language));
    notices.push(words::model_needed(language, "1.6 GB", "huggingface.co"));
    for outcome in [Ok(()), Err("x".to_string())] {
        notices.push(words::pulled(&outcome, "qwen3:8b", language));
        for chosen in [false, true] {
            notices.push(words::fetched(&outcome, chosen, language));
        }
    }
    // Each one sent again with its meeting named first (D92): its title kept.
    let named: Vec<_> = notices
        .iter()
        .map(|notice| words::for_meeting(notice.clone(), "x", language))
        .collect();
    notices.extend(named);
    notices
}

#[test]
fn a_notification_s_title_holds_50_characters() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/words.rs")).unwrap();
    // Read over the whole file, not line by line: a signature may span several lines.
    let notifying: Vec<_> = source
        .match_indices("fn ")
        .filter(|(at, _)| source[..*at].ends_with(|c: char| c.is_whitespace()))
        .filter_map(|(at, _)| {
            let rest = &source[at + 3..];
            let name = &rest[..rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?];
            // Its parameters, to the parenthesis that closes them; then what it returns, to its body.
            let (mut depth, mut close) = (0, None);
            for (index, c) in rest.char_indices().skip_while(|&(_, c)| c != '(') {
                depth += match c {
                    '(' => 1,
                    ')' => -1,
                    _ => 0,
                };
                if depth == 0 {
                    close = Some(index);
                    break;
                }
            }
            let after = &rest[close? + 1..];
            let returns = &after[..after.find(['{', ';'])?];
            returns.split_once("->")?.1.contains("Notice").then_some(name)
        })
        .collect();
    assert_eq!(
        notifying, NOTIFYING,
        "a function that sends a notification: read it in `every_notice`"
    );
    let mut wrong: Vec<_> = BOTH
        .iter()
        .flat_map(|&language| every_notice(language))
        .map(|notice| notice.title)
        .filter(|title| title.chars().count() > 50)
        .collect();
    wrong.sort();
    wrong.dedup();
    assert_eq!(
        wrong, TITLES_TOO_LONG,
        "a title over 50 characters, or one put right: take it from the list"
    );
}

/// Every sentence of the window is read by the code that shows it — the window's script, or the
/// shell's own code —: one that nothing reads any more is removed, not kept unseen (D94).
#[test]
fn every_sentence_of_the_window_is_read_by_the_code() {
    let root = env!("CARGO_MANIFEST_DIR");
    // The window's scripts, each: window.js and those beside it (D122's engines.js).
    let mut code = String::new();
    for entry in std::fs::read_dir(format!("{root}/ui")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "js") {
            code += &std::fs::read_to_string(path).unwrap();
        }
    }
    for entry in std::fs::read_dir(format!("{root}/src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "rs") && !path.ends_with("words.rs") {
            code += &std::fs::read_to_string(path).unwrap();
        }
    }
    let read = |field: &str| {
        code.match_indices(&format!(".{field}"))
            .any(|(at, found)| !code[at + found.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_'))
    };
    let unread: std::collections::BTreeSet<String> = sentences()
        .into_iter()
        .filter(|sentence| sentence.function == "window")
        .filter_map(|sentence| sentence.field.map(|(field, _)| field))
        .filter(|field| !read(field))
        .collect();
    assert!(unread.is_empty(), "sentences no code reads: remove them\n{unread:?}");
}
