//! What the user is told, in one place: every sentence of the shell is here, in each language
//! it speaks, so that it is read as a whole and a language is added by adding its sentences.

use crate::{
    recording::{Said, Source},
    registry::Good,
    settings::Refused,
    summarising::Cause,
    transcription::Made,
    trial::Left,
};

/// A language the shell speaks. The interface follows the system's, unless the user chose one (D64);
/// any other is English.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    French,
}

impl Language {
    /// The language of a locale as a system names it: `fr-FR`, `fr_CA.UTF-8`, `en-GB`.
    pub fn of(locale: &str) -> Self {
        let language = locale.split(['-', '_', '.', '@']).next().unwrap_or_default();
        if language.eq_ignore_ascii_case("fr") {
            Self::French
        } else {
            Self::English
        }
    }

    /// The one a code names, as the settings write it — `fr`, `en` —; none for a language it does
    /// not speak.
    pub fn named(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "fr" => Some(Self::French),
            "en" => Some(Self::English),
            _ => None,
        }
    }

    /// Its code, as a transcript's language is asked for: `fr` or `en`.
    pub fn code(self) -> &'static str {
        match self {
            Self::French => "fr",
            Self::English => "en",
        }
    }

    /// The language the user's system is set to.
    pub fn system() -> Self {
        Self::of(&sys_locale::get_locale().unwrap_or_default())
    }
}

/// Whether the desktop says a notification was clicked, so that a click opens the meeting: under
/// Linux, its notification server does; elsewhere a notification promises no click (D44).
const CLICKED: bool = cfg!(target_os = "linux");

/// A notification: a title that says it all at a glance, and one or two sentences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

/// A day as the window's `Intl` says it, with CLDR's abbreviated months: « 18 oct. », "Oct 18".
/// One formatter for every day Kitokey says beside a meeting (D92, D93).
pub fn day(day: chrono::NaiveDate, language: Language) -> String {
    // Shows: in a notification's body, the day a meeting was recorded. When: a meeting is named by
    // its day and hour, not recorded today. At most: no limit. Placeholders: none.
    use chrono::Datelike;
    // CLDR's abbreviated months, as `Intl.DateTimeFormat` gives them with `month: "short"`.
    let month = day.month0() as usize;
    match language {
        Language::English => {
            let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            format!("{} {}", months[month], day.day())
        }
        Language::French => {
            let months = [
                "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc.",
            ];
            format!("{} {}", day.day(), months[month])
        }
    }
}

/// An hour as the window's `Intl` says it, two digits: « 09:42 », "09:42 AM".
pub fn hour(time: chrono::NaiveTime, language: Language) -> String {
    // Shows: in a notification's body, the hour a meeting was recorded. When: a meeting is named by
    // its hour. At most: no limit. Placeholders: none.
    match language {
        Language::English => time.format("%I:%M %p").to_string(),
        // `%R`: the hour and minutes on 24 hours, as `%H:%M`.
        Language::French => time.format("%R").to_string(),
    }
}

/// A meeting named in a sentence (D92): the title it was given, else when it was recorded — its
/// hour on the day it is said, its day and hour after. Used where a sentence names the meeting it
/// is about; the window names it in its list and page itself.
pub fn meeting(title: Option<&str>, started: Option<chrono::NaiveDateTime>, today: chrono::NaiveDate, language: Language) -> String {
    // Shows: in a notification's body, the meeting it is about. When: a meeting is named, without a
    // title of its own. At most: no limit. Placeholders: {} its hour, or its day then its hour.
    use Language::{English, French};
    match (title.map(str::trim).filter(|title| !title.is_empty()), started, language) {
        (Some(title), _, French) => format!("«\u{a0}{title}\u{a0}»"),
        (Some(title), _, English) => format!("“{title}”"),
        (None, Some(started), French) if started.date() == today => format!("la réunion de {}", hour(started.time(), French)),
        (None, Some(started), English) if started.date() == today => format!("the {} meeting", hour(started.time(), English)),
        (None, Some(started), French) => {
            format!("la réunion du {} à {}", day(started.date(), French), hour(started.time(), French))
        }
        (None, Some(started), English) => {
            format!(
                "the meeting of {} at {}",
                day(started.date(), English),
                hour(started.time(), English)
            )
        }
        (None, None, French) => "la réunion".into(),
        (None, None, English) => "the meeting".into(),
    }
}

/// The notification that ends a meeting's making, its meeting named first (D92).
pub fn for_meeting(notice: Notice, meeting: &str, language: Language) -> Notice {
    // Shows: a notification's body, before the sentences it had. When: the meeting's last step
    // ended — written or failed —, no window of Kitokey having the focus. At most: no limit.
    // Placeholders: {meeting} the meeting, as `meeting` names it.
    let named = match language {
        Language::French => format!("Pour {meeting}."),
        Language::English => format!("For {meeting}."),
    };
    Notice {
        body: format!("{named} {}", notice.body),
        ..notice
    }
}

/// A length as a person says it: seconds under a minute, then minutes, then hours — each
/// to the nearest, so that a minute and fifty-nine seconds is two minutes.
pub fn length(seconds: f64) -> String {
    let seconds = if seconds.is_finite() { seconds.max(0.0) } else { 0.0 };
    if seconds < 59.5 {
        return format!("{} s", seconds.round() as u64);
    }
    let minutes = (seconds / 60.0).round() as u64;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{minutes} min"),
        (hours, minutes) => format!("{hours} h {minutes:02}"),
    }
}

/// What a program said of a failure, made fit to show: without the name it puts before its
/// sentence, without brackets of its own, on one line, and short. A program that said nothing
/// gives nothing.
fn detail(said: &str) -> String {
    let said = said.trim();
    let said = ["kitokey-capture:", "kitokey-transcription:"]
        .iter()
        .find_map(|name| said.strip_prefix(name))
        .unwrap_or(said)
        .trim();
    let said = said.strip_prefix('(').and_then(|inner| inner.strip_suffix(')')).unwrap_or(said);
    // One line, whatever it held: a notification is no place for a program's line ends.
    let said = said
        .split(|c: char| c.is_whitespace() || c.is_control())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    match said.char_indices().nth(200) {
        Some((cut, _)) => format!("{}…", &said[..cut]),
        None => said.to_string(),
    }
}

/// What the user is told of something a recording said — or nothing, when it calls for no
/// word. `reminder` is the setting: tell, at each start, that announcing the recording is
/// the user's to do. French is written as French is printed: a space that does not break
/// before `:` and `?`.
pub fn of(said: &Said, reminder: bool, language: Language) -> Option<Notice> {
    use Language::{English, French};
    let (title, body) = match (said, language) {
        (Said::Started(..), _) if !reminder => return None,
        // What follows says it: the device named, or the side with none.
        (Said::Heard(_) | Said::Level(..) | Said::MicrophoneBack, _) => return None,
        // The microphone is named, so that one other than the call's shows at once.
        (Said::Started(_, microphone, _), English) => {
            let named = microphone
                .as_ref()
                .map(|name| format!("Microphone recorded: {name}. "))
                .unwrap_or_default();
            return notice(
                "Recording",
                format!("{named}Tell the participants that this meeting is being recorded."),
            );
        }
        (Said::Started(_, microphone, _), French) => {
            let named = microphone
                .as_ref()
                .map(|name| format!("Micro enregistré\u{a0}: {name}. "))
                .unwrap_or_default();
            return notice(
                "Enregistrement en cours",
                format!("{named}Prévenez les participants que cette réunion est enregistrée."),
            );
        }
        (Said::Unavailable(Source::Me, _), English) => {
            ("Your microphone is not recorded for now", "The meeting’s sound is still recorded.")
        }
        (Said::Unavailable(Source::Me, _), French) => (
            "Votre micro n’est pas enregistré pour l’instant",
            "Le son de la réunion est toujours enregistré.",
        ),
        (Said::MicrophoneGone, English) => (
            "The microphone you chose is not there",
            "Your microphone is recorded as on Auto for now, and the one you chose as soon as it is back.",
        ),
        (Said::MicrophoneGone, French) => (
            "Le micro choisi n’est pas là",
            "Votre micro est enregistré comme sur Auto pour l’instant, et celui choisi dès qu’il revient.",
        ),
        (Said::Unavailable(Source::Others, _), English) => ("The meeting’s sound is not recorded", "Your microphone is still recorded."),
        (Said::Unavailable(Source::Others, _), French) => {
            ("Le son de la réunion n’est pas enregistré", "Votre micro est toujours enregistré.")
        }
        // The device recorded changed (D31): what is recorded now is named.
        (Said::Moved(Source::Me, Some(name)), English) => return notice("Microphone changed", format!("Now recorded: {name}.")),
        (Said::Moved(Source::Me, Some(name)), French) => return notice("Micro changé", format!("Enregistré maintenant\u{a0}: {name}.")),
        (Said::Moved(Source::Others, Some(name)), English) => {
            return notice("Sound output changed", format!("The meeting’s sound is now recorded from {name}."));
        }
        (Said::Moved(Source::Others, Some(name)), French) => {
            return notice(
                "Sortie audio changée",
                format!("Le son de la réunion est maintenant enregistré depuis {name}."),
            );
        }
        (Said::Moved(Source::Me, None), English) => (
            "Microphone changed",
            "Your microphone is still recorded, from the one your computer uses now.",
        ),
        (Said::Moved(Source::Me, None), French) => (
            "Micro changé",
            "Votre micro est toujours enregistré, depuis celui qu’utilise maintenant votre ordinateur.",
        ),
        (Said::Moved(Source::Others, None), English) => (
            "Sound output changed",
            "The meeting’s sound is still recorded, from the output your computer uses now.",
        ),
        (Said::Moved(Source::Others, None), French) => (
            "Sortie audio changée",
            "Le son de la réunion est toujours enregistré, depuis la sortie qu’utilise maintenant votre ordinateur.",
        ),
        // None left: that side, and nothing of the other, which may have none either. Of the
        // microphone, never why: the system took it away, or a call let go of it (D66).
        (Said::Lost(Source::Me), English) => ("Your microphone is not recorded for now", "The recording goes on."),
        (Said::Lost(Source::Me), French) => ("Votre micro n’est pas enregistré pour l’instant", "L’enregistrement continue."),
        (Said::Lost(Source::Others), English) => (
            "The meeting’s sound is not recorded for now",
            "Your computer has no sound output left. An output plugged in is recorded at once; the recording goes on.",
        ),
        (Said::Lost(Source::Others), French) => (
            "Le son de la réunion n’est plus enregistré pour l’instant",
            "Votre ordinateur n’a plus de sortie audio. Une sortie branchée est enregistrée aussitôt\u{202f}; l’enregistrement continue.",
        ),
        (Said::Nothing, _) => return sends_nothing(None, language),
        (Said::Silent(Source::Me), English) => (
            "No sound from your microphone",
            "Nothing was heard for ten seconds. The recording goes on.",
        ),
        (Said::Silent(Source::Me), French) => (
            "Votre micro ne capte aucun son",
            "Rien n’a été entendu depuis dix secondes. L’enregistrement continue.",
        ),
        (Said::Silent(Source::Others), English) => (
            "No sound from the meeting",
            "Nothing was heard for ten seconds: is the call playing on this device? The recording goes on.",
        ),
        (Said::Silent(Source::Others), French) => (
            "Aucun son venant de la réunion",
            "Rien n’a été entendu depuis dix secondes\u{a0}: l’appel passe-t-il par cet appareil\u{202f}? L’enregistrement continue.",
        ),
        (Said::Stopped(_, seconds), English) => return notice("Recording stopped", format!("{} recorded.", length(*seconds))),
        (Said::Stopped(_, seconds), French) => {
            // A single second or minute is agreed as one: « 1 min enregistrée », « 16 min enregistrées ».
            let said = length(*seconds);
            let agreed = if matches!(said.as_str(), "0 s" | "1 s" | "1 min") {
                "enregistrée"
            } else {
                "enregistrées"
            };
            return notice("Enregistrement arrêté", format!("{said} {agreed}."));
        }
        (Said::Failed(why), English) if detail(why).is_empty() => (
            "The recording stopped by itself",
            "The program that records ended without saying why. What was recorded is in the session’s folder.",
        ),
        (Said::Failed(why), French) if detail(why).is_empty() => (
            "L’enregistrement s’est arrêté tout seul",
            "Le programme d’enregistrement s’est terminé sans dire pourquoi. Ce qui a été enregistré est gardé dans le dossier de la réunion.",
        ),
        (Said::Failed(_), English) => ("The recording stopped by itself", "What was recorded is in the session’s folder."),
        (Said::Failed(_), French) => (
            "L’enregistrement s’est arrêté tout seul",
            "Ce qui a été enregistré est gardé dans le dossier de la réunion.",
        ),
    };
    notice(title, body.to_string())
}

/// A notice. What a program or the system said of a failure is never in it: it goes to the
/// log, and to the window's page in developer mode (D29).
fn notice(title: &str, body: String) -> Option<Notice> {
    Some(Notice {
        title: title.to_string(),
        body,
    })
}

/// The tray's menu: what each entry says, in the state the recording is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub start: &'static str,
    pub stop: &'static str,
    pub stopping: &'static str,
    pub quit: &'static str,
    /// Opens the window.
    pub open: &'static str,
    /// What the tray's icon says of itself while a recording runs.
    pub recording: &'static str,
    /// Makes the transcript of the last recording, again when it failed.
    pub transcribe: &'static str,
    /// The same entry while a transcript is being made.
    pub transcribing: &'static str,
    /// Has the minutes of the last recording written, again when it failed.
    pub minutes: &'static str,
    /// The same entry while they are being written.
    pub minuting: &'static str,
    /// Brings back the recording bar the user hid (D90).
    pub show_bar: &'static str,
}

pub fn menu(language: Language) -> Menu {
    match language {
        Language::English => Menu {
            start: "Start recording",
            stop: "Stop recording",
            stopping: "Stopping…",
            quit: "Quit Kitokey",
            open: "Open Kitokey",
            recording: "Kitokey — recording",
            transcribe: "Transcribe the last recording",
            transcribing: "Transcribing…",
            minutes: "Write the minutes of the last recording",
            minuting: "Writing the minutes…",
            show_bar: "Show the bar",
        },
        Language::French => Menu {
            start: "Démarrer un enregistrement",
            stop: "Arrêter l’enregistrement",
            stopping: "Arrêt en cours…",
            quit: "Quitter Kitokey",
            open: "Ouvrir Kitokey",
            recording: "Kitokey — enregistrement en cours",
            transcribe: "Transcrire le dernier enregistrement",
            transcribing: "Transcription en cours…",
            minutes: "Rédiger le compte rendu de la dernière réunion",
            minuting: "Compte rendu en cours…",
            show_bar: "Afficher la barre",
        },
    }
}

