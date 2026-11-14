//! What the user is told, in one place: every sentence of the shell is here, in each language
//! it speaks, so that it is read as a whole and a language is added by adding its sentences.

use crate::{
    recording::{Said, Source},
    registry::Good,
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
    /// Shows: the window, as units. When: after every size (o, ko, Mo, Go). At most: about 3
    /// characters. Placeholders: none.
    pub units: [&'static str; 4],
    /// What a screen reader calls the window's sections.
    /// Shows: the sidebar, as a name for a screen reader. When: screen reader: name of the
    /// sidebar's icon navigation. At most: no limit. Placeholders: none.
    pub sections: &'static str,
    /// Shows: the sidebar, as a name for a screen reader. When: screen reader: name of the sidebar.
    /// At most: no limit. Placeholders: none.
    pub meetings: &'static str,
    /// How many meetings weigh the size beside it (board 4's foot).
    /// Shows: the sidebar's foot, as a status. When: always, under the list: how many meetings
    /// weigh the size beside it; plural. At most: about 24 characters. Placeholders: {} the number
    /// of meetings.
    pub weight: &'static str,
    /// The same, for one meeting.
    /// Shows: the sidebar's foot, as a status. When: same, exactly one meeting. At most: about 24
    /// characters. Placeholders: {} the number of meetings, 1.
    pub weight_one: &'static str,
    /// A picture of the two weights, for whoever does not see it.
    /// Shows: the sidebar's foot, as a name for a screen reader. When: screen reader: description
    /// of the audio/text bar. At most: no limit. Placeholders: {} the audio's size, then the
    /// texts'.
    pub weight_described: &'static str,
    /// Shows: the sidebar, as a heading. When: group heading in the meeting list. At most: about 24
    /// characters. Placeholders: none.
    pub this_week: &'static str,
    /// Shows: the sidebar, as a heading. When: group heading in the meeting list. At most: about 24
    /// characters. Placeholders: none.
    pub last_week: &'static str,
    /// Earlier in the current month: the month's name.
    /// Shows: the sidebar, as a heading. When: group heading: earlier this month. At most: about 24
    /// characters. Placeholders: {} month name.
    pub earlier_in: &'static str,
    /// Sessions whose start cannot be read as a day.
    /// Shows: the sidebar, as a heading. When: group heading for meetings whose date cannot be
    /// read. At most: about 24 characters. Placeholders: none.
    pub undated: &'static str,
    /// Shows: the sidebar, as a list label. When: on a meeting's row when it has minutes. At most:
    /// 18 characters. Placeholders: none.
    pub minutes: &'static str,
    /// Shows: the sidebar, as a list label. When: on a meeting's row when it has a transcript.
    /// At most: 18 characters. Placeholders: none.
    pub transcript: &'static str,
    /// Shows: the sidebar, as a list label. When: on a meeting's row: audio and its size. At most:
    /// 18 characters. Placeholders: {} the audio's size.
    pub audio: &'static str,
    /// Shows: the sidebar, as a list label. When: on a meeting's row whose audio was deleted.
    /// At most: 18 characters. Placeholders: none.
    pub audio_freed: &'static str,
    /// What a meeting's audio weighs, beside its length in its header (D89).
    /// Shows: a meeting's page, as a status. When: header of a meeting, beside its duration: audio
    /// size. At most: about 16 characters. Placeholders: {} the audio's size.
    pub audio_weighs: &'static str,
    /// Shows: the sidebar, as a list label. When: on the row of the meeting being recorded.
    /// At most: 18 characters. Placeholders: none.
    pub recording: &'static str,
    /// Shows: the sidebar, as a list label. When: on the row of a recording that ended abnormally.
    /// At most: 18 characters. Placeholders: none.
    pub interrupted: &'static str,
    /// No session in the folder: the folder.
    /// Shows: the window with no meeting, as a heading. When: no meeting in the folder yet.
    /// At most: no limit. Placeholders: {} folder.
    pub none: &'static str,
    /// The sidebar's search (D61): what its field says, and none of the meetings holding what is typed.
    /// No meeting yet: what to do.
    /// Shows: the window with no meeting, as a help line. When: under the empty-state title: how to
    /// start. At most: no limit. Placeholders: none.
    pub none_do: &'static str,
    /// Shows: the window with no meeting, as an error. When: the meetings folder cannot be read.
    /// At most: no limit. Placeholders: {} folder.
    pub list_unread: &'static str,
    /// Shows: the window with no meeting, as a help line. When: under that error: what to do.
    /// At most: no limit. Placeholders: none.
    pub list_unread_do: &'static str,
    /// Shows: a meeting's page, as an error. When: "Try again" on a failed step was refused.
    /// At most: no limit. Placeholders: {} failure sentence.
    pub again_refused: &'static str,
    pub hearing_models_refused: &'static str,
    /// Shows: the sidebar, as an error. When: the folded/unfolded state could not be saved.
    /// At most: no limit. Placeholders: {} failure sentence.
    pub fold_not_kept: &'static str,
    /// Shows: the agreement before a meeting leaves, as an error. When: sending to the online
    /// service was refused. At most: no limit. Placeholders: {} failure sentence.
    pub agreement_refused: &'static str,
    /// Shows: a meeting's page, as an error. When: a meeting's page could not open. At most: no
    /// limit. Placeholders: {} failure sentence.
    pub meeting_not_opened: &'static str,
    /// Shows: the window, as an error. When: a page failed to render. At most: no limit.
    /// Placeholders: none.
    pub page_not_shown: &'static str,
    /// Shows: a meeting's page, as an error. When: the meeting context could not be re-read.
    /// At most: no limit. Placeholders: {} failure sentence.
    pub context_not_read: &'static str,
    /// Shows: the sidebar and the recording bar, as a button. When: record button label while the
    /// recording starts. At most: about 24 characters. Placeholders: none.
    pub record_starting: &'static str,
    /// Shows: the sidebar and the recording bar, as a button. When: record button label while the
    /// recording stops. At most: about 24 characters. Placeholders: none.
    pub record_stopping: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a button. When: closes the Redo panel. At most:
    /// about 16 characters. Placeholders: none.
    pub redo_cancel: &'static str,
    pub saved: &'static str,
    /// Shows: the recording bar, as a help line. When: the user wanted the meeting confidential
    /// from the bar, but it is not. At most: no limit. Placeholders: none.
    pub small_not_confidential: &'static str,
    /// A recording refused, the trial over (PDR-0005, D83): why and what stays, in one sentence; the one gesture.
    /// Shows: the sidebar, as an error. When: recording refused: the trial is over. At most: no
    /// limit. Placeholders: none.
    pub trial_over: &'static str,
    /// Shows: the sidebar, as a button. When: beside trial_over. At most: about 24 characters.
    /// Placeholders: none.
    pub buy: &'static str,
    /// Settings on a trial licence (D83): the name of what they say is left of it (`trial_left`).
    pub trial: &'static str,
    /// Shows: the sidebar, as a field's placeholder. When: search field over the meeting list.
    /// At most: about 26 characters. Placeholders: none.
    pub find_meetings: &'static str,
    /// The sidebar folded and brought back (D61): each button's name; the shortcut is added beside.
    /// Shows: the sidebar, as a name for a screen reader. When: icon button (tooltip + screen
    /// reader) that folds the sidebar; shortcut appended. At most: no limit. Placeholders: none.
    pub fold: &'static str,
    /// Shows: any page of the window, as a name for a screen reader. When: icon button (tooltip +
    /// screen reader) that brings the sidebar back. At most: no limit. Placeholders: none.
    pub unfold: &'static str,
    /// Shows: the sidebar, as a status. When: search finds no meeting. At most: no limit.
    /// Placeholders: none.
    pub none_found: &'static str,
    /// One meeting's page.
    /// Shows: any page of the window, as a link. When: back link to the meeting list. At most:
    /// about 16 characters. Placeholders: none.
    pub back: &'static str,
    /// Shows: a meeting's page, as a name for a screen reader. When: icon button (tooltip): open
    /// the meeting's folder in the file manager. At most: no limit. Placeholders: none.
    pub open_folder: &'static str,
    /// The menu of a meeting's rarer actions (D89), and a setting's name before its value.
    /// Shows: a meeting's page, as a name for a screen reader. When: icon button (tooltip) of the
    /// menu of rarer actions. At most: no limit. Placeholders: none.
    pub more_actions: &'static str,
    /// Shows: the window, as a format. When: a setting's name before its value: "{}:". At most: no
    /// limit. Placeholders: {} a setting's name.
    pub labelled: &'static str,
    /// Shows: a meeting's page, as a heading. When: heading of the minutes card. At most: about 20
    /// characters. Placeholders: none.
    pub minutes_title: &'static str,
    /// Shows: a meeting's page, as a name for a screen reader. When: name of the transcript card.
    /// At most: about 20 characters. Placeholders: none.
    pub transcript_title: &'static str,
    /// Shows: a meeting's page, as a field's placeholder. When: search field over the transcript.
    /// At most: about 30 characters. Placeholders: none.
    pub search: &'static str,
    /// The minutes' maker: the model, the provider, when.
    /// Shows: a meeting's page, as a status. When: under minutes: the model that wrote them.
    /// At most: no limit. Placeholders: {} model name.
    pub written_by: &'static str,
    /// Its provider, when known, said after the model.
    /// Shows: a meeting's page, as a status. When: appended after written_by: the provider.
    /// At most: no limit. Placeholders: {} provider.
    pub written_via: &'static str,
    /// Between a day and its hour: "mar. 13 oct. à 12:21", "Tue, Oct 13 at 12:21 PM".
    /// Shows: a meeting's page, as a format. When: between a day and its hour. At most: no limit.
    /// Placeholders: none.
    pub at: &'static str,
    /// The transcript's maker: the model, on this computer.
    /// Shows: a meeting's page, as a status. When: under the transcript: the model that made it.
    /// At most: no limit. Placeholders: {} model.
    pub heard_by: &'static str,
    /// Shows: a meeting's page, as a label. When: speaker side in the transcript: the user's
    /// microphone. At most: about 12 characters. Placeholders: none.
    pub here: &'static str,
    /// Shows: a meeting's page, as a label. When: speaker side in the transcript: the other
    /// participants. At most: about 12 characters. Placeholders: none.
    pub remote: &'static str,
    /// A voice told within a track: its label.
    /// Shows: a meeting's page, as a label. When: a voice told apart within a side. At most: about
    /// 12 characters. Placeholders: {} number.
    pub voice: &'static str,
    /// Shows: a meeting's page, as a label. When: legend of the speaking-time drawing. At most: no
    /// limit. Placeholders: none.
    pub here_described: &'static str,
    /// Shows: a meeting's page, as a label. When: legend of the speaking-time drawing. At most: no
    /// limit. Placeholders: none.
    pub remote_described: &'static str,
    /// Who spoke over the meeting, for whoever does not see the drawing.
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: the speaking-
    /// time drawing. At most: no limit. Placeholders: none.
    pub who_spoke: &'static str,
    /// Shows: a meeting's page, as a status. When: transcript card, none yet. At most: no limit.
    /// Placeholders: none.
    pub no_transcript: &'static str,
    /// Shows: a meeting's page, as a status. When: minutes card, none yet. At most: no limit.
    /// Placeholders: none.
    pub no_minutes: &'static str,
    /// Shows: a meeting's page, as a help line. When: transcript still being written: what is shown
    /// is partial. At most: no limit. Placeholders: none.
    pub being_made: &'static str,
    /// Shows: a meeting's page, as an error. When: a file of a newer/older format cannot be read.
    /// At most: no limit. Placeholders: none.
    pub unreadable: &'static str,
    /// Lines of a transcript that are not the contract's, left out: how many.
    /// Shows: a meeting's page, as a help line. When: some lines of the transcript file are
    /// skipped; plural. At most: no limit. Placeholders: {} count.
    pub unread: &'static str,
    /// Shows: a meeting's page, as a status. When: search in the transcript finds nothing. At most:
    /// no limit. Placeholders: none.
    pub no_match: &'static str,
    /// The settings' page.
    /// Shows: the sidebar, as a name for a screen reader. When: icon link to Settings (tooltip) and
    /// Settings page title. At most: no limit. Placeholders: none.
    pub settings: &'static str,
    pub after_title: &'static str,
    /// Each choice of what follows a recording: its name, then what it does.
    pub after_nothing: [&'static str; 2],
    pub after_transcript: [&'static str; 2],
    pub after_minutes: [&'static str; 2],
    pub models_title: &'static str,
    /// What the models' card says under its title, and under its steps.
    pub models_intro: &'static str,
    pub models_note: &'static str,
    /// The transcription model's state, at the page's end.
    pub model_state: &'static str,
    pub model_ready: &'static str,
    pub model_here: &'static str,
    pub model_missing: &'static str,
    /// One the user named, not on this computer: theirs, nothing offered (D36).
    pub model_named_missing: &'static str,
    pub model_coming: &'static str,
    /// Shows: a meeting's page, as a button. When: download the transcription model. At most: about
    /// 40 characters. Placeholders: {} host.
    pub model_offer: &'static str,
    /// The short list of models to download (D37): its heading, the gesture, how far, and the
    /// model chosen that is not here yet, in the choice.
    pub offered_title: &'static str,
    pub offered_fetch: &'static str,
    pub offered_far: &'static str,
    pub model_chosen_missing: &'static str,
    /// The minutes' model of the list, fetched by Ollama (D37): its size, what it is good for,
    /// and Ollama needed when none answers.
    pub offered_ollama_size: &'static str,
    pub offered_ollama_good: &'static str,
    pub offered_needs_ollama: &'static str,
    /// The list read again on the user's gesture (D40): the button, its host filled in; and
    /// what is said when it could not be.
    pub offered_refresh: &'static str,
    pub offered_refresh_failed: &'static str,
    pub offered_refresh_waits: &'static str,
    /// Apple's SpeechAnalyzer among who hears, on a Mac with macOS 26 (D42): its name in the
    /// choice; its model not on the Mac yet; the gesture that has macOS fetch it, and from whom;
    /// while it comes; when it did not.
    pub speech_option: &'static str,
    pub speech_missing: &'static str,
    pub speech_fetch: &'static str,
    pub speech_fetching: &'static str,
    pub speech_failed: &'static str,
    /// A service that hears (D43): the choice's group, its fields, the agreement before anything is
    /// sent to its host, the gestures, and what they say.
    pub service_group: &'static str,
    pub service_option: &'static str,
    /// A model the service's list does not hold, typed (D63).
    pub service_other_model: &'static str,
    pub service_address: &'static str,
    pub service_model: &'static str,
    pub service_key: &'static str,
    pub service_agree: &'static str,
    pub service_save: &'static str,
    pub service_test: &'static str,
    pub service_tested: &'static str,
    pub service_saved: &'static str,
    pub service_agree_first: &'static str,
    pub service_address_bad: &'static str,
    pub service_model_named: &'static str,
    /// The model's name as people say it, and its size once installed.
    pub installed: &'static str,
    /// Another number of days, typed; and what is said of one that is none.
    pub keep_other: &'static str,
    pub keep_not_days: &'static str,
    /// The engine that hears, shipped with Kitokey; and the user's home, as a folder is said.
    pub builtin: &'static str,
    pub home: &'static str,
    pub transcription: &'static str,
    pub on_this_computer: &'static str,
    pub reminder: [&'static str; 2],
    /// When an application takes the microphone (D30): its name and line, as board 4 has them.
    pub offer: [&'static str; 2],
    /// Its two choices: offer to record, do nothing.
    pub offer_choices: [&'static str; 2],
    /// Started with the user's session (D32): its name and line.
    pub autostart: [&'static str; 2],
    /// On Windows, the switch held (D84): by the user in Windows' Startup settings, where it is
    /// turned back on; by a policy.
    pub autostart_held: [&'static str; 2],
    pub keep_audio: [&'static str; 2],
    /// The settings' last card, and its one switch: what a program said of a failure, shown on
    /// the page under the sentence (D29).
    pub advanced_title: &'static str,
    pub developer: [&'static str; 2],
    /// Under a making not made, in developer mode: what its program said ({}).
    /// Shows: a meeting's page, as a status. When: developer mode: raw program message under a
    /// failure. At most: no limit. Placeholders: {} raw text.
    pub program_said: &'static str,
    /// A meeting's minutes, their three sections as the page names them (D11's second board),
    /// whatever titles the model wrote: `minutes/v1` holds them in this order.
    /// Shows: a meeting's page, as a heading. When: the three section titles of minutes. At most:
    /// about 20 characters. Placeholders: none.
    pub minutes_sections: [&'static str; 3],
    /// The actions' three cells, named for assistive technology alone (D46), and what a meeting
    /// left unsettled, as the summary writes it.
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: column names of
    /// the actions table. At most: about 12 characters. Placeholders: none.
    pub actions_cells: [&'static str; 3],
    /// Shows: a meeting's page, as a status. When: written by the summary model when an action has
    /// no owner/date; MATCHED by code. At most: no limit. Placeholders: none.
    pub unsettled: &'static str,
    /// A model of this computer that holds the minutes' form, as Settings offer it ({}).
    pub model_recommended: &'static str,
    /// A meeting's player (D26): its button, both ways, and why a meeting has none.
    /// Shows: a meeting's page, as a button. When: player button at rest. At most: about 12
    /// characters. Placeholders: none.
    pub play: &'static str,
    /// Shows: a meeting's page, as a button. When: player button while playing. At most: about 12
    /// characters. Placeholders: none.
    pub pause: &'static str,
    /// Shows: a meeting's page, as a help line. When: audio deleted: no player. At most: no limit.
    /// Placeholders: none.
    pub no_player: &'static str,
    /// Its audio is there, and no track of it is one this version plays.
    /// Shows: a meeting's page, as a help line. When: audio present but no playable track. At most:
    /// no limit. Placeholders: none.
    pub not_playable: &'static str,
    /// A line of the minutes' time, as a screen reader says it (D75): what a click on it does.
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: a time in the
    /// minutes, click plays from there. At most: no limit. Placeholders: {} time.
    pub time_listen: &'static str,
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: a time in the
    /// minutes, click shows the transcript passage. At most: no limit. Placeholders: {} time.
    pub time_passage: &'static str,
    /// A track the window could not play; and the drawing as the playback's slider, named.
    /// Shows: a meeting's page, as an error. When: playback failed. At most: no limit.
    /// Placeholders: none.
    pub not_played: &'static str,
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: the playback
    /// slider. At most: no limit. Placeholders: none.
    pub playback: &'static str,
    /// In developer mode: the logs' level, the logs' folder, a meeting's diagnostic copied.
    pub log_level: [&'static str; 2],
    pub log_info: &'static str,
    pub log_debug: &'static str,
    pub logs: [&'static str; 2],
    pub open_logs: &'static str,
    pub logs_not_opened: &'static str,
    /// Shows: a meeting's page, as a menu entry. When: developer: menu item copies a diagnostic.
    /// At most: about 30 characters. Placeholders: none.
    pub copy_diagnostic: &'static str,
    /// Shows: a meeting's page, as a status. When: after copying. At most: about 24 characters.
    /// Placeholders: none.
    pub diagnostic_copied: &'static str,
    /// Shows: a meeting's page, as a status. When: nothing to copy. At most: no limit.
    /// Placeholders: none.
    pub diagnostic_empty: &'static str,
    /// Shows: a meeting's page, as an error. When: copy failed. At most: about 24 characters.
    /// Placeholders: none.
    pub diagnostic_not_copied: &'static str,
    /// The folder the sessions go to, and the button that opens the system's dialog for it.
    pub sessions_folder: [&'static str; 2],
    pub choose: &'static str,
    /// The recording bar, its icon in the sidebar's foot (D89), what it says (D90).
    /// Shows: the sidebar, as a name for a screen reader. When: icon button (tooltip): open the
    /// recording bar. At most: no limit. Placeholders: none.
    pub small: &'static str,
    /// Shows: the recording bar and « Vérifier le son », as a label. When: meter name: the user's
    /// mic. At most: about 20 characters. Placeholders: none.
    pub small_mic: &'static str,
    /// Shows: the recording bar and « Vérifier le son », as a label. When: meter name: the
    /// meeting's sound. At most: about 20 characters. Placeholders: none.
    pub small_meeting: &'static str,
    /// Shows: the recording bar, as a help line. When: during recording: what happens at stop.
    /// At most: no limit. Placeholders: none.
    pub small_note: &'static str,
    /// What the bar says of each side (D49, D90): silent two minutes, never as an error; a
    /// microphone that sends nothing, as the notifications say it; a side with no device.
    /// Shows: the recording bar, as a status. When: mic silent for 2 min (not an error). At most:
    /// about 60 characters. Placeholders: none.
    pub bar_silent_me: &'static str,
    /// Shows: the recording bar, as a status. When: meeting sound silent for 2 min. At most: about
    /// 60 characters. Placeholders: none.
    pub bar_silent_others: &'static str,
    /// Shows: the recording bar, as an error. When: mic sends pure zeros. At most: no limit.
    /// Placeholders: none.
    pub small_nothing: &'static str,
    /// Shows: the recording bar, as a status. When: no mic device for now. At most: no limit.
    /// Placeholders: none.
    pub small_waiting_me: &'static str,
    /// Shows: the recording bar, as a status. When: no output device for now. At most: no limit.
    /// Placeholders: none.
    pub small_waiting_others: &'static str,
    /// The reminder said where the user looks, the first seconds of a recording (D50).
    /// Shows: the recording bar, as a help line. When: first seconds of recording: tell
    /// participants. At most: no limit. Placeholders: none.
    pub small_announce: &'static str,
    /// Shows: the recording bar, as a button. When: copies a ready sentence for the call's chat.
    /// At most: about 24 characters. Placeholders: none.
    pub copy_announcement: &'static str,
    /// Shows: the recording bar, as an error. When: copy failed. At most: about 16 characters.
    /// Placeholders: none.
    pub announcement_not_copied: &'static str,
    /// Shows: the recording bar, as a name for a screen reader. When: icon button (tooltip): open
    /// the full window. At most: no limit. Placeholders: none.
    pub small_full: &'static str,
    /// The bar's options (D90): opened under its row, closed again.
    /// Shows: the recording bar, as a name for a screen reader. When: icon button (tooltip): unfold
    /// options. At most: no limit. Placeholders: none.
    pub bar_more: &'static str,
    /// Shows: the recording bar, as a name for a screen reader. When: icon button (tooltip): fold
    /// options. At most: no limit. Placeholders: none.
    pub bar_less: &'static str,
    /// Shows: the recording bar, as a name for a screen reader. When: icon button (tooltip): hide
    /// the bar. At most: no limit. Placeholders: none.
    pub bar_hide: &'static str,
    /// Shows: the recording bar, as a label. When: microphone picker. At most: no limit.
    /// Placeholders: {} mic name.
    pub small_microphone: &'static str,
    /// Shows: the recording bar, as a name for a screen reader. When: screen reader: the mic
    /// picker. At most: no limit. Placeholders: none.
    pub small_microphones: &'static str,
    /// Shows: the recording bar, as a choice of a list. When: mic choice: automatic. At most: about
    /// 8 characters. Placeholders: none.
    pub microphone_auto: &'static str,
    /// Shows: the recording bar, as a help line. When: under "Auto". At most: no limit.
    /// Placeholders: none.
    pub microphone_auto_says: &'static str,
    /// Shows: the recording bar, as a help line. When: under the chosen mic, unplugged. At most: no
    /// limit. Placeholders: none.
    pub microphone_absent: &'static str,
    /// Shows: the recording bar, as an error. When: mic list unavailable. At most: no limit.
    /// Placeholders: none.
    pub microphones_unread: &'static str,
    /// Shows: the recording bar, as an error. When: mic choice not saved. At most: no limit.
    /// Placeholders: none.
    pub microphone_not_kept: &'static str,
    /// Shows: the recording bar, as a status. When: during recording: chosen mic gone. At most: no
    /// limit. Placeholders: none.
    pub small_microphone_gone: &'static str,
    /// Recording, under the microphone (D90, board 6): the one recorded, a change for the next.
    /// Shows: the recording bar, as a help line. When: during recording: mic in use. At most: no
    /// limit. Placeholders: {} mic name.
    pub bar_microphone_recorded: &'static str,
    /// Shows: the recording bar, as a help line. When: during recording: a change is for next time.
    /// At most: no limit. Placeholders: none.
    pub bar_microphone_next: &'static str,
    /// The context typed before the start, in the row at rest (D91, board 5): its placeholder, its ✕,
    /// and under the row while one is kept, that it waits for the next recording.
    /// Shows: the recording bar, as a field's placeholder. When: context field before a recording.
    /// At most: about 28 characters. Placeholders: none.
    pub bar_context: &'static str,
    /// Shows: the recording bar, as a name for a screen reader. When: icon button (tooltip): clear
    /// context. At most: no limit. Placeholders: none.
    pub bar_context_clear: &'static str,
    /// Shows: the recording bar, as a help line. When: context kept for next recording. At most: no
    /// limit. Placeholders: none.
    pub bar_context_kept: &'static str,
    /// « Avancé » (D90, board 6; D91), closed by default, in the bar opened and in *Refaire…*: the
    /// meeting's own models, Settings' choice first, said as such — what it holds said on it.
    /// Shows: the recording bar and Redo…, as a button. When: disclosure: per-meeting models.
    /// At most: about 30 characters. Placeholders: none.
    pub bar_advanced: &'static str,
    /// Shows: the recording bar and Redo…, as a label. When: per-meeting transcription model.
    /// At most: about 30 characters. Placeholders: none.
    pub models_transcription: &'static str,
    /// Shows: the recording bar and Redo…, as a label. When: per-meeting minutes model. At most:
    /// about 30 characters. Placeholders: none.
    pub models_minutes: &'static str,
    /// Shows: the recording bar and Redo…, as a choice of a list. When: first choice: Settings'
    /// model. At most: about 60 characters. Placeholders: {} model.
    pub models_settings: &'static str,
    /// Shows: the recording bar and Redo…, as a choice of a list. When: no model chosen in
    /// Settings. At most: about 30 characters. Placeholders: none.
    pub models_none: &'static str,
    /// Shows: the recording bar and Redo…, as a help line. When: per-meeting choice does not change
    /// Settings. At most: no limit. Placeholders: none.
    pub models_own_note: &'static str,
    /// Shows: the recording bar and Redo…, as an error. When: choice not saved. At most: no limit.
    /// Placeholders: {} failure sentence.
    pub models_not_kept: &'static str,
    /// Shows: a meeting's page, as an error. When: per-meeting models unreadable. At most: no
    /// limit. Placeholders: {} failure sentence.
    pub models_not_read: &'static str,
    /// A meeting's own model gone (D91): Settings' choice stands in, as Auto does for a microphone.
    /// Shows: the recording bar and Redo…, as a status. When: per-meeting transcription model
    /// missing. At most: no limit. Placeholders: none.
    pub models_gone_transcription: &'static str,
    /// Shows: the recording bar and Redo…, as a status. When: per-meeting minutes model's provider
    /// no longer configured. At most: no limit. Placeholders: none.
    pub models_gone_minutes: &'static str,
    /// A service the meeting chose, not used while it is said confidential (D60).
    /// Shows: the recording bar and Redo…, as a status. When: online service ignored: meeting
    /// confidential. At most: no limit. Placeholders: none.
    pub models_unused_confidential: &'static str,
    /// Under the bar's row at rest, « Avancé » closed: the models chosen for the next recording, named
    /// short (D91) — one, or more joined by « · ».
    /// Shows: the recording bar, as a help line. When: at rest: models kept for next recording.
    /// At most: no limit. Placeholders: {} the models kept, joined by « · ».
    pub bar_models_kept: &'static str,
    /// Shows: the recording bar, as a help line. When: same, one model. At most: no limit.
    /// Placeholders: {} the model kept.
    pub bar_model_kept: &'static str,
    /// « Vérifier le son » (D88, board 7): its title, and its row in Settings.
    pub check_sound: &'static str,
    pub check_sound_says: &'static str,
    /// A source capture could not open; none named yet.
    /// Shows: « Vérifier le son », as a status. When: a source cannot be opened. At most: about 16
    /// characters. Placeholders: none.
    pub check_unavailable: &'static str,
    /// The microphone listened to is a Bluetooth headset's: its call profile while the panel shows.
    /// Shows: « Vérifier le son », as a help line. When: Bluetooth headset switches to call
    /// profile. At most: no limit. Placeholders: none.
    pub check_bluetooth: &'static str,
    /// The test sound, and what was heard of it: on the meeting's side, or not, and what to do.
    /// Shows: « Vérifier le son », as a button. When: plays a test sound. At most: about 24
    /// characters. Placeholders: none.
    pub check_play: &'static str,
    /// Shows: « Vérifier le son », as a status. When: test sound heard. At most: no limit.
    /// Placeholders: none.
    pub check_heard: &'static str,
    /// Shows: « Vérifier le son », as an error. When: test sound not heard. At most: no limit.
    /// Placeholders: none.
    pub check_not_heard: &'static str,
    /// A call holds a microphone: no test sound, it would reach the call.
    /// Shows: « Vérifier le son », as a help line. When: a call holds the mic: no test sound.
    /// At most: no limit. Placeholders: none.
    pub check_call: &'static str,
    /// Shows: « Vérifier le son », as a help line. When: mic stays open while panel shows. At most:
    /// no limit. Placeholders: none.
    pub check_open_note: &'static str,
    /// Shows: « Vérifier le son », as a name for a screen reader. When: icon button (tooltip):
    /// close panel. At most: no limit. Placeholders: none.
    pub check_close: &'static str,
    /// It could not listen; or a recording runs.
    /// Shows: « Vérifier le son », as an error. When: cannot listen. At most: no limit.
    /// Placeholders: none.
    pub check_failed: &'static str,
    /// Shows: « Vérifier le son », as a status. When: a recording runs. At most: no limit.
    /// Placeholders: none.
    pub check_recording: &'static str,
    /// Freeing space (D11's third board).
    /// Shows: the sidebar and « Faire de la place », as a name for a screen reader. When: icon link
    /// (tooltip) and page title. At most: about 24 characters. Placeholders: none.
    pub free: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a heading. When: question. At most: no
    /// limit. Placeholders: none.
    pub free_what: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a name, then its line. When: radio:
    /// remove audio only. At most: no limit. Placeholders: none.
    pub free_audio: [&'static str; 2],
    /// Shows: « Faire de la place », the sidebar's foot, as a name, then its line. When: radio:
    /// remove whole meeting. At most: no limit. Placeholders: none.
    pub free_whole: [&'static str; 2],
    /// Shows: « Faire de la place », the sidebar's foot, as a heading. When: list ordering label.
    /// At most: about 30 characters. Placeholders: none.
    pub free_heaviest: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a button. When: tick meetings older
    /// than 30 days. At most: about 36 characters. Placeholders: none.
    pub free_older: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a status. When: after a size; plural.
    /// At most: no limit. Placeholders: {} count meetings.
    pub free_total: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a status. When: same, one meeting.
    /// At most: no limit. Placeholders: none.
    pub free_one: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a help line. When: goes to trash,
    /// restorable. At most: no limit. Placeholders: none.
    pub free_note: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a help line. When: consequence of
    /// removing audio. At most: no limit. Placeholders: none.
    pub free_audio_note: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a button. When: confirm: audio to
    /// trash. At most: about 36 characters. Placeholders: none.
    pub free_do_audio: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a button. When: confirm: meetings to
    /// trash. At most: about 36 characters. Placeholders: none.
    pub free_do_whole: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a status. When: a meeting cannot be
    /// freed now. At most: no limit. Placeholders: none.
    pub free_busy: &'static str,
    /// Shows: « Faire de la place », the sidebar's foot, as a status. When: after freeing. At most:
    /// no limit. Placeholders: none.
    pub free_done: &'static str,
    /// *Refaire…* on the minutes (D11's sixth board): the panel's title, question, note and gesture.
    /// Shows: a meeting's page, as a name for a screen reader. When: icon button (tooltip) opening
    /// the Redo panel. At most: no limit. Placeholders: none.
    pub redo: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a heading. When: redo the minutes. At most: no
    /// limit. Placeholders: none.
    pub redo_title: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a help line. When: uses this meeting's models.
    /// At most: no limit. Placeholders: none.
    pub redo_with: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a help line. When: current minutes go to trash
    /// once new ones are ready. At most: no limit. Placeholders: none.
    pub redo_note: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a button. When: confirm redo. At most: about 16
    /// characters. Placeholders: none.
    pub redo_do: &'static str,
    /// Shows: the recording bar and Redo…, as a choice of a list. When: appended to a local model's
    /// name. At most: about 20 characters. Placeholders: none.
    pub on_computer: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a heading. When: redo the transcript. At most:
    /// no limit. Placeholders: none.
    pub redo_transcript_title: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a help line. When: what happens to the current
    /// transcript. At most: no limit. Placeholders: none.
    pub redo_transcript_note: &'static str,
    /// What a meeting lacks, offered on its page; what is said once asked; where a step ran.
    /// Shows: a meeting's page, as a button. When: meeting lacks minutes. At most: about 30
    /// characters. Placeholders: none.
    pub make_minutes: &'static str,
    /// Shows: a meeting's page, as a button. When: meeting lacks a transcript. At most: about 30
    /// characters. Placeholders: none.
    pub make_transcript: &'static str,
    /// What is made in the background, said on the pages (D25): the minutes being written (by,
    /// for how long), waiting their turn ({} before), waiting for the user's agreement to send them
    /// ({} the host) with the way to give it, not written ({} the reason); on a list's row.
    /// Shows: a meeting's page, as a status. When: minutes being written (followed by model and
    /// elapsed time). At most: no limit. Placeholders: none.
    pub minutes_making: &'static str,
    /// Shows: a meeting's page, as a status. When: queued. At most: no limit. Placeholders: {}
    /// number before it.
    pub minutes_waiting: &'static str,
    /// Shows: a meeting's page, as a status. When: waiting for consent. At most: no limit.
    /// Placeholders: {} host.
    pub minutes_awaiting: &'static str,
    /// Shows: a meeting's page, as a link. When: opens the consent page. At most: about 24
    /// characters. Placeholders: none.
    pub agree_open: &'static str,
    /// Shows: a meeting's page, as an error. When: minutes failed (cause follows). At most: no
    /// limit. Placeholders: none.
    pub minutes_not_made: &'static str,
    /// Shows: a meeting's page, as a button. When: retry a failed step. At most: about 16
    /// characters. Placeholders: none.
    pub try_again: &'static str,
    /// Shows: the sidebar, as a list label. When: row: minutes being written. At most: 18
    /// characters. Placeholders: none.
    pub chip_making: &'static str,
    /// Shows: the sidebar, as a list label. When: row: queued. At most: 18 characters.
    /// Placeholders: none.
    pub chip_waiting: &'static str,
    /// Shows: the sidebar, as a list label. When: row: consent needed. At most: 18 characters.
    /// Placeholders: none.
    pub chip_awaiting: &'static str,
    /// Shows: the sidebar, as a list label. When: row: transcript being made. At most: 18
    /// characters. Placeholders: none.
    pub chip_transcribing: &'static str,
    /// Shows: the sidebar, as a list label. When: row: minutes failed. At most: 18 characters.
    /// Placeholders: none.
    pub chip_failed: &'static str,
    /// The same for the transcript: being made (by, for how long), not made ({} the reason).
    /// Shows: a meeting's page, as a status. When: transcript being made. At most: no limit.
    /// Placeholders: none.
    pub transcript_making: &'static str,
    /// Shows: a meeting's page, as an error. When: transcript failed. At most: no limit.
    /// Placeholders: none.
    pub transcript_not_made: &'static str,
    /// Shows: the sidebar, as a list label. When: row: transcript failed. At most: 18 characters.
    /// Placeholders: none.
    pub chip_transcript_failed: &'static str,
    /// Who writes the minutes chosen on the settings' page, not yet saved: who still does ({}).
    pub minutes_unsaved: &'static str,
    pub minutes_unsaved_none: &'static str,
    /// A transcript waiting for the default Whisper model: offered in the menu, or coming.
    pub transcript_model_needed: &'static str,
    pub transcript_model_coming: &'static str,
    pub ran_online: &'static str,
    /// The header's button, at rest and while a recording runs.
    pub record_start: &'static str,
    /// Shows: the sidebar, as a button. When: stop recording. At most: about 24 characters.
    /// Placeholders: none.
    pub record_stop: &'static str,
    /// The agreement before a meeting first leaves for a host (D11's fifth board): what leaves,
    /// what stays, to whom; the user's word, once or for this service; on this computer instead.
    /// Shows: the agreement before a meeting leaves, as a label. When: small heading above the
    /// title. At most: no limit. Placeholders: none.
    pub agree_eyebrow: &'static str,
    /// Shows: the agreement before a meeting leaves, as a heading. When: always. At most: no limit.
    /// Placeholders: {} meeting name.
    pub agree_title: &'static str,
    /// Shows: the agreement before a meeting leaves, as a help line. When: under the title.
    /// At most: no limit. Placeholders: none.
    pub agree_lead: &'static str,
    /// Shows: the agreement before a meeting leaves, as a name, then its line. When: what leaves:
    /// the transcript; {} minutes, {} words. At most: no limit. Placeholders: {} filled by the
    /// window.
    pub agree_leaves: [&'static str; 2],
    /// Shows: the agreement before a meeting leaves, as a name, then its line. When: what leaves:
    /// the context; {} words. At most: no limit. Placeholders: {} the context's number of words.
    pub agree_context: [&'static str; 2],
    /// Shows: the agreement before a meeting leaves, as a name, then its line. When: what stays:
    /// the audio. At most: no limit. Placeholders: none.
    pub agree_stays: [&'static str; 2],
    /// Shows: the agreement before a meeting leaves, as a label. When: recipient row. At most:
    /// about 16 characters. Placeholders: none.
    pub agree_to: &'static str,
    /// Shows: the agreement before a meeting leaves, as a label. When: model row. At most: about 16
    /// characters. Placeholders: none.
    pub agree_model: &'static str,
    /// Shows: the agreement before a meeting leaves, as a label. When: checkbox: don't ask again
    /// for this service. At most: no limit. Placeholders: none.
    pub agree_remember: &'static str,
    /// Shows: the agreement before a meeting leaves, as a button. When: primary: send. At most:
    /// about 30 characters. Placeholders: none.
    pub agree_send: &'static str,
    /// Shows: the agreement before a meeting leaves, as a button. When: refuse. At most: about 20
    /// characters. Placeholders: none.
    pub agree_not: &'static str,
    /// Shows: the agreement before a meeting leaves, as a link. When: alternative: write locally.
    /// At most: no limit. Placeholders: none.
    pub agree_local: &'static str,
    /// Shows: the agreement before a meeting leaves, as a button. When: confirm local writing.
    /// At most: about 30 characters. Placeholders: none.
    pub agree_local_write: &'static str,
    /// The global shortcut: its keys as granted, none granted, where it changes, the button there.
    pub shortcut: [&'static str; 2],
    pub shortcut_none: &'static str,
    pub shortcut_where: &'static str,
    pub shortcut_open: &'static str,
    /// *Tester*: the button, what it sends and to whom, and what it found.
    pub test: &'static str,
    pub test_sends: &'static str,
    pub testing: &'static str,
    pub test_answered: &'static str,
    pub test_failed: &'static str,
    pub keep_always: &'static str,
    /// A number of days the audio is kept.
    pub keep_days: &'static str,
    /// The settings file could not be read: why.
    /// A change could not be saved: why.
    /// Shows: the window, as an error. When: a setting/change not saved. At most: no limit.
    /// Placeholders: {} failure sentence (lower-case start).
    pub not_saved: &'static str,
    /// Who writes the minutes was saved, but the keyring did not keep the key: why.
    pub key_not_kept: &'static str,
    pub minutes_step: &'static str,
    pub ollama_group: &'static str,
    pub online_group: &'static str,
    pub other_group: &'static str,
    pub custom: &'static str,
    pub address: &'static str,
    pub model_name: &'static str,
    pub key: &'static str,
    pub key_kept: &'static str,
    pub save: &'static str,
    pub where_online: &'static str,
    /// The model saved, which this computer's Ollama does not have ({} the model).
    pub ollama_lacks: &'static str,
    /// This computer's Ollama, saved, not answering — or answering with no model.
    pub ollama_silent: &'static str,
    pub not_chosen: &'static str,
    /// The first run's assistant (D58): its question, who writes the minutes, the setting said
    /// plainly, and its way back from Settings. Each answer is its name and its line.
    pub welcome_title: &'static str,
    pub welcome_question: &'static str,
    pub welcome_examples: &'static str,
    pub welcome_often: [&'static str; 2],
    pub welcome_rarely: [&'static str; 2],
    pub welcome_unsure: [&'static str; 2],
    pub welcome_skip: &'static str,
    pub welcome_cancel: &'static str,
    pub welcome_next: &'static str,
    pub welcome_back: &'static str,
    pub welcome_writer: &'static str,
    pub welcome_here: [&'static str; 2],
    pub welcome_service: [&'static str; 2],
    pub welcome_private: [&'static str; 2],
    pub welcome_connected: [&'static str; 2],
    /// The summary's rows: who transcribes, who writes the minutes, the reminder.
    pub welcome_rows: [&'static str; 3],
    pub welcome_on_this_computer: &'static str,
    /// Online, no service chosen yet; one chosen already.
    pub welcome_online: &'static str,
    pub welcome_online_chosen: &'static str,
    /// Ollama and its model ({} the model as people say it).
    pub welcome_ollama_by: &'static str,
    pub welcome_reminder_on: &'static str,
    pub welcome_reminder_off: &'static str,
    /// Shows: the first run's assistant and a meeting's page, as an error. When: Ollama not
    /// answering: minutes wait. At most: no limit. Placeholders: none.
    pub welcome_ollama_silent: &'static str,
    /// Ollama answers without the model ({} the model).
    pub welcome_ollama_lacks: &'static str,
    /// The model's download ({} its size), and where from ({} the host).
    pub welcome_fetch: &'static str,
    pub welcome_fetch_from: &'static str,
    pub welcome_done: &'static str,
    pub welcome_later: &'static str,
    pub welcome_choose: &'static str,
    pub welcome_again: [&'static str; 2],
    /// A meeting's context (D59): the small window's disclosure, closed then open; the field's
    /// label, its example and why it helps; its count ({} the most, {} what is left — one, then
    /// more); another script than Latin or Cyrillic; why it is closed; what changed, the
    /// transcript alone or with the minutes, and the gesture that makes them again.
    /// Shows: a meeting's page, as a button. When: context empty: add one. At most: about 24
    /// characters. Placeholders: none.
    pub context_add: &'static str,
    pub context_edit: &'static str,
    pub context_open: &'static str,
    /// Shows: a meeting's page, as a label. When: context row name, followed by ':'. At most: about
    /// 16 characters. Placeholders: none.
    pub context_title: &'static str,
    /// Shows: a meeting's page, as a label. When: context field label. At most: no limit.
    /// Placeholders: none.
    pub context_label: &'static str,
    /// Shows: a meeting's page, as a field's placeholder. When: context field example. At most: no
    /// limit. Placeholders: none.
    pub context_example: &'static str,
    /// Shows: a meeting's page, as a help line. When: why context helps. At most: no limit.
    /// Placeholders: none.
    pub context_why: &'static str,
    /// Shows: a meeting's page, as a status. When: count. At most: no limit. Placeholders: {} max
    /// characters.
    pub context_most: &'static str,
    /// Shows: a meeting's page, as a status. When: count left; [one, many]. At most: no limit.
    /// Placeholders: {} number.
    pub context_left: [&'static str; 2],
    /// Shows: a meeting's page, as a help line. When: non-Latin script warning. At most: no limit.
    /// Placeholders: none.
    pub context_script: &'static str,
    /// Shows: a meeting's page, as a status. When: context locked during processing. At most: no
    /// limit. Placeholders: none.
    pub context_making: &'static str,
    /// Shows: a meeting's page, as a status. When: context locked: transcript unfinished. At most:
    /// no limit. Placeholders: none.
    pub context_unfinished: &'static str,
    /// Shows: a meeting's page, as a help line. When: transcript (or + minutes) made with another
    /// context; [transcript only, both]. At most: no limit. Placeholders: none.
    pub context_changed: [&'static str; 2],
    /// Shows: a meeting's page, as a button. When: redo with new context. At most: about 30
    /// characters. Placeholders: none.
    pub context_redo: &'static str,
    /// The language a meeting is spoken in (D65).
    /// Shows: a meeting's page, as a label. When: row: spoken language. At most: about 20
    /// characters. Placeholders: none.
    pub language_label: &'static str,
    /// Shows: a meeting's page, as a choice of a list. When: "As in Settings", followed by — value.
    /// At most: about 30 characters. Placeholders: none.
    pub language_default: &'static str,
    pub language_detected: &'static str,
    pub meetings_language: &'static str,
    /// Shows: a meeting's page, as an error. When: always. At most: no limit. Placeholders: {}
    /// failure sentence.
    pub language_not_saved: &'static str,
    /// Shows: a meeting's page, as a menu entry. When: rarer actions: rename. At most: about 30
    /// characters. Placeholders: none.
    pub rename_meeting: &'static str,
    /// Shows: a meeting's page, as a field's placeholder. When: name field. At most: about 30
    /// characters. Placeholders: none.
    pub name_meeting: &'static str,
    /// Shows: a meeting's page, as an error. When: always. At most: no limit. Placeholders: {}
    /// failure sentence.
    pub name_not_saved: &'static str,
    /// Shows: a meeting's page, as a choice of a list. When: appended "detected". At most: about 12
    /// characters. Placeholders: none.
    pub language_detected_short: &'static str,
    /// SpeechAnalyzer's language when the settings name none: the system's, never the app's (D64).
    pub language_as_system: &'static str,
    /// A meeting's own language SpeechAnalyzer does not hear: the one it hears in, said (D65).
    /// Shows: a meeting's page, on a Mac, as a choice of a list. When: {} language — SpeechAnalyzer
    /// hears {}. At most: no limit. Placeholders: {} the language chosen, then the one heard.
    pub language_heard_as: &'static str,
    /// The language Kitokey speaks (D64): its row in Settings, and the system's, its default.
    pub app_language: [&'static str; 2],
    pub app_language_system: &'static str,
    /// A transcript's line corrected by whoever listens (D62).
    /// Shows: a meeting's page, as a name for a screen reader. When: pencil on a transcript line.
    /// At most: no limit. Placeholders: {} time.
    pub correct_line: &'static str,
    /// Shows: a meeting's page, as a list label. When: line edited by the user. At most: 18
    /// characters. Placeholders: none.
    pub corrected: &'static str,
    /// Shows: a meeting's page, as an error. When: empty correction. At most: no limit.
    /// Placeholders: none.
    pub correction_empty: &'static str,
    /// Shows: a meeting's page, as an error. When: always. At most: no limit. Placeholders: {}
    /// failure sentence.
    pub correction_failed: &'static str,
    /// Shows: a meeting's page, as a help line. When: minutes older than transcript corrections.
    /// At most: no limit. Placeholders: none.
    pub corrected_since: &'static str,
    /// Shows: a meeting's page, as a button. When: redo minutes with corrections. At most: about 36
    /// characters. Placeholders: none.
    pub redo_corrections: &'static str,
    /// The minutes copied in one click, said in place (D45).
    /// Shows: a meeting's page, as a name for a screen reader. When: icon button (tooltip): copy
    /// minutes. At most: no limit. Placeholders: none.
    pub copy_minutes: &'static str,
    /// Shows: a meeting's page, as a status. When: after copy. At most: about 12 characters.
    /// Placeholders: none.
    pub minutes_copied: &'static str,
    /// Shows: a meeting's page, as an error. When: copy failed. At most: no limit. Placeholders:
    /// none.
    pub minutes_not_copied: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a help line. When: warning: corrections lost.
    /// At most: no limit. Placeholders: none.
    pub redo_loses_corrections: &'static str,
    /// Shows: a meeting's page, as an error. When: context not saved. At most: no limit.
    /// Placeholders: none.
    pub context_not_saved: &'static str,
    /// A meeting said confidential (D60): the switch's name and its words; the list's word beside
    /// the lock; what had left before ({} what, then each: the sound, the transcript, {} a host);
    /// the switch not saved; the minutes waiting for Ollama holding no model, and the fetch of {}.
    /// Shows: a meeting's page and the recording bar, as a label. When: switch name. At most: about
    /// 24 characters. Placeholders: none.
    pub confidential_name: &'static str,
    /// Shows: a meeting's page and the recording bar, as a help line. When: switch line. At most:
    /// no limit. Placeholders: none.
    pub confidential_says: &'static str,
    /// Shows: the sidebar, as a list label. When: row: confidential. At most: 18 characters.
    /// Placeholders: none.
    pub confidential_chip: &'static str,
    /// Shows: a meeting's page, as a help line. When: data already sent before marking
    /// confidential. At most: no limit. Placeholders: {} list.
    pub confidential_left: &'static str,
    /// Shows: a meeting's page, as a status. When: list item. At most: no limit. Placeholders: {}
    /// host.
    pub confidential_left_sound: &'static str,
    /// Shows: a meeting's page, as a status. When: list item. At most: no limit. Placeholders: {}
    /// host.
    pub confidential_left_transcript: &'static str,
    /// Shows: a meeting's page, as an error. When: switch not saved. At most: no limit.
    /// Placeholders: none.
    pub confidential_not_saved: &'static str,
    /// Shows: a meeting's page, as an error. When: confidential + Ollama has no model. At most: no
    /// limit. Placeholders: none.
    pub confidential_ollama_empty: &'static str,
    /// Shows: a meeting's page, as a button. When: always. At most: about 30 characters.
    /// Placeholders: {} model.
    pub confidential_fetch: &'static str,
}

pub fn window(language: Language) -> Window {
    match language {
        Language::English => Window {
            locale: "en",
            units: ["B", "kB", "MB", "GB"],
            sections: "Sections",
            meetings: "Meetings",
            weight: "{} meetings",
            weight_one: "{} meeting",
            weight_described: "The audio takes {}; the transcripts and minutes, {}.",
            this_week: "This week",
            last_week: "Last week",
            earlier_in: "Earlier in {}",
            undated: "Date unreadable",
            minutes: "Minutes",
            transcript: "Transcript",
            audio: "Audio · {}",
            audio_freed: "Audio deleted",
            audio_weighs: "audio {}",
            recording: "Recording now",
            interrupted: "Interrupted",
            none: "No meeting in {} yet.",
            none_do: "Start a recording: its button, its shortcut, or the tray’s icon.",
            list_unread: "The meetings in {} cannot be read.",
            list_unread_do: "Check that this folder’s disk is connected and that you may read it, then try again.",
            again_refused: "It could not start again: {}",
            hearing_models_refused: "This service’s models could not be read, you can type one: {}",
            fold_not_kept: "The sidebar will be back as it was at the next start: {}",
            agreement_refused: "This meeting cannot be sent: {}",
            meeting_not_opened: "This meeting could not be opened: {}",
            page_not_shown: "This page could not be shown: try again.",
            context_not_read: "The context could not be read again: {}",
            record_starting: "Starting…",
            record_stopping: "Stopping…",
            redo_cancel: "Cancel",
            saved: "Saved",
            small_not_confidential: "This meeting is not confidential: open it to mark it.",
            trial_over: "Your trial is over; your meetings stay yours, readable, playable and exportable.",
            buy: "Buy Kitokey",
            trial: "Free trial",
            find_meetings: "Search the meetings",
            fold: "Fold the sidebar",
            unfold: "Show the sidebar",
            none_found: "No meeting holds these words.",
            back: "Meetings",
            open_folder: "Open the folder",
            more_actions: "More actions",
            labelled: "{}:",
            minutes_title: "Minutes",
            transcript_title: "Transcript",
            search: "Search the meeting",
            written_by: "Written by {}",
            written_via: ", via {}",
            at: "at",
            heard_by: "Written by {}, on this computer",
            here: "Here",
            remote: "Remote",
            voice: "voice {}",
            here_described: "Here — your microphone",
            remote_described: "Remote — what the machine plays",
            who_spoke: "Who spoke over the meeting: here above, remote below",
            no_transcript: "No transcript yet.",
            no_minutes: "No minutes yet.",
            being_made: "Being written — what is here is true all the same.",
            unreadable: "This file cannot be read by this version of Kitokey.",
            unread: "{} line(s) of the file could not be read and are left out.",
            no_match: "Nothing said matches.",
            settings: "Settings",
            after_title: "After a recording",
            after_nothing: ["Do nothing", "The audio is kept; you start what follows when you want."],
            after_transcript: ["Write the transcript", "On this computer, with no network."],
            after_minutes: [
                "Write the transcript, then the minutes",
                "Each follows the other; a notification says when the minutes are ready.",
            ],
            models_title: "Models",
            models_intro: "For each step, you choose who works: a model on this computer, an online service, or your own address.",
            models_note: "The models “on this computer” are those Kitokey finds installed. A key is kept in your system’s keyring, never in a file. Nothing leaves this computer without your agreement: a transcription service you name receives your meetings’ sound and their context, a service that writes the minutes their text and context only.",
            model_state: "Transcription model",
            model_ready: "Ready",
            model_here: "Installed · {} · runs on this computer, with no network",
            model_missing: "Not on this computer yet.",
            model_named_missing: "Not on this computer: put it back where it was, or choose another.",
            model_coming: "Being downloaded.",
            model_offer: "Download the model ({}, {})",
            offered_title: "Models to download",
            offered_fetch: "Download",
            offered_far: "Downloading… {} %",
            model_chosen_missing: "{} — not on this computer yet",
            offered_ollama_size: "about 5 GB",
            offered_ollama_good: "For the minutes, on this computer, through Ollama",
            offered_needs_ollama: "Needs Ollama — install it from ollama.com",
            offered_refresh: "Update the list ({})",
            offered_refresh_failed: "The list could not be updated: the models above are still offered.",
            offered_refresh_waits: "The list cannot be updated yet: it waits on the key that signs it. The models above are still offered.",
            speech_option: "Apple's SpeechAnalyzer, on this Mac",
            speech_missing: "Apple's model is not on this Mac yet",
            speech_fetch: "Download Apple's model (macOS fetches it from Apple)",
            speech_fetching: "macOS is downloading Apple's model…",
            speech_failed: "Apple's model was not downloaded. Try again later; Whisper still hears.",
            service_group: "A service",
            service_option: "A service I name",
            service_other_model: "Another model…",
            service_address: "Its address",
            service_model: "Its model",
            service_key: "Its key, if it asks for one",
            service_agree: "I agree that my meetings’ sound is sent to {}",
            service_save: "Save",
            service_test: "Test",
            service_tested: "It hears, with each segment’s times: a sound of Kitokey’s own was transcribed.",
            service_saved: "Saved: {} hears your next meetings.",
            service_agree_first: "Nothing is saved: agree first that your meetings’ sound is sent to {}.",
            service_address_bad: "Nothing is saved: the address starts with https:// or http://, then the service’s host — for example https://api.groq.com/openai/v1.",
            service_model_named: "Nothing is saved: name the model the service hears with — for example whisper-large-v3.",
            installed: "installed, {}",
            keep_other: "Another number of days…",
            keep_not_days: "{} is not a number of days: from 1 to 36,500",
            builtin: "built into Kitokey",
            home: "Home folder",
            transcription: "Transcription",
            on_this_computer: "On this computer — nothing leaves it",
            reminder: [
                "Remind me to tell the participants",
                "At the start, a line reminds you to say the meeting is recorded.",
            ],
            offer: [
                "When an application takes the microphone",
                "A call starting, most often. Kitokey never records without you knowing.",
            ],
            offer_choices: ["Offer to record", "Do nothing"],
            autostart: [
                "Start Kitokey when you log in",
                "In the tray, no window: a call is offered even after a restart.",
            ],
            autostart_held: [
                "Turned off in Windows: turn it back on in Settings › Apps › Startup.",
                "Set by your organisation: it cannot be changed here.",
            ],
            advanced_title: "Advanced",
            developer: [
                "Developer mode",
                "Under a failure, shows what Kitokey’s program said of it — for whoever looks into it.",
            ],
            program_said: "What the program said: {}",
            minutes_sections: ["Highlights", "Actions", "Key points"],
            actions_cells: ["Who", "What", "By when"],
            unsettled: "not specified",
            model_recommended: "{} — recommended",
            play: "Listen",
            pause: "Pause",
            time_listen: "Listen from {}",
            time_passage: "See the passage at {}",
            not_playable: "Its audio cannot be listened to here: it holds no track this version plays.",
            not_played: "This meeting could not be played here. “Open the folder” shows its tracks, which another player reads.",
            playback: "Where the meeting is played from",
            no_player: "Its audio was freed: the meeting can no longer be listened to. Its transcript and minutes stay.",
            log_level: ["Logs’ level", "Read at Kitokey’s next start, by it and its programs."],
            log_info: "Usual",
            log_debug: "Detailed",
            logs: [
                "Logs",
                "What Kitokey did, one file a day, kept fourteen days — never a word of a meeting.",
            ],
            open_logs: "Open the logs",
            logs_not_opened: "The logs’ folder could not be opened.",
            copy_diagnostic: "Copy a diagnostic",
            diagnostic_copied: "Diagnostic copied",
            diagnostic_empty: "Nothing logged for this meeting",
            diagnostic_not_copied: "Diagnostic not copied",
            sessions_folder: [
                "Meetings folder",
                "New meetings are written here. Those already recorded stay where they were: the window and the audio's keeping follow this folder.",
            ],
            choose: "Choose…",
            small: "The recording bar",
            small_mic: "Your microphone",
            small_meeting: "The meeting’s sound",
            small_note: "Once stopped, the transcript writes itself. You will be told.",
            bar_silent_me: "Your microphone: nothing for 2 min — the recording goes on.",
            bar_silent_others: "The meeting: nothing for 2 min — the recording goes on.",
            small_nothing: "Nothing comes from it: check it is not muted on the device itself, and that it is ready.",
            small_waiting_me: "Your microphone is not recorded for now.",
            small_waiting_others: "The meeting’s sound is not recorded for now.",
            small_announce: "Tell the participants this meeting is being recorded.",
            copy_announcement: "Copy the announcement",
            announcement_not_copied: "Not copied.",
            small_full: "The whole window",
            bar_more: "More options",
            bar_less: "Fewer options",
            bar_hide: "Hide the bar",
            small_microphone: "Microphone: {}",
            small_microphones: "The microphone of the next recording",
            microphone_auto: "Auto",
            microphone_auto_says: "The one your call holds, otherwise your computer’s.",
            microphone_absent: "It is absent for now: Auto stands in.",
            microphones_unread: "The list cannot be read for now: Auto still works. Open it again in a moment.",
            microphone_not_kept: "Your choice of microphone could not be kept: choose it again.",
            small_microphone_gone: "The microphone you chose is not there: Auto stands in.",
            bar_microphone_recorded: "Recorded: {}. Changing it applies to the next recording.",
            bar_microphone_next: "Changing it applies to the next recording.",
            bar_context: "Meeting context",
            bar_context_clear: "Clear the context",
            bar_context_kept: "Kept for the next recording, until you clear it.",
            bar_advanced: "Advanced — models",
            models_transcription: "Transcription model",
            models_minutes: "Minutes model",
            models_settings: "Settings: {}",
            models_none: "no model chosen",
            models_own_note: "For this meeting alone; Settings do not change.",
            models_not_kept: "This choice could not be kept: {}",
            models_not_read: "The models could not be read: {}",
            models_gone_transcription: "The transcription model chosen for this meeting is no longer here: Settings’ stands in.",
            models_gone_minutes: "The minutes model chosen for this meeting goes through a service Settings do not name: Settings’ stands in.",
            models_unused_confidential: "The service chosen for this meeting is not used while it is confidential: it is made on this computer.",
            bar_models_kept: "Models kept: {}",
            bar_model_kept: "Model kept: {}",
            check_sound: "Check the sound",
            check_sound_says: "Your microphone and the meeting’s sound, before a recording.",
            check_unavailable: "Unavailable",
            check_bluetooth: "While this panel is open, your Bluetooth headset switches to call quality; it comes back once it is closed.",
            check_play: "Play a test sound",
            check_heard: "Kitokey hears this computer’s sound.",
            check_not_heard: "Kitokey did not hear the test sound. Check that the sound comes out of this computer and not another device, then play it again.",
            check_call: "A call holds the microphone: no test sound, it would go into the call. Play a sound in the call and watch the meter.",
            check_open_note: "The microphone stays open as long as this panel is.",
            check_close: "Close",
            check_failed: "Kitokey cannot listen to your microphone or the meeting’s sound for now: close this panel and open it again.",
            check_recording: "A recording is running: its meters are in the bar.",
            free: "Free space",
            free_what: "What do you want to remove?",
            free_audio: [
                "The audio only",
                "The minutes and the transcript still read. The audio is nearly all the weight.",
            ],
            free_whole: [
                "The whole meeting",
                "The audio, the transcript and the minutes: nothing of the meeting is left.",
            ],
            free_heaviest: "Heaviest first",
            free_older: "Tick those older than 30 days",
            free_total: "freed · {} meetings",
            free_one: "freed · 1 meeting",
            free_note: "Nothing is erased for good: everything goes to your system’s trash. To undo, restore from the trash.",
            free_audio_note: "Without its audio, a meeting can no longer be transcribed again by a better engine.",
            free_do_audio: "Move their audio to the trash",
            free_do_whole: "Move these meetings to the trash",
            free_busy: "being recorded, transcribed or its minutes written",
            free_done: "Done: in the trash.",
            redo: "Redo…",
            redo_title: "Write the minutes again",
            redo_with: "Made again with this meeting’s models: “Advanced” to choose them.",
            redo_note: "The current minutes stay in place until the new ones are ready, then go to the trash.",
            redo_do: "Redo",
            on_computer: "on this computer",
            redo_transcript_title: "Write the transcript again",
            redo_transcript_note: "The current transcript is kept until the new one is whole, then goes to the trash; if the new one fails, the current comes back.",
            make_minutes: "Write the minutes",
            make_transcript: "Write the transcript",
            minutes_making: "Writing the minutes",
            minutes_waiting: "Waiting its turn — {} before it",
            minutes_awaiting: "Waiting for your agreement to send it to {}",
            agree_open: "Review and agree",
            minutes_not_made: "The minutes were not written. The transcript is untouched.",
            try_again: "Try again",
            chip_making: "Minutes…",
            chip_waiting: "Waiting",
            chip_awaiting: "Your agreement",
            chip_transcribing: "Transcript…",
            chip_failed: "Minutes not written",
            transcript_making: "Writing the transcript",
            transcript_not_made: "The transcript was not written. The recording is untouched.",
            chip_transcript_failed: "Transcript not written",
            minutes_unsaved: "Not saved yet — the minutes are still written by {}.",
            minutes_unsaved_none: "Not saved yet — no minutes are written until it is.",
            transcript_model_needed: "Waiting for the Whisper model.",
            transcript_model_coming: "Waiting for the Whisper model, being downloaded.",
            ran_online: "online",
            record_start: "Start a recording",
            record_stop: "Stop the recording",
            agree_eyebrow: "Before the first minutes online",
            agree_title: "The text of “{}” is about to leave your computer",
            agree_lead: "To write the minutes, Kitokey sends the transcript to the service you chose. Here is exactly what leaves.",
            agree_leaves: [
                "Leaves: the transcript",
                "The words spoken, on your side and remote — {} minutes, about {} words.",
            ],
            agree_context: ["Leaves: the context you attached", "About {} words."],
            agree_stays: ["Stays here: the audio", "No sound is ever sent."],
            agree_to: "Recipient",
            agree_model: "Model",
            agree_remember: "Don’t ask again for this service",
            agree_send: "Send the transcript",
            agree_not: "Don’t send",
            agree_local: "Write the minutes on this computer instead",
            agree_local_write: "Write here",
            shortcut: ["Shortcut", "Starts and stops a recording, wherever you are."],
            shortcut_none: "Not granted by the desktop",
            shortcut_where: "The desktop grants it: it changes in the desktop's keyboard settings.",
            shortcut_open: "Keyboard settings",
            test: "Test",
            test_sends: "Test sends {} one sentence of Kitokey's, nothing of a meeting.",
            testing: "Testing…",
            test_answered: "The model answers.",
            test_failed: "The model does not answer: {}",
            keep_audio: [
                "Keep the audio",
                "After that, the audio goes to the trash; the transcript and minutes stay.",
            ],
            keep_always: "Always",
            keep_days: "{} days",
            not_saved: "The setting was not saved: {}",
            key_not_kept: "The choice is saved, but the key was not kept in the system's keyring: {}",
            minutes_step: "Minutes",
            ollama_group: "On this computer — found in Ollama",
            online_group: "Online",
            other_group: "Other",
            custom: "Custom…",
            address: "Address (OpenAI-compatible)",
            model_name: "Model",
            key: "Key",
            key_kept: "A key is kept in the system's keyring; a new one replaces it.",
            save: "Save",
            where_online: "Online — the transcript's text is sent",
            ollama_lacks: "Ollama, on this computer, has no model named {}: choose one in the list, or fetch it with Ollama.",
            ollama_silent: "Ollama, on this computer, does not answer, or has no model: start it, or fetch a model with it.",
            not_chosen: "Not chosen yet",
            welcome_title: "Let’s set Kitokey up with one question.",
            welcome_question: "Do your meetings discuss confidential matters?",
            welcome_examples: "Clients, human resources, health, legal matters, unpublished figures…",
            welcome_often: ["Yes, often", "Nothing must leave this computer."],
            welcome_rarely: ["Rarely or never", "I can trust a service I choose with a meeting’s text."],
            welcome_unsure: ["I don’t know", "Kitokey will choose the most discreet."],
            welcome_skip: "Skip — everything on this computer",
            welcome_cancel: "Cancel",
            welcome_next: "Next",
            welcome_back: "Back",
            welcome_writer: "Who writes the minutes?",
            welcome_here: [
                "This computer",
                "Nothing leaves it. It needs Ollama, which is free; a little slower.",
            ],
            welcome_service: [
                "An online service",
                "Faster and finer. The meeting’s text is sent to it, never its sound. You will choose it in Settings, and your agreement will be asked once before the first sending.",
            ],
            welcome_private: ["Private mode", "Your meetings do not leave this computer."],
            welcome_connected: [
                "Connected mode",
                "The minutes are written by an online service; the transcription stays on this computer.",
            ],
            welcome_rows: ["Transcription", "Minutes", "Reminder"],
            welcome_on_this_computer: "On this computer",
            welcome_online: "An online service, to choose now",
            welcome_online_chosen: "An online service",
            welcome_ollama_by: "Ollama · {}",
            welcome_reminder_on: "Tell the participants at the start of each meeting",
            welcome_reminder_off: "Off — Settings turn it back on",
            welcome_ollama_silent: "Ollama does not answer on this computer: the minutes wait for it. It is free, at ollama.com.",
            welcome_ollama_lacks: "Ollama does not hold {} yet: the minutes wait for it. Settings fetch it in one click.",
            welcome_fetch: "Download the model ({})",
            welcome_fetch_from: "From {}, checked before it is used.",
            welcome_done: "Finish",
            welcome_later: "Later",
            welcome_choose: "Choose my service",
            welcome_again: [
                "Set up again",
                "The first start’s question, your current setting chosen; nothing changes before its last step.",
            ],
            context_add: "Add the context",
            context_edit: "Edit the context",
            context_open: "The meeting’s context",
            context_title: "Context",
            context_label: "What is the meeting about? Who takes part?",
            context_example: "E.g. the Q4 budget with Inès Durand and the purchasing team",
            context_why: "The transcription hears it as the first words said: names are spelt as here. The minutes use it to give each action to the right person.",
            context_most: "{} characters at most",
            context_left: ["{} left", "{} left"],
            context_script: "In this script, Whisper may keep only its end, a service may refuse it.",
            context_making: "The context cannot change while the transcript or the minutes are being made.",
            context_unfinished: "The transcript is unfinished: finish it (Try again) before changing the context.",
            context_changed: [
                "The transcript was made with another context.",
                "The transcript and the minutes were made with another context.",
            ],
            context_redo: "Redo with this context",
            language_label: "Spoken language",
            language_default: "As in Settings",
            language_detected: "Detected by the transcription",
            meetings_language: "Language of the meetings",
            language_not_saved: "The language was not saved: {}",
            rename_meeting: "Rename this meeting",
            name_meeting: "Name this meeting",
            name_not_saved: "The name was not saved: {}",
            language_detected_short: "detected",
            language_as_system: "The system’s",
            language_heard_as: "{} — SpeechAnalyzer hears {}",
            app_language: [
                "Language of Kitokey",
                "Its window, its menu and its notifications; what the system draws itself keeps the system’s.",
            ],
            app_language_system: "The system’s language",
            correct_line: "Correct the line at {}",
            corrected: "corrected",
            correction_empty: "A line is not left empty: Escape leaves it as it was.",
            correction_failed: "The correction was not saved: {}",
            corrected_since: "The transcript was corrected after these minutes were written.",
            redo_corrections: "Redo with the corrections",
            copy_minutes: "Copy the minutes",
            minutes_copied: "Copied",
            minutes_not_copied: "The minutes could not be copied.",
            redo_loses_corrections: "Your corrections to this transcript will not be kept.",
            context_not_saved: "The context could not be saved.",
            confidential_name: "Confidential meeting",
            confidential_says: "Nothing leaves this computer",
            confidential_chip: "Confidential",
            confidential_left: "Had already left before: {}.",
            confidential_left_sound: "the sound, to {}",
            confidential_left_transcript: "the transcript, to {}",
            confidential_not_saved: "The meeting could not be said confidential: try again.",
            confidential_ollama_empty: "Ollama holds no model on this computer: the minutes wait for it.",
            confidential_fetch: "Download {}",
        },
        Language::French => Window {
            locale: "fr",
            units: ["o", "ko", "Mo", "Go"],
            sections: "Rubriques",
            meetings: "Réunions",
            weight: "{} réunions",
            weight_one: "{} réunion",
            weight_described: "L’audio occupe {}\u{202f}; les transcriptions et les comptes rendus, {}.",
            this_week: "Cette semaine",
            last_week: "La semaine dernière",
            earlier_in: "Plus tôt en {}",
            undated: "Date inconnue",
            minutes: "Compte rendu",
            transcript: "Transcription",
            audio: "Audio · {}",
            audio_freed: "Audio supprimé",
            audio_weighs: "audio {}",
            recording: "Enregistrement en cours",
            interrupted: "Interrompu",
            none: "Aucune réunion dans {} pour l’instant.",
            none_do: "Démarrez un enregistrement\u{a0}: son bouton, son raccourci, ou l’icône de la barre système.",
            list_unread: "Les réunions de {} ne peuvent pas être lues.",
            list_unread_do: "Vérifiez que le disque de ce dossier est branché et que vous pouvez le lire, puis réessayez.",
            again_refused: "Impossible de recommencer\u{a0}: {}",
            hearing_models_refused: "Les modèles de ce service n’ont pas pu être lus, vous pouvez en saisir un\u{a0}: {}",
            fold_not_kept: "La barre latérale reviendra comme avant au prochain démarrage\u{a0}: {}",
            agreement_refused: "Cette réunion ne peut pas être envoyée\u{a0}: {}",
            meeting_not_opened: "Cette réunion n’a pas pu être ouverte\u{a0}: {}",
            page_not_shown: "Cette page n’a pas pu être affichée\u{a0}: réessayez.",
            context_not_read: "Le contexte n’a pas pu être relu\u{a0}: {}",
            record_starting: "Démarrage…",
            record_stopping: "Arrêt…",
            redo_cancel: "Annuler",
            saved: "Enregistré",
            small_not_confidential: "Cette réunion n’est pas confidentielle\u{a0}: ouvrez-la pour la marquer.",
            trial_over: "L’essai est terminé\u{202f}; vos réunions restent à vous, lisibles, jouables et exportables.",
            buy: "Acheter Kitokey",
            trial: "Essai gratuit",
            find_meetings: "Chercher dans les réunions",
            fold: "Replier la barre latérale",
            unfold: "Afficher la barre latérale",
            none_found: "Aucune réunion ne contient ces mots.",
            back: "Réunions",
            open_folder: "Ouvrir le dossier",
            more_actions: "Plus d’actions",
            labelled: "{}\u{a0}:",
            minutes_title: "Compte rendu",
            transcript_title: "Transcription",
            search: "Rechercher dans la transcription",
            written_by: "Écrit par {}",
            written_via: ", via {}",
            at: "à",
            heard_by: "Transcrit par {}, sur cet ordinateur",
            here: "Ici",
            remote: "À distance",
            voice: "voix {}",
            here_described: "Ici — votre micro",
            remote_described: "À distance — le son de l’ordinateur",
            who_spoke: "Qui a parlé au fil de la réunion\u{a0}: ici en haut, à distance en bas",
            no_transcript: "Pas encore de transcription.",
            no_minutes: "Pas encore de compte rendu.",
            being_made: "En cours d’écriture — ce qui est là est vrai quand même.",
            unreadable: "Ce fichier ne peut pas être lu par cette version de Kitokey.",
            unread: "{} ligne(s) du fichier n’ont pas pu être lues et sont laissées de côté.",
            no_match: "Aucun passage ne correspond.",
            settings: "Réglages",
            after_title: "Après un enregistrement",
            after_nothing: [
                "Ne rien faire",
                "L’audio est gardé\u{202f}; vous lancez la suite quand vous voulez.",
            ],
            after_transcript: ["Écrire la transcription", "Sur cet ordinateur, sans réseau."],
            after_minutes: [
                "Écrire la transcription, puis le compte rendu",
                "Tout s’enchaîne\u{202f}; une notification prévient quand le compte rendu est prêt.",
            ],
            models_title: "Modèles",
            models_intro: "Pour chaque étape, vous choisissez qui travaille\u{a0}: un modèle sur cet ordinateur, un service en ligne, ou votre propre adresse.",
            models_note: "Les modèles «\u{a0}sur cet ordinateur\u{a0}» sont ceux que Kitokey trouve installés. Une clé est gardée dans le trousseau de votre système, jamais dans un fichier. Rien ne quitte cet ordinateur sans votre accord\u{a0}: un service de transcription que vous choisissez reçoit le son de vos réunions et leur contexte, un service de compte rendu leur texte et leur contexte seulement.",
            model_state: "Modèle de transcription",
            model_ready: "Prêt",
            model_here: "Installé · {} · tourne sur cet ordinateur, sans réseau",
            model_missing: "Pas encore sur cet ordinateur.",
            model_named_missing: "Pas sur cet ordinateur\u{a0}: remettez-le à sa place, ou choisissez-en un autre.",
            model_coming: "En cours de téléchargement.",
            model_offer: "Télécharger le modèle ({}, {})",
            offered_title: "Modèles à télécharger",
            offered_fetch: "Télécharger",
            offered_far: "Téléchargement… {} %",
            model_chosen_missing: "{} — pas encore sur cet ordinateur",
            offered_ollama_size: "environ 5 Go",
            offered_ollama_good: "Pour les comptes rendus, sur cet ordinateur, par Ollama",
            offered_needs_ollama: "Nécessite Ollama, à installer depuis ollama.com",
            offered_refresh: "Mettre à jour la liste ({})",
            offered_refresh_failed: "La liste n’a pas pu être mise à jour\u{a0}: les modèles ci-dessus restent proposés.",
            offered_refresh_waits: "La liste ne peut pas encore être mise à jour\u{a0}: elle attend la clé qui la signe. Les modèles ci-dessus restent proposés.",
            speech_option: "SpeechAnalyzer d’Apple, sur ce Mac",
            speech_missing: "le modèle d’Apple n’est pas encore sur ce Mac",
            speech_fetch: "Télécharger le modèle d’Apple (macOS le télécharge depuis Apple)",
            speech_fetching: "macOS télécharge le modèle d’Apple…",
            speech_failed: "Le modèle d’Apple n’a pas été téléchargé. Réessayez plus tard\u{202f}; Whisper reste disponible.",
            service_group: "Un service",
            service_option: "Un service que je choisis",
            service_other_model: "Un autre modèle…",
            service_address: "Son adresse",
            service_model: "Son modèle",
            service_key: "Sa clé, s’il en demande une",
            service_agree: "J’accepte que le son de mes réunions soit envoyé à {}",
            service_save: "Enregistrer",
            service_test: "Tester",
            service_tested: "Il entend, avec les temps de chaque segment\u{a0}: un son propre à Kitokey a été transcrit.",
            service_saved: "Enregistré\u{a0}: {} transcrira vos prochaines réunions.",
            service_agree_first: "Rien n’est enregistré\u{a0}: acceptez d’abord que le son de vos réunions soit envoyé à {}.",
            service_address_bad: "Rien n’est enregistré\u{a0}: l’adresse commence par https:// ou http://, puis l’hôte du service — par exemple https://api.groq.com/openai/v1.",
            service_model_named: "Rien n’est enregistré\u{a0}: nommez le modèle avec lequel le service entend — par exemple whisper-large-v3.",
            installed: "installé, {}",
            keep_other: "Autre nombre de jours…",
            keep_not_days: "{} n’est pas un nombre de jours\u{a0}: de 1 à 36 500",
            builtin: "intégré à Kitokey",
            home: "Dossier personnel",
            transcription: "Transcription",
            on_this_computer: "Sur cet ordinateur — rien ne sort",
            reminder: [
                "Rappeler de prévenir les participants",
                "Au démarrage, une ligne rappelle de dire que la réunion est enregistrée.",
            ],
            offer: [
                "Quand une application prend le micro",
                "Un appel qui commence, le plus souvent. Kitokey n’enregistre jamais sans que vous le sachiez.",
            ],
            offer_choices: ["Proposer d’enregistrer", "Ne rien faire"],
            autostart: [
                "Lancer Kitokey à l’ouverture de session",
                "Dans la barre système, sans fenêtre\u{a0}: un appel vous est proposé même après un redémarrage.",
            ],
            autostart_held: [
                "Désactivé dans Windows\u{a0}: réactivez-le dans Paramètres › Applications › Démarrage.",
                "Défini par votre organisation\u{a0}: il ne peut pas être changé ici.",
            ],
            advanced_title: "Avancé",
            developer: [
                "Mode développeur",
                "Affiche le détail technique sous chaque erreur, pour aider au diagnostic.",
            ],
            program_said: "Ce qu’a dit le programme\u{a0}: {}",
            minutes_sections: ["Points saillants", "Actions", "Points clés"],
            actions_cells: ["Qui", "Quoi", "Pour quand"],
            unsettled: "non précisé",
            model_recommended: "{} — recommandé",
            play: "Écouter",
            pause: "Pause",
            time_listen: "Écouter depuis {}",
            time_passage: "Voir le passage de {}",
            not_playable: "Cette version de Kitokey ne peut pas lire cet audio.",
            not_played: "Lecture impossible ici. Ouvrez le dossier pour l’écouter avec un autre lecteur.",
            playback: "Position de lecture",
            no_player: "L’audio a été supprimé pour faire de la place. La transcription et le compte rendu restent.",
            log_level: ["Niveau des journaux", "S’applique au prochain lancement de Kitokey."],
            log_info: "Habituel",
            log_debug: "Détaillé",
            logs: [
                "Journaux",
                "Ce que Kitokey a fait, un fichier par jour, gardé quatorze jours — jamais un mot d’une réunion.",
            ],
            open_logs: "Ouvrir les journaux",
            logs_not_opened: "Le dossier des journaux n’a pas pu être ouvert.",
            copy_diagnostic: "Copier un diagnostic",
            diagnostic_copied: "Diagnostic copié",
            diagnostic_empty: "Rien de journalisé pour cette réunion",
            diagnostic_not_copied: "Diagnostic non copié",
            sessions_folder: [
                "Dossier des réunions",
                "Les nouvelles réunions y sont écrites. Celles déjà enregistrées restent où elles étaient\u{a0}: la fenêtre et la garde de l’audio suivent ce dossier.",
            ],
            choose: "Choisir…",
            small: "La barre d’enregistrement",
            small_mic: "Votre micro",
            small_meeting: "Le son de la réunion",
            bar_silent_me: "Votre micro\u{a0}: rien depuis 2\u{a0}min — l’enregistrement continue.",
            bar_silent_others: "La réunion\u{a0}: rien depuis 2\u{a0}min — l’enregistrement continue.",
            small_nothing: "Rien n’en arrive\u{a0}: vérifiez qu’il n’est pas coupé sur l’appareil lui-même, et qu’il est prêt.",
            small_waiting_me: "Votre micro n’est pas enregistré pour l’instant.",
            small_waiting_others: "Son de la réunion non enregistré pour l’instant.",
            small_note: "À l’arrêt, la transcription s’écrit toute seule. Vous serez prévenu.",
            small_announce: "Prévenez les participants que cette réunion est enregistrée.",
            copy_announcement: "Copier l’annonce",
            announcement_not_copied: "Pas copiée.",
            small_full: "Toute la fenêtre",
            bar_more: "Plus d’options",
            bar_less: "Moins d’options",
            bar_hide: "Masquer la barre",
            small_microphone: "Micro\u{a0}: {}",
            small_microphones: "Le micro du prochain enregistrement",
            microphone_auto: "Auto",
            microphone_auto_says: "Celui que tient votre appel, sinon celui de votre ordinateur.",
            microphone_absent: "Il est absent pour l’instant\u{a0}: Auto le remplace.",
            microphones_unread: "La liste ne peut pas être lue pour l’instant\u{a0}: Auto fonctionne toujours. Rouvrez-la dans un instant.",
            microphone_not_kept: "Votre choix de micro n’a pas pu être gardé\u{a0}: choisissez-le de nouveau.",
            small_microphone_gone: "Le micro choisi n’est pas là\u{a0}: Auto le remplace.",
            bar_microphone_recorded: "Enregistré\u{a0}: {}. Le changer vaut pour le prochain enregistrement.",
            bar_microphone_next: "Le changer vaut pour le prochain enregistrement.",
            bar_context: "Contexte de la réunion",
            bar_context_clear: "Effacer le contexte",
            bar_context_kept: "Gardé pour le prochain enregistrement, jusqu’à ce que vous l’effaciez.",
            bar_advanced: "Avancé — modèles",
            models_transcription: "Modèle de transcription",
            models_minutes: "Modèle du compte rendu",
            models_settings: "Réglages\u{a0}: {}",
            models_none: "aucun modèle choisi",
            models_own_note: "Pour cette réunion seule\u{202f}; les Réglages ne changent pas.",
            models_not_kept: "Ce choix n’a pas pu être gardé\u{a0}: {}",
            models_not_read: "Les modèles n’ont pas pu être lus\u{a0}: {}",
            models_gone_transcription: "Le modèle de transcription choisi pour cette réunion n’est plus là\u{a0}: celui des Réglages le remplace.",
            models_gone_minutes: "Le modèle du compte rendu choisi pour cette réunion passe par un service que les Réglages ne nomment pas\u{a0}: celui des Réglages le remplace.",
            models_unused_confidential: "Le service choisi pour cette réunion n’est pas utilisé tant qu’elle est confidentielle\u{a0}: elle est faite sur cet ordinateur.",
            bar_models_kept: "Modèles gardés\u{a0}: {}",
            bar_model_kept: "Modèle gardé\u{a0}: {}",
            check_sound: "Vérifier le son",
            check_sound_says: "Votre micro et le son de la réunion, avant un enregistrement.",
            check_unavailable: "Indisponible",
            check_bluetooth: "Tant que ce panneau est ouvert, votre casque Bluetooth passe en qualité d’appel\u{202f}; il revient à la fermeture.",
            check_play: "Jouer un son de test",
            check_heard: "Kitokey entend le son de cet ordinateur.",
            check_not_heard: "Kitokey n’a pas entendu le son de test. Vérifiez que le son sort de cet ordinateur et pas d’un autre appareil, puis rejouez-le.",
            check_call: "Un appel tient le micro\u{a0}: pas de son de test, il partirait dans l’appel. Faites jouer un son dans l’appel et regardez la jauge.",
            check_open_note: "Le micro reste ouvert tant que ce panneau l’est.",
            check_close: "Fermer",
            check_failed: "Kitokey ne peut écouter ni votre micro ni le son de la réunion pour l’instant\u{a0}: fermez ce panneau et rouvrez-le.",
            check_recording: "Un enregistrement est en cours\u{a0}: ses jauges sont dans la barre.",
            free: "Faire de la place",
            free_what: "Que voulez-vous retirer\u{202f}?",
            free_audio: [
                "L’audio seulement",
                "Le compte rendu et la transcription restent lisibles. C’est l’audio qui pèse presque tout.",
            ],
            free_whole: [
                "La réunion entière",
                "L’audio, la transcription et le compte rendu\u{a0}: il ne reste rien de la réunion.",
            ],
            free_heaviest: "Les plus lourdes d’abord",
            free_older: "Cocher celles de plus de 30 jours",
            free_total: "libérés · {} réunions",
            free_one: "libérés · 1 réunion",
            free_note: "Rien n’est effacé pour de bon\u{a0}: tout part dans la corbeille de votre système. Pour annuler, restaurez depuis la corbeille.",
            free_audio_note: "Sans son audio, une réunion ne pourra plus être transcrite à nouveau par un meilleur moteur.",
            free_do_audio: "Mettre leur audio à la corbeille",
            free_do_whole: "Mettre ces réunions à la corbeille",
            free_busy: "en cours d’enregistrement, de transcription ou de rédaction du compte rendu",
            free_done: "C’est fait\u{a0}: dans la corbeille.",
            redo: "Refaire…",
            redo_title: "Refaire le compte rendu",
            redo_with: "Refait avec les modèles de cette réunion\u{a0}: «\u{a0}Avancé\u{a0}» pour les choisir.",
            redo_note: "Le compte rendu actuel reste en place tant que le nouveau n’est pas prêt, et se retrouve ensuite dans la corbeille.",
            redo_do: "Refaire",
            on_computer: "sur cet ordinateur",
            redo_transcript_title: "Refaire la transcription",
            redo_transcript_note: "La transcription actuelle est gardée tant que la nouvelle n’est pas entière, puis part à la corbeille\u{202f}; si la nouvelle échoue, l’actuelle revient.",
            make_minutes: "Faire le compte rendu",
            make_transcript: "Faire la transcription",
            minutes_making: "Rédaction du compte rendu",
            minutes_waiting: "En file d’attente · {} avant",
            minutes_awaiting: "En attente de votre accord pour l’envoyer à {}",
            agree_open: "Voir et accepter",
            minutes_not_made: "Le compte rendu n’a pas été fait. La transcription est intacte.",
            try_again: "Réessayer",
            chip_making: "Compte rendu…",
            chip_waiting: "En attente",
            chip_awaiting: "Accord requis",
            chip_transcribing: "Transcription…",
            chip_failed: "Compte rendu non fait",
            transcript_making: "Transcription en cours",
            transcript_not_made: "La transcription n’a pas été faite. L’enregistrement est intact.",
            chip_transcript_failed: "Transcription non faite",
            minutes_unsaved: "Modification non enregistrée\u{a0}: {} reste utilisé.",
            minutes_unsaved_none: "Pas encore enregistré — aucun compte rendu n’est fait tant que ce n’est pas le cas.",
            transcript_model_needed: "En attente du modèle Whisper.",
            transcript_model_coming: "En attente du modèle Whisper, en cours de téléchargement.",
            ran_online: "en ligne",
            record_start: "Démarrer un enregistrement",
            record_stop: "Arrêter l’enregistrement",
            agree_eyebrow: "Avant le premier compte rendu en ligne",
            agree_title: "Le texte de «\u{a0}{}\u{a0}» va quitter votre ordinateur",
            agree_lead: "Pour écrire le compte rendu, Kitokey envoie la transcription au service que vous avez choisi. Voici exactement ce qui part.",
            agree_leaves: [
                "Part\u{a0}: la transcription",
                "Les mots prononcés, de votre côté et à distance — {} minutes, environ {} mots.",
            ],
            agree_context: ["Part\u{a0}: le contexte que vous avez joint", "Environ {} mots."],
            agree_stays: ["Reste ici\u{a0}: l’audio", "Aucun son n’est envoyé, jamais."],
            agree_to: "Destinataire",
            agree_model: "Modèle",
            agree_remember: "Ne plus demander pour ce service",
            agree_send: "Envoyer la transcription",
            agree_not: "Ne pas envoyer",
            agree_local: "Écrire plutôt le compte rendu sur cet ordinateur",
            agree_local_write: "Rédiger sur cet ordinateur",
            shortcut: ["Raccourci", "Démarre et arrête un enregistrement, où que vous soyez."],
            shortcut_none: "Aucun raccourci défini",
            shortcut_where: "Il se modifie dans les réglages clavier du système.",
            shortcut_open: "Réglages du clavier",
            test: "Tester",
            test_sends: "Le test envoie une courte phrase à {} — rien de vos réunions.",
            testing: "Test en cours…",
            test_answered: "Ça marche\u{a0}: le modèle répond.",
            test_failed: "Le modèle ne répond pas\u{a0}: {}",
            keep_audio: [
                "Garder l’audio",
                "Passé ce délai, l’audio part à la corbeille\u{202f}; la transcription et le compte rendu restent.",
            ],
            keep_always: "Toujours",
            keep_days: "{} jours",
            not_saved: "Le réglage n’a pas été enregistré\u{a0}: {}",
            key_not_kept: "Le choix est enregistré, mais la clé n’a pas été gardée dans le trousseau du système\u{a0}: {}",
            minutes_step: "Compte rendu",
            ollama_group: "Sur cet ordinateur — trouvés dans Ollama",
            online_group: "En ligne",
            other_group: "Autre",
            custom: "Personnalisé…",
            address: "Adresse (compatible OpenAI)",
            model_name: "Modèle",
            key: "Clé d’API",
            key_kept: "Une clé est gardée dans le trousseau du système\u{202f}; une nouvelle la remplace.",
            save: "Enregistrer",
            where_online: "En ligne — le texte de la transcription est envoyé",
            ollama_lacks: "Ollama, sur cet ordinateur, n’a pas de modèle nommé {}\u{a0}: choisissez-en un dans la liste, ou téléchargez-le avec Ollama.",
            ollama_silent: "Ollama ne répond pas ou n’a aucun modèle\u{a0}: lancez Ollama, puis téléchargez un modèle.",
            not_chosen: "Pas encore choisi",
            welcome_title: "Réglons Kitokey en une question.",
            welcome_question: "Vos réunions parlent-elles de sujets confidentiels\u{202f}?",
            welcome_examples: "Clients, ressources humaines, santé, juridique, chiffres non publiés…",
            welcome_often: ["Oui, souvent", "Rien ne doit quitter cet ordinateur."],
            welcome_rarely: [
                "Rarement ou jamais",
                "Je peux confier le texte d’une réunion à un service que je choisis.",
            ],
            welcome_unsure: ["Je ne sais pas", "Kitokey choisira le plus discret."],
            welcome_skip: "Passer — tout sur cet ordinateur",
            welcome_cancel: "Annuler",
            welcome_next: "Suivant",
            welcome_back: "Retour",
            welcome_writer: "Qui écrit le compte rendu\u{202f}?",
            welcome_here: ["Cet ordinateur", "Rien ne sort. Il faut Ollama, gratuit\u{202f}; un peu plus lent."],
            welcome_service: [
                "Un service en ligne",
                "Plus rapide et plus fin. Le texte de la réunion lui est envoyé, jamais le son. Vous le choisirez dans Réglages, et votre accord sera demandé une fois avant le premier envoi.",
            ],
            welcome_private: ["Mode privé", "Vos réunions ne quittent pas cet ordinateur."],
            welcome_connected: [
                "Mode connecté",
                "Le compte rendu est écrit par un service en ligne\u{202f}; la transcription reste sur cet ordinateur.",
            ],
            welcome_rows: ["Transcription", "Compte rendu", "Rappel"],
            welcome_on_this_computer: "Sur cet ordinateur",
            welcome_online: "Un service en ligne, à choisir maintenant",
            welcome_online_chosen: "Un service en ligne",
            welcome_ollama_by: "Ollama · {}",
            welcome_reminder_on: "Prévenir les participants au début de chaque réunion",
            welcome_reminder_off: "Désactivé — Réglages le remettent",
            welcome_ollama_silent: "Ollama ne répond pas sur cet ordinateur\u{a0}: le compte rendu l’attend. Il est gratuit, sur ollama.com.",
            welcome_ollama_lacks: "Ollama n’a pas encore {}\u{a0}: le compte rendu l’attend. Réglages le téléchargent en un clic.",
            welcome_fetch: "Télécharger le modèle ({})",
            welcome_fetch_from: "Depuis {}, vérifié avant d’être utilisé.",
            welcome_done: "Terminer",
            welcome_later: "Plus tard",
            welcome_choose: "Choisir mon service",
            welcome_again: [
                "Refaire le réglage",
                "La question du premier lancement, votre réglage actuel choisi\u{202f}; rien ne change avant la dernière étape.",
            ],
            context_add: "Ajouter le contexte",
            context_edit: "Modifier le contexte",
            context_open: "Contexte de la réunion",
            context_title: "Contexte",
            context_label: "De quoi parle la réunion\u{202f}? Qui y participe\u{202f}?",
            context_example: "Ex.\u{a0}: budget du T4 avec Inès Durand et l’équipe achats",
            context_why: "La transcription l’entend comme les premiers mots\u{a0}: les noms y sont écrits comme ici. Le compte rendu s’en sert pour attribuer les actions.",
            context_most: "{} caractères au plus",
            context_left: ["{} restant", "{} restants"],
            context_script: "Dans cette écriture, Whisper peut n’en garder que la fin, un service le refuser.",
            context_making: "Le contexte ne change pas pendant que la transcription ou le compte rendu se fait.",
            context_unfinished: "La transcription n’est pas terminée\u{a0}: terminez-la (Réessayer) avant de changer le contexte.",
            context_changed: [
                "La transcription a été faite avec un autre contexte.",
                "La transcription et le compte rendu ont été faits avec un autre contexte.",
            ],
            context_redo: "Refaire avec ce contexte",
            language_label: "Langue parlée",
            language_default: "Celle des Réglages",
            language_detected: "Détectée par la transcription",
            meetings_language: "Langue des réunions",
            language_not_saved: "La langue n’a pas été enregistrée\u{a0}: {}",
            rename_meeting: "Renommer cette réunion",
            name_meeting: "Nommer cette réunion",
            name_not_saved: "Le nom n’a pas été enregistré\u{a0}: {}",
            language_detected_short: "détectée",
            language_as_system: "Celle du système",
            language_heard_as: "{} — SpeechAnalyzer entend {}",
            app_language: [
                "Langue de Kitokey",
                "Sa fenêtre, son menu et ses notifications\u{202f}; ce que le système dessine lui-même garde la sienne.",
            ],
            app_language_system: "Langue du système",
            correct_line: "Corriger la ligne de {}",
            corrected: "corrigé",
            correction_empty: "Une ligne ne reste pas vide\u{a0}: Échap la laisse comme elle était.",
            correction_failed: "La correction n’a pas été enregistrée\u{a0}: {}",
            corrected_since: "La transcription a été corrigée après ce compte rendu.",
            redo_corrections: "Refaire avec les corrections",
            copy_minutes: "Copier le compte rendu",
            minutes_copied: "Copié",
            minutes_not_copied: "Le compte rendu n’a pas pu être copié.",
            redo_loses_corrections: "Vos corrections de cette transcription ne seront pas gardées.",
            context_not_saved: "Le contexte n’a pas pu être enregistré.",
            confidential_name: "Réunion confidentielle",
            confidential_says: "Rien ne quitte cet ordinateur",
            confidential_chip: "Confidentielle",
            confidential_left: "Déjà parti avant\u{a0}: {}.",
            confidential_left_sound: "le son vers {}",
            confidential_left_transcript: "la transcription vers {}",
            confidential_not_saved: "La réunion n’a pas pu être marquée confidentielle\u{a0}: réessayez.",
            confidential_ollama_empty: "Ollama n’a aucun modèle sur cet ordinateur\u{a0}: le compte rendu l’attend.",
            confidential_fetch: "Télécharger {}",
        },
    }
}

/// What can stand in the shell's own way, before or beside a recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    /// The program that records could not be started: what the system said of it.
    NoRecorder(String),
    /// The settings file was not read: the sentence that says which file, and what in it.
    Settings(String),
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
    SettingsUnread,
    SettingsLocked,
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
        (Failure::SettingsUnread, English) => "the settings file cannot be read: correct it, then try again.",
        (Failure::SettingsUnread, French) => "le fichier des réglages ne peut pas être lu\u{a0}: corrigez-le, puis réessayez.",
        (Failure::SettingsLocked, English) => "The settings file cannot be read: Kitokey does not write over it until it is corrected.",
        (Failure::SettingsLocked, French) => {
            "Le fichier des réglages ne peut pas être lu\u{a0}: Kitokey ne l’écrit pas tant qu’il n’est pas corrigé."
        }
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
        (Trouble::Settings(why), English) => {
            return said(
                "Your settings were not read",
                format!("Kitokey runs with its defaults until the file is corrected: {}.", detail(why)),
            );
        }
        (Trouble::Settings(why), French) => {
            return said(
                "Vos réglages n’ont pas été lus",
                format!(
                    "Kitokey fonctionne avec ses réglages par défaut tant que le fichier n’est pas corrigé\u{a0}: {}.",
                    detail(why)
                ),
            );
        }
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