/// The window's words. A `{}` is where the window puts a number, a size or a month: it formats
/// those itself, in the language named by `locale`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Window {
    pub locale: &'static str,
    /// Bytes, kilobytes, megabytes, gigabytes.
    pub units: [&'static str; 4],
    /// What a screen reader calls the window's sections.
    pub sections: &'static str,
    pub meetings: &'static str,
    /// How many meetings weigh the size beside it (board 4's foot).
    pub weight: &'static str,
    /// The same, for one meeting.
    pub weight_one: &'static str,
    pub this_week: &'static str,
    pub last_week: &'static str,
    /// Earlier in the current month: the month's name.
    pub earlier_in: &'static str,
    /// Sessions whose start cannot be read as a day.
    pub undated: &'static str,
    pub minutes: &'static str,
    pub transcript: &'static str,
    pub audio: &'static str,
    pub audio_freed: &'static str,
    /// What a meeting's audio weighs, beside its length in its header (D89).
    pub audio_weighs: &'static str,
    pub recording: &'static str,
    pub interrupted: &'static str,
    /// No session in the folder (#775): said without its path, which `none_folder` says under the action.
    pub none: &'static str,
    /// Shows: the empty pane, a discreet line under its action (#775). When: no meeting in the folder.
    /// At most: no limit, a path wraps. Placeholders: `{}`, the folder's whole path.
    pub none_folder: &'static str,
    /// The sidebar's search (D61): what its field says, and none of the meetings holding what is typed.
    /// No meeting yet: what to do.
    pub none_do: &'static str,
    pub list_unread: &'static str,
    pub list_unread_do: &'static str,
    /// The settings file cannot be read: said above every page but Settings (#770) — never read since
    /// Kitokey started, then read before and its last settings held.
    /// Shows: a banner above every page but Settings. When: the settings file cannot be read and was
    /// never read since Kitokey started. At most: no limit. Placeholders: none.
    pub settings_banner: &'static str,
    /// Shows: a banner above every page but Settings. When: the settings file cannot be read, read
    /// before, its last settings held. At most: no limit. Placeholders: none.
    pub settings_banner_kept: &'static str,
    pub again_refused: &'static str,
    pub fold_not_kept: &'static str,
    pub agreement_refused: &'static str,
    pub meeting_not_opened: &'static str,
    pub page_not_shown: &'static str,
    pub context_not_read: &'static str,
    pub record_starting: &'static str,
    pub record_stopping: &'static str,
    pub redo_cancel: &'static str,
    /// Shows: Settings, beside a setting just saved. When: for a moment after it is. At most: one
    /// line. Placeholders: none.
    pub saved: &'static str,
    pub small_not_confidential: &'static str,
    /// A recording refused, the trial over (PDR-0005, D83): why and what stays, in one sentence; the one gesture.
    pub trial_over: &'static str,
    pub buy: &'static str,
    /// Settings on a trial licence (D83): the name of what they say is left of it (`trial_left`).
    /// Shows: Settings, its first card, its label, beside what is left (`trial_left`). When: on a
    /// trial licence (D83). At most: one line. Placeholders: none.
    pub trial: &'static str,
    pub find_meetings: &'static str,
    /// The sidebar folded and brought back (D61): each button's name; the shortcut is added beside.
    pub fold: &'static str,
    pub unfold: &'static str,
    pub none_found: &'static str,
    /// One meeting's page.
    pub back: &'static str,
    pub open_folder: &'static str,
    /// Shows: a meeting's page, beside « Ouvrir le dossier »: the icon's name, said and as its tooltip
    /// (D100). When: always. At most: one line. Placeholders: none.
    pub trash_meeting: &'static str,
    /// Shows: a meeting's page, the question in place once the trash's icon is pressed: its title (D100).
    /// When: the icon pressed. At most: one line. Placeholders: none.
    pub trash_ask: &'static str,
    /// Shows: the trash's question, under its title: what the meeting holds (D100). When: it holds an
    /// audio, a transcript or a summary. At most: one line. Placeholders: {} the parts it holds, listed
    /// in the window's language from `trash_parts`.
    pub trash_holds: &'static str,
    /// Shows: inside `trash_holds`, its audio, its transcript, its summary (D100). When: the meeting
    /// holds it. At most: one line each. Placeholders: none.
    pub trash_parts: [&'static str; 3],
    /// Shows: the trash's question, under what it holds: that the system's trash gives it back (D100).
    /// When: always with the question. At most: one line. Placeholders: none.
    pub trash_restore: &'static str,
    /// Shows: the trash's question, its first button, focused first (D100). When: always with the
    /// question. At most: one line, never cut. Placeholders: none.
    pub trash_cancel: &'static str,
    /// Shows: the trash's question, its second button (D100). When: always with the question. At most:
    /// one line, never cut. Placeholders: none.
    pub trash_confirm: &'static str,
    /// Shows: the trash's question, under its buttons, said to assistive technology too (D100). When: the
    /// meeting is being recorded or made, or the system's trash refused it. At most: no limit.
    /// Placeholders: {} the reason.
    pub trash_refused: &'static str,
    /// The menu of a meeting's rarer actions (D89), and a setting's name before its value.
    pub more_actions: &'static str,
    pub labelled: &'static str,
    pub minutes_title: &'static str,
    pub transcript_title: &'static str,
    pub search: &'static str,
    /// The minutes' maker: the model, the provider, when.
    pub written_by: &'static str,
    /// Its provider, when known, said after the model.
    pub written_via: &'static str,
    /// Between a day and its hour: "13 oct. à 12:21", "Oct 13 at 12:21 PM".
    pub at: &'static str,
    /// The transcript's maker: the model, on this computer.
    pub heard_by: &'static str,
    /// Shows: a meeting's page, the transcript's maker line: that day and hour first, its model after —
    /// "Transcrit le 13 nov. à 09:42 · Whisper small" (D93). When: the transcript says when it was made
    /// whole. At most: one line. Placeholders: {} the day and hour, in local time, without the year.
    pub transcribed_on: &'static str,
    /// Shows: a meeting's page, the summary's maker line: that day and hour first, their model after —
    /// "Compte rendu du 13 nov. à 09:45 · Qwen3 8B" (D93). When: the summary says when it was made.
    /// At most: one line. Placeholders: {} the day and hour, in local time, without the year.
    pub minutes_of: &'static str,
    /// Shows: a meeting's page, the line of its transcript and of its minutes, after the model — "en 3 s",
    /// "in 31 min 52 s" (D97). When: the version says how long its making took. At most: one line.
    /// Placeholders: `{}`, the time taken, its units the window's.
    pub made_in: &'static str,
    pub here: &'static str,
    pub remote: &'static str,
    /// A voice told within a track: its label.
    pub voice: &'static str,
    pub here_described: &'static str,
    pub remote_described: &'static str,
    /// Who spoke over the meeting, for whoever does not see the drawing.
    pub who_spoke: &'static str,
    pub no_transcript: &'static str,
    pub no_minutes: &'static str,
    pub being_made: &'static str,
    pub unreadable: &'static str,
    /// Lines of a transcript that are not the contract's, left out: how many.
    pub unread: &'static str,
    pub no_match: &'static str,
    /// The settings' page.
    pub settings: &'static str,
    /// Shows: Settings, « Après un enregistrement », its heading. When: always. At most: one line.
    /// Placeholders: none.
    pub after_title: &'static str,
    /// Each choice of what follows a recording: its name, then what it does.
    /// Shows: Settings, « Après un enregistrement », a choice: its name, then its line under it.
    /// When: always. At most: one line each. Placeholders: none.
    pub after_nothing: [&'static str; 2],
    /// Shows: Settings, « Après un enregistrement », a choice: its name, then its line under it.
    /// When: always. At most: one line each. Placeholders: none.
    pub after_transcript: [&'static str; 2],
    /// Shows: Settings, « Après un enregistrement », a choice: its name, then its line under it.
    /// When: always. At most: one line each. Placeholders: none.
    pub after_minutes: [&'static str; 2],
    /// Shows: Settings, « Modèles », its heading. When: always. At most: one line. Placeholders:
    /// none.
    pub models_title: &'static str,
    /// What the models' card says under its title, and under its steps.
    /// Shows: Settings, « Modèles », under its heading. When: always. At most: no limit.
    /// Placeholders: none.
    pub models_intro: &'static str,
    /// Shows: Settings, « Modèles », its last line: where a key is kept, what leaves the computer.
    /// When: always. At most: no limit. Placeholders: none.
    pub models_note: &'static str,
    /// The transcription model's state, at the page's end.
    /// Shows: Settings, « Modèles », the transcription's step, the label of the transcription
    /// model's state, under its choice. When: Whisper hears. At most: one line. Placeholders: none.
    pub model_state: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a tag beside that state. When: the
    /// model is on this computer. At most: one line. Placeholders: none.
    pub model_ready: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, that state. When: the model is on
    /// this computer. At most: one line. Placeholders: {} its size.
    pub model_here: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, that state. When: the model chosen
    /// is Kitokey's and not here yet. At most: one line. Placeholders: none.
    pub model_missing: &'static str,
    /// One the user named, not on this computer: theirs, nothing offered (D36).
    /// Shows: Settings, « Modèles », the transcription's step, that state, as an error. When: a
    /// model the user named is not on this computer (D36). At most: no limit. Placeholders: none.
    pub model_named_missing: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, that state. When: the model is being
    /// downloaded. At most: one line. Placeholders: none.
    pub model_coming: &'static str,
    pub model_offer: &'static str,
    /// The short list of models to download (D37): its heading, the gesture, how far, and the
    /// model chosen that is not here yet, in the choice.
    /// Shows: Settings, « Modèles », the heading of the folded list of models to download (D37),
    /// and its name for a screen reader. When: always. At most: one line. Placeholders: none.
    pub offered_title: &'static str,
    /// Shows: Settings, « Modèles », models to download, a model's button. When: it is not on this
    /// computer. At most: one line. Placeholders: none.
    pub offered_fetch: &'static str,
    /// Shows: Settings, « Modèles », models to download, in place of that button. When: the model
    /// is being downloaded. At most: one line. Placeholders: {} how far, in percent.
    pub offered_far: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, the Whisper model chosen, in the
    /// list. When: it is not on this computer yet (D37). At most: one line. Placeholders: {} the
    /// model, as people say it.
    pub model_chosen_missing: &'static str,
    /// The minutes' model of the list, fetched by Ollama (D37): its size, what it is good for,
    /// and Ollama needed when none answers.
    /// Shows: Settings, « Modèles », models to download, the summary's model's size. When: always.
    /// At most: one line. Placeholders: none.
    pub offered_ollama_size: &'static str,
    /// Shows: Settings, « Modèles », models to download, what the summary's model is good for.
    /// When: always. At most: one line. Placeholders: none.
    pub offered_ollama_good: &'static str,
    /// Shows: Settings, « Modèles », models to download, beside the summary's model. When: no
    /// Ollama answers on this computer. At most: one line. Placeholders: none.
    pub offered_needs_ollama: &'static str,
    /// The list read again on the user's gesture (D40): the button, its host filled in; and
    /// what is said when it could not be.
    /// Shows: Settings, « Modèles », models to download, a button: the list read again, its host
    /// shown (D40). When: always. At most: one line. Placeholders: {} the host.
    pub offered_refresh: &'static str,
    /// Shows: Settings, « Modèles », models to download, under that button. When: the list could
    /// not be read again, or was not signed. At most: no limit. Placeholders: none.
    pub offered_refresh_failed: &'static str,
    /// Apple's SpeechAnalyzer among who hears, on a Mac with macOS 26 (D42): its name in the
    /// choice; its model not on the Mac yet; the gesture that has macOS fetch it, and from whom;
    /// while it comes; when it did not.
    /// Shows: Settings, « Modèles », the transcription's step, and the first run's assistant (D58),
    /// a choice of who hears. When: on a Mac with macOS 26 (D42). At most: one line. Placeholders:
    /// none.
    pub speech_option: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, appended to Apple's SpeechAnalyzer
    /// in the list. When: its model is not on the Mac yet. At most: one line. Placeholders: none.
    pub speech_missing: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, and the first run's assistant (D58),
    /// a button. When: Apple's SpeechAnalyzer chosen, its model not on the Mac yet. At most: one
    /// line. Placeholders: none.
    pub speech_fetch: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a status in place of that button.
    /// When: macOS downloads Apple's model. At most: one line. Placeholders: none.
    pub speech_fetching: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, an error under the choice. When:
    /// macOS did not download Apple's model. At most: no limit. Placeholders: none.
    pub speech_failed: &'static str,
    /// A service that hears (D43): its fields, the agreement before anything is sent to its host,
    /// the gestures, and what they say.
    pub service_option: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, folded « Avancé »: a field's label,
    /// an address of one's own. When: a service hears. At most: one line. Placeholders: none.
    pub service_address: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a field's label, its list or «
    /// Avancé »'s. When: a service hears. At most: one line. Placeholders: none.
    pub service_model: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a field's label. When: a service
    /// hears. At most: one line. Placeholders: none.
    pub service_key: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a checkbox before anything is sent
    /// (D43). When: a service hears whose host is not this computer. At most: no limit.
    /// Placeholders: {} the host.
    pub service_agree: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, the button that saves the service.
    /// When: a service hears. At most: one line. Placeholders: none.
    pub service_save: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a button. When: a service is saved.
    /// At most: one line. Placeholders: none.
    pub service_test: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, under « Tester ». When: the service
    /// transcribed Kitokey's own sound. At most: no limit. Placeholders: none.
    pub service_tested: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, under « Enregistrer ». When: the
    /// service is saved. At most: no limit. Placeholders: {} the host, or this computer said so
    /// (#775).
    pub service_saved: &'static str,
    /// Shows: Settings, « Transcription », under the service's form, once saved. When: the address saved is
    /// this computer's, where nothing leaves it (#775). At most: one line. Placeholders: none.
    pub service_saved_here: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, under « Enregistrer ». When: the
    /// agreement is not ticked. At most: no limit. Placeholders: {} the host.
    pub service_agree_first: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, under « Enregistrer ». When: the
    /// address typed is not an http(s) address. At most: no limit. Placeholders: none.
    pub service_address_bad: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, under « Enregistrer ». When: no
    /// model is named. At most: no limit. Placeholders: none.
    pub service_model_named: &'static str,
    /// The model's name as people say it, and its size once installed.
    /// Shows: Settings, « Modèles », the transcription's step, a Whisper model of the list, after
    /// its name. When: the model is on this computer. At most: one line. Placeholders: {} its size.
    pub installed: &'static str,
    /// Another number of days, typed; and what is said of one that is none.
    /// Shows: Settings, « Après un enregistrement », that list's last choice, then the field it
    /// opens. When: always. At most: one line. Placeholders: none.
    pub keep_other: &'static str,
    /// Shows: Settings, « Après un enregistrement », under that field. When: what is typed is no
    /// number of days the file allows. At most: no limit. Placeholders: {} what was typed.
    pub keep_not_days: &'static str,
    /// The engine that hears, shipped with Kitokey; and the user's home, as a folder is said.
    /// Shows: Settings, « Modèles », the transcription's step, a tag beside Whisper, built into
    /// Kitokey. When: always. At most: one line. Placeholders: none.
    pub builtin: &'static str,
    /// Shows: Settings, « Après un enregistrement », the folder's path, in place of the user's
    /// home. When: the folder is in it. At most: one line. Placeholders: none.
    pub home: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, its label. When: always. At most:
    /// one line. Placeholders: none.
    pub transcription: &'static str,
    /// Shows: Settings, « Modèles », the line under a step's choice, and models to download. When:
    /// a model of this computer is chosen. At most: one line. Placeholders: none.
    pub on_this_computer: &'static str,
    /// Shows: Settings, « Général », a switch: its name, then its line under it. When: always. At
    /// most: one line for the name, the line wraps. Placeholders: none.
    pub reminder: [&'static str; 2],
    /// When an application takes the microphone (D30): its name and line, as board 4 has them.
    /// Shows: Settings, « Général », a list: its name, then its line under it (D30). When: always.
    /// At most: one line for the name, the line wraps. Placeholders: none.
    pub offer: [&'static str; 2],
    /// Its two choices: offer to record, do nothing.
    /// Shows: Settings, « Général », that list's two choices. When: always. At most: one line each.
    /// Placeholders: none.
    pub offer_choices: [&'static str; 2],
    /// Started with the user's session (D32): its name and line.
    /// Shows: Settings, « Général », a switch: its name, then its line under it (D32). When: the
    /// system offers it. At most: one line for the name, the line wraps. Placeholders: none.
    pub autostart: [&'static str; 2],
    /// On Windows, the switch held (D84): by the user in Windows' Startup settings, where it is
    /// turned back on; by a policy.
    /// Shows: Settings, « Général », that switch's line in place of its own: held by the user in
    /// Windows' settings, or by a policy (D84). When: on Windows, the switch held. At most: no
    /// limit. Placeholders: none.
    pub autostart_held: [&'static str; 2],
    /// Shows: Settings, « Après un enregistrement », a list: its name, then its line under it.
    /// When: always. At most: one line for the name, the line wraps. Placeholders: none.
    pub keep_audio: [&'static str; 2],
    /// The settings' last group: Kitokey itself, as GNOME's preferences name it (D95).
    /// Shows: Settings, as a heading. When: always. At most: no limit. Placeholders: none.
    pub general_title: &'static str,
    /// Folded at the bottom of « Général », and its one switch: what a program said of a failure,
    /// shown on the page under the sentence (D29).
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, its heading; and a step's «
    /// Avancé » in Settings, « Modèles ». When: always. At most: one line. Placeholders: none.
    pub advanced_title: &'static str,
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, a switch: its name, then its
    /// line under it (D29). When: always. At most: one line for the name, the line wraps.
    /// Placeholders: none.
    pub developer: [&'static str; 2],
    /// Under a making not made, in developer mode: what its program said ({}).
    pub program_said: &'static str,
    /// A meeting's minutes, their three sections as the page names them (D11's second board),
    /// whatever titles the model wrote: `minutes/v1` holds them in this order.
    pub minutes_sections: [&'static str; 3],
    /// The actions' three cells, named for assistive technology alone (D46), and what a meeting
    /// left unsettled, as the summary writes it.
    pub actions_cells: [&'static str; 3],
    pub unsettled: &'static str,
    /// A model of this computer that holds the minutes' form, as Settings offer it ({}).
    /// Shows: Settings, « Modèles », the summary's step, a model of this computer in the list.
    /// When: it holds the summary's form. At most: one line. Placeholders: {} the model.
    pub model_recommended: &'static str,
    /// A meeting's player (D26): its button, both ways, and why a meeting has none.
    pub play: &'static str,
    pub pause: &'static str,
    pub no_player: &'static str,
    /// Its audio is there, and no track of it is one this version plays.
    pub not_playable: &'static str,
    /// A line of the minutes' time, as a screen reader says it (D75): what a click on it does.
    pub time_listen: &'static str,
    pub time_passage: &'static str,
    /// A track the window could not play; and the drawing as the playback's slider, named.
    pub not_played: &'static str,
    pub playback: &'static str,
    /// In developer mode: the logs' level, the logs' folder, a meeting's diagnostic copied.
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, a list: its name, then its
    /// line under it. When: the developer mode is on. At most: one line for the name, the line
    /// wraps. Placeholders: none.
    pub log_level: [&'static str; 2],
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, the logs' level, a choice.
    /// When: the developer mode is on. At most: one line. Placeholders: none.
    pub log_info: &'static str,
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, the logs' level, a choice.
    /// When: the developer mode is on. At most: one line. Placeholders: none.
    pub log_debug: &'static str,
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, a row: its name, then its
    /// line under it. When: the developer mode is on. At most: one line for the name, the line
    /// wraps. Placeholders: none.
    pub logs: [&'static str; 2],
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, the logs' row, a button.
    /// When: the developer mode is on. At most: one line. Placeholders: none.
    pub open_logs: &'static str,
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, under that button. When: the
    /// logs' folder could not be opened. At most: no limit. Placeholders: none.
    pub logs_not_opened: &'static str,
    pub copy_diagnostic: &'static str,
    pub diagnostic_copied: &'static str,
    pub diagnostic_empty: &'static str,
    pub diagnostic_not_copied: &'static str,
    /// The folder the sessions go to, and the button that opens the system's dialog for it.
    /// Shows: Settings, « Après un enregistrement », a row: its name, then its line under it. When:
    /// always. At most: one line for the name, the line wraps. Placeholders: none.
    pub sessions_folder: [&'static str; 2],
    /// Shows: Settings, « Après un enregistrement », the folder's button: the system's dialog.
    /// When: always. At most: one line. Placeholders: none.
    pub choose: &'static str,
    /// The recording bar, its icon in the sidebar's foot (D89), what it says (D90).
    pub small: &'static str,
    pub small_mic: &'static str,
    pub small_meeting: &'static str,
    pub small_note: &'static str,
    /// What the bar says of each side (D49, D90): silent two minutes, never as an error; a
    /// microphone that sends nothing, as the notifications say it; a side with no device.
    pub bar_silent_me: &'static str,
    pub bar_silent_others: &'static str,
    pub small_nothing: &'static str,
    pub small_waiting_me: &'static str,
    pub small_waiting_others: &'static str,
    /// The reminder said where the user looks, the first seconds of a recording (D50).
    pub small_announce: &'static str,
    pub copy_announcement: &'static str,
    pub announcement_not_copied: &'static str,
    pub small_full: &'static str,
    /// The bar's options (D90): opened under its row, closed again.
    pub bar_more: &'static str,
    pub bar_less: &'static str,
    pub bar_hide: &'static str,
    pub small_microphone: &'static str,
    pub small_microphones: &'static str,
    pub microphone_auto: &'static str,
    pub microphone_auto_says: &'static str,
    pub microphone_absent: &'static str,
    pub microphones_unread: &'static str,
    pub microphone_not_kept: &'static str,
    pub small_microphone_gone: &'static str,
    /// Recording, under the microphone (D90, board 6): the one recorded, a change for the next.
    pub bar_microphone_recorded: &'static str,
    pub bar_microphone_next: &'static str,
    /// The context typed before the start, in the row at rest (D91, board 5): its placeholder, its ✕,
    /// and under the row while one is kept, that it waits for the next recording.
    pub bar_context: &'static str,
    pub bar_context_clear: &'static str,
    pub bar_context_kept: &'static str,
    /// The meeting last stopped and its transcript under way (D92). Shows: the bar at rest, under its
    /// row, in a polite live region. When: after a stop, until the transcript ends. At most: about 60
    /// characters. Placeholders: {} the meeting, as a sentence opens on it: « La réunion de 17:31 ».
    pub bar_last_hearing: &'static str,
    /// The same, its summary under way (D92). Shows: the bar at rest, under its row. When: the
    /// transcript ended, the summary being written. At most: about 60 characters. Placeholders: {} the meeting.
    pub bar_last_summary: &'static str,
    /// The same, made (D92). Shows: the bar at rest, under its row, `bar_last_open` beside it. When: its
    /// last step written. At most: about 60 characters. Placeholders: {} the meeting.
    pub bar_last_ready: &'static str,
    /// The same, cancelled from its page (D92). Shows: the bar at rest, `bar_last_open` beside it. When:
    /// a step stopped by « Annuler ». At most: about 60 characters. Placeholders: {} the meeting.
    pub bar_last_stopped: &'static str,
    /// The same, a step failed (D92). Shows: the bar at rest, `bar_last_open` beside it. When: its last
    /// step failed. At most: no limit. Placeholders: {} the meeting, then {} the cause as its page says it.
    pub bar_last_failed: &'static str,
    /// The button beside the meeting last stopped (D92). Shows: the bar at rest. When: the meeting
    /// ready, stopped or failed; a press opens its page. At most: 12 characters. Placeholders: none.
    pub bar_last_open: &'static str,
    /// « Avancé » (D90, board 6; D91), closed by default, in the bar opened and in *Refaire…*: the
    /// meeting's own models, Settings' choice first, said as such — what it holds said on it.
    pub bar_advanced: &'static str,
    pub models_transcription: &'static str,
    pub models_minutes: &'static str,
    pub models_settings: &'static str,
    pub models_none: &'static str,
    pub models_own_note: &'static str,
    pub models_not_kept: &'static str,
    pub models_not_read: &'static str,
    /// A meeting's own model gone (D91): Settings' choice stands in, as Auto does for a microphone.
    pub models_gone_transcription: &'static str,
    pub models_gone_minutes: &'static str,
    /// A service the meeting chose, not used while it is said confidential (D60).
    pub models_unused_confidential: &'static str,
    /// Under the bar's row at rest, « Avancé » closed: the models chosen for the next recording, named
    /// short (D91) — one, or more joined by « · ».
    pub bar_models_kept: &'static str,
    pub bar_model_kept: &'static str,
    /// « Vérifier le son » (D88, board 7): its title, and its row in Settings.
    /// Shows: « Vérifier le son » (D88), its title, opened from Settings, « Général » or the bar;
    /// and its row there. When: always. At most: one line. Placeholders: none.
    pub check_sound: &'static str,
    /// Shows: Settings, « Général », the line under « Vérifier le son », and the bar's. When:
    /// always. At most: no limit. Placeholders: none.
    pub check_sound_says: &'static str,
    /// A source capture could not open; none named yet.
    pub check_unavailable: &'static str,
    /// The microphone listened to is a Bluetooth headset's: its call profile while the panel shows.
    pub check_bluetooth: &'static str,
    /// The test sound, and what was heard of it: on the meeting's side, or not, and what to do.
    pub check_play: &'static str,
    pub check_heard: &'static str,
    pub check_not_heard: &'static str,
    /// A call holds a microphone: no test sound, it would reach the call.
    pub check_call: &'static str,
    pub check_open_note: &'static str,
    pub check_close: &'static str,
    /// It could not listen; or a recording runs.
    pub check_failed: &'static str,
    pub check_recording: &'static str,
    /// Freeing space (D11's third board).
    pub free: &'static str,
    pub free_what: &'static str,
    pub free_audio: [&'static str; 2],
    pub free_whole: [&'static str; 2],
    pub free_heaviest: &'static str,
    pub free_older: &'static str,
    pub free_total: &'static str,
    pub free_one: &'static str,
    pub free_note: &'static str,
    pub free_audio_note: &'static str,
    pub free_do_audio: &'static str,
    pub free_do_whole: &'static str,
    pub free_busy: &'static str,
    pub free_done: &'static str,
    /// *Refaire…* on the minutes (D11's sixth board): the panel's title, question, note and gesture.
    pub redo: &'static str,
    pub redo_title: &'static str,
    pub redo_with: &'static str,
    pub redo_note: &'static str,
    /// Shows: the summary's redo panel, its one button that starts, naming the making and its model
    /// (D96): « Refaire le compte rendu avec Qwen3 14b ». When: a summary model is known, the meeting's,
    /// the one chosen in « Avancé » or Settings'; else `redo_title`. At most: wraps inside the panel.
    /// Placeholders: {} the model, as people say it.
    pub redo_minutes_do: &'static str,
    /// Shows: the transcript's redo panel, its one button that starts, naming its model (D96):
    /// « Refaire la transcription avec Whisper Large V3 Turbo ». When: a transcription model is known;
    /// else `redo_transcript_title`. At most: wraps inside the panel. Placeholders: {} the model, as
    /// people say it.
    pub redo_transcript_do: &'static str,
    pub on_computer: &'static str,
    pub redo_transcript_title: &'static str,
    pub redo_transcript_note: &'static str,
    /// What a meeting lacks, offered on its page; what is said once asked; where a step ran.
    pub make_minutes: &'static str,
    pub make_transcript: &'static str,
    /// Shows: a meeting's page, the button beside the summary's list of models, its transcript whole and
    /// no summary yet (D93): « Faire le compte rendu avec Qwen3 14b ». When: a model is chosen there or
    /// named by Settings; else `make_minutes`. At most: no limit. Placeholders: {} the model, as people say it.
    pub make_minutes_with: &'static str,
    /// Shows: a meeting's page, above the button that makes the summary again, the model that made the
    /// one shown (D93). When: another model is chosen in its line. At most: no limit. Placeholders: {}
    /// the model, as its line says it.
    pub minutes_made_with: &'static str,
    /// Shows: a meeting's page, above the button that makes the transcript again, the model that made
    /// the one shown (D93). When: another model is chosen in its line. At most: no limit.
    /// Placeholders: {} the model, as its line says it.
    pub transcript_made_with: &'static str,
    /// What is made in the background, said on the pages (D25): the minutes being written (by,
    /// for how long), waiting their turn ({} before), waiting for the user's agreement to send them
    /// ({} the host) with the way to give it, not written ({} the reason); on a list's row.
    pub minutes_making: &'static str,
    pub minutes_waiting: &'static str,
    pub minutes_awaiting: &'static str,
    pub agree_open: &'static str,
    pub minutes_not_made: &'static str,
    pub try_again: &'static str,
    pub chip_making: &'static str,
    pub chip_waiting: &'static str,
    pub chip_awaiting: &'static str,
    pub chip_transcribing: &'static str,
    pub chip_failed: &'static str,
    /// Shows: a meeting's chip, in the list and on its page (#772). When: its minutes are kept and the
    /// last Refaire… of them failed. At most: one line. Placeholders: none.
    pub chip_redo_failed: &'static str,
    /// Shows: a meeting's page, the minutes' state, before the cause when there is one (#772). When:
    /// its minutes are kept and the last Refaire… of them failed. At most: no limit. Placeholders: none.
    pub minutes_redo_failed: &'static str,
    /// The same for the transcript: being made (by, for how long), not made ({} the reason).
    pub transcript_making: &'static str,
    pub transcript_not_made: &'static str,
    pub chip_transcript_failed: &'static str,
    /// Who writes the minutes chosen on the settings' page, not yet saved: who still does ({}).
    /// Shows: Settings, « Modèles », the summary's step, under « Enregistrer ». When: another
    /// choice is made, not saved yet. At most: no limit. Placeholders: {} who writes it today.
    pub minutes_unsaved: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, under « Enregistrer ». When: a choice is
    /// made, none saved yet. At most: no limit. Placeholders: none.
    pub minutes_unsaved_none: &'static str,
    /// A transcript waiting for the default Whisper model: offered in the menu, or coming.
    /// Shows: a meeting's page, the transcript's card. When: the transcript waits for Kitokey's
    /// Whisper model. At most: no limit. Placeholders: none.
    pub transcript_model_needed: &'static str,
    /// Shows: a meeting's page, the transcript's card. When: the transcript waits for Kitokey's
    /// Whisper model, being downloaded. At most: no limit. Placeholders: none.
    pub transcript_model_coming: &'static str,
    /// The button that ends what is being made of a meeting, or takes it out of the line (D92).
    /// Shows: a button on a meeting's page, beside what is being made. When: its transcript or minutes
    /// are being made, or waiting in line. At most: no limit. Placeholders: none.
    pub making_cancel: &'static str,
    /// « Annuler » refused — its meeting gone meanwhile (D92). Shows: a meeting's page, beside the
    /// making's line, until the next gesture on it. When: the shell refused to end or take out the
    /// making. At most: no limit. Placeholders: {} why it was refused.
    pub making_cancel_failed: &'static str,
    /// What is left of a meeting's making, in minutes, from the speeds measured here (D92). Shows: in a
    /// meeting's page bar, after the step. When: both steps' speeds are known and more than thirty
    /// seconds are left. At most: « about 100 min ». Placeholders: {} the minutes, rounded.
    pub making_left: &'static str,
    /// Said on its page once the user cancelled its transcript, or its minutes: what was there stays.
    /// Shows: a meeting's page, where its transcript is. When: the user cancelled its transcript.
    /// At most: no limit. Placeholders: none.
    pub transcript_stopped: &'static str,
    /// Shows: a meeting's page, where its minutes are. When: the user cancelled its minutes.
    /// At most: no limit. Placeholders: none.
    pub minutes_stopped: &'static str,
    /// The header's button, at rest and while a recording runs.
    /// Shows: the button that records: in the window's header, its tooltip too; in the empty
    /// window; in « Vérifier le son »; and the recording bar's, as its name for a screen reader.
    /// When: no recording runs. At most: one line. Placeholders: none.
    pub record_start: &'static str,
    pub record_stop: &'static str,
    /// The agreement before a meeting first leaves for a host (D11's fifth board): what leaves,
    /// what stays, to whom; the user's word, once or for this service; on this computer instead.
    pub agree_eyebrow: &'static str,
    pub agree_title: &'static str,
    pub agree_lead: &'static str,
    pub agree_leaves: [&'static str; 2],
    pub agree_context: [&'static str; 2],
    pub agree_stays: [&'static str; 2],
    pub agree_to: &'static str,
    pub agree_model: &'static str,
    pub agree_remember: &'static str,
    pub agree_send: &'static str,
    pub agree_not: &'static str,
    pub agree_local: &'static str,
    pub agree_local_write: &'static str,
    /// The global shortcut: its keys as granted, none granted, where it changes, the button there.
    /// Shows: Settings, « Général », a row: its name, then its line under it. When: always. At
    /// most: one line for the name, the line wraps. Placeholders: none.
    pub shortcut: [&'static str; 2],
    /// Shows: Settings, « Général », the shortcut's keys, in their place. When: the desktop granted
    /// none. At most: one line. Placeholders: none.
    pub shortcut_none: &'static str,
    /// Shows: Settings, « Général », under the shortcut's keys. When: always. At most: no limit.
    /// Placeholders: none.
    pub shortcut_where: &'static str,
    /// Shows: Settings, « Général », a button: the desktop's keyboard settings. When: the desktop
    /// has them. At most: one line. Placeholders: none.
    pub shortcut_open: &'static str,
    /// *Tester*: the button, what it sends and to whom, and what it found.
    /// Shows: Settings, « Modèles », the summary's step, a button: the model asked one sentence.
    /// When: a model is saved. At most: one line. Placeholders: none.
    pub test: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, beside « Tester » while it asks. When: the
    /// model is a service's. At most: no limit. Placeholders: {} the host.
    pub test_sends: &'static str,
    /// Shows: Settings, « Modèles », beside « Tester » or a key's test. When: while it asks. At
    /// most: one line. Placeholders: none.
    pub testing: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, beside « Tester ». When: the model
    /// answered. At most: one line. Placeholders: none.
    pub test_answered: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, beside « Tester », said by the shell.
    /// When: the model did not answer. At most: no limit. Placeholders: {} the failure's sentence.
    pub test_failed: &'static str,
    /// Shows: Settings, « Après un enregistrement », that list's first choice. When: always. At
    /// most: one line. Placeholders: none.
    pub keep_always: &'static str,
    /// A number of days the audio is kept.
    /// Shows: Settings, « Après un enregistrement », that list's choices. When: always. At most:
    /// one line. Placeholders: {} a number of days.
    pub keep_days: &'static str,
    /// The settings file could not be read: why.
    /// A change could not be saved: why.
    pub not_saved: &'static str,
    /// Who writes the minutes was saved, but the keyring did not keep the key: why.
    /// Shows: Settings, « Modèles », under « Enregistrer ». When: the choice was saved, the keyring
    /// did not keep the key. At most: no limit. Placeholders: {} why.
    pub key_not_kept: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, its label. When: always. At most: one
    /// line. Placeholders: none.
    pub minutes_step: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, folded « Avancé »: a field's label. When:
    /// an address of one's own. At most: one line. Placeholders: none.
    pub address: &'static str,
    /// Shows: Settings, « Modèles », a model's list or field, its label. When: a step's model is
    /// chosen. At most: one line. Placeholders: none.
    pub model_name: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, a field's label. When: a service writes
    /// it. At most: one line. Placeholders: none.
    pub key: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, the key field's placeholder. When: a key
    /// is kept in the keyring. At most: one line. Placeholders: none.
    pub key_kept: &'static str,
    /// Settings, « Modèles », the summary's first step (D95): the choice beside « Sur cet ordinateur ».
    /// Shows: Settings, « Modèles », the summary's step, its first choice, beside « Sur cet
    /// ordinateur » (D95). When: always. At most: one line. Placeholders: none.
    pub where_service: &'static str,
    /// The label of the list of the services offered by name (D95).
    /// Shows: Settings, « Modèles », the label of the list of services named, in the
    /// transcription's and the summary's steps (D95). When: a service is chosen. At most: one line.
    /// Placeholders: none.
    pub service_name: &'static str,
    /// The button that asks the service its models with the key typed: the key tested at once (D95).
    /// Shows: Settings, « Modèles », the button beside a service's key: it asks the service its
    /// models with that key (D95). When: a service is chosen. At most: one line. Placeholders:
    /// none.
    pub key_test: &'static str,
    /// Shows: Settings, « Modèles », the button that saves a step's choice. When: always. At most:
    /// one line. Placeholders: none.
    pub save: &'static str,
    /// Shows: Settings, « Modèles », the summary's step, the line under its choice. When: a service
    /// writes the summary. At most: one line. Placeholders: none.
    pub where_online: &'static str,
    /// The model saved, which this computer's Ollama does not have ({} the model).
    /// Shows: Settings, « Modèles », the summary's step, under its choice. When: the model saved is
    /// not in Ollama. At most: no limit. Placeholders: {} the model.
    pub ollama_lacks: &'static str,
    /// This computer's Ollama, saved, not answering — or answering with no model.
    /// Shows: Settings, « Modèles », the summary's step, under its choice. When: Ollama is saved
    /// and does not answer, or has no model. At most: no limit. Placeholders: none.
    pub ollama_silent: &'static str,
    /// The first run's assistant (D58): its question, who writes the minutes, the setting said
    /// plainly, and its way back from Settings. Each answer is its name and its line.
    /// Shows: the first run's assistant (D58), above its question. When: always. At most: one line.
    /// Placeholders: none.
    pub welcome_title: &'static str,
    /// Shows: the first run's assistant (D58), its first question, a heading. When: its first step.
    /// At most: one line. Placeholders: none.
    pub welcome_question: &'static str,
    /// Shows: the first run's assistant (D58), under its first question. When: its first step. At
    /// most: no limit. Placeholders: none.
    pub welcome_examples: &'static str,
    /// Shows: the first run's assistant (D58), an answer to its first question: its name, then its
    /// line under it. When: its first step. At most: one line each. Placeholders: none.
    pub welcome_often: [&'static str; 2],
    /// Shows: the first run's assistant (D58), an answer to its first question: its name, then its
    /// line under it. When: its first step. At most: one line each. Placeholders: none.
    pub welcome_rarely: [&'static str; 2],
    /// Shows: the first run's assistant (D58), an answer to its first question: its name, then its
    /// line under it. When: its first step. At most: one line each. Placeholders: none.
    pub welcome_unsure: [&'static str; 2],
    /// Shows: the first run's assistant (D58), a link under its first step. When: the first run. At
    /// most: one line. Placeholders: none.
    pub welcome_skip: &'static str,
    /// Shows: the first run's assistant (D58), a link under its first step. When: opened again from
    /// Settings, « Général ». At most: one line. Placeholders: none.
    pub welcome_cancel: &'static str,
    /// Shows: the first run's assistant (D58), its button to the next step. When: its first two
    /// steps. At most: one line. Placeholders: none.
    pub welcome_next: &'static str,
    /// Shows: the first run's assistant (D58), its link to the step before. When: its second and
    /// last steps. At most: one line. Placeholders: none.
    pub welcome_back: &'static str,
    /// Shows: the first run's assistant (D58), its second question, a heading. When: its second
    /// step, after « Rarement ou jamais ». At most: one line. Placeholders: none.
    pub welcome_writer: &'static str,
    /// Shows: the first run's assistant (D58), an answer to its second question: its name, then its
    /// line under it. When: its second step. At most: one line each. Placeholders: none.
    pub welcome_here: [&'static str; 2],
    /// Shows: the first run's assistant (D58), an answer to its second question: its name, then its
    /// line under it. When: its second step. At most: one line each. Placeholders: none.
    pub welcome_service: [&'static str; 2],
    /// Shows: the first run's assistant (D58), its last step's heading: its name, then its line
    /// under it. When: everything on this computer. At most: one line each. Placeholders: none.
    pub welcome_private: [&'static str; 2],
    /// Shows: the first run's assistant (D58), its last step's heading: its name, then its line
    /// under it. When: a service writes the summary. At most: one line each. Placeholders: none.
    pub welcome_connected: [&'static str; 2],
    /// The summary's rows: who transcribes, who writes the minutes, the reminder.
    /// Shows: the first run's assistant (D58), its last step: the names of its three rows. When:
    /// its last step. At most: one line each. Placeholders: none.
    pub welcome_rows: [&'static str; 3],
    /// Shows: the first run's assistant (D58), a row's value; and Settings, « Modèles », the
    /// summary's step, its first choice. When: a step is on this computer. At most: one line.
    /// Placeholders: none.
    pub welcome_on_this_computer: &'static str,
    /// Online, no service chosen yet; one chosen already.
    /// Shows: the first run's assistant (D58), the summary's row. When: a service is to be chosen.
    /// At most: one line. Placeholders: none.
    pub welcome_online: &'static str,
    /// Shows: the first run's assistant (D58), the summary's row. When: a service is chosen
    /// already. At most: one line. Placeholders: none.
    pub welcome_online_chosen: &'static str,
    /// Ollama and its model ({} the model as people say it).
    /// Shows: the first run's assistant (D58), the summary's row. When: Ollama writes it. At most:
    /// one line. Placeholders: {} the model, as people say it.
    pub welcome_ollama_by: &'static str,
    /// Shows: the first run's assistant (D58), the reminder's row. When: the reminder is on. At
    /// most: one line. Placeholders: none.
    pub welcome_reminder_on: &'static str,
    /// Shows: the first run's assistant (D58), the reminder's row. When: the reminder is off. At
    /// most: one line. Placeholders: none.
    pub welcome_reminder_off: &'static str,
    pub welcome_ollama_silent: &'static str,
    /// Ollama answers without the model ({} the model).
    /// Shows: the first run's assistant (D58), its last step, under its rows. When: Ollama answers
    /// without the summary's model. At most: no limit. Placeholders: {} the model.
    pub welcome_ollama_lacks: &'static str,
    /// The model's download ({} its size), and where from ({} the host).
    /// Shows: the first run's assistant (D58), its last step's button. When: Whisper's model is not
    /// here yet. At most: one line. Placeholders: {} its size.
    pub welcome_fetch: &'static str,
    /// Shows: the first run's assistant (D58), under that button. When: Whisper's model is not here
    /// yet. At most: no limit. Placeholders: {} the host.
    pub welcome_fetch_from: &'static str,
    /// Shows: the first run's assistant (D58), its last step's button. When: on this computer, no
    /// model left to fetch. At most: one line. Placeholders: none.
    pub welcome_done: &'static str,
    /// Shows: the first run's assistant (D58), its last step, a link. When: no model is left to
    /// fetch (`welcome_later_waits` else). At most: one line. Placeholders: none.
    pub welcome_later: &'static str,
    /// The welcome's « Later » when the transcription's model is still to be downloaded, saying what it
    /// leaves waiting (#781). Shows: the welcome's last screen, a link-styled button. When: the model is
    /// not on this computer and the welcome offers to download it. At most: about 50 characters. Placeholders: none.
    pub welcome_later_waits: &'static str,
    /// Shows: the first run's assistant (D58), its last step's button: to Settings' summary choice.
    /// When: a service writes the summary. At most: one line. Placeholders: none.
    pub welcome_choose: &'static str,
    /// Shows: Settings, « Général », the row that opens the first run's assistant (D58) again: its
    /// name, then its line under it. When: always. At most: one line for the name, the line wraps.
    /// Placeholders: none.
    pub welcome_again: [&'static str; 2],
    /// A meeting's context (D59): its button's value while none is written; the field's
    /// label, its example and why it helps; its count ({} the most, {} what is left — one, then
    /// more); another script than Latin or Cyrillic; why it is closed; what changed, the
    /// transcript alone or with the minutes, and the gesture that makes them again.
    pub context_add: &'static str,
    pub context_title: &'static str,
    pub context_label: &'static str,
    pub context_example: &'static str,
    pub context_why: &'static str,
    pub context_most: &'static str,
    pub context_left: [&'static str; 2],
    pub context_script: &'static str,
    pub context_making: &'static str,
    pub context_unfinished: &'static str,
    pub context_changed: [&'static str; 2],
    pub context_redo: &'static str,
    /// The language a meeting is spoken in (D65).
    pub language_label: &'static str,
    pub language_default: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, the language of the meetings, its
    /// first choice. When: Whisper or a service hears. At most: one line. Placeholders: none.
    pub language_detected: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a list's label. When: always. At
    /// most: one line. Placeholders: none.
    pub meetings_language: &'static str,
    pub language_not_saved: &'static str,
    pub rename_meeting: &'static str,
    pub name_meeting: &'static str,
    pub name_not_saved: &'static str,
    pub language_detected_short: &'static str,
    /// SpeechAnalyzer's language when the settings name none: the system's, never the app's (D64).
    /// Shows: Settings, « Modèles », the transcription's step, the language of the meetings, its
    /// first choice. When: Apple's SpeechAnalyzer hears (D64). At most: one line. Placeholders:
    /// none.
    pub language_as_system: &'static str,
    /// A meeting's own language SpeechAnalyzer does not hear: the one it hears in, said (D65).
    pub language_heard_as: &'static str,
    /// The language Kitokey speaks (D64): its row in Settings, and the system's, its default.
    /// Shows: Settings, « Général », a list: its name, then its line under it (D64). When: always.
    /// At most: one line for the name, the line wraps. Placeholders: none.
    pub app_language: [&'static str; 2],
    /// Shows: Settings, « Général », that list's first choice. When: always. At most: one line.
    /// Placeholders: none.
    pub app_language_system: &'static str,
    /// Shows: Settings, « Général », the appearance's row (D98): its name, then what it changes. When:
    /// always. At most: one line each. Placeholders: none.
    pub appearance: [&'static str; 2],
    /// Shows: Settings, « Général », the appearance's list, in that order (D98). When: always. At most:
    /// 18 characters each. Placeholders: none.
    pub appearance_choices: [&'static str; 3],
    /// A transcript's line corrected by whoever listens (D62).
    pub correct_line: &'static str,
    pub corrected: &'static str,
    pub correction_empty: &'static str,
    pub correction_failed: &'static str,
    pub corrected_since: &'static str,
    pub redo_corrections: &'static str,
    /// The minutes copied in one click, said in place (D45).
    pub copy_minutes: &'static str,
    pub minutes_copied: &'static str,
    pub minutes_not_copied: &'static str,
    pub redo_loses_corrections: &'static str,
    pub context_not_saved: &'static str,
    /// A meeting said confidential (D60): the switch's name and its words; the list's word beside
    /// the lock; what had left before ({} what, then each: the sound, the transcript, {} a host);
    /// the switch not saved; the minutes waiting for Ollama holding no model, and the fetch of {}.
    pub confidential_name: &'static str,
    pub confidential_says: &'static str,
    pub confidential_chip: &'static str,
    pub confidential_left: &'static str,
    pub confidential_left_sound: &'static str,
    pub confidential_left_transcript: &'static str,
    pub confidential_not_saved: &'static str,
    pub confidential_ollama_empty: &'static str,
    pub confidential_fetch: &'static str,
}

pub fn window(language: Language) -> Window {
    match language {
        Language::English => Window {
            locale: "en",
            units: ["B", "kB", "MB", "GB"],
            sections: "Navigation",
            meetings: "Meetings",
            weight: "{} meetings",
            weight_one: "{} meeting",
            this_week: "This week",
            last_week: "Last week",
            earlier_in: "Earlier in {}",
            undated: "No date",
            minutes: "Summary",
            transcript: "Transcript",
            audio: "Audio · {}",
            audio_freed: "Audio deleted",
            audio_weighs: "audio {}",
            recording: "Recording",
            interrupted: "Interrupted",
            none: "No meeting yet.",
            none_folder: "Meetings are saved in {}.",
            none_do: "To begin, click Start recording, press the keyboard shortcut, or use the Kitokey icon.",
            list_unread: "Can’t read the meetings in {}.",
            list_unread_do: "Make sure the drive is connected and you have access to this folder, then try again.",
            settings_banner: "Settings cannot be read: your meetings are not shown.",
            settings_banner_kept: "Settings cannot be read: the last ones read stay in use, and no change is saved until the file is fixed.",
            again_refused: "Couldn’t try again: {}",
            fold_not_kept: "The sidebar position won’t be remembered: {}",
            agreement_refused: "Couldn’t send this meeting: {}",
            meeting_not_opened: "Couldn’t open this meeting: {}",
            page_not_shown: "Couldn’t show this page. Try again.",
            context_not_read: "Couldn’t reload the context: {}",
            record_starting: "Starting…",
            record_stopping: "Stopping…",
            redo_cancel: "Cancel",
            saved: "Saved",
            small_not_confidential: "This meeting isn’t confidential. Open it to mark it as confidential.",
            trial_over: "Your trial has ended. Your meetings stay yours: you can still read, play and export them.",
            buy: "Buy Kitokey",
            trial: "Free trial",
            find_meetings: "Search meetings",
            fold: "Hide sidebar",
            unfold: "Show sidebar",
            none_found: "No meetings found.",
            back: "Meetings",
            open_folder: "Open folder",
            trash_meeting: "Move to trash",
            trash_ask: "Move this meeting to the trash?",
            trash_holds: "It holds {}.",
            trash_parts: ["its audio", "its transcript", "its summary"],
            trash_restore: "You can restore it from your system’s trash.",
            trash_cancel: "Cancel",
            trash_confirm: "Move to trash",
            trash_refused: "Not moved to the trash: {}",
            more_actions: "More actions",
            labelled: "{}:",
            minutes_title: "Summary",
            transcript_title: "Transcript",
            search: "Search the transcript",
            written_by: "Written by {}",
            written_via: ", via {}",
            at: "at",
            heard_by: "Transcribed by {} on this computer",
            transcribed_on: "Transcribed on {}",
            minutes_of: "Summary of {}",
            made_in: "in {}",
            here: "You",
            remote: "Others",
            voice: "voice {}",
            here_described: "You — your microphone",
            remote_described: "Others — the call’s audio",
            who_spoke: "Who spoke when: you on top, others below",
            no_transcript: "No transcript yet.",
            no_minutes: "No summary yet.",
            being_made: "Still transcribing — more is on its way.",
            unreadable: "This version of Kitokey can’t read this file.",
            unread: "{} unreadable line(s) skipped.",
            no_match: "No results.",
            settings: "Settings",
            after_title: "After a recording",
            after_nothing: ["Do nothing", "The audio is kept. Transcribe it whenever you like."],
            after_transcript: ["Transcribe", "On this computer, offline."],
            after_minutes: [
                "Transcribe, then summarize",
                "One after the other. You’ll get a notification when the summary is ready.",
            ],
            models_title: "Models",
            models_intro: "For each step, choose what does the work: a model on this computer, an online service, or your own server.",
            models_note: "“On this computer” lists the models installed. API keys are stored in your system keychain, never in a file. Nothing leaves this computer without your consent: a transcription service receives your meetings’ audio and context; a summary service only their text and context.",
            model_state: "Transcription model",
            model_ready: "Ready",
            model_here: "Installed · {} · works offline",
            model_missing: "Not downloaded yet.",
            model_named_missing: "Not found on this computer. Put the file back or choose another model.",
            model_coming: "Downloading…",
            model_offer: "Download the model ({}, {})",
            offered_title: "Models to download",
            offered_fetch: "Download",
            offered_far: "Downloading… {}%",
            model_chosen_missing: "{} — not downloaded yet",
            offered_ollama_size: "about 5 GB",
            offered_ollama_good: "For summaries, on this computer, with Ollama",
            offered_needs_ollama: "Requires Ollama (free, at ollama.com)",
            offered_refresh: "Refresh list ({})",
            offered_refresh_failed: "Couldn’t refresh the list. The models above are still available.",
            speech_option: "Apple SpeechAnalyzer, on this Mac",
            speech_missing: "Apple’s model not downloaded yet",
            speech_fetch: "Download Apple’s model (macOS fetches it from Apple)",
            speech_fetching: "macOS is downloading Apple’s model…",
            speech_failed: "Couldn’t download Apple’s model. Try again later; Whisper is still available.",
            service_option: "A service I name",
            service_address: "Service address",
            service_model: "Model",
            service_key: "API key (if required)",
            service_agree: "I agree to send my meetings’ audio to {}",
            service_save: "Save",
            service_test: "Test",
            service_tested: "It works: a test clip was transcribed with timestamps.",
            service_saved: "Saved. {} will transcribe your next meetings.",
            service_saved_here: "Saved: this computer hears your next meetings.",
            service_agree_first: "Not saved: first agree to send the audio to {}.",
            service_address_bad: "Not saved: the address must start with https:// or http://, for example https://api.groq.com/openai/v1.",
            service_model_named: "Not saved: enter the model name, for example whisper-large-v3.",
            installed: "installed, {}",
            keep_other: "Other duration…",
            keep_not_days: "“{}” isn’t a valid number of days (1 to 36,500)",
            builtin: "built in",
            home: "Home folder",
            transcription: "Transcription",
            on_this_computer: "On this computer — nothing is sent",
            reminder: [
                "Remind me to tell participants",
                "When a recording starts, a reminder prompts you to announce it.",
            ],
            offer: [
                "When an app uses the microphone",
                "Usually a call starting. Kitokey never records without you knowing.",
            ],
            offer_choices: ["Offer to record", "Do nothing"],
            autostart: [
                "Open Kitokey at login",
                "Quietly, with no window — so Kitokey still offers to record your calls after a restart.",
            ],
            autostart_held: [
                "Turned off in Windows: turn it back on in Settings › Apps › Startup.",
                "Set by your organisation: it cannot be changed here.",
            ],
            general_title: "General",
            advanced_title: "Advanced",
            developer: ["Developer mode", "Shows technical details under each error, to help troubleshoot."],
            program_said: "Technical details: {}",
            minutes_sections: ["Highlights", "Actions", "Key points"],
            actions_cells: ["Who", "What", "When"],
            unsettled: "not specified",
            model_recommended: "{} — recommended",
            play: "Play",
            pause: "Pause",
            time_listen: "Play from {}",
            time_passage: "Go to {} in the transcript",
            not_playable: "This audio can’t be played in Kitokey.",
            not_played: "Can’t play this meeting here. Open the folder to listen in another player.",
            playback: "Playback position",
            no_player: "The audio was deleted to free up space. The transcript and summary remain.",
            log_level: ["Log level", "Applies the next time Kitokey starts."],
            log_info: "Normal",
            log_debug: "Detailed",
            logs: ["Logs", "One file per day, kept 14 days. No meeting content."],
            open_logs: "Open logs",
            logs_not_opened: "Couldn’t open the logs folder.",
            copy_diagnostic: "Copy a diagnostic",
            diagnostic_copied: "Diagnostic copied",
            diagnostic_empty: "No logs for this meeting",
            diagnostic_not_copied: "Couldn’t copy",
            sessions_folder: [
                "Meetings folder",
                "New meetings are saved here. Existing ones stay in the old folder: Kitokey only shows and cleans up this one.",
            ],
            choose: "Choose…",
            small: "Recording bar",
            small_mic: "Your microphone",
            small_meeting: "The meeting’s sound",
            small_note: "When you stop, transcription starts on its own. You’ll get a notification.",
            bar_silent_me: "Your microphone: nothing for 2 min — the recording goes on.",
            bar_silent_others: "The meeting: nothing for 2 min — the recording goes on.",
            small_nothing: "Nothing comes from it: check it is not muted on the device itself, and that it is ready.",
            small_waiting_me: "Your microphone isn’t being recorded right now.",
            small_waiting_others: "Meeting audio isn’t being recorded right now.",
            small_announce: "Let participants know this meeting is being recorded.",
            copy_announcement: "Copy an announcement",
            announcement_not_copied: "Couldn’t copy.",
            small_full: "Open full window",
            bar_more: "More options",
            bar_less: "Fewer options",
            bar_hide: "Hide bar",
            small_microphone: "Microphone: {}",
            small_microphones: "Microphone for the next recording",
            microphone_auto: "Auto",
            microphone_auto_says: "The one your call uses, otherwise your computer’s.",
            microphone_absent: "Unplugged for now — Auto takes over.",
            microphones_unread: "Can’t list microphones right now. Auto still works — try again in a moment.",
            microphone_not_kept: "Your microphone choice wasn’t saved. Choose it again.",
            small_microphone_gone: "The microphone you chose is unplugged — Auto takes over.",
            bar_microphone_recorded: "Recording from {}. Changes apply to the next recording.",
            bar_microphone_next: "Changes apply to the next recording.",
            bar_context: "Topic, participants…",
            bar_context_clear: "Clear context",
            bar_context_kept: "Kept for the next recording.",
            bar_last_hearing: "{}: transcribing",
            bar_last_summary: "{}: summary being written",
            bar_last_ready: "{}: ready",
            bar_last_stopped: "{}: stopped",
            bar_last_failed: "{}: {}",
            bar_last_open: "Open",
            bar_advanced: "Advanced — models",
            models_transcription: "Transcription model",
            models_minutes: "Summary model",
            models_settings: "From Settings: {}",
            models_none: "no model chosen",
            models_own_note: "For this meeting only; Settings stay the same.",
            models_not_kept: "Choice not saved: {}",
            models_not_read: "Couldn’t load the models: {}",
            models_gone_transcription: "The transcription model chosen for this meeting is no longer installed — the one in Settings will be used.",
            models_gone_minutes: "The summary service chosen for this meeting is no longer set up — the one in Settings will be used.",
            models_unused_confidential: "Confidential meeting: the online service isn’t used — everything stays on this computer.",
            bar_models_kept: "Models: {}",
            bar_model_kept: "Model: {}",
            check_sound: "Check the sound",
            check_sound_says: "Test your microphone and meeting audio before recording.",
            check_unavailable: "Unavailable",
            check_bluetooth: "While this panel is open, your Bluetooth headset switches to call mode (lower sound quality). It goes back to normal when you close it.",
            check_play: "Play a test sound",
            check_heard: "Kitokey can hear this computer’s sound.",
            check_not_heard: "Kitokey didn’t hear the test sound. Make sure sound is playing from this computer, not another device, then try again.",
            check_call: "A call is using the microphone, so no test sound — the call would hear it. Have someone speak in the call and watch the meter.",
            check_open_note: "The microphone stays on while this panel is open.",
            check_close: "Close",
            check_failed: "Kitokey can’t listen to your microphone or the meeting audio right now. Close this panel and open it again.",
            check_recording: "A recording is in progress — its meters are in the recording bar.",
            free: "Free up space",
            free_what: "What do you want to remove?",
            free_audio: [
                "Audio only",
                "Summaries and transcripts stay readable. Audio takes up almost all the space.",
            ],
            free_whole: ["Whole meeting", "Audio, transcript and summary — nothing is kept."],
            free_heaviest: "Largest first",
            free_older: "Select those older than 30 days",
            free_total: "freed · {} meetings",
            free_one: "freed · 1 meeting",
            free_note: "Nothing is deleted permanently: everything goes to the trash, where you can restore it.",
            free_audio_note: "Without the audio, the meeting can’t be transcribed again, even with a better model.",
            free_do_audio: "Move audio to trash",
            free_do_whole: "Move meetings to trash",
            free_busy: "in progress",
            free_done: "Done — moved to the trash.",
            redo: "Redo…",
            redo_title: "Write the summary again",
            redo_with: "Uses this meeting’s models (choose them under “Advanced”).",
            redo_note: "The current summary stays in place until the new one is ready, then goes to the trash.",
            redo_minutes_do: "Write the summary again with {}",
            redo_transcript_do: "Write the transcript again with {}",
            on_computer: "on this computer",
            redo_transcript_title: "Write the transcript again",
            redo_transcript_note: "The current transcript stays until the new one is finished, then goes to the trash. If something goes wrong, it’s kept.",
            make_minutes: "Write summary",
            make_transcript: "Transcribe",
            make_minutes_with: "Write summary with {}",
            minutes_made_with: "This summary was written with {}.",
            transcript_made_with: "This transcript was made with {}.",
            minutes_making: "Writing summary",
            minutes_waiting: "Queued · {} ahead",
            minutes_awaiting: "Waiting for your OK to send to {}",
            agree_open: "Review",
            minutes_not_made: "The summary couldn’t be written. The transcript is unchanged.",
            try_again: "Try again",
            chip_making: "Summary…",
            chip_waiting: "Waiting",
            chip_awaiting: "Needs OK",
            chip_transcribing: "Transcript…",
            chip_failed: "Summary failed",
            chip_redo_failed: "Redo failed",
            minutes_redo_failed: "The last redo failed. The summary shown is the one before it.",
            transcript_making: "Transcribing",
            transcript_not_made: "The transcript couldn’t be made. The audio is safe.",
            chip_transcript_failed: "Transcript failed",
            minutes_unsaved: "Not saved — {} is still used.",
            minutes_unsaved_none: "Not saved — no summaries will be written until you save.",
            transcript_model_needed: "Waiting for the transcription model.",
            transcript_model_coming: "Waiting for the transcription model (downloading).",
            making_cancel: "Cancel",
            making_cancel_failed: "Kitokey could not cancel: {}",
            making_left: "about {} min",
            transcript_stopped: "Transcription stopped. You can start it again, with another model if you like.",
            minutes_stopped: "Minutes stopped. You can start them again, with another model if you like.",
            record_start: "Start recording",
            record_stop: "Stop recording",
            agree_eyebrow: "First online summary",
            agree_title: "The text of “{}” is about to leave your computer",
            agree_lead: "To write the summary, Kitokey sends the transcript to the service you chose. Here’s exactly what’s sent.",
            agree_leaves: ["Sent: the transcript", "What everyone said — {} minutes, about {} words."],
            agree_context: ["Sent: the context you added", "About {} words."],
            agree_stays: ["Stays here: the audio", "No audio is ever sent."],
            agree_to: "Recipient",
            agree_model: "Model",
            agree_remember: "Don’t ask again for this service",
            agree_send: "Send the transcript",
            agree_not: "Don’t send",
            agree_local: "Write it on this computer instead",
            agree_local_write: "Write here",
            shortcut: ["Keyboard shortcut", "Starts or stops a recording from anywhere."],
            shortcut_none: "No shortcut set",
            shortcut_where: "You can change it in your system’s keyboard settings.",
            shortcut_open: "Keyboard settings",
            test: "Test",
            test_sends: "The test sends a short sentence to {} — nothing from your meetings.",
            testing: "Testing…",
            test_answered: "It works: the model responds.",
            test_failed: "The model isn’t responding: {}",
            keep_audio: [
                "Keep audio",
                "After this, the audio goes to the trash; the transcript and summary stay.",
            ],
            keep_always: "Always",
            keep_days: "{} days",
            not_saved: "Not saved: {}",
            key_not_kept: "Choice saved, but the key couldn’t be stored in your system keychain: {}",
            minutes_step: "Summary",
            address: "Address (OpenAI-compatible)",
            model_name: "Model",
            key: "API key",
            key_kept: "Key saved — enter a new one to replace it",
            where_service: "A service",
            service_name: "Service",
            key_test: "Check the key",
            save: "Save",
            where_online: "Online — the transcript text is sent",
            ollama_lacks: "Ollama has no model named {} on this computer. Pick one from the list or download it with Ollama.",
            ollama_silent: "Ollama isn’t responding or has no models. Start Ollama, then download a model.",
            welcome_title: "Let’s set up Kitokey with one question.",
            welcome_question: "Do your meetings cover confidential topics?",
            welcome_examples: "Clients, HR, health, legal, unpublished figures…",
            welcome_often: ["Yes, often", "Nothing must leave this computer."],
            welcome_rarely: ["Rarely or never", "I’m fine sending a meeting’s text to a service I choose."],
            welcome_unsure: ["I’m not sure", "Kitokey will pick the most private option."],
            welcome_skip: "Skip (keep everything on this computer)",
            welcome_cancel: "Cancel",
            welcome_next: "Next",
            welcome_back: "Back",
            welcome_writer: "Who writes the summary?",
            welcome_here: ["This computer", "Nothing is sent. Requires Ollama (free); a bit slower."],
            welcome_service: [
                "An online service",
                "Faster and more accurate. Only the meeting’s text is sent, never the audio. You’ll pick the service in Settings, and you’ll be asked before the first send.",
            ],
            welcome_private: ["Private mode", "Your meetings never leave this computer."],
            welcome_connected: [
                "Connected mode",
                "Summaries are written by an online service; transcription stays on this computer.",
            ],
            welcome_rows: ["Transcription", "Summary", "Reminder"],
            welcome_on_this_computer: "On this computer",
            welcome_online: "An online service (to choose)",
            welcome_online_chosen: "An online service",
            welcome_ollama_by: "Ollama · {}",
            welcome_reminder_on: "Remind me to tell participants at the start",
            welcome_reminder_off: "Off (you can turn it on in Settings)",
            welcome_ollama_silent: "Ollama isn’t responding on this computer, so the summary will wait. Ollama is free at ollama.com.",
            welcome_ollama_lacks: "Ollama doesn’t have {} yet, so the summary will wait. Download it in one click in Settings.",
            welcome_fetch: "Download the model ({})",
            welcome_fetch_from: "From {}, verified before use.",
            welcome_done: "Finish",
            welcome_later: "Later",
            welcome_later_waits: "Later — your meetings will wait for the model",
            welcome_choose: "Choose my service",
            welcome_again: [
                "Run setup again",
                "Goes back through the first-launch questions with your current choices. Nothing changes until the last step.",
            ],
            context_add: "Add context",
            context_title: "Context",
            context_label: "What’s the meeting about? Who’s attending?",
            context_example: "E.g. Q4 budget with Inès Durand and the purchasing team",
            context_why: "Helps the transcript spell names and terms correctly, and the summary assign each action to the right person.",
            context_most: "{} characters max",
            context_left: ["{} left", "{} left"],
            context_script: "In this script, the context may be cut short or refused by some services.",
            context_making: "You can’t edit the context while the transcript or summary is being made.",
            context_unfinished: "The transcript isn’t finished. Click Try again to finish it before editing the context.",
            context_changed: [
                "The transcript was made with a different context.",
                "The transcript and summary were made with a different context.",
            ],
            context_redo: "Redo with this context",
            language_label: "Spoken language",
            language_default: "Same as Settings",
            language_detected: "Detect automatically",
            meetings_language: "Meeting language",
            language_not_saved: "Language not saved: {}",
            rename_meeting: "Rename this meeting",
            name_meeting: "Name this meeting",
            name_not_saved: "Name not saved: {}",
            language_detected_short: "detected",
            language_as_system: "System language",
            language_heard_as: "{} — SpeechAnalyzer transcribes in {}",
            app_language: [
                "Kitokey language",
                "Window, menu and notifications. System dialogs keep the system language.",
            ],
            app_language_system: "System language",
            appearance: [
                "Appearance",
                "Every window of Kitokey, the bar too; System follows the system’s light or dark.",
            ],
            appearance_choices: ["System", "Light", "Dark"],
            correct_line: "Edit the line at {}",
            corrected: "edited",
            correction_empty: "A line can’t be empty. Press Escape to cancel.",
            correction_failed: "Edit not saved: {}",
            corrected_since: "The transcript was edited after this summary was written.",
            redo_corrections: "Redo with edits",
            copy_minutes: "Copy summary",
            minutes_copied: "Copied",
            minutes_not_copied: "Couldn’t copy the summary.",
            redo_loses_corrections: "Your edits to this transcript will be lost.",
            context_not_saved: "Couldn’t save the context.",
            confidential_name: "Confidential meeting",
            confidential_says: "Nothing leaves this computer",
            confidential_chip: "Confidential",
            confidential_left: "Already sent earlier: {}.",
            confidential_left_sound: "the audio to {}",
            confidential_left_transcript: "the transcript to {}",
            confidential_not_saved: "Couldn’t mark the meeting as confidential. Try again.",
            confidential_ollama_empty: "Ollama has no model on this computer, so the summary will wait.",
            confidential_fetch: "Download {}",
        },
        Language::French => Window {
            locale: "fr",
            units: ["o", "ko", "Mo", "Go"],
            sections: "Navigation",
            meetings: "Réunions",
            weight: "{} réunions",
            weight_one: "{} réunion",
            this_week: "Cette semaine",
            last_week: "La semaine dernière",
            earlier_in: "Plus tôt en {}",
            undated: "Sans date",
            minutes: "Compte rendu",
            transcript: "Transcription",
            audio: "Audio · {}",
            audio_freed: "Audio supprimé",
            audio_weighs: "audio {}",
            recording: "Enregistrement",
            interrupted: "Interrompu",
            none: "Aucune réunion pour l’instant.",
            none_folder: "Les réunions sont enregistrées dans {}.",
            none_do: "Pour commencer, cliquez sur Démarrer un enregistrement, utilisez le raccourci clavier ou l’icône Kitokey.",
            list_unread: "Impossible de lire les réunions de {}.",
            list_unread_do: "Vérifiez que le disque est branché et que vous avez accès à ce dossier, puis réessayez.",
            settings_banner: "Les réglages ne peuvent pas être lus\u{a0}: vos réunions ne sont pas affichées.",
            settings_banner_kept: "Les réglages ne peuvent pas être lus\u{a0}: les derniers lus restent en usage, et aucun changement n’est enregistré tant que le fichier n’est pas corrigé.",
            again_refused: "Impossible de relancer\u{a0}: {}",
            fold_not_kept: "La position de la barre latérale ne sera pas mémorisée\u{a0}: {}",
            agreement_refused: "Impossible d’envoyer cette réunion\u{a0}: {}",
            meeting_not_opened: "Impossible d’ouvrir cette réunion\u{a0}: {}",
            page_not_shown: "Impossible d’afficher cette page. Réessayez.",
            context_not_read: "Impossible de relire le contexte\u{a0}: {}",
            record_starting: "Démarrage…",
            record_stopping: "Arrêt…",
            redo_cancel: "Annuler",
            saved: "Enregistré",
            small_not_confidential: "Cette réunion n’est pas confidentielle. Ouvrez-la pour la marquer comme telle.",
            trial_over: "Votre essai est terminé. Vos réunions restent à vous\u{a0}: vous pouvez toujours les lire, les écouter et les exporter.",
            buy: "Acheter Kitokey",
            trial: "Essai gratuit",
            find_meetings: "Rechercher",
            fold: "Masquer la barre latérale",
            unfold: "Afficher la barre latérale",
            none_found: "Aucune réunion trouvée.",
            back: "Réunions",
            open_folder: "Ouvrir le dossier",
            trash_meeting: "Mettre à la corbeille",
            trash_ask: "Mettre cette réunion à la corbeille\u{202f}?",
            trash_holds: "Elle contient {}.",
            trash_parts: ["son audio", "sa transcription", "son compte rendu"],
            trash_restore: "Vous pourrez la restaurer depuis la corbeille de votre système.",
            trash_cancel: "Annuler",
            trash_confirm: "Mettre à la corbeille",
            trash_refused: "Pas mise à la corbeille\u{a0}: {}",
            more_actions: "Plus d’actions",
            labelled: "{}\u{a0}:",
            minutes_title: "Compte rendu",
            transcript_title: "Transcription",
            search: "Rechercher dans le texte",
            written_by: "Rédigé par {}",
            written_via: ", via {}",
            at: "à",
            heard_by: "Transcrit par {}, sur cet ordinateur",
            transcribed_on: "Transcrit le {}",
            minutes_of: "Compte rendu du {}",
            made_in: "en {}",
            here: "Vous",
            remote: "Les autres",
            voice: "voix {}",
            here_described: "Vous — votre micro",
            remote_described: "Les autres — le son de l’appel",
            who_spoke: "Temps de parole au fil de la réunion\u{a0}: vous en haut, les autres en bas",
            no_transcript: "Pas encore de transcription.",
            no_minutes: "Pas encore de compte rendu.",
            being_made: "Transcription en cours\u{a0}: la suite arrive.",
            unreadable: "Cette version de Kitokey ne sait pas lire ce fichier.",
            unread: "{} ligne(s) illisible(s) ignorée(s).",
            no_match: "Aucun résultat.",
            settings: "Réglages",
            after_title: "Après un enregistrement",
            after_nothing: [
                "Ne rien faire",
                "L’audio est conservé. Vous lancez la transcription quand vous voulez.",
            ],
            after_transcript: ["Transcrire", "Sur cet ordinateur, sans connexion."],
            after_minutes: [
                "Transcrire, puis rédiger le compte rendu",
                "Tout s’enchaîne. Une notification vous prévient quand le compte rendu est prêt.",
            ],
            models_title: "Modèles",
            models_intro: "Pour chaque étape, choisissez qui fait le travail\u{a0}: un modèle sur cet ordinateur, un service en ligne ou votre propre serveur.",
            models_note: "«\u{a0}Sur cet ordinateur\u{a0}» liste les modèles installés. Les clés d’API sont rangées dans le trousseau du système, jamais dans un fichier. Rien ne quitte cet ordinateur sans votre accord\u{a0}: un service de transcription reçoit le son et le contexte de vos réunions\u{202f}; un service de compte rendu, seulement leur texte et leur contexte.",
            model_state: "Modèle de transcription",
            model_ready: "Prêt",
            model_here: "Installé · {} · fonctionne hors connexion",
            model_missing: "Pas encore téléchargé.",
            model_named_missing: "Introuvable sur cet ordinateur. Remettez le fichier à sa place ou choisissez un autre modèle.",
            model_coming: "Téléchargement en cours…",
            model_offer: "Télécharger le modèle ({}, {})",
            offered_title: "Modèles à télécharger",
            offered_fetch: "Télécharger",
            offered_far: "Téléchargement… {}\u{202f}%",
            model_chosen_missing: "{} — pas encore téléchargé",
            offered_ollama_size: "environ 5 Go",
            offered_ollama_good: "Pour le compte rendu, sur cet ordinateur, avec Ollama",
            offered_needs_ollama: "Nécessite Ollama (gratuit, sur ollama.com)",
            offered_refresh: "Actualiser la liste ({})",
            offered_refresh_failed: "Impossible d’actualiser la liste. Les modèles ci-dessus restent disponibles.",
            speech_option: "SpeechAnalyzer d’Apple, sur ce Mac",
            speech_missing: "modèle d’Apple pas encore téléchargé",
            speech_fetch: "Télécharger le modèle d’Apple (macOS le télécharge depuis Apple)",
            speech_fetching: "macOS télécharge le modèle d’Apple…",
            speech_failed: "Impossible de télécharger le modèle d’Apple. Réessayez plus tard\u{202f}; Whisper reste disponible.",
            service_option: "Un service que je choisis",
            service_address: "Adresse du service",
            service_model: "Modèle",
            service_key: "Clé d’API (si demandée)",
            service_agree: "J’accepte que le son de mes réunions soit envoyé à {}",
            service_save: "Enregistrer",
            service_test: "Tester",
            service_tested: "Ça fonctionne\u{a0}: un extrait audio de test a été transcrit avec ses horodatages.",
            service_saved: "Enregistré\u{a0}: {} transcrira vos prochaines réunions.",
            service_saved_here: "Enregistré\u{a0}: cet ordinateur transcrira vos prochaines réunions.",
            service_agree_first: "Non enregistré\u{a0}: cochez d’abord l’accord d’envoi du son à {}.",
            service_address_bad: "Non enregistré\u{a0}: l’adresse doit commencer par https:// ou http://, par exemple https://api.groq.com/openai/v1.",
            service_model_named: "Non enregistré\u{a0}: indiquez le nom du modèle, par exemple whisper-large-v3.",
            installed: "installé, {}",
            keep_other: "Autre durée…",
            keep_not_days: "«\u{a0}{}\u{a0}» n’est pas un nombre de jours valide (de 1 à 36 500)",
            builtin: "intégré",
            home: "Dossier personnel",
            transcription: "Transcription",
            on_this_computer: "Sur cet ordinateur — rien n’est envoyé",
            reminder: [
                "Me rappeler de prévenir les participants",
                "Au début de chaque enregistrement, un rappel vous invite à annoncer que la réunion est enregistrée.",
            ],
            offer: [
                "Quand une application utilise le micro",
                "Le plus souvent, un appel qui commence. Kitokey n’enregistre jamais à votre insu.",
            ],
            offer_choices: ["Proposer d’enregistrer", "Ne rien faire"],
            autostart: [
                "Lancer Kitokey à l’ouverture de session",
                "Discrètement, sans fenêtre\u{a0}: Kitokey vous propose d’enregistrer vos appels même après un redémarrage.",
            ],
            autostart_held: [
                "Désactivé dans Windows\u{a0}: réactivez-le dans Paramètres › Applications › Démarrage.",
                "Défini par votre organisation\u{a0}: il ne peut pas être changé ici.",
            ],
            general_title: "Général",
            advanced_title: "Avancé",
            developer: [
                "Mode développeur",
                "Affiche le détail technique sous chaque erreur, pour aider au diagnostic.",
            ],
            program_said: "Détail technique\u{a0}: {}",
            minutes_sections: ["Points saillants", "Actions", "Points clés"],
            actions_cells: ["Qui", "Quoi", "Pour quand"],
            unsettled: "non précisé",
            model_recommended: "{} — recommandé",
            play: "Écouter",
            pause: "Pause",
            time_listen: "Écouter à partir de {}",
            time_passage: "Voir le passage à {}",
            not_playable: "Cet audio ne peut pas être lu dans Kitokey.",
            not_played: "Lecture impossible ici. Ouvrez le dossier pour l’écouter avec un autre lecteur.",
            playback: "Position de lecture",
            no_player: "L’audio a été supprimé pour faire de la place. La transcription et le compte rendu restent.",
            log_level: ["Niveau des journaux", "S’applique au prochain lancement de Kitokey."],
            log_info: "Normal",
            log_debug: "Détaillé",
            logs: ["Journaux", "Un fichier par jour, conservé 14 jours. Aucun contenu de réunion."],
            open_logs: "Ouvrir les journaux",
            logs_not_opened: "Impossible d’ouvrir le dossier des journaux.",
            copy_diagnostic: "Copier un diagnostic",
            diagnostic_copied: "Diagnostic copié",
            diagnostic_empty: "Aucun journal pour cette réunion",
            diagnostic_not_copied: "Copie impossible",
            sessions_folder: [
                "Dossier des réunions",
                "Les nouvelles réunions sont enregistrées ici. Les réunions existantes restent dans l’ancien dossier\u{a0}: Kitokey n’affiche et ne nettoie que celui-ci.",
            ],
            choose: "Choisir…",
            small: "Barre d’enregistrement",
            small_mic: "Votre micro",
            small_meeting: "Le son de la réunion",
            bar_silent_me: "Votre micro\u{a0}: rien depuis 2\u{a0}min — l’enregistrement continue.",
            bar_silent_others: "La réunion\u{a0}: rien depuis 2\u{a0}min — l’enregistrement continue.",
            small_nothing: "Rien n’en arrive\u{a0}: vérifiez qu’il n’est pas coupé sur l’appareil lui-même, et qu’il est prêt.",
            small_waiting_me: "Votre micro n’est pas enregistré pour l’instant.",
            small_waiting_others: "Le son de la réunion n’est pas enregistré pour l’instant.",
            small_note: "À l’arrêt, la transcription démarre toute seule. Une notification vous préviendra.",
            small_announce: "Prévenez les participants que cette réunion est enregistrée.",
            copy_announcement: "Copier l’annonce",
            announcement_not_copied: "Non copiée.",
            small_full: "Ouvrir la fenêtre complète",
            bar_more: "Plus d’options",
            bar_less: "Moins d’options",
            bar_hide: "Masquer la barre",
            small_microphone: "Micro\u{a0}: {}",
            small_microphones: "Micro du prochain enregistrement",
            microphone_auto: "Auto",
            microphone_auto_says: "Celui de votre appel, sinon celui de l’ordinateur.",
            microphone_absent: "Débranché pour l’instant\u{a0}: Auto prend le relais.",
            microphones_unread: "Liste des micros indisponible pour l’instant. Auto fonctionne\u{202f}; réessayez dans un instant.",
            microphone_not_kept: "Votre choix de micro n’a pas été enregistré. Choisissez-le à nouveau.",
            small_microphone_gone: "Le micro choisi est débranché\u{a0}: Auto prend le relais.",
            bar_microphone_recorded: "Micro utilisé\u{a0}: {}. Un changement s’appliquera au prochain enregistrement.",
            bar_microphone_next: "Un changement s’appliquera au prochain enregistrement.",
            bar_context: "Sujet, participants…",
            bar_context_clear: "Effacer le contexte",
            bar_context_kept: "Conservé pour le prochain enregistrement.",
            bar_last_hearing: "{}\u{a0}: transcription en cours",
            bar_last_summary: "{}\u{a0}: compte rendu en cours",
            bar_last_ready: "{}\u{a0}: prête",
            bar_last_stopped: "{}\u{a0}: arrêtée",
            bar_last_failed: "{}\u{a0}: {}",
            bar_last_open: "Ouvrir",
            bar_advanced: "Avancé — modèles",
            models_transcription: "Modèle de transcription",
            models_minutes: "Modèle du compte rendu",
            models_settings: "Celui des Réglages\u{a0}: {}",
            models_none: "aucun modèle choisi",
            models_own_note: "Pour cette réunion uniquement\u{202f}; les Réglages ne changent pas.",
            models_not_kept: "Choix non enregistré\u{a0}: {}",
            models_not_read: "Impossible de lire les modèles\u{a0}: {}",
            models_gone_transcription: "Le modèle de transcription choisi pour cette réunion n’est plus installé\u{a0}: celui des Réglages sera utilisé.",
            models_gone_minutes: "Le service choisi pour le compte rendu de cette réunion n’est plus configuré\u{a0}: celui des Réglages sera utilisé.",
            models_unused_confidential: "Réunion confidentielle\u{a0}: le service en ligne choisi n’est pas utilisé, tout se fait sur cet ordinateur.",
            bar_models_kept: "Modèles retenus\u{a0}: {}",
            bar_model_kept: "Modèle retenu\u{a0}: {}",
            check_sound: "Vérifier le son",
            check_sound_says: "Testez votre micro et le son de la réunion avant d’enregistrer.",
            check_unavailable: "Indisponible",
            check_bluetooth: "Tant que ce panneau est ouvert, votre casque Bluetooth passe en mode appel (son de moindre qualité). Il revient à la normale à la fermeture.",
            check_play: "Jouer un son de test",
            check_heard: "Kitokey entend bien le son de cet ordinateur.",
            check_not_heard: "Kitokey n’a pas entendu le son de test. Vérifiez que le son sort bien de cet ordinateur (et pas d’un autre appareil), puis réessayez.",
            check_call: "Un appel utilise le micro\u{a0}: pas de son de test, il serait entendu dans l’appel. Faites parler quelqu’un dans l’appel et regardez la jauge.",
            check_open_note: "Le micro reste actif tant que ce panneau est ouvert.",
            check_close: "Fermer",
            check_failed: "Kitokey ne parvient pas à écouter votre micro ni le son de la réunion. Fermez ce panneau et rouvrez-le.",
            check_recording: "Un enregistrement est en cours\u{a0}: ses jauges sont dans la barre d’enregistrement.",
            free: "Faire de la place",
            free_what: "Que voulez-vous supprimer\u{202f}?",
            free_audio: [
                "L’audio seulement",
                "Le compte rendu et la transcription restent lisibles. L’audio représente presque tout l’espace.",
            ],
            free_whole: [
                "La réunion entière",
                "Audio, transcription et compte rendu\u{a0}: rien n’est conservé.",
            ],
            free_heaviest: "Les plus volumineuses d’abord",
            free_older: "Cocher celles de plus de 30 jours",
            free_total: "libérés · {} réunions",
            free_one: "libérés · 1 réunion",
            free_note: "Rien n’est supprimé définitivement\u{a0}: tout va dans la corbeille, d’où vous pouvez le restaurer.",
            free_audio_note: "Sans l’audio, la réunion ne pourra plus être retranscrite, même avec un meilleur modèle.",
            free_do_audio: "Mettre l’audio à la corbeille",
            free_do_whole: "Mettre les réunions à la corbeille",
            free_busy: "en cours de traitement",
            free_done: "C’est fait\u{a0}: tout est dans la corbeille.",
            redo: "Refaire…",
            redo_title: "Refaire le compte rendu",
            redo_with: "Refait avec les modèles de cette réunion (à choisir dans «\u{a0}Avancé\u{a0}»).",
            redo_note: "Le compte rendu actuel reste en place tant que le nouveau n’est pas prêt, et se retrouve ensuite dans la corbeille.",
            redo_minutes_do: "Refaire le compte rendu avec {}",
            redo_transcript_do: "Refaire la transcription avec {}",
            on_computer: "sur cet ordinateur",
            redo_transcript_title: "Refaire la transcription",
            redo_transcript_note: "La transcription actuelle reste en place jusqu’à ce que la nouvelle soit terminée, puis va à la corbeille. En cas d’échec, elle est conservée.",
            make_minutes: "Rédiger le compte rendu",
            make_transcript: "Transcrire",
            make_minutes_with: "Rédiger le compte rendu avec {}",
            minutes_made_with: "Ce compte rendu a été fait avec {}.",
            transcript_made_with: "Cette transcription a été faite avec {}.",
            minutes_making: "Rédaction du compte rendu",
            minutes_waiting: "En attente · {} avant",
            minutes_awaiting: "En attente de votre accord pour l’envoi à {}",
            agree_open: "Voir et accepter",
            minutes_not_made: "Le compte rendu n’a pas pu être rédigé. La transcription n’a pas changé.",
            try_again: "Réessayer",
            chip_making: "Compte rendu…",
            chip_waiting: "En attente",
            chip_awaiting: "Accord requis",
            chip_transcribing: "Transcription…",
            chip_failed: "Échec compte rendu",
            chip_redo_failed: "Refaire échoué",
            minutes_redo_failed: "Le dernier Refaire a échoué. Le compte rendu affiché est le précédent.",
            transcript_making: "Transcription en cours",
            transcript_not_made: "La transcription n’a pas pu être faite. L’audio est intact.",
            chip_transcript_failed: "Non transcrite",
            minutes_unsaved: "Non enregistré\u{a0}: {} reste utilisé.",
            minutes_unsaved_none: "Non enregistré\u{a0}: aucun compte rendu ne sera rédigé d’ici là.",
            transcript_model_needed: "En attente du modèle de transcription.",
            transcript_model_coming: "En attente du modèle de transcription (téléchargement en cours).",
            making_cancel: "Annuler",
            making_cancel_failed: "Kitokey n’a pas pu annuler\u{a0}: {}",
            making_left: "environ {}\u{a0}min",
            transcript_stopped: "Transcription arrêtée. Vous pouvez la relancer, avec un autre modèle si besoin.",
            minutes_stopped: "Compte rendu arrêté. Vous pouvez le relancer, avec un autre modèle si besoin.",
            record_start: "Démarrer un enregistrement",
            record_stop: "Arrêter l’enregistrement",
            agree_eyebrow: "Premier compte rendu en ligne",
            agree_title: "Le texte de «\u{a0}{}\u{a0}» va quitter votre ordinateur",
            agree_lead: "Pour rédiger le compte rendu, Kitokey envoie la transcription au service que vous avez choisi. Voici exactement ce qui est envoyé.",
            agree_leaves: [
                "Envoyé\u{a0}: la transcription",
                "Les paroles de tous les participants — {} minutes, environ {} mots.",
            ],
            agree_context: ["Envoyé\u{a0}: le contexte que vous avez ajouté", "Environ {} mots."],
            agree_stays: ["Reste ici\u{a0}: l’audio", "Aucun son n’est jamais envoyé."],
            agree_to: "Destinataire",
            agree_model: "Modèle",
            agree_remember: "Ne plus demander pour ce service",
            agree_send: "Envoyer la transcription",
            agree_not: "Ne pas envoyer",
            agree_local: "Rédiger plutôt sur cet ordinateur",
            agree_local_write: "Rédiger ici",
            shortcut: ["Raccourci clavier", "Démarre ou arrête un enregistrement, où que vous soyez."],
            shortcut_none: "Aucun raccourci défini",
            shortcut_where: "Vous pouvez le modifier dans les réglages clavier du système.",
            shortcut_open: "Réglages du clavier",
            test: "Tester",
            test_sends: "Le test envoie une courte phrase à {}, rien de vos réunions.",
            testing: "Test en cours…",
            test_answered: "Ça fonctionne\u{a0}: le modèle répond.",
            test_failed: "Le modèle ne répond pas\u{a0}: {}",
            keep_audio: [
                "Conserver l’audio",
                "Passé ce délai, l’audio va à la corbeille\u{202f}; la transcription et le compte rendu restent.",
            ],
            keep_always: "Toujours",
            keep_days: "{} jours",
            not_saved: "Non enregistré\u{a0}: {}",
            key_not_kept: "Choix enregistré, mais la clé n’a pas pu être rangée dans le trousseau du système\u{a0}: {}",
            minutes_step: "Compte rendu",
            address: "Adresse (compatible OpenAI)",
            model_name: "Modèle",
            key: "Clé d’API",
            key_kept: "Clé enregistrée — saisissez-en une nouvelle pour la remplacer",
            where_service: "Un service",
            service_name: "Service",
            key_test: "Vérifier la clé",
            save: "Enregistrer",
            where_online: "En ligne — le texte de la transcription est envoyé",
            ollama_lacks: "Ollama n’a pas de modèle nommé {} sur cet ordinateur. Choisissez-en un dans la liste ou téléchargez-le avec Ollama.",
            ollama_silent: "Ollama ne répond pas ou n’a aucun modèle. Lancez Ollama, puis téléchargez un modèle.",
            welcome_title: "Configurons Kitokey en une question.",
            welcome_question: "Vos réunions abordent-elles des sujets confidentiels\u{202f}?",
            welcome_examples: "Clients, ressources humaines, santé, juridique, chiffres non publiés…",
            welcome_often: ["Oui, souvent", "Rien ne doit quitter cet ordinateur."],
            welcome_rarely: [
                "Rarement ou jamais",
                "Je peux confier le texte d’une réunion à un service de mon choix.",
            ],
            welcome_unsure: ["Je ne sais pas", "Kitokey choisira l’option la plus discrète."],
            welcome_skip: "Passer (tout sur cet ordinateur)",
            welcome_cancel: "Annuler",
            welcome_next: "Suivant",
            welcome_back: "Retour",
            welcome_writer: "Qui rédige le compte rendu\u{202f}?",
            welcome_here: [
                "Cet ordinateur",
                "Rien n’est envoyé. Nécessite Ollama (gratuit)\u{202f}; un peu plus lent.",
            ],
            welcome_service: [
                "Un service en ligne",
                "Plus rapide et plus précis. Seul le texte de la réunion est envoyé, jamais le son. Vous choisirez le service dans les Réglages\u{202f}; votre accord sera demandé avant le premier envoi.",
            ],
            welcome_private: ["Mode privé", "Vos réunions ne quittent pas cet ordinateur."],
            welcome_connected: [
                "Mode connecté",
                "Le compte rendu est rédigé par un service en ligne\u{202f}; la transcription reste sur cet ordinateur.",
            ],
            welcome_rows: ["Transcription", "Compte rendu", "Rappel"],
            welcome_on_this_computer: "Sur cet ordinateur",
            welcome_online: "Un service en ligne, à choisir",
            welcome_online_chosen: "Un service en ligne",
            welcome_ollama_by: "Ollama · {}",
            welcome_reminder_on: "Prévenir les participants au début de chaque réunion",
            welcome_reminder_off: "Désactivé (réactivable dans les Réglages)",
            welcome_ollama_silent: "Ollama ne répond pas sur cet ordinateur\u{a0}: le compte rendu attendra. Ollama est gratuit, sur ollama.com.",
            welcome_ollama_lacks: "Ollama n’a pas encore {}\u{a0}: le compte rendu attendra. Téléchargez-le en un clic dans les Réglages.",
            welcome_fetch: "Télécharger le modèle ({})",
            welcome_fetch_from: "Depuis {}, vérifié avant utilisation.",
            welcome_done: "Terminer",
            welcome_later: "Plus tard",
            welcome_later_waits: "Plus tard — vos réunions attendront le modèle",
            welcome_choose: "Choisir mon service",
            welcome_again: [
                "Refaire la configuration",
                "Reprend les questions du premier lancement, avec vos choix actuels. Rien ne change avant la dernière étape.",
            ],
            context_add: "Ajouter un contexte",
            context_title: "Contexte",
            context_label: "De quoi parle la réunion\u{202f}? Qui y participe\u{202f}?",
            context_example: "Ex.\u{a0}: budget du T4 avec Inès Durand et l’équipe achats",
            context_why: "Aide la transcription à bien écrire les noms et les termes du métier, et le compte rendu à attribuer chaque action à la bonne personne.",
            context_most: "{} caractères maximum",
            context_left: ["{} restant", "{} restants"],
            context_script: "Dans cet alphabet, le contexte peut être tronqué ou refusé par certains services.",
            context_making: "Le contexte ne peut pas être modifié pendant la transcription ou la rédaction du compte rendu.",
            context_unfinished: "La transcription n’est pas terminée. Cliquez sur Réessayer pour la finir avant de modifier le contexte.",
            context_changed: [
                "La transcription a été faite avec un autre contexte.",
                "La transcription et le compte rendu ont été faits avec un autre contexte.",
            ],
            context_redo: "Refaire avec ce contexte",
            language_label: "Langue parlée",
            language_default: "Celle des Réglages",
            language_detected: "Détection automatique",
            meetings_language: "Langue des réunions",
            language_not_saved: "Langue non enregistrée\u{a0}: {}",
            rename_meeting: "Renommer cette réunion",
            name_meeting: "Nommer cette réunion",
            name_not_saved: "Nom non enregistré\u{a0}: {}",
            language_detected_short: "détectée",
            language_as_system: "Celle du système",
            language_heard_as: "{} — SpeechAnalyzer transcrit en {}",
            app_language: [
                "Langue de Kitokey",
                "Fenêtre, menu et notifications. Les boîtes de dialogue du système gardent la langue du système.",
            ],
            app_language_system: "Langue du système",
            appearance: [
                "Apparence",
                "Toutes les fenêtres de Kitokey, la barre aussi\u{202f}; Système suit le clair ou le sombre du système.",
            ],
            appearance_choices: ["Système", "Clair", "Sombre"],
            correct_line: "Corriger la ligne à {}",
            corrected: "corrigé",
            correction_empty: "Une ligne ne peut pas être vide. Appuyez sur Échap pour annuler.",
            correction_failed: "Correction non enregistrée\u{a0}: {}",
            corrected_since: "La transcription a été corrigée depuis la rédaction de ce compte rendu.",
            redo_corrections: "Refaire avec les corrections",
            copy_minutes: "Copier le compte rendu",
            minutes_copied: "Copié",
            minutes_not_copied: "Impossible de copier le compte rendu.",
            redo_loses_corrections: "Vos corrections de cette transcription seront perdues.",
            context_not_saved: "Impossible d’enregistrer le contexte.",
            confidential_name: "Réunion confidentielle",
            confidential_says: "Rien ne quitte cet ordinateur",
            confidential_chip: "Confidentielle",
            confidential_left: "Déjà envoyé auparavant\u{a0}: {}.",
            confidential_left_sound: "le son à {}",
            confidential_left_transcript: "la transcription à {}",
            confidential_not_saved: "Impossible de marquer la réunion comme confidentielle. Réessayez.",
            confidential_ollama_empty: "Ollama n’a aucun modèle sur cet ordinateur\u{a0}: le compte rendu attendra.",
            confidential_fetch: "Télécharger {}",
        },
    }
}

/// What can stand in the shell's own way, before or beside a recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    /// The program that records could not be started: what the system said of it.
    NoRecorder(String),
    /// The settings file was not read: which setting, on which line (#783).
    Settings(Refused),
    /// The desktop did not bind the global shortcut: what it said of it.
    Shortcut(String),
    /// The model a transcript is heard with is not on the machine: where it was looked for.
    NoModel(String),
    /// The program that hears could not be started: what the system said of it.
    NoTranscriber(String),
    /// The meetings' folder could not be made or written: where it is.
    NoFolder(String),
    /// A transcript was asked for, and no recording is in the folder of sessions.
    NoSession,
}

/// What the window's page shows of what a program said of a failure: it, in developer mode;
/// else nothing — the page says the failure in plain words (D29).
pub fn shown_detail(said: &str, developer: bool) -> String {
    if developer { said.to_string() } else { String::new() }
}

/// Why a command the page called failed: each is one sentence below. What no gesture of the user
/// can reach — the page refuses it first, or offers nothing else — is `Unexpected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Unexpected,
    NoMeeting,
    /// Said at the list, where its page was asked: nowhere to go back to.
    MeetingGone,
    Busy,
    NotComplete,
    MeetingFile,
    Trash(Refusal),
    Confidential,
    NoneChosen,
    NoTranscript,
    ProviderChanged,
    ModelUnnamed,
    ModelAbsent,
    BadAddress,
    FolderUnnamed,
    FolderNotWhole,
    FolderUnmade,
    Autostart,
    /// The settings file refused, which setting and where (#783): under the command it stopped,
    /// and, `SettingsLocked`, alone on Settings.
    SettingsUnread(Refused),
    SettingsLocked(Refused),
    SettingsUnwritten,
    FolderNotOpened,
    KeyboardSettings,
    Keyring,
    ProgramNotStarted,
    TestRefused,
}

/// A failure's sentence. Most follow the page's own « not saved: » and so start in lower case; those
/// the page shows alone, `FolderNotOpened`, `KeyboardSettings` and `SettingsLocked`, are whole.
pub fn failure(failure: Failure, language: Language) -> String {
    use Language::*;
    match (failure, language) {
        (Failure::Trash(refusal), _) => return audio_trashed(language, &Err(refusal)).body,
        (Failure::Unexpected, English) => "Kitokey could not do this: try again, and restart it if it happens again.",
        (Failure::Unexpected, French) => "Kitokey n’a pas pu le faire\u{a0}: réessayez, et redémarrez-le si cela se reproduit.",
        (Failure::MeetingGone, English) => "this meeting is no longer in the meetings’ folder.",
        (Failure::MeetingGone, French) => "cette réunion n’est plus dans le dossier des réunions.",
        (Failure::NoMeeting, English) => "this meeting is no longer in the meetings’ folder: go back to the list.",
        (Failure::NoMeeting, French) => "cette réunion n’est plus dans le dossier des réunions\u{a0}: revenez à la liste.",
        (Failure::Busy, English) => "this meeting is being recorded or made: try again once that is done.",
        (Failure::Busy, French) => "cette réunion est en cours d’enregistrement ou de préparation\u{a0}: réessayez une fois cela terminé.",
        (Failure::NotComplete, English) => "its transcript is not complete: make it again first.",
        (Failure::NotComplete, French) => "sa transcription n’est pas complète\u{a0}: refaites-la d’abord.",
        (Failure::MeetingFile, English) => {
            "its folder could not be written to: check that the disk has room and that you may write there, then try again."
        }
        (Failure::MeetingFile, French) => {
            "son dossier n’a pas pu être écrit\u{a0}: vérifiez qu’il reste de la place et que vous pouvez y écrire, puis réessayez."
        }
        (Failure::Confidential, English) => "this meeting is confidential: nothing is sent.",
        (Failure::Confidential, French) => "cette réunion est confidentielle\u{a0}: rien n’est envoyé.",
        (Failure::NoneChosen, English) => "nothing is chosen yet: choose it in Settings, then try again.",
        (Failure::NoneChosen, French) => "aucun modèle choisi\u{a0}: choisissez-en un dans les réglages, puis réessayez.",
        (Failure::NoTranscript, English) => "this meeting has no transcript to send: make it first.",
        (Failure::NoTranscript, French) => "cette réunion n’a pas de transcription à envoyer\u{a0}: faites-la d’abord.",
        (Failure::ProviderChanged, English) => "who writes the minutes has changed: look at this page again.",
        (Failure::ProviderChanged, French) => "le modèle qui rédige le compte rendu a changé\u{a0}: revoyez cette page.",
        (Failure::ModelUnnamed, English) => "name a model, then try again.",
        (Failure::ModelUnnamed, French) => "nommez un modèle, puis réessayez.",
        (Failure::ModelAbsent, English) => "this model is not on this computer: fetch it in Settings.",
        (Failure::ModelAbsent, French) => "ce modèle n’est pas sur cet ordinateur\u{a0}: téléchargez-le dans les réglages.",
        (Failure::BadAddress, English) => "this address cannot be used: write it whole, starting with https://.",
        (Failure::BadAddress, French) => "cette adresse n’est pas utilisable\u{a0}: écrivez-la en entier, en commençant par https://.",
        (Failure::FolderUnnamed, English) => "name a folder, then try again.",
        (Failure::FolderUnnamed, French) => "nommez un dossier, puis réessayez.",
        (Failure::FolderNotWhole, English) => "write the folder’s whole path, from / or from ~.",
        (Failure::FolderNotWhole, French) => "écrivez le chemin entier du dossier, depuis / ou depuis ~.",
        (Failure::FolderUnmade, English) => "this folder could not be made: check that you may write there.",
        (Failure::FolderUnmade, French) => "ce dossier n’a pas pu être créé\u{a0}: vérifiez que vous pouvez y écrire.",
        (Failure::Autostart, English) => "starting at login could not be changed: check that your home folder can be written to.",
        (Failure::Autostart, French) => {
            "le démarrage à l’ouverture de session n’a pas pu être changé\u{a0}: vérifiez que votre dossier personnel peut être écrit."
        }
        // Shows: under a command the window refused, after « not saved: », then the refusal.
        // When: the settings file cannot be read as settings. At most: no limit. Placeholders: {} the refusal.
        (Failure::SettingsUnread(why), English) => return format!("the settings file cannot be read. {}", refused(&why, language)),
        (Failure::SettingsUnread(why), French) => {
            return format!("le fichier des réglages ne peut pas être lu. {}", refused(&why, language));
        }
        // Shows: alone on Settings, then the refusal. When: the settings file cannot be read.
        // At most: no limit. Placeholders: {} the refusal.
        (Failure::SettingsLocked(why), English) => {
            return format!(
                "The settings file cannot be read: Kitokey does not write over it until it is corrected. {}",
                refused(&why, language)
            );
        }
        (Failure::SettingsLocked(why), French) => {
            return format!(
                "Le fichier des réglages ne peut pas être lu\u{a0}: Kitokey ne l’écrit pas tant qu’il n’est pas corrigé. {}",
                refused(&why, language)
            );
        }
        // Shows: under a command the window refused, after « not saved: », or alone where the page
        // shows it (the folder, the keyboard settings). When: the command named by its arm fails.
        // At most: no limit. Placeholders: none.
        (Failure::SettingsUnwritten, English) => "the settings file could not be written: check that the disk has room, then try again.",
        (Failure::SettingsUnwritten, French) => {
            "le fichier des réglages n’a pas pu être écrit\u{a0}: vérifiez qu’il reste de la place, puis réessayez."
        }
        (Failure::FolderNotOpened, English) => "The folder could not be opened: open it from your file manager.",
        (Failure::FolderNotOpened, French) => "Le dossier n’a pas pu être ouvert\u{a0}: ouvrez-le depuis votre gestionnaire de fichiers.",
        (Failure::KeyboardSettings, English) => "The keyboard settings cannot be opened from here: open them from your system’s settings.",
        (Failure::KeyboardSettings, French) => {
            "Les réglages du clavier ne s’ouvrent pas d’ici\u{a0}: ouvrez-les depuis ceux de votre système."
        }
        (Failure::Keyring, English) => "the system’s keyring does not answer: unlock it, then try again.",
        (Failure::Keyring, French) => "le trousseau du système ne répond pas\u{a0}: déverrouillez-le, puis réessayez.",
        (Failure::ProgramNotStarted, English) => "a program of Kitokey’s could not be started: install Kitokey again.",
        (Failure::ProgramNotStarted, French) => "un programme de Kitokey n’a pas pu démarrer\u{a0}: réinstallez Kitokey.",
        (Failure::TestRefused, English) => "check its address, its name and its key, then try again.",
        (Failure::TestRefused, French) => "vérifiez son adresse, son nom et sa clé, puis réessayez.",
    }
    .to_string()
}

/// What a command that failed hands the page: a sentence of these words, and only here made — and,
/// in developer mode, the raw text on the line under it (D47).
#[derive(Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct Told(String);

/// A failure as the page is told it.
pub fn told(which: Failure, detail: &str, language: Language, developer: bool) -> Told {
    let sentence = failure(which, language);
    let shown = shown_detail(detail, developer);
    Told(if shown.is_empty() {
        sentence
    } else {
        format!("{sentence}\n{shown}")
    })
}

/// *Tester* failed with no cause named: the sentence that says the model does not answer, whatever came.
pub fn test_refused(told: Told, language: Language) -> Told {
    Told(window(language).test_failed.replacen("{}", &told.0, 1))
}

/// *Tester* failed for a cause its program named: that sentence alone, the raw text only in developer mode.
pub fn test_caused(which: Cause, detail: &str, language: Language, developer: bool) -> Told {
    let shown = shown_detail(detail, developer);
    let sentence = cause(which, language);
    Told(if shown.is_empty() {
        sentence.to_string()
    } else {
        format!("{sentence}\n{shown}")
    })
}

/// A quantisation's or a precision's tag, as a model's id ends with it: q4_K_M, iq3_xs, fp16, bf16,
/// f32, int8 — what only machines read.
fn quantised(tag: &str) -> bool {
    let tag = tag.to_ascii_lowercase();
    let q = tag.strip_prefix('i').unwrap_or(&tag);
    if let Some(rest) = q.strip_prefix('q') {
        let mut chars = rest.chars();
        if chars.next().is_some_and(|c| c.is_ascii_digit()) {
            let more = chars.as_str();
            return more.is_empty()
                || more.strip_prefix('_').is_some_and(|parts| {
                    parts
                        .split('_')
                        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()))
                });
        }
    }
    let float = |t: &str, bits: &str| t.strip_prefix('f').is_some_and(|t| t.strip_prefix('p').unwrap_or(t) == bits);
    float(tag.strip_prefix('b').unwrap_or(&tag), "16") || float(&tag, "32") || tag == "int4" || tag == "int8"
}

/// A date of release, as a model's id ends with it: 20241022, 2024-08-06, 06-2025.
fn dated(tag: &str) -> bool {
    let digits = |t: &str, n: usize| t.len() == n && t.chars().all(|c| c.is_ascii_digit());
    let parts: Vec<&str> = tag.split('-').collect();
    match parts[..] {
        [one] => digits(one, 8),
        [year, month, day] => digits(year, 4) && digits(month, 2) && digits(day, 2),
        [month, year] => digits(month, 2) && digits(year, 4),
        _ => false,
    }
}

/// `name` without the first tail, after one of `separators`, that `tail` recognises — cut where a
/// character begins, never inside one.
fn without_tail<'a>(name: &'a str, separators: &[char], tail: impl Fn(&str) -> bool) -> &'a str {
    for (at, c) in name.char_indices() {
        let (before, from) = name.split_at(at);
        let mut rest = from.chars();
        rest.next();
        if separators.contains(&c) && tail(rest.as_str()) {
            return before;
        }
    }
    name
}

/// What JavaScript's `\s` matches: its white space and line ends — U+FEFF among them, U+0085 not.
fn js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

/// What people say of a model: the page's `modelSaid`, character for character (`tests/model_said.rs`
/// runs both) — the last part of its id, without what only machines read: a file's extension, a tag
/// "latest", a date of release, a quantisation.
pub fn model_said(model: &str) -> String {
    let mut name = model.split('/').rfind(|part| !part.is_empty()).unwrap_or(model);
    // One extension, the last.
    if let Some((before, extension)) = name.rsplit_once('.')
        && (extension.eq_ignore_ascii_case("gguf") || extension.eq_ignore_ascii_case("bin"))
    {
        name = before;
    }
    // Its tag "latest", after a colon.
    if let Some((before, tag)) = name.rsplit_once(':')
        && tag.eq_ignore_ascii_case("latest")
    {
        name = before;
    }
    name = without_tail(name, &['-', '_', ':', '.'], quantised);
    name = without_tail(name, &['-', '_', ':'], dated);
    let machine = |word: &str| quantised(word) || ["qat", "k", "m", "s", "l"].contains(&word.to_ascii_lowercase().as_str());
    let mut words: Vec<&str> = name
        .split(|c: char| c == '-' || c == '_' || c == ':' || js_space(c))
        .filter(|w| !w.is_empty())
        .collect();
    while words.len() > 1 && words.last().is_some_and(|word| machine(word)) {
        words.pop();
    }
    if words.is_empty() || (words.len() == 1 && machine(words[0])) {
        return model.to_string();
    }
    let short = |w: &str| (1..=2).contains(&w.len()) && w.chars().all(|c| c.is_ascii_digit());
    let mut said: Vec<String> = Vec::new();
    for word in words {
        let mut chars = word.chars();
        let first = chars.next();
        // A version written with dashes — claude-sonnet-4-5 — is said with its point.
        if short(word) && said.last().is_some_and(|last| last.split('.').all(short)) {
            let last = said.last_mut().expect("checked above");
            last.push('.');
            last.push_str(word);
        } else if word.to_lowercase() == "gpt" {
            said.push("GPT".into());
        } else if let Some(first) = first.filter(char::is_ascii_lowercase) {
            said.push(format!("{}{}", first.to_ascii_uppercase(), chars.as_str()));
        } else {
            said.push(word.to_string());
        }
    }
    said.join(" ")
}

/// The cause's sentence, said of the meeting's own model when it was its (`own`, as its id is) and the
/// gesture is another model — then « Refaire… › Avancé » —, else Settings' sentence: the one the page
/// and the notification both say (D91).
pub fn cause_said(cause: Cause, own: Option<&str>, language: Language) -> String {
    match own.zip(cause_own(cause, language)) {
        Some((model, sentence)) => sentence.replacen("{}", &model_said(model), 1),
        None => self::cause(cause, language).to_string(),
    }
}

/// Why minutes were not made by the meeting's own model, chosen in « Avancé » (D91), where the gesture
/// is another model: `{}` its name, the page's; chosen again in *Refaire…*, never in Settings. None
/// where the gesture is the provider's — a key, credit, an address —: `cause`'s sentence says it.
pub fn cause_own(cause: Cause, language: Language) -> Option<&'static str> {
    use Language::{English, French};
    Some(match (cause, language) {
        (Cause::TooSlow, English) => {
            "{}, chosen for this meeting, did not answer in time: try again later, or choose a faster model in Redo…\u{a0}› Advanced."
        }
        (Cause::TooSlow, French) => {
            "{}, choisi pour cette réunion, n’a pas répondu à temps\u{a0}: réessayez plus tard, ou choisissez un modèle plus rapide dans Refaire…\u{a0}› Avancé."
        }
        (Cause::UnknownModel, English) => {
            "The provider does not know {}, chosen for this meeting: choose another in Redo…\u{a0}› Advanced."
        }
        (Cause::UnknownModel, French) => {
            "Le fournisseur ne connaît pas {}, choisi pour cette réunion\u{a0}: choisissez-en un autre dans Refaire…\u{a0}› Avancé."
        }
        (Cause::Down, English) => {
            "The provider answers that it is failing: try again later. On this computer, {}, chosen for this meeting, may be too large for the memory: choose a smaller one in Redo…\u{a0}› Advanced."
        }
        (Cause::Down, French) => {
            "Le fournisseur répond qu’il est en panne\u{a0}: réessayez plus tard. Sur cet ordinateur, {}, choisi pour cette réunion, est peut-être trop gros pour la mémoire\u{a0}: choisissez-en un plus petit dans Refaire…\u{a0}› Avancé."
        }
        (Cause::CutShort, English) => {
            "{}, chosen for this meeting, stopped before the end: choose one that writes longer answers in Redo…\u{a0}› Advanced."
        }
        (Cause::CutShort, French) => {
            "{}, choisi pour cette réunion, s’est arrêté avant la fin\u{a0}: choisissez-en un qui écrit plus long dans Refaire…\u{a0}› Avancé."
        }
        (Cause::NotMinutes, English) => {
            "{}, chosen for this meeting, did not write minutes: try again, or choose another model in Redo…\u{a0}› Advanced."
        }
        (Cause::NotMinutes, French) => {
            "{}, choisi pour cette réunion, n’a pas écrit de compte rendu\u{a0}: réessayez, ou choisissez un autre modèle dans Refaire…\u{a0}› Avancé."
        }
        _ => return None,
    })
}

/// Why minutes were not made, and what to do, as kitokey-summary coded it (summarising/v1).
pub fn cause(cause: Cause, language: Language) -> &'static str {
    use Language::{English, French};
    match (cause, language) {
        (Cause::Unreachable, English) => "The provider could not be reached: check that it runs, or the network, then try again.",
        (Cause::Unreachable, French) => "Le fournisseur n’a pas pu être joint\u{a0}: vérifiez qu’il tourne, ou le réseau, puis réessayez.",
        (Cause::TooSlow, English) => {
            "The provider did not answer in time: try again later, or choose a faster model in Kitokey’s settings."
        }
        (Cause::TooSlow, French) => {
            "Le fournisseur n’a pas répondu à temps\u{a0}: réessayez plus tard, ou choisissez un modèle plus rapide dans les réglages de Kitokey."
        }
        (Cause::RefusedKey, English) => "The provider refused the key: give it again in Kitokey’s settings.",
        (Cause::RefusedKey, French) => "Le fournisseur a refusé la clé\u{a0}: saisissez-la de nouveau dans les réglages de Kitokey.",
        (Cause::NoCredit, English) => "The provider refused for lack of credit: add credit with it, then try again.",
        (Cause::NoCredit, French) => "Le fournisseur a refusé faute de crédit\u{a0}: ajoutez du crédit chez lui, puis réessayez.",
        (Cause::UnknownModel, English) => "The provider does not know this model or this address: choose them again in Kitokey’s settings.",
        (Cause::UnknownModel, French) => {
            "Le fournisseur ne connaît pas ce modèle ou cette adresse\u{a0}: choisissez-les de nouveau dans les réglages de Kitokey."
        }
        (Cause::TooMany, English) => "The provider refuses for too many requests, or a quota used up: wait, or check your plan with it.",
        (Cause::TooMany, French) => {
            "Le fournisseur refuse pour trop de demandes, ou un quota épuisé\u{a0}: patientez, ou vérifiez votre forfait chez lui."
        }
        (Cause::Down, English) => {
            "The provider answers that it is failing: try again later. On this computer, this model may be too large for the memory: choose a smaller one."
        }
        (Cause::Down, French) => {
            "Le fournisseur répond qu’il est en panne\u{a0}: réessayez plus tard. Sur cet ordinateur, ce modèle est peut-être trop gros pour la mémoire\u{a0}: choisissez-en un plus petit."
        }
        (Cause::NoTimes, English) => {
            "This model gives no times for what it hears: choose one that gives each segment’s times in Kitokey’s settings."
        }
        (Cause::NoTimes, French) => {
            "Ce modèle ne donne pas les temps de ce qu’il entend\u{a0}: choisissez-en un qui donne ceux de chaque segment dans les réglages de Kitokey."
        }
        (Cause::Redirected, English) => "The provider sends the request to another address: check the address in Kitokey’s settings.",
        (Cause::Redirected, French) => {
            "Le fournisseur renvoie la demande vers une autre adresse\u{a0}: vérifiez l’adresse dans les réglages de Kitokey."
        }
        (Cause::ProviderFailed, English) => "The provider failed to answer: try again later.",
        (Cause::ProviderFailed, French) => "Le fournisseur n’a pas su répondre\u{a0}: réessayez plus tard.",
        (Cause::CutShort, English) => "The model stopped before the end: choose one that writes longer answers in Kitokey’s settings.",
        (Cause::CutShort, French) => {
            "Le modèle s’est arrêté avant la fin\u{a0}: choisissez-en un qui écrit plus long dans les réglages de Kitokey."
        }
        (Cause::NotMinutes, English) => "The model did not write minutes: try again, or choose another model in Kitokey’s settings.",
        (Cause::NotMinutes, French) => {
            "Le modèle n’a pas écrit de compte rendu\u{a0}: réessayez, ou choisissez un autre modèle dans les réglages de Kitokey."
        }
        (Cause::TranscriptUnusable, English) => "The transcript cannot be read: make it again first.",
        (Cause::TranscriptUnusable, French) => "La transcription ne peut pas être lue\u{a0}: refaites-la d’abord.",
        (Cause::NoSpeech, English) => "The transcript holds no speech: there is nothing to write minutes of. Check the recording.",
        (Cause::NoSpeech, French) => {
            "La transcription ne contient aucune parole\u{a0}: il n’y a rien à résumer. Vérifiez l’enregistrement."
        }
        (Cause::Settings, English) => "The provider’s address, model or key cannot be used as given: correct them in Kitokey’s settings.",
        (Cause::Settings, French) => {
            "L’adresse, le modèle ou la clé du fournisseur ne peuvent pas être utilisés tels quels\u{a0}: corrigez-les dans les réglages de Kitokey."
        }
        (Cause::Unwritable, English) => "The minutes could not be written in the meeting’s folder: free space, or check the folder.",
        (Cause::Unwritable, French) => {
            "Le compte rendu n’a pas pu être écrit dans le dossier de la réunion\u{a0}: libérez de la place, ou vérifiez le dossier."
        }
    }
}

/// What the user is told of a settings file refused (#783): which setting, on which line, and the
/// gesture — never `settings.rs`'s own sentence, which stays for the log and developer mode.
pub fn refused(refused: &Refused, language: Language) -> String {
    use Language::{English, French};
    // Shows: after the sentence of a notification or a page that says the file was not read.
    // When: `settings::read` refuses the file. At most: no limit. Placeholders: {line} the line,
    // {key} the setting's name as the file writes it — one of Kitokey's own, never what the user typed.
    match (*refused, language) {
        (Refused::Unopened, English) => "Kitokey cannot open its settings file: check that you may read it.".into(),
        (Refused::Unopened, French) => "Kitokey ne peut pas ouvrir son fichier de réglages\u{a0}: vérifiez que vous pouvez le lire.".into(),
        (Refused::Unparsed { line, key: Some(key) }, English) => {
            format!("Line {line}: Kitokey does not understand the value of “{key}”. Correct it, or remove the line.")
        }
        (Refused::Unparsed { line, key: Some(key) }, French) => {
            format!("Ligne {line}\u{a0}: Kitokey ne comprend pas la valeur de «\u{a0}{key}\u{a0}». Corrigez-la, ou supprimez la ligne.")
        }
        (Refused::Unparsed { line, key: None }, English) => {
            format!("Line {line} is not written as a setting. Correct it, or remove it.")
        }
        (Refused::Unparsed { line, key: None }, French) => {
            format!("La ligne {line} n’est pas écrite comme un réglage. Corrigez-la, ou supprimez-la.")
        }
        (Refused::SessionsNotWhole { line }, English) => {
            format!("Line {line}: “sessions” gives a folder’s whole path, from / or from ~. Correct it.")
        }
        (Refused::SessionsNotWhole { line }, French) => {
            format!("Ligne {line}\u{a0}: «\u{a0}sessions\u{a0}» donne le chemin entier d’un dossier, depuis / ou depuis ~. Corrigez-le.")
        }
        (Refused::ModelNotWhole { line }, English) => {
            format!("Line {line}: “model” gives a file’s whole path, from / or from ~. Correct it.")
        }
        (Refused::ModelNotWhole { line }, French) => {
            format!("Ligne {line}\u{a0}: «\u{a0}model\u{a0}» donne le chemin entier d’un fichier, depuis / ou depuis ~. Corrigez-le.")
        }
        (Refused::KeepAudio { line }, English) => {
            format!("Line {line}: “keep_audio” is “always” or a number of days, from 1 to 36,500. Correct it.")
        }
        (Refused::KeepAudio { line }, French) => format!(
            "Ligne {line}\u{a0}: «\u{a0}keep_audio\u{a0}» vaut «\u{a0}always\u{a0}» ou un nombre de jours, de 1 à 36\u{202f}500. Corrigez-le."
        ),
    }
}

/// What the user is told of a trouble: what it is, and what still works. Where the user has
/// something to put right — the settings file's line, the place the model goes —, it says where;
/// never what a program or the system said (D29).
pub fn trouble(trouble: &Trouble, language: Language) -> Notice {
    use Language::{English, French};
    let said = |title: &str, body: String| notice(title, body).expect("a trouble is always said");
    let (title, body) = match (trouble, language) {
        (Trouble::NoRecorder(_), English) => (
            "Kitokey cannot record",
            "Its recording program could not be started. Installing Kitokey again puts it back.",
        ),
        (Trouble::NoRecorder(_), French) => (
            "Kitokey ne peut pas enregistrer",
            "Son programme d’enregistrement n’a pas pu démarrer. Réinstaller Kitokey le remet en place.",
        ),
        // Shows: a notification at a start, its body then the refusal. When: the settings file cannot
        // be read. At most: no limit. Placeholders: {} the refusal.
        (Trouble::Settings(why), English) => {
            return said(
                "Your settings were not read",
                format!(
                    "Kitokey runs with its defaults until the file is corrected. {}",
                    refused(why, language)
                ),
            );
        }
        (Trouble::Settings(why), French) => {
            return said(
                "Vos réglages n’ont pas été lus",
                format!(
                    "Kitokey fonctionne avec ses réglages par défaut tant que le fichier n’est pas corrigé. {}",
                    refused(why, language)
                ),
            );
        }
        // Shows: a notification, its title then its body. When: the shortcut is not bound, the model or
        // the program that hears is missing, the meetings' folder cannot be written, or no recording is
        // there to transcribe. At most: the title, 50 characters; the body, no limit. Placeholders:
        // {at} where the model is looked for, or the folder.
        (Trouble::Shortcut(_), English) => (
            "The keyboard shortcut is not set",
            "The menu of Kitokey’s icon still starts and stops a recording.",
        ),
        (Trouble::Shortcut(_), French) => (
            "Le raccourci clavier n’est pas actif",
            "Le menu de l’icône Kitokey permet toujours de démarrer et d’arrêter un enregistrement.",
        ),
        (Trouble::NoModel(at), English) => {
            return said(
                "No transcript: the model is missing",
                format!("The recording is kept. Place the Whisper model at {at}, then use “Transcribe the last recording”."),
            );
        }
        (Trouble::NoModel(at), French) => {
            return said(
                "Pas de transcription\u{a0}: le modèle manque",
                format!(
                    "L’enregistrement est conservé. Placez le modèle Whisper à l’emplacement {at}, puis utilisez «\u{a0}Transcrire le dernier enregistrement\u{a0}»."
                ),
            );
        }
        (Trouble::NoTranscriber(_), English) => (
            "Kitokey cannot transcribe",
            "Its transcription program could not be started. The recording is kept; installing Kitokey again puts the program back.",
        ),
        (Trouble::NoFolder(at), English) => {
            return said(
                "Kitokey cannot record",
                format!("Its meetings folder cannot be written: {at}. Choose another one in Kitokey’s settings."),
            );
        }
        (Trouble::NoFolder(at), French) => {
            return said(
                "Kitokey ne peut pas enregistrer",
                format!(
                    "Son dossier des réunions ne peut pas être écrit\u{a0}: {at}. Choisissez-en un autre dans les réglages de Kitokey."
                ),
            );
        }
        (Trouble::NoSession, English) => (
            "No recording to transcribe",
            "Record a meeting first: its transcript follows the stop.",
        ),
        (Trouble::NoSession, French) => (
            "Aucun enregistrement à transcrire",
            "Enregistrez d’abord une réunion\u{a0}: sa transcription suit l’arrêt.",
        ),
        (Trouble::NoTranscriber(_), French) => (
            "Kitokey ne peut pas transcrire",
            "Son programme de transcription n’a pas pu démarrer. L’enregistrement est conservé\u{202f}; réinstaller Kitokey remet le programme en place.",
        ),
    };
    said(title, body.to_string())
}

/// What the user is told of a transcript's end: there, with where it is — or not, with why,
/// and how to have it made again.
pub fn made(made: &Made, language: Language) -> Notice {
    use Language::{English, French};
    let (title, body) = match (made, language) {
        // A click opens the meeting in Kitokey where the desktop says it was clicked (D44).
        (Made::Written(_), English) => (
            "Transcript ready",
            if CLICKED {
                "Click to open it."
            } else {
                "It is in Kitokey’s window."
            },
        ),
        (Made::Written(_), French) => (
            "Transcription prête",
            if CLICKED {
                "Cliquez pour l’ouvrir."
            } else {
                "Elle est dans la fenêtre de Kitokey."
            },
        ),
        (Made::Failed(_), English) => (
            "The transcript was not made",
            "The recording is untouched. “Transcribe the last recording” in Kitokey’s menu tries again from where it stopped.",
        ),
        (Made::Failed(_), French) => (
            "La transcription n’a pas été faite",
            "L’enregistrement est intact. «\u{a0}Transcrire le dernier enregistrement\u{a0}», dans le menu de Kitokey, reprend là où elle s’est arrêtée.",
        ),
    };
    notice(title, body.to_string()).expect("a transcript's end is always said")
}

/// What becomes of the minutes asked for, or chained after a transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Minutes {
    Ready,
    /// Ready, and the minutes they replace could not go to the trash: kept beside them, by this name.
    ReadyKeptBeside(String),
    /// Not made, and why when kitokey-summary named a cause this version knows.
    Failed(Option<Cause>),
    /// Made again (*Refaire…*) and not made: the ones there stayed; and why, as above.
    RedoFailed(Option<Cause>),
    /// Not made by the meeting's own model (its id), for a cause whose gesture is another model: said
    /// as the page says it (D91); `again` as for `RedoFailed`.
    FailedOwn {
        cause: Cause,
        model: String,
        again: bool,
    },
    /// Online, and the user has not agreed to send this meeting to this host yet.
    Waiting(String),
    /// No one is chosen to write them.
    Unchosen,
    /// Who writes them, in the settings, is no provider's address: the address as written.
    NoProvider(String),
    /// The last recording has no whole transcript.
    NoTranscript,
    /// The meeting is confidential, and Ollama does not answer (`silent`) or holds no model (D60).
    OllamaWaits {
        silent: bool,
    },
}

impl Minutes {
    /// Not made: as a first making, or as minutes made again (*Refaire…*) — the ones there stayed.
    /// The same, by the meeting's own model when it was its (`own`): named where its gesture is another
    /// model, else as `failed` says it.
    pub fn failed_by(cause: Option<Cause>, again: bool, own: Option<&str>, language: Language) -> Self {
        match (cause, own) {
            (Some(cause), Some(model)) if cause_own(cause, language).is_some() => Minutes::FailedOwn {
                cause,
                model: model.to_string(),
                again,
            },
            _ => Self::failed(cause, again),
        }
    }

    pub fn failed(cause: Option<Cause>, again: bool) -> Self {
        if again {
            Minutes::RedoFailed(cause)
        } else {
            Minutes::Failed(cause)
        }
    }
}

pub fn minutes(said: &Minutes, language: Language) -> Notice {
    use Language::{English, French};
    let entry = menu(language).minutes;
    let (title, body) = match (said, language) {
        (Minutes::Ready, English) => (
            "Minutes ready",
            if CLICKED {
                "Click to open it."
            } else {
                "They are in Kitokey’s window, beside the transcript."
            }
            .to_string(),
        ),
        (Minutes::ReadyKeptBeside(name), English) => (
            "Minutes ready",
            format!("The ones they replace could not go to the trash: they are kept beside them, as {name}."),
        ),
        (Minutes::ReadyKeptBeside(name), French) => (
            "Compte rendu prêt",
            format!("Celui qu’il remplace n’a pas pu aller à la corbeille\u{a0}: il est gardé à côté, sous {name}."),
        ),
        (Minutes::Ready, French) => (
            "Compte rendu prêt",
            if CLICKED {
                "Cliquez pour l’ouvrir."
            } else {
                "Il est dans la fenêtre de Kitokey, à côté de la transcription."
            }
            .to_string(),
        ),
        // Its cause named, the cause's gesture; else the menu's entry, which tries again.
        (Minutes::Failed(Some(cause)), English) => (
            "The minutes were not written",
            format!("The transcript is untouched. {}", self::cause(*cause, language)),
        ),
        (Minutes::Failed(Some(cause)), French) => (
            "Le compte rendu n’a pas été fait",
            format!("La transcription est intacte. {}", self::cause(*cause, language)),
        ),
        (Minutes::Failed(None), English) => (
            "The minutes were not written",
            format!("The transcript is untouched. “{entry}”, in Kitokey’s menu, tries again."),
        ),
        (Minutes::Failed(None), French) => (
            "Le compte rendu n’a pas été fait",
            format!("La transcription est intacte. «\u{a0}{entry}\u{a0}», dans le menu de Kitokey, le refait."),
        ),
        (Minutes::RedoFailed(cause), English) => (
            "The minutes were not made again",
            match cause {
                Some(cause) => format!("The previous ones stayed in place. {}", self::cause(*cause, language)),
                None => "The previous ones stayed in place.".to_string(),
            },
        ),
        (Minutes::RedoFailed(cause), French) => (
            "Le compte rendu n’a pas été refait",
            match cause {
                Some(cause) => format!("Celui d’avant est resté en place. {}", self::cause(*cause, language)),
                None => "Celui d’avant est resté en place.".to_string(),
            },
        ),
        // By the meeting's own model: named, and chosen again in « Avancé », as its page says it (D91).
        (
            Minutes::FailedOwn {
                cause,
                model,
                again: false,
            },
            English,
        ) => (
            "The minutes were not written",
            format!("The transcript is untouched. {}", cause_said(*cause, Some(model), language)),
        ),
        (
            Minutes::FailedOwn {
                cause,
                model,
                again: false,
            },
            French,
        ) => (
            "Le compte rendu n’a pas été fait",
            format!("La transcription est intacte. {}", cause_said(*cause, Some(model), language)),
        ),
        (Minutes::FailedOwn { cause, model, again: true }, English) => (
            "The minutes were not made again",
            format!("The previous ones stayed in place. {}", cause_said(*cause, Some(model), language)),
        ),
        (Minutes::FailedOwn { cause, model, again: true }, French) => (
            "Le compte rendu n’a pas été refait",
            format!("Celui d’avant est resté en place. {}", cause_said(*cause, Some(model), language)),
        ),
        (Minutes::Waiting(host), English) => (
            "The minutes wait for your agreement",
            format!("Nothing is sent to {host} before you agree, in Kitokey’s window."),
        ),
        (Minutes::Waiting(host), French) => (
            "Le compte rendu attend votre accord",
            format!("Rien n’est envoyé à {host} avant votre accord, dans la fenêtre de Kitokey."),
        ),
        (Minutes::Unchosen, English) => ("No one is chosen to write the minutes", "Choose in Kitokey’s settings.".to_string()),
        (Minutes::Unchosen, French) => (
            "Aucun modèle choisi pour les comptes rendus",
            "Choisissez-le dans les réglages de Kitokey.".to_string(),
        ),
        (Minutes::NoProvider(address), English) => (
            "The minutes were not written",
            format!("“{address}” is not a provider’s address: choose who writes the minutes in Kitokey’s settings."),
        ),
        (Minutes::NoProvider(address), French) => (
            "Le compte rendu n’a pas été fait",
            format!(
                "«\u{a0}{address}\u{a0}» n’est pas l’adresse d’un fournisseur\u{a0}: choisissez qui écrit le compte rendu dans les réglages de Kitokey."
            ),
        ),
        (Minutes::NoTranscript, English) => (
            "No minutes without a transcript",
            format!("The last recording has no whole transcript: “{}” first.", menu(language).transcribe),
        ),
        (Minutes::NoTranscript, French) => (
            "Pas de compte rendu sans transcription",
            format!(
                "Le dernier enregistrement n’a pas de transcription entière\u{a0}: «\u{a0}{}\u{a0}» d’abord.",
                menu(language).transcribe
            ),
        ),
        (Minutes::OllamaWaits { silent: true }, English) => (
            "The minutes wait for Ollama",
            "The meeting is confidential, and Ollama does not answer on this computer. It is free, at ollama.com.".to_string(),
        ),
        (Minutes::OllamaWaits { silent: true }, French) => (
            "Le compte rendu attend Ollama",
            "La réunion est confidentielle, et Ollama ne répond pas sur cet ordinateur. Il est gratuit, sur ollama.com.".to_string(),
        ),
        (Minutes::OllamaWaits { silent: false }, English) => (
            "The minutes wait for Ollama",
            format!(
                "The meeting is confidential, and Ollama holds no model yet: Kitokey’s window fetches {} in one click.",
                crate::providers::RECOMMENDED
            ),
        ),
        (Minutes::OllamaWaits { silent: false }, French) => (
            "Le compte rendu attend Ollama",
            format!(
                "La réunion est confidentielle, et Ollama n’a encore aucun modèle\u{a0}: la fenêtre de Kitokey télécharge {} en un clic.",
                crate::providers::RECOMMENDED
            ),
        ),
    };
    notice(title, body).expect("the minutes' end is always said")
}

/// Said when a transcript made again replaced one the trash refused: kept beside it; or, the
/// new one failed (`not_put_back`), when the one there could not be put back: where it is.
pub fn transcript_kept_beside(language: Language, name: &str, not_put_back: bool) -> Notice {
    let (title, body) = match (language, not_put_back) {
        (Language::English, false) => (
            "The previous transcript is kept",
            format!("It could not go to the trash: it is beside the new one, as {name}."),
        ),
        (Language::French, false) => (
            "L’ancienne transcription est gardée",
            format!("Elle n’a pas pu aller à la corbeille\u{a0}: elle est à côté de la nouvelle, sous {name}."),
        ),
        (Language::English, true) => (
            "The transcript could not be put back",
            format!("It is in the meeting’s folder, as {name}."),
        ),
        (Language::French, true) => (
            "La transcription n’a pas pu être remise en place",
            format!("Elle est dans le dossier de la réunion, sous {name}."),
        ),
    };
    Notice { title: title.into(), body }
}

/// What a recording's devices are, as its notices need them: each side with no device — not
/// opened at the start, or none left since (D31).
#[derive(Debug, Default)]
pub struct Sides {
    gone: [bool; 2],
    /// What each side is recorded from, to name a microphone that sends nothing.
    devices: crate::recording::Devices,
    /// The microphone's going was said: once a recording, a call taking and leaving it being no
    /// news each time. Its return is never said: the small window and the page show it (D66).
    me_out_said: bool,
    /// The microphone waits for its first device: taking it is no change.
    me_waited: bool,
    /// The microphone last named, by the start or by a notice.
    me_last: Option<String>,
}

impl Sides {
    /// The notice of what a recording said, as [`of`] says it, knowing the sides: a side gone
    /// says the other is still recorded only when it is; the meeting's sound back after none is
    /// recorded again.
    pub fn told(&mut self, said: &Said, reminder: bool, language: Language) -> Option<Notice> {
        self.devices.heard(said);
        match said {
            // None left, it was said: the zeros the recording keeps in its place are no news.
            Said::Nothing if self.gone[0] => return None,
            Said::Nothing => return sends_nothing(self.devices.names()[0].as_deref(), language),
            Said::Started(_, me, _) => self.me_last = me.clone(),
            // What stands in comes next: the notice says it already. Before the start, `Started` names it.
            Said::MicrophoneGone => self.me_waited = self.me_last.is_some(),
            Said::Unavailable(Source::Me, _) | Said::Lost(Source::Me) => {
                self.gone[0] = true;
                self.me_waited |= matches!(said, Said::Unavailable(..));
                if std::mem::replace(&mut self.me_out_said, true) {
                    return None;
                }
                return if self.gone[1] {
                    of(&Said::Lost(Source::Me), reminder, language)
                } else {
                    Some(lost_beside(Source::Me, language))
                };
            }
            Said::Moved(Source::Me, name) => {
                self.gone[0] = false;
                let first = std::mem::take(&mut self.me_waited);
                let same = *name == std::mem::replace(&mut self.me_last, name.clone());
                if first || same {
                    return None;
                }
            }
            Said::Unavailable(Source::Others, _) => self.gone[1] = true,
            Said::Lost(Source::Others) => {
                self.gone[1] = true;
                if !self.gone[0] {
                    return Some(lost_beside(Source::Others, language));
                }
            }
            Said::Moved(Source::Others, name) if self.gone[1] => {
                self.gone[1] = false;
                return Some(back(name.as_deref(), language));
            }
            _ => {}
        }
        of(said, reminder, language)
    }
}

/// The microphone sends nothing at all (#361): named when the recording said which one it is.
fn sends_nothing(name: Option<&str>, language: Language) -> Option<Notice> {
    let body = match (name, language) {
        (Some(name), Language::English) => format!(
            "Nothing at all has come from {name} for ten seconds, not even a room’s faint noise: check it is not muted on the device itself, and that it is ready. The recording goes on."
        ),
        (None, Language::English) => "Nothing at all has come from your microphone for ten seconds, not even a room’s faint noise: check it is not muted on the device itself, and that it is ready. The recording goes on.".to_string(),
        (Some(name), Language::French) => format!(
            "Rien du tout n’arrive du micro {name} depuis dix secondes, pas même le léger bruit d’une pièce\u{a0}: vérifiez qu’il n’est pas coupé sur l’appareil lui-même, et qu’il est prêt. L’enregistrement continue."
        ),
        (None, Language::French) => "Rien du tout n’arrive de votre micro depuis dix secondes, pas même le léger bruit d’une pièce\u{a0}: vérifiez qu’il n’est pas coupé sur l’appareil lui-même, et qu’il est prêt. L’enregistrement continue.".to_string(),
    };
    let title = match language {
        Language::English => "Your microphone sends nothing",
        Language::French => "Votre micro ne transmet rien",
    };
    notice(title, body)
}

/// A side with no device left while the other is still recorded.
fn lost_beside(source: Source, language: Language) -> Notice {
    let (title, body) = match (source, language) {
        (Source::Me, Language::English) => ("Your microphone is not recorded for now", "The meeting’s sound is still recorded."),
        (Source::Me, Language::French) => (
            "Votre micro n’est pas enregistré pour l’instant",
            "Le son de la réunion est toujours enregistré.",
        ),
        (Source::Others, Language::English) => (
            "The meeting’s sound is not recorded for now",
            "Your computer has no sound output left. Your microphone is still recorded, and an output plugged in is recorded at once.",
        ),
        (Source::Others, Language::French) => (
            "Le son de la réunion n’est plus enregistré pour l’instant",
            "Votre ordinateur n’a plus de sortie audio. Votre micro est toujours enregistré, et une sortie branchée est enregistrée aussitôt.",
        ),
    };
    Notice {
        title: title.into(),
        body: body.into(),
    }
}

/// Said when the meeting's sound, with no device left, is recorded again, from this one (D31).
pub fn back(name: Option<&str>, language: Language) -> Notice {
    let (title, body) = match language {
        Language::English => (
            "The meeting’s sound is recorded again",
            name.map_or("From the output your computer uses now.".into(), |name| format!("From {name}.")),
        ),
        Language::French => (
            "Le son de la réunion est de nouveau enregistré",
            name.map_or("Depuis la sortie qu’utilise maintenant votre ordinateur.".into(), |name| {
                format!("Depuis {name}.")
            }),
        ),
    };
    Notice { title: title.into(), body }
}

/// The offer to record a call another application started (D30): the notice, naming the
/// application, and its three gestures — record, not this time, never offer.
pub fn offer(application: &str, language: Language) -> (Notice, [&'static str; 3]) {
    match language {
        Language::English => (
            Notice {
                title: format!("{application} is using your microphone"),
                body: "Record this call with Kitokey?".into(),
            },
            ["Record", "Not this time", "Never offer"],
        ),
        Language::French => (
            Notice {
                title: format!("{application} utilise votre micro"),
                body: "Enregistrer cet appel avec Kitokey\u{202f}?".into(),
            },
            ["Enregistrer", "Pas cette fois", "Ne plus proposer"],
        ),
    }
}

/// Said after a recording's stop when its transcript is made next, without another gesture.
pub fn follows(language: Language) -> &'static str {
    match language {
        Language::English => "The transcript follows.",
        Language::French => "La transcription suit.",
    }
}

/// The menu's entry that fetches the default model: what it is, how big, and from where — the
/// user's click on it is their agreement.
pub fn download(language: Language, size: &str, host: &str) -> String {
    match language {
        Language::English => format!("Download the model ({size}, {host})"),
        Language::French => format!("Télécharger le modèle ({size}, {host})"),
    }
}

/// The same entry while the model comes.
pub fn downloading(language: Language, percent: u64) -> String {
    match language {
        Language::English => format!("Downloading the model… {percent} %"),
        Language::French => format!("Téléchargement du modèle… {percent}\u{a0}%"),
    }
}

/// The entry while the model comes, from the bytes held: the share come — then, once every
/// byte is there, that its fingerprint is being checked.
pub fn fetching(language: Language, held: u64, bytes: u64) -> String {
    if held >= bytes {
        return match language {
            Language::English => "Checking the model…",
            Language::French => "Vérification du modèle…",
        }
        .to_string();
    }
    downloading(language, held * 100 / bytes.max(1))
}

/// Why the system's trash did not take the audio, in the user's words rather than the system's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    NotAllowed,
    NoSpace,
    Other,
}

/// Said when the audio of meetings kept longer than the settings say went to the system's trash;
/// or, `Err`, why it could not.
pub fn audio_trashed(language: Language, outcome: &Result<(usize, u32), Refusal>) -> Notice {
    let (title, body) = match (language, outcome) {
        (Language::English, Ok((1, days))) => (
            "Audio moved to the trash".to_string(),
            format!(
                "The audio of 1 meeting kept more than {} is in the trash; its transcript and minutes stay.",
                english_days(*days)
            ),
        ),
        (Language::English, Ok((meetings, days))) => (
            "Audio moved to the trash".to_string(),
            format!(
                "The audio of {meetings} meetings kept more than {} is in the trash; their transcripts and minutes stay.",
                english_days(*days)
            ),
        ),
        (Language::French, Ok((1, days))) => (
            "Audio mis à la corbeille".to_string(),
            format!(
                "L’audio d’une réunion gardé plus {} est dans la corbeille\u{202f}; sa transcription et son compte rendu restent.",
                french_days(*days)
            ),
        ),
        (Language::French, Ok((meetings, days))) => (
            "Audio mis à la corbeille".to_string(),
            format!(
                "L’audio de {meetings} réunions gardé plus {} est dans la corbeille\u{202f}; leurs transcriptions et leurs comptes rendus restent.",
                french_days(*days)
            ),
        ),
        // The trash may have taken some before it refused: what is said holds either way.
        (Language::English, Err(refusal)) => (
            "The audio was not all moved to the trash".to_string(),
            format!(
                "{} What was not moved stays where it was; nothing was deleted.",
                match refusal {
                    Refusal::NotAllowed => "The trash refused access.",
                    Refusal::NoSpace => "The trash has no room left.",
                    Refusal::Other => "The system's trash did not take it.",
                }
            ),
        ),
        (Language::French, Err(refusal)) => (
            "L’audio n’a pas été entièrement mis à la corbeille".to_string(),
            format!(
                "{} Ce qui n’a pas été déplacé reste à sa place\u{202f}; rien n’a été supprimé.",
                match refusal {
                    Refusal::NotAllowed => "La corbeille a refusé l’accès.",
                    Refusal::NoSpace => "La corbeille n’a plus de place.",
                    Refusal::Other => "La corbeille du système ne l’a pas pris.",
                }
            ),
        ),
    };
    Notice { title, body }
}

/// What is left of the trial, said in Settings (D83): its days and its meetings; the days alone once
/// the meetings are made, the meetings alone once the days have passed.
pub fn trial_left(left: Left, language: Language) -> String {
    let counted = |count: u32, one: &str, many: &str| {
        if count == 1 {
            format!("1 {one}")
        } else {
            format!("{count} {many}")
        }
    };
    let (days, meetings) = match language {
        Language::English => (counted(left.days, "day", "days"), counted(left.meetings, "meeting", "meetings")),
        Language::French => (counted(left.days, "jour", "jours"), counted(left.meetings, "réunion", "réunions")),
    };
    match (language, left.days, left.meetings) {
        (Language::English, 0, _) => format!("{meetings} left."),
        (Language::English, _, 0) => format!("{days} left."),
        (Language::English, ..) => format!("{days} and {meetings} left; the trial ends once both are used up."),
        (Language::French, 0, _) => format!("Il reste {meetings}."),
        (Language::French, _, 0) => format!("Il reste {days}."),
        (Language::French, ..) => format!("Il reste {days} et {meetings}\u{202f}; l’essai prend fin une fois les deux épuisés."),
    }
}

fn english_days(days: u32) -> String {
    if days == 1 { "1 day".to_string() } else { format!("{days} days") }
}

fn french_days(days: u32) -> String {
    if days == 1 {
        "d’un jour".to_string()
    } else {
        format!("de {days} jours")
    }
}

/// Said when a transcript needs the default model and it is not on the machine: nothing is
/// fetched until the user chooses the menu's entry.
/// Wholeheard became Kitokey (D86), as Summit became Wholeheard (D28): said once, at the first start, when something was carried over.
pub fn carried_over(language: Language) -> Notice {
    let (title, body) = match language {
        Language::English => (
            "Wholeheard is now Kitokey",
            "Your settings, your meetings and the transcription model are carried over; the keyboard shortcut may be asked once more.",
        ),
        Language::French => (
            "Wholeheard s’appelle maintenant Kitokey",
            "Vos réglages, vos réunions et le modèle de transcription sont repris\u{202f}; le raccourci clavier peut être redemandé une fois.",
        ),
    };
    Notice {
        title: title.into(),
        body: body.into(),
    }
}

/// What the user held under an earlier name could not all be carried over: left where they were, tried again next start.
/// Said in plain words: the technical reason goes to what Kitokey writes for whoever debugs it.
pub fn carry_failed(language: Language) -> Notice {
    let (title, body) = match language {
        Language::English => (
            "Wholeheard’s settings were not all carried over",
            "Nothing is lost: they stay where they were, and it is tried again at the next start.",
        ),
        Language::French => (
            "Les réglages de Wholeheard n’ont pas tous été repris",
            "Rien n’est perdu\u{a0}: ils restent où ils étaient, et la reprise est tentée de nouveau au prochain démarrage.",
        ),
    };
    Notice {
        title: title.into(),
        body: body.into(),
    }
}

pub fn model_needed(language: Language, size: &str, host: &str) -> Notice {
    let entry = download(language, size, host);
    let (title, body) = match language {
        Language::English => (
            "The transcription model is needed",
            format!("It is fetched once, on your word: choose “{entry}” in Kitokey’s menu. The recording is kept."),
        ),
        Language::French => (
            "Le modèle de transcription est nécessaire",
            format!(
                "Il est téléchargé une fois, sur votre accord\u{a0}: choisissez «\u{a0}{entry}\u{a0}» dans le menu de Kitokey. L’enregistrement est conservé."
            ),
        ),
    };
    notice(title, body).expect("always said")
}

/// Said when Ollama's pull of a model ended: there, offered first for the minutes — or not, and
/// offered again (D37).
pub fn pulled(outcome: &Result<(), String>, model: &str, language: Language) -> Notice {
    let (title, body) = match (outcome, language) {
        (Ok(()), Language::English) => ("Model ready", format!("{model} is in Ollama: it is offered first for the minutes.")),
        (Ok(()), Language::French) => (
            "Modèle prêt",
            format!("{model} est dans Ollama\u{a0}: il est proposé en premier pour les comptes rendus."),
        ),
        (Err(_), Language::English) => ("The model was not downloaded", "Nothing of it is used. It is offered again.".into()),
        (Err(_), Language::French) => (
            "Le modèle n’a pas été téléchargé",
            "Rien n’en est utilisé. Il est proposé à nouveau.".into(),
        ),
    };
    notice(title, body).expect("always said")
}

/// What a model of Settings' list is good for, as the list says it (D37, D40).
pub fn offered_good(language: Language, good: Good) -> &'static str {
    match (language, good) {
        (Language::English, Good::Recommended) => "Recommended: the most exact, quick on a recent computer",
        (Language::French, Good::Recommended) => "Recommandé\u{a0}: le plus juste, rapide sur un ordinateur récent",
        (Language::English, Good::Faster) => "Faster, less exact: for a modest computer",
        (Language::French, Good::Faster) => "Plus rapide, moins juste\u{a0}: pour un ordinateur modeste",
    }
}

/// Said when the model's fetch ended: ready, and the transcript begun — or not, with why, and
/// that nothing of it was used.
pub fn fetched(outcome: &Result<(), String>, chosen: bool, language: Language) -> Notice {
    use Language::{English, French};
    let (title, body) = match (outcome, chosen, language) {
        (Ok(()), true, English) => (
            "Model ready",
            "Checked against its published fingerprint. The transcript is being made.",
        ),
        (Ok(()), true, French) => ("Modèle prêt", "Vérifié par son empreinte publiée. La transcription est en cours."),
        // Another than the one chosen: the choice stays the user's (D37).
        (Ok(()), false, English) => (
            "Model ready",
            "Checked against its published fingerprint. Choose it in Settings for your next transcripts.",
        ),
        (Ok(()), false, French) => (
            "Modèle prêt",
            "Vérifié par son empreinte publiée. Choisissez-le dans les Réglages pour vos prochaines transcriptions.",
        ),
        (Err(_), _, English) => ("The model was not downloaded", "Nothing of it is used. It is offered again."),
        (Err(_), _, French) => (
            "Le modèle n’a pas été téléchargé",
            "Rien n’en est utilisé. Il est proposé à nouveau.",
        ),
    };
    notice(title, body.to_string()).expect("always said")
}

/// The sentence « Copier l'annonce » puts on the clipboard, for the call's chat (D50): in the
/// meeting's language when it says French or English, else in the window's.
pub fn announcement(meeting: Option<&str>, window: Language) -> &'static str {
    let language = match meeting {
        Some("fr") => Language::French,
        Some("en") => Language::English,
        _ => window,
    };
    match language {
        Language::French => {
            "J’enregistre cette réunion pour en faire le compte rendu. Dites-le-moi si vous préférez que je ne le fasse pas."
        }
        Language::English => "I’m recording this meeting to write up its minutes. Tell me if you’d rather I didn’t.",
    }
}
