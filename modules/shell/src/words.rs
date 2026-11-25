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
    // Shows: a length wherever one is said: the bar, a stop's notification, a meeting's header.
    // When: a recording's length. At most: about 8 characters. Placeholders: {} the seconds,
    // {minutes} the minutes, {hours} the hours.
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
        // Shows: a notification, its title then its body. When: a recording starts, the reminder
        // on. At most: the title, 50 characters; the body, no limit. Placeholders: {name} the
        // microphone recorded, {named} the sentence that names it.
        (Said::Started(_, microphone, _), English) => {
            let named = microphone.as_ref().map(|name| format!("Microphone: {name}. ")).unwrap_or_default();
            return notice(
                "Recording",
                format!("{named}Remember to tell participants this meeting is being recorded."),
            );
        }
        (Said::Started(_, microphone, _), French) => {
            let named = microphone.as_ref().map(|name| format!("Micro\u{a0}: {name}. ")).unwrap_or_default();
            return notice(
                "Enregistrement en cours",
                format!("{named}Pensez à prévenir les participants que la réunion est enregistrée."),
            );
        }
        // Shows: a notification, its title then its body. When: a side cannot be opened at the
        // start, or the microphone chosen is not there. At most: the title, 50 characters; the
        // body, no limit. Placeholders: none.
        (Said::Unavailable(Source::Me, _), English) => {
            ("Your microphone isn’t being recorded", "The meeting audio is still being recorded.")
        }
        (Said::Unavailable(Source::Me, _), French) => (
            "Votre micro n’est pas enregistré",
            "Le son de la réunion, lui, est bien enregistré.",
        ),
        (Said::MicrophoneGone, English) => (
            "Chosen microphone not found",
            "Kitokey is using the default microphone and will switch back as soon as yours is reconnected.",
        ),
        (Said::MicrophoneGone, French) => (
            "Micro choisi introuvable",
            "Kitokey utilise le micro par défaut et reprendra le vôtre dès qu’il sera rebranché.",
        ),
        (Said::Unavailable(Source::Others, _), English) => {
            ("Meeting audio isn’t being recorded", "Your microphone is still being recorded.")
        }
        (Said::Unavailable(Source::Others, _), French) => (
            "Le son de la réunion n’est pas enregistré",
            "Votre micro, lui, est bien enregistré.",
        ),
        // The device recorded changed (D31): what is recorded now is named.
        // Shows: a notification, its title then its body. When: a side is recorded from another
        // device. At most: the title, 50 characters; the body, no limit. Placeholders: {name} the
        // device now recorded.
        (Said::Moved(Source::Me, Some(name)), English) => return notice("Microphone changed", format!("Now recording from {name}.")),
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
        (Said::Moved(Source::Me, None), English) => ("Microphone changed", "Now recording from your computer’s current microphone."),
        (Said::Moved(Source::Me, None), French) => (
            "Micro changé",
            "Kitokey enregistre désormais le micro utilisé par votre ordinateur.",
        ),
        (Said::Moved(Source::Others, None), English) => (
            "Sound output changed",
            "Meeting audio is now recorded from your computer’s current output.",
        ),
        (Said::Moved(Source::Others, None), French) => (
            "Sortie audio changée",
            "Le son de la réunion est désormais enregistré depuis la sortie actuelle de votre ordinateur.",
        ),
        // None left: that side, and nothing of the other, which may have none either. Of the
        // microphone, never why: the system took it away, or a call let go of it (D66).
        // Shows: a notification, its title then its body. When: a side has no device left during a
        // recording. At most: the title, 50 characters; the body, no limit. Placeholders: none.
        (Said::Lost(Source::Me), English) => ("Your microphone isn’t being recorded", "Recording continues."),
        (Said::Lost(Source::Me), French) => ("Votre micro n’est pas enregistré", "L’enregistrement continue."),
        (Said::Lost(Source::Others), English) => (
            "Meeting audio is no longer being recorded",
            "Your computer has no audio output. Plug one in to resume; recording continues.",
        ),
        (Said::Lost(Source::Others), French) => (
            "Le son de la réunion n’est plus enregistré",
            "Votre ordinateur n’a plus de sortie audio. Branchez-en une pour reprendre\u{202f}; l’enregistrement continue.",
        ),
        (Said::Nothing, _) => return sends_nothing(None, language),
        // Shows: a notification, its title then its body. When: a side has heard nothing for ten
        // seconds. At most: the title, 50 characters; the body, no limit. Placeholders: none.
        (Said::Silent(Source::Me), English) => ("No sound from your microphone", "Nothing for 10 seconds. Recording continues."),
        (Said::Silent(Source::Me), French) => ("Aucun son de votre micro", "Rien depuis 10 secondes. L’enregistrement continue."),
        (Said::Silent(Source::Others), English) => (
            "No sound from the meeting",
            "Nothing for 10 seconds — is the call playing on this computer? Recording continues.",
        ),
        (Said::Silent(Source::Others), French) => (
            "Aucun son de la réunion",
            "Rien depuis 10 secondes\u{a0}: l’appel passe-t-il bien par cet ordinateur\u{202f}? L’enregistrement continue.",
        ),
        // Shows: a notification, its title then its body. When: a recording stops. At most: the title,
        // 50 characters; the body, no limit. Placeholders: {} its length; French builds its body
        // apart, the participle agreed with the length.
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
        // Shows: a notification, its title then its body. When: the program that records ends by
        // itself. At most: the title, 50 characters; the body, no limit. Placeholders: none.
        (Said::Failed(why), English) if detail(why).is_empty() => (
            "Recording stopped unexpectedly",
            "Something went wrong. Everything recorded so far is saved in the meeting’s folder.",
        ),
        (Said::Failed(why), French) if detail(why).is_empty() => (
            "L’enregistrement s’est arrêté",
            "Une erreur inattendue l’a interrompu. Ce qui a été enregistré est conservé dans le dossier de la réunion.",
        ),
        (Said::Failed(_), English) => (
            "Recording stopped unexpectedly",
            "Everything recorded so far is saved in the meeting’s folder.",
        ),
        (Said::Failed(_), French) => (
            "L’enregistrement s’est arrêté",
            "Ce qui a été enregistré est conservé dans le dossier de la réunion.",
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
    /// Shows: the Kitokey icon's menu, as a menu entry. When: the recording is at rest. At most:
    /// about 40 characters. Placeholders: none.
    pub start: &'static str,
    /// Shows: the Kitokey icon's menu, as a menu entry. When: a recording runs. At most: about 40
    /// characters. Placeholders: none.
    pub stop: &'static str,
    /// Shows: the Kitokey icon's menu, as a menu entry. When: a recording is told to stop. At most:
    /// about 40 characters. Placeholders: none.
    pub stopping: &'static str,
    /// Shows: the Kitokey icon's menu, as a menu entry. When: always. At most: about 40 characters.
    /// Placeholders: none.
    pub quit: &'static str,
    /// Opens the window.
    /// Shows: the Kitokey icon's menu, as a menu entry. When: always. At most: about 40 characters.
    /// Placeholders: none.
    pub open: &'static str,
    /// What the tray's icon says of itself while a recording runs.
    /// Shows: the Kitokey icon, as its tooltip. When: a recording runs, or is told to stop.
    /// At most: about 40 characters. Placeholders: none.
    pub recording: &'static str,
    /// Makes the transcript of the last recording, again when it failed.
    /// Shows: the Kitokey icon's menu, as a menu entry. When: a recording is there to transcribe.
    /// At most: about 40 characters. Placeholders: none.
    pub transcribe: &'static str,
    /// The same entry while a transcript is being made.
    /// Shows: the Kitokey icon's menu, as a menu entry. When: its transcript is made. At most:
    /// about 40 characters. Placeholders: none.
    pub transcribing: &'static str,
    /// Has the minutes of the last recording written, again when it failed.
    /// Shows: the Kitokey icon's menu, as a menu entry. When: a recording is there whose minutes
    /// can be written. At most: about 40 characters. Placeholders: none.
    pub minutes: &'static str,
    /// The same entry while they are being written.
    /// Shows: the Kitokey icon's menu, as a menu entry. When: its minutes are written. At most:
    /// about 40 characters. Placeholders: none.
    pub minuting: &'static str,
    /// Brings back the recording bar the user hid (D90).
    /// Shows: the Kitokey icon's menu, as a menu entry. When: the user hid the recording bar.
    /// At most: about 40 characters. Placeholders: none.
    pub show_bar: &'static str,
}

/// The words a file exported shows that no contract carries (`exporting/v1`'s input, D109): the
/// transcript's heading, who spoke, and the transcript's three columns.
pub struct Exported {
    pub transcript: &'static str,
    pub me: &'static str,
    pub others: &'static str,
    pub time: &'static str,
    pub speaker: &'static str,
    pub text: &'static str,
}

pub fn exported(language: Language) -> Exported {
    match language {
        // Shows: an exported file, the transcript's heading, who spoke, and its three columns (D109).
        // When: the transcript is included. At most: 24 characters. Placeholders: none.
        Language::English => Exported {
            transcript: "Transcript",
            me: "You",
            others: "Others",
            time: "Time",
            speaker: "Who",
            text: "Text",
        },
        Language::French => Exported {
            transcript: "Transcription",
            me: "Vous",
            others: "Les autres",
            time: "Heure",
            speaker: "Qui",
            text: "Texte",
        },
    }
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
            transcribe: "Transcribe last recording",
            transcribing: "Transcribing…",
            minutes: "Write summary of last recording",
            minuting: "Writing summary…",
            show_bar: "Show recording bar",
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
            minutes: "Rédiger le compte rendu du dernier enregistrement",
            minuting: "Rédaction du compte rendu…",
            show_bar: "Afficher la barre d’enregistrement",
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
    /// No session in the folder (#775): said without its path, which `none_folder` says under the action.
    /// Shows: the window with no meeting, as a heading. When: no meeting in the folder yet.
    /// At most: no limit. Placeholders: none.
    pub none: &'static str,
    /// Shows: the empty pane, a discreet line under its action (#775). When: no meeting in the folder.
    /// At most: no limit, a path wraps. Placeholders: `{}`, the folder's whole path.
    pub none_folder: &'static str,
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
    /// The settings file cannot be read: said above every page but Settings (#770) — never read since
    /// Kitokey started, then read before and its last settings held.
    /// Shows: a banner above every page but Settings. When: the settings file cannot be read and was
    /// never read since Kitokey started. At most: no limit. Placeholders: none.
    pub settings_banner: &'static str,
    /// Shows: a banner above every page but Settings. When: the settings file cannot be read, read
    /// before, its last settings held. At most: no limit. Placeholders: none.
    pub settings_banner_kept: &'static str,
    /// Shows: a meeting's page, as an error. When: "Try again" on a failed step was refused.
    /// At most: no limit. Placeholders: {} failure sentence.
    pub again_refused: &'static str,
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
    /// Shows: Settings, beside a setting just saved. When: for a moment after it is. At most: one
    /// line. Placeholders: none.
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
    /// Shows: Settings, its first card, its label, beside what is left (`trial_left`). When: on a
    /// trial licence (D83). At most: one line. Placeholders: none.
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
    /// Between a day and its hour: "13 oct. à 12:21", "Oct 13 at 12:21 PM".
    /// Shows: a meeting's page, as a format. When: between a day and its hour. At most: no limit.
    /// Placeholders: none.
    pub at: &'static str,
    /// The transcript's maker: the model, on this computer.
    /// Shows: a meeting's page, as a status. When: under the transcript: the model that made it.
    /// At most: no limit. Placeholders: {} model.
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
    /// The transcript's speakers named by the user (#1205): the button that opens the two fields.
    /// Shows: a meeting's page, as a button's tooltip and name. When: transcript card, a v3 meeting with its
    /// transcript. At most: about 30 characters. Placeholders: none.
    pub speakers_open: &'static str,
    /// Shows: a meeting's page, as a button. When: the speakers' names being typed. At most: one or two words.
    /// Placeholders: none.
    pub speakers_save: &'static str,
    /// Shows: a meeting's page, as a button. When: the speakers' names being typed. At most: one or two words.
    /// Placeholders: none.
    pub speakers_cancel: &'static str,
    /// Shows: a meeting's page, as a hint under the fields. When: the speakers' names being typed. At most: one
    /// line. Placeholders: none.
    pub speakers_hint: &'static str,
    /// Shows: a meeting's page, as an error under the fields, what was typed kept. When: the names could not be
    /// kept in the meeting's folder. At most: one line. Placeholders: none.
    pub speakers_not_saved: &'static str,
    /// Who spoke over the meeting, for whoever does not see the drawing.
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: the
    /// speaking-time drawing. At most: no limit. Placeholders: none.
    pub who_spoke: &'static str,
    /// Shows: a meeting's page, in its who-spoke area (#1116). When: the meeting has no transcript yet. At most:
    /// one line. Placeholders: none.
    pub who_spoke_later: &'static str,
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
    /// Shows: a meeting's page, as a button. When: download the transcription model. At most: about
    /// 40 characters. Placeholders: {} the model's size, then {} its host.
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
    /// The part of « Après un enregistrement » holding the destinations a summary can be sent to
    /// (D110, PDR-0007), its heading. Shows: Settings, a sub-heading. When: always. At most: a few words. Placeholders: none.
    pub destinations_title: &'static str,
    /// Under that heading: what the card is for, and that nothing leaves without a gesture.
    /// Shows: Settings, « Après un enregistrement », « Envoi des comptes rendus », secondary text. When: always. At most: two lines.
    /// Placeholders: none.
    pub destinations_said: &'static str,
    /// The same card with no destination yet. Shows: Settings, in place of the list. When: none is
    /// set up. At most: one line. Placeholders: none.
    pub destinations_none: &'static str,
    /// Each kind of destination, as the list and the choice name it (`posting/v1`'s kinds).
    /// Shows: Settings, a list's choices and each row. When: always. At most: two words each.
    /// Placeholders: none.
    pub destination_kinds: [&'static str; 3],
    /// The three fields' labels: its kind, its name, its address. Shows: Settings, labels. When:
    /// always. At most: a few words each. Placeholders: none.
    pub destination_fields: [&'static str; 3],
    /// Under the address field: what to paste, and that it is kept in the keychain, never shown whole.
    /// Shows: Settings, secondary text. When: always. At most: two lines. Placeholders: none.
    pub destination_address_said: &'static str,
    /// The button that adds it. Shows: Settings, a button. When: always. At most: one word.
    /// Placeholders: none.
    pub destination_add: &'static str,
    /// The disclosure that unfolds the form adding a destination, folded at first (E5: no address
    /// asked in the common case). Shows: Settings, « Envoi des comptes rendus », a button. When:
    /// always. At most: three words. Placeholders: none.
    pub destination_open: &'static str,
    /// Each destination's button that forgets it, its address out of the keychain. Shows: Settings,
    /// a button on each row. When: a destination is set up. At most: one word. Placeholders: none.
    pub destination_forget: &'static str,
    /// Said once a destination is added. Shows: Settings, under the fields, as a status. When: after
    /// « Ajouter » kept it. At most: one line. Placeholders: {} the destination's name.
    pub destination_added: &'static str,
    /// The destination not added: why, the failure's sentence after it. Shows: Settings, under the
    /// fields, as a status. When: « Ajouter » refused. At most: one line and the failure's.
    /// Placeholders: {} the failure's sentence (lower-case start).
    pub destination_not_added: &'static str,
    /// The button that opens the send panel (D110). Shows: a meeting's page, the summary's actions. When: a
    /// summary, a destination set up, the meeting not confidential. At most: one word. Placeholders: none.
    pub send_open: &'static str,
    /// Each destination's button in the send panel (PDR-0007 §2). Shows: a meeting's page, the send panel. When:
    /// the panel is open. At most: four words. Placeholders: {} the destination's name.
    pub send_to: &'static str,
    /// Before a destination's first send, what leaves and where, by its name, and that it cannot be taken back
    /// (PDR-0007 §3). Shows: a meeting's page, the send panel. When: the destination was never sent to. At most:
    /// three lines. Placeholders: {} the destination's name.
    pub send_notice: &'static str,
    /// A confidential meeting's send asked once more, every time (D119, PDR-0007 §4 as amended on 2026-11-24).
    /// Shows: a meeting's page, the send panel, above « Envoyer » and « Annuler ». When: a destination is chosen
    /// for a confidential meeting. At most: one sentence. Placeholders: {} the destination's name.
    pub send_confidential_ask: &'static str,
    /// The button that sends after the notice. Shows: a meeting's page, the send panel. When: the notice is shown.
    /// At most: one word. Placeholders: none.
    pub send_confirm: &'static str,
    /// The button that closes the panel, nothing sent. Shows: a meeting's page, the send panel. When: the panel is
    /// open. At most: one word. Placeholders: none.
    pub send_cancel: &'static str,
    /// While kitokey-send runs. Shows: a meeting's page, as a status. When: a send was asked. At most: one line.
    /// Placeholders: {} the destination's name.
    pub send_sending: &'static str,
    /// A send that arrived. Shows: a meeting's page, as a status. When: kitokey-send said sent. At most: one line.
    /// Placeholders: {} the destination's name, {} the time.
    pub send_done: &'static str,
    /// Each send recorded, listed under the summary (PDR-0007 §5). Shows: a meeting's page, a list item. When: the
    /// meeting was sent. At most: one line. Placeholders: {} the destination's name, {} the day and time.
    pub send_listed: &'static str,
    /// posting/v1's address.refused: what to do. Shows: a meeting's page, as a status. When: a send refused before
    /// it left. At most: two lines. Placeholders: {} the destination's name.
    pub send_address_refused: &'static str,
    /// posting/v1's meeting.unreadable. Shows: a meeting's page, as a status. When: a send failed. At most: one
    /// line. Placeholders: none.
    pub send_unreadable: &'static str,
    /// posting/v1's meeting.confidential. Shows: a meeting's page, as a status. When: a confidential meeting's
    /// send not confirmed (D119). At most: one line. Placeholders: none.
    pub send_confidential: &'static str,
    /// posting/v1's meeting.no_summary. Shows: a meeting's page, as a status. When: a send failed. At most: one
    /// line. Placeholders: none.
    pub send_no_summary: &'static str,
    /// posting/v1's summary.too_large: send it as a file instead. Shows: a meeting's page, as a status. When: a
    /// send failed. At most: one line. Placeholders: {} the destination's name.
    pub send_too_large: &'static str,
    /// posting/v1's network.unreachable: nothing left. Shows: a meeting's page, as a status. When: a send failed.
    /// At most: one line. Placeholders: none.
    pub send_offline: &'static str,
    /// posting/v1's destination.silent: it may have arrived, look before sending again. Shows: a meeting's page, as
    /// a status. When: no answer in time. At most: two lines. Placeholders: {} the destination's name.
    pub send_silent: &'static str,
    /// posting/v1's destination.gone: set it up again. Shows: a meeting's page, as a status. When: the webhook is
    /// deleted or revoked. At most: two lines. Placeholders: {} the destination's name.
    pub send_gone: &'static str,
    /// posting/v1's destination.busy: wait, then send again. Shows: a meeting's page, as a status. When: the
    /// destination said 429. At most: one line. Placeholders: {} the destination's name, {} seconds.
    pub send_busy: &'static str,
    /// posting/v1's destination.busy when the destination said no delay: wait, then send again. Shows: a meeting's
    /// page, as a status. When: the destination said 429 without Retry-After. At most: one line. Placeholders: {}
    /// the destination's name.
    pub send_busy_later: &'static str,
    /// posting/v1's destination.refused: the kind may not match. Shows: a meeting's page, as a status. When:
    /// another 3xx or 4xx. At most: two lines. Placeholders: {} the destination's name.
    pub send_refused: &'static str,
    /// posting/v1's destination.failed: try later. Shows: a meeting's page, as a status. When: a 5xx. At most: one
    /// line. Placeholders: {} the destination's name.
    pub send_failed: &'static str,
    /// A send that failed without a code a person acts on. Shows: a meeting's page, as a status. When: kitokey-send
    /// failed otherwise. At most: one line. Placeholders: none.
    pub send_not_sent: &'static str,
    /// Before Kitokey's version, at the foot of « Général », as GNOME's « About » and Windows' Settings
    /// say it (#1089); the number follows, as `--version` gives it. Shows: Settings, « Général », its
    /// last line, in secondary text the user can select to copy. When: always. At most: one word.
    /// Placeholders: none.
    pub version_said: &'static str,
    /// Shows: Settings, « Général », folded « Avancé » at its bottom, a switch: its name, then its
    /// line under it (D29). When: always. At most: one line for the name, the line wraps.
    /// Placeholders: none.
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
    /// A meeting's minutes in a structure (`minutes/v5`, D111): the sections of the five
    /// structures beyond `general`, as the page names them, whatever titles the model wrote, in the
    /// order of `minutes::SECTION_IDS` after its first three — updates, feedback, progress, blockers,
    /// needs, questions, next steps, background, answers, status, risks, decisions.
    /// Shows: a meeting's page, as a heading. When: the section titles of minutes made in a
    /// structure. At most: about 20 characters. Placeholders: none.
    pub structure_sections: [&'static str; 12],
    /// The actions' three cells, named for assistive technology alone (D46), and what a meeting
    /// left unsettled, as the summary writes it.
    /// Shows: a meeting's page, as a name for a screen reader. When: screen reader: column names of
    /// the actions table. At most: about 12 characters. Placeholders: none.
    pub actions_cells: [&'static str; 3],
    /// Shows: a meeting's page, as a status. When: written by the summary model when an action has
    /// no owner/date; MATCHED by code. At most: no limit. Placeholders: none.
    pub unsettled: &'static str,
    /// A model of this computer that holds the minutes' form, as Settings offer it ({}).
    /// Shows: Settings, « Modèles », the summary's step, a model of this computer in the list.
    /// When: it holds the summary's form. At most: one line. Placeholders: {} the model.
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
    /// Shows: Settings, « Après un enregistrement », a row: its name, then its line under it. When:
    /// always. At most: one line for the name, the line wraps. Placeholders: none.
    pub sessions_folder: [&'static str; 2],
    /// Shows: Settings, « Après un enregistrement », the folder's button: the system's dialog.
    /// When: always. At most: one line. Placeholders: none.
    pub choose: &'static str,
    /// « Exporter chaque compte rendu » (D109): the folder every new summary is written to too.
    /// Shows: Settings, « Après un enregistrement », a row: its name, then its line under it. When:
    /// always. At most: one line for the name, the line wraps. Placeholders: none.
    pub export_folder: [&'static str; 2],
    /// Shows: Settings, « Après un enregistrement », the export row's folder, as its value. When: no
    /// folder chosen. At most: about 20 characters. Placeholders: none.
    pub export_none: &'static str,
    /// Shows: Settings, « Après un enregistrement », the export row, as a button. When: a folder is
    /// chosen. At most: about 20 characters. Placeholders: none.
    pub export_stop: &'static str,
    /// Shows: a meeting's page, as a note under its summary (D109). When: a folder is chosen and the
    /// meeting is confidential, so never written there (D60). At most: one line. Placeholders: none.
    pub export_confidential: &'static str,
    /// Shows: a meeting's page, as an error under its summary (D109). When: its summary could not be
    /// written to the export folder. At most: two lines. Placeholders: none.
    pub export_failed: &'static str,
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
    /// Shows: « Vérifier le son » (D88), its title, opened from Settings, « Général » or the bar;
    /// and its row there. When: always. At most: one line. Placeholders: none.
    pub check_sound: &'static str,
    /// Shows: Settings, « Général », the line under « Vérifier le son », and the bar's. When:
    /// always. At most: no limit. Placeholders: none.
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
    /// « Dates mentionnées » (D115): the dates a summary's actions name, offered to the user's calendar.
    /// Shows: a meeting's page, a button under its actions, and the panel's title. When: an action's by when
    /// names a date. At most: one line. Placeholders: none.
    pub dates_open: &'static str,
    /// Shows: the « Dates mentionnées » panel, under its title. When: it is open. At most: two lines.
    /// Placeholders: none.
    pub dates_says: &'static str,
    /// Shows: the « Dates mentionnées » panel, the label of the day the dates are read against. When: it is
    /// open. At most: about 20 characters. Placeholders: none.
    pub dates_day: &'static str,
    /// Shows: the « Dates mentionnées » panel, a button. When: a meeting not confidential. At most: about 28
    /// characters. Placeholders: none.
    pub dates_google: &'static str,
    /// Shows: the « Dates mentionnées » panel, a button. When: always, a confidential meeting's too. At most: about
    /// 40 characters. Placeholders: none.
    pub dates_ics: &'static str,
    /// Shows: the « Dates mentionnées » panel, as a status. When: the `.ics` is written. At most: one line.
    /// Placeholders: {} the file's name.
    pub dates_saved: &'static str,
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
    /// Shows: the recording bar and Redo…, as a choice of a list; a meeting's page, after the model
    /// of minutes this computer wrote (#1080). When: appended to a local model's name. At most:
    /// about 20 characters. Placeholders: none.
    pub on_computer: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a heading. When: redo the transcript. At most:
    /// no limit. Placeholders: none.
    pub redo_transcript_title: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as a help line. When: what happens to the current
    /// transcript. At most: no limit. Placeholders: none.
    pub redo_transcript_note: &'static str,
    /// Shows: a meeting's page, under its summary, as a button (D113). When: a redo replaced a version,
    /// kept beside. At most: no limit. Placeholders: {} when that version was made and its model, as the
    /// page's line says them.
    pub previous_back: &'static str,
    /// Shows: a meeting's page, under « Revenir à la version précédente », as a help line (D113). When:
    /// the button shows. At most: no limit. Placeholders: none.
    pub previous_help: &'static str,
    /// Shows: a meeting's page, under « Revenir à la version précédente », as an error (D113). When: the
    /// two versions could not swap. At most: no limit. Placeholders: {} what is wrong, as the failure says it.
    pub previous_failed: &'static str,
    /// Shows: a meeting's page, its Redo… panel, as an error above its button (#771). When: the redo
    /// could not start. At most: no limit. Placeholders: {} what is wrong and what to do, as the failure
    /// says it.
    pub redo_not_done: &'static str,
    /// Shows: a meeting's page, its transcript's Redo… panel, under its note: the summary is not made
    /// again with it (the decider, 2026-11-18, D113). When: the meeting has a summary. At most: no limit.
    /// Placeholders: none.
    pub redo_keeps_minutes: &'static str,
    /// What a meeting lacks, offered on its page; what is said once asked; where a step ran.
    /// Shows: a meeting's page, as a button. When: meeting lacks minutes. At most: about 30
    /// characters. Placeholders: none.
    pub make_minutes: &'static str,
    /// Shows: a meeting's page, as a button. When: meeting lacks a transcript. At most: about 30
    /// characters. Placeholders: none.
    pub make_transcript: &'static str,
    /// Shows: a meeting's page, the button beside the summary's list of models, its transcript whole and
    /// no summary yet (D93): « Faire le compte rendu avec Qwen3 14b ». When: a model is chosen there or
    /// named by Settings; else `make_minutes`. At most: no limit. Placeholders: {} the model, as people say it.
    pub make_minutes_with: &'static str,
    /// Shows: a meeting's page, above the button that makes the summary again, the model that made the
    /// one shown (D93). When: another model is chosen in its line. At most: no limit. Placeholders: {}
    /// the model, as its line says it.
    pub minutes_made_with: &'static str,
    /// Shows: a meeting's page, under the summary's line: a language model wrote it ([Store
    /// Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) 11.16, #1092).
    /// When: a summary is shown. At most: one line. Placeholders: none.
    pub minutes_ai: &'static str,
    /// Shows: under a summary's section title, in the soft colour (#1234). When: the section was made with
    /// nothing in it. At most: 20 characters. Placeholders: none.
    pub section_empty: &'static str,
    /// Shows: a meeting's page, above the button that makes the transcript again, the model that made
    /// the one shown (D93). When: another model is chosen in its line. At most: no limit.
    /// Placeholders: {} the model, as its line says it.
    pub transcript_made_with: &'static str,
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
    /// Shows: a meeting's page, its transcript's card, as a button (#933). When: the transcript was not made.
    /// At most: one line in the card. Placeholders: none.
    pub transcript_try_again: &'static str,
    /// Shows: a meeting's page, its summary's card, as a button (#933). When: the summary was not made, or
    /// Ollama did not answer. At most: one line in the card. Placeholders: none.
    pub minutes_try_again: &'static str,
    /// Shows: a meeting's page, its transcript's card, as a button (#933). When: the model is missing; once
    /// fetched, the transcript is made. At most: one line. Placeholders: {} the model's size, {} its host.
    pub transcript_fetch_then: &'static str,
    /// Shows: a meeting's page, its summary's card, as a button (#933). When: Ollama holds no model; once
    /// fetched, the summary is made. At most: one line. Placeholders: {} the model, as Ollama names it.
    pub minutes_fetch_then: &'static str,
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
    /// Shows: a meeting's chip, in the list and on its page (#772). When: its minutes are kept and the
    /// last Refaire… of them failed. At most: one line. Placeholders: none.
    pub chip_redo_failed: &'static str,
    /// Shows: a meeting's page, the minutes' state, before the cause when there is one (#772). When:
    /// its minutes are kept and the last Refaire… of them failed. At most: no limit. Placeholders: none.
    pub minutes_redo_failed: &'static str,
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
    /// the transcript. At most: no limit. Placeholders: in the line, {} the minutes, then {} the
    /// words.
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
    /// Shows: Settings, « Modèles », the summary's step, beside « Tester » while it asks. When: the
    /// model is this computer's Ollama's (#1080): no address said. At most: no limit. Placeholders: none.
    pub test_sends_here: &'static str,
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
    /// Shows: the window, as an error. When: a setting/change not saved. At most: no limit.
    /// Placeholders: {} failure sentence (lower-case start).
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
    /// Shows: the first run's assistant and a meeting's page, as an error. When: Ollama not
    /// answering: minutes wait. At most: no limit. Placeholders: none.
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
    /// Shows: a meeting's page, as a button. When: context empty: add one. At most: about 24
    /// characters. Placeholders: none.
    pub context_add: &'static str,
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
    /// Shows: a meeting's context card, the button under `context_changed`, naming what it starts (D113):
    /// the transcript, then the summary when one follows it. When: the context changed after they were
    /// made; the second where the meeting has a summary or Settings make one after a transcript, else the
    /// first. At most: about 45 characters. Placeholders: none.
    pub context_redo: [&'static str; 2],
    /// The language a meeting is spoken in (D65).
    /// Shows: a meeting's page, as a label. When: row: spoken language. At most: about 20
    /// characters. Placeholders: none.
    pub language_label: &'static str,
    /// Shows: a meeting's page, as a choice of a list: its first, when Settings name a language —
    /// detecting it, `language_detected` says it as Settings do (#1074). When: always. At most:
    /// about 30 characters. Placeholders: {} the language Settings name.
    pub language_default: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, the language of the meetings, its
    /// first choice. When: Whisper or a service hears. At most: one line. Placeholders: none.
    pub language_detected: &'static str,
    /// Shows: Settings, « Modèles », the transcription's step, a list's label. When: always. At
    /// most: one line. Placeholders: none.
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
    /// SpeechAnalyzer's language when the settings name none: the system's, never the app's (D64).
    /// Shows: Settings, « Modèles », the transcription's step, the language of the meetings, its
    /// first choice. When: Apple's SpeechAnalyzer hears (D64). At most: one line. Placeholders:
    /// none.
    pub language_as_system: &'static str,
    /// A meeting's own language SpeechAnalyzer does not hear: the one it hears in, said (D65).
    /// Shows: a meeting's page, on a Mac, as a choice of a list. When: {} language — SpeechAnalyzer
    /// hears {}. At most: no limit. Placeholders: {} the language chosen, then the one heard.
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
    /// Shows: a meeting's page, as a button naming what it starts, the summary (D113). When: redo minutes
    /// with corrections. At most: about 45 characters. Placeholders: none.
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
    /// Shows: a meeting's page, the transcript's copy button, as its name for a screen reader and its
    /// tooltip (D109). When: a transcript is there. At most: 24 characters. Placeholders: none.
    pub copy_transcript: &'static str,
    /// Shows: a meeting's page, beside the transcript's copy button, in its live region (D109). When:
    /// the clipboard refused the copy. At most: one line. Placeholders: none.
    pub transcript_not_copied: &'static str,
    /// Shows: the clipboard, each line of a transcript copied (D109). When: the transcript's copy
    /// button is pressed. At most: no limit. Placeholders: `{}` the time it was said at, `{}` who said
    /// it, as the page names them, `{}` what was said.
    pub transcript_copied_line: &'static str,
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
    /// Shows: a meeting's page, as a status. When: list item, its summary sent to a destination (D110,
    /// PDR-0007 §4). At most: no limit. Placeholders: {} the destination's name.
    pub confidential_left_summary: &'static str,
    /// Shows: a meeting's page, as an error. When: switch not saved. At most: no limit.
    /// Placeholders: none.
    pub confidential_not_saved: &'static str,
    /// Shows: a meeting's page, as an error. When: confidential + Ollama has no model. At most: no
    /// limit. Placeholders: none.
    pub confidential_ollama_empty: &'static str,
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
            heard_by: "Transcribed by {}, on this computer",
            transcribed_on: "Transcribed on {}",
            minutes_of: "Summary of {}",
            made_in: "in {}",
            here: "You",
            remote: "Others",
            voice: "voice {}",
            here_described: "You — your microphone",
            remote_described: "Others — the call’s audio",
            speakers_open: "Name the speakers…",
            speakers_save: "Save",
            speakers_cancel: "Cancel",
            speakers_hint: "Kept with this meeting. Leave a field empty to keep “You” or “Others”.",
            speakers_not_saved: "Kitokey could not save these names. Try again.",
            who_spoke: "Who spoke when: you on top, others below",
            who_spoke_later: "Who spoke when appears once it is transcribed.",
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
            destinations_title: "Sending summaries",
            destinations_said: "The channels you can send a summary to. Nothing is sent unless you ask, and never a confidential meeting.",
            destinations_none: "No destination yet.",
            destination_kinds: ["Webhook", "Discord", "Slack"],
            destination_fields: ["Kind", "Name", "Webhook address"],
            destination_address_said: "Paste the channel’s webhook address. It’s kept in your system keychain and never shown in full again.",
            destination_add: "Add",
            destination_open: "Add a destination",
            destination_forget: "Remove",
            destination_added: "Destination {} added.",
            destination_not_added: "Destination not added: {}",
            send_open: "Send…",
            send_to: "Send to “{}”",
            send_notice: "The title, the summary and its actions go to “{}”. Never the audio, never the transcript. A summary sent cannot be taken back.",
            send_confidential_ask: "This meeting is confidential: its summary will leave this computer for “{}”.",
            send_confirm: "Send",
            send_cancel: "Cancel",
            send_sending: "Sending to “{}”…",
            send_done: "Sent to “{}” at {}.",
            send_listed: "Sent to “{}” on {}",
            send_address_refused: "The address of “{}” is refused. Remove it, then add the channel’s https webhook address in Settings.",
            send_unreadable: "Kitokey couldn’t read this meeting. Check its folder, then try again.",
            send_confidential: "This meeting is confidential: it’s sent only once you confirm.",
            send_no_summary: "There’s no summary to send yet.",
            send_too_large: "The summary is too long for “{}”. Export it as a file instead.",
            send_offline: "No connection: nothing was sent. Check your connection, then try again.",
            send_silent: "“{}” didn’t answer: the summary may have arrived; look in the channel before sending again.",
            send_gone: "“{}” no longer exists or was revoked. Set the destination up again, with a new address.",
            send_busy: "“{}” is receiving too many sends. Try again in {} s.",
            send_busy_later: "“{}” is receiving too many sends. Try again later.",
            send_refused: "“{}” refused it. Check that the destination’s kind matches its address.",
            send_failed: "“{}” has a problem. Try again later.",
            send_not_sent: "The summary couldn’t be sent. Try again.",
            version_said: "Version",
            developer: ["Developer mode", "Shows technical details under each error, to help troubleshoot."],
            program_said: "Technical details: {}",
            minutes_sections: ["Highlights", "Actions", "Key points"],
            structure_sections: [
                "Updates",
                "Feedback",
                "Progress",
                "Blockers",
                "Needs",
                "Questions",
                "Next steps",
                "Background",
                "Answers",
                "Status",
                "Risks",
                "Decisions",
            ],
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
            export_folder: [
                "Export every summary",
                "Each new summary is also written to this folder, in Markdown, named by the meeting’s date and time. Redoing a summary replaces its file in the folder.",
            ],
            export_none: "No folder",
            export_stop: "Stop exporting",
            export_confidential: "Confidential: not exported.",
            export_failed: "Kitokey could not write the summary to the export folder. Choose another folder in Settings.",
            small: "Recording bar",
            small_mic: "Your microphone",
            small_meeting: "Meeting audio",
            small_note: "When you stop, transcription starts on its own. You’ll get a notification.",
            bar_silent_me: "Your microphone: silent for 2 min — still recording.",
            bar_silent_others: "Meeting: silent for 2 min — still recording.",
            small_nothing: "No signal: check the microphone isn’t muted on the device and that it’s switched on.",
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
            dates_open: "Dates mentioned",
            dates_says: "Tick the ones to put in your calendar. Nothing leaves this computer until you choose where.",
            dates_day: "Meeting day",
            dates_google: "Open in Google Calendar",
            dates_ics: "Save as a calendar file (.ics)",
            dates_saved: "Saved in {}.",
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
            previous_back: "Go back to the previous version ({})",
            previous_help: "The version you leave becomes the previous one: the same button brings it back.",
            previous_failed: "Couldn’t go back to the previous version: {}",
            redo_not_done: "Not redone: {}",
            redo_keeps_minutes: "The current summary is kept; write it again afterwards if needed.",
            make_minutes: "Write summary",
            make_transcript: "Transcribe",
            make_minutes_with: "Write summary with {}",
            minutes_made_with: "This summary was written with {}.",
            minutes_ai: "Written by AI: read it before you share it.",
            section_empty: "Nothing noted",
            transcript_made_with: "This transcript was made with {}.",
            minutes_making: "Writing summary",
            minutes_waiting: "Queued · {} ahead",
            minutes_awaiting: "Waiting for your OK to send to {}",
            agree_open: "Review",
            minutes_not_made: "The summary couldn’t be written. The transcript is unchanged.",
            try_again: "Try again",
            transcript_try_again: "Try the transcript again",
            minutes_try_again: "Try the summary again",
            transcript_fetch_then: "Download the model ({}, {}) and transcribe",
            minutes_fetch_then: "Download {} and write the summary",
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
            agree_send: "Send and write",
            agree_not: "Don’t send",
            agree_local: "Write the summary on this computer instead",
            agree_local_write: "Write the summary here",
            shortcut: ["Keyboard shortcut", "Starts or stops a recording from anywhere."],
            shortcut_none: "No shortcut set",
            shortcut_where: "You can change it in your system’s keyboard settings.",
            shortcut_open: "Keyboard settings",
            test: "Test",
            test_sends: "The test sends a short sentence to {} — nothing from your meetings.",
            test_sends_here: "The test sends a short sentence to Ollama, on this computer — nothing from your meetings.",
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
            context_redo: ["Write the transcript again", "Write the transcript and the summary again"],
            language_label: "Spoken language",
            language_default: "{}, from Settings",
            language_detected: "Detect automatically",
            meetings_language: "Meeting language",
            language_not_saved: "Language not saved: {}",
            rename_meeting: "Rename this meeting",
            name_meeting: "Name this meeting",
            name_not_saved: "Name not saved: {}",
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
            redo_corrections: "Write the summary again with your edits",
            copy_minutes: "Copy summary",
            minutes_copied: "Copied",
            minutes_not_copied: "Couldn’t copy the summary.",
            copy_transcript: "Copy transcript",
            transcript_not_copied: "Couldn’t copy the transcript.",
            transcript_copied_line: "[{}] {}: {}",
            redo_loses_corrections: "Your edits to this transcript will be lost.",
            context_not_saved: "Couldn’t save the context.",
            confidential_name: "Confidential meeting",
            confidential_says: "Nothing leaves this computer",
            confidential_chip: "Confidential",
            confidential_left: "Already sent earlier: {}.",
            confidential_left_sound: "the audio to {}",
            confidential_left_transcript: "the transcript to {}",
            confidential_left_summary: "the summary to “{}”",
            confidential_not_saved: "Couldn’t mark the meeting as confidential. Try again.",
            confidential_ollama_empty: "Ollama has no model on this computer, so the summary will wait.",
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
            speakers_open: "Nommer les intervenants…",
            speakers_save: "Enregistrer",
            speakers_cancel: "Annuler",
            speakers_hint: "Gardés avec cette réunion. Laissez un champ vide pour garder «\u{a0}Vous\u{a0}» ou «\u{a0}Les autres\u{a0}».",
            speakers_not_saved: "Kitokey n’a pas pu enregistrer ces noms. Réessayez.",
            who_spoke: "Temps de parole au fil de la réunion\u{a0}: vous en haut, les autres en bas",
            who_spoke_later: "Le temps de parole s’affichera après la transcription.",
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
            destinations_title: "Envoi des comptes rendus",
            destinations_said: "Les canaux où vous pourrez envoyer un compte rendu. Rien n’est envoyé sans que vous le demandiez, et jamais une réunion confidentielle.",
            destinations_none: "Aucune destination pour l’instant.",
            destination_kinds: ["Webhook", "Discord", "Slack"],
            destination_fields: ["Type", "Nom", "Adresse du webhook"],
            destination_address_said: "Collez l’adresse du webhook du canal. Elle est gardée dans le trousseau du système et n’est plus jamais affichée en entier.",
            destination_add: "Ajouter",
            destination_open: "Ajouter une destination",
            destination_forget: "Supprimer",
            destination_added: "Destination {} ajoutée.",
            destination_not_added: "Destination non ajoutée\u{a0}: {}",
            send_open: "Envoyer…",
            send_to: "Envoyer à « {} »",
            send_notice: "Le titre, le compte rendu et ses actions partent vers « {} ». Jamais le son, jamais la transcription. Un compte rendu envoyé ne peut pas être repris.",
            send_confidential_ask: "Cette réunion est confidentielle\u{a0}: son compte rendu va quitter cet ordinateur pour «\u{a0}{}\u{a0}».",
            send_confirm: "Envoyer",
            send_cancel: "Annuler",
            send_sending: "Envoi vers « {} »…",
            send_done: "Envoyé à « {} » à {}.",
            send_listed: "Envoyé à « {} » le {}",
            send_address_refused: "L’adresse de « {} » est refusée. Retirez-la, puis ajoutez l’adresse https du webhook du canal dans les Réglages.",
            send_unreadable: "Kitokey n’a pas pu lire cette réunion. Vérifiez son dossier, puis réessayez.",
            send_confidential: "Réunion confidentielle\u{a0}: elle n’est envoyée qu’après votre confirmation.",
            send_no_summary: "Pas encore de compte rendu à envoyer.",
            send_too_large: "Compte rendu trop long pour « {} ». Exportez-le en fichier à la place.",
            send_offline: "Pas de connexion : rien n’est parti. Vérifiez la connexion, puis réessayez.",
            send_silent: "« {} » n’a pas répondu : le compte rendu est peut-être arrivé ; regardez dans le canal avant de renvoyer.",
            send_gone: "« {} » n’existe plus ou a été révoqué. Reconfigurez la destination avec une nouvelle adresse.",
            send_busy: "« {} » reçoit trop d’envois. Réessayez dans {} s.",
            send_busy_later: "« {} » reçoit trop d’envois. Réessayez plus tard.",
            send_refused: "« {} » a refusé l’envoi. Vérifiez que le type de la destination correspond à son adresse.",
            send_failed: "« {} » a un problème. Réessayez plus tard.",
            send_not_sent: "Le compte rendu n’a pas pu partir. Réessayez.",
            version_said: "Version",
            developer: [
                "Mode développeur",
                "Affiche le détail technique sous chaque erreur, pour aider au diagnostic.",
            ],
            program_said: "Détail technique\u{a0}: {}",
            minutes_sections: ["Points saillants", "Actions", "Points clés"],
            structure_sections: [
                "Avancement",
                "Retours",
                "Avancées",
                "Blocages",
                "Besoins",
                "Questions",
                "Prochaines étapes",
                "Parcours",
                "Réponses",
                "État",
                "Risques",
                "Décisions",
            ],
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
            export_folder: [
                "Exporter chaque compte rendu",
                "Chaque nouveau compte rendu est aussi écrit dans ce dossier, en Markdown, nommé par la date et l’heure de la réunion. Refaire un compte rendu remplace son fichier dans le dossier.",
            ],
            export_none: "Aucun dossier",
            export_stop: "Ne plus exporter",
            export_confidential: "Confidentielle\u{a0}: pas exportée.",
            export_failed: "Impossible d’écrire le compte rendu dans le dossier d’export. Choisissez un autre dossier dans les Réglages.",
            small: "Barre d’enregistrement",
            small_mic: "Votre micro",
            small_meeting: "Le son de la réunion",
            bar_silent_me: "Votre micro\u{a0}: rien depuis 2\u{a0}min — enregistrement en cours.",
            bar_silent_others: "La réunion\u{a0}: rien depuis 2\u{a0}min — enregistrement en cours.",
            small_nothing: "Aucun signal\u{a0}: vérifiez que le micro n’est pas coupé sur l’appareil et qu’il est allumé.",
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
            dates_open: "Dates mentionnées",
            dates_says: "Cochez celles à mettre dans votre agenda. Rien ne quitte cet ordinateur avant que vous choisissiez où.",
            dates_day: "Jour de la réunion",
            dates_google: "Ouvrir dans Google Agenda",
            dates_ics: "Enregistrer un fichier d’agenda (.ics)",
            dates_saved: "Enregistré dans {}.",
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
            previous_back: "Revenir à la version précédente ({})",
            previous_help: "La version quittée devient la précédente\u{a0}: le même bouton la ramène.",
            previous_failed: "Impossible de revenir à la version précédente\u{a0}: {}",
            redo_not_done: "Pas refait\u{a0}: {}",
            redo_keeps_minutes: "Le compte rendu actuel est gardé\u{202f}; refaites-le ensuite si besoin.",
            make_minutes: "Rédiger le compte rendu",
            make_transcript: "Transcrire",
            make_minutes_with: "Rédiger le compte rendu avec {}",
            minutes_made_with: "Ce compte rendu a été fait avec {}.",
            minutes_ai: "Rédigé par une IA\u{a0}: relisez-le avant de le partager.",
            section_empty: "Rien de noté",
            transcript_made_with: "Cette transcription a été faite avec {}.",
            minutes_making: "Rédaction du compte rendu",
            minutes_waiting: "En attente · {} avant",
            minutes_awaiting: "En attente de votre accord pour l’envoi à {}",
            agree_open: "Voir et accepter",
            minutes_not_made: "Le compte rendu n’a pas pu être rédigé. La transcription n’a pas changé.",
            try_again: "Réessayer",
            transcript_try_again: "Réessayer la transcription",
            minutes_try_again: "Réessayer le compte rendu",
            transcript_fetch_then: "Télécharger le modèle ({}, {}) et transcrire",
            minutes_fetch_then: "Télécharger {} et rédiger le compte rendu",
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
            agree_send: "Envoyer et rédiger",
            agree_not: "Ne pas envoyer",
            agree_local: "Rédiger plutôt le compte rendu sur cet ordinateur",
            agree_local_write: "Rédiger le compte rendu ici",
            shortcut: ["Raccourci clavier", "Démarre ou arrête un enregistrement, où que vous soyez."],
            shortcut_none: "Aucun raccourci défini",
            shortcut_where: "Vous pouvez le modifier dans les réglages clavier du système.",
            shortcut_open: "Réglages du clavier",
            test: "Tester",
            test_sends: "Le test envoie une courte phrase à {}, rien de vos réunions.",
            test_sends_here: "Le test envoie une courte phrase à Ollama, sur cet ordinateur — rien de vos réunions.",
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
            context_redo: ["Refaire la transcription", "Refaire la transcription et le compte rendu"],
            language_label: "Langue parlée",
            language_default: "{}, selon les Réglages",
            language_detected: "Détection automatique",
            meetings_language: "Langue des réunions",
            language_not_saved: "Langue non enregistrée\u{a0}: {}",
            rename_meeting: "Renommer cette réunion",
            name_meeting: "Nommer cette réunion",
            name_not_saved: "Nom non enregistré\u{a0}: {}",
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
            redo_corrections: "Refaire le compte rendu avec les corrections",
            copy_minutes: "Copier le compte rendu",
            minutes_copied: "Copié",
            minutes_not_copied: "Impossible de copier le compte rendu.",
            copy_transcript: "Copier la transcription",
            transcript_not_copied: "Impossible de copier la transcription.",
            transcript_copied_line: "[{}] {}\u{a0}: {}",
            redo_loses_corrections: "Vos corrections de cette transcription seront perdues.",
            context_not_saved: "Impossible d’enregistrer le contexte.",
            confidential_name: "Réunion confidentielle",
            confidential_says: "Rien ne quitte cet ordinateur",
            confidential_chip: "Confidentielle",
            confidential_left: "Déjà envoyé auparavant\u{a0}: {}.",
            confidential_left_sound: "le son à {}",
            confidential_left_transcript: "la transcription à {}",
            confidential_left_summary: "le compte rendu à « {} »",
            confidential_not_saved: "Impossible de marquer la réunion comme confidentielle. Réessayez.",
            confidential_ollama_empty: "Ollama n’a aucun modèle sur cet ordinateur\u{a0}: le compte rendu attendra.",
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
    LinkNotOpened,
    DatesNotSaved,
    KeyboardSettings,
    Keyring,
    ProgramNotStarted,
    TestRefused,
    /// A destination (D110): no name a person reads, a name already given, or an address refused.
    DestinationUnnamed,
    DestinationNamed,
    DestinationRefused,
}

/// A failure's sentence. Most follow the page's own « not saved: » and so start in lower case; those
/// the page shows alone, `FolderNotOpened`, `LinkNotOpened`, `KeyboardSettings` and `SettingsLocked`, are whole.
pub fn failure(failure: Failure, language: Language) -> String {
    use Language::*;
    match (failure, language) {
        // Shows: under a command the window refused, after « not saved: », or alone where the page
        // shows it. When: the command named by its arm fails. At most: no limit. Placeholders:
        // none.
        (Failure::Trash(refusal), _) => return audio_trashed(language, &Err(refusal)).body,
        (Failure::Unexpected, English) => "Something went wrong. Try again; if it keeps happening, restart Kitokey.",
        (Failure::Unexpected, French) => "Une erreur s’est produite. Réessayez\u{202f}; si le problème persiste, redémarrez Kitokey.",
        (Failure::MeetingGone, English) => "this meeting is no longer in the meetings folder.",
        (Failure::MeetingGone, French) => "cette réunion n’est plus dans le dossier des réunions.",
        (Failure::NoMeeting, English) => "this meeting is no longer in the meetings folder. Go back to the list.",
        (Failure::NoMeeting, French) => "cette réunion n’est plus dans le dossier des réunions. Revenez à la liste.",
        (Failure::Busy, English) => "this meeting is being processed. Try again when it’s done.",
        (Failure::Busy, French) => "cette réunion est en cours de traitement. Réessayez une fois terminé.",
        (Failure::NotComplete, English) => "its transcript is incomplete. Redo it first.",
        (Failure::NotComplete, French) => "sa transcription est incomplète. Refaites-la d’abord.",
        (Failure::MeetingFile, English) => "can’t write to its folder. Check disk space and folder permissions, then try again.",
        (Failure::MeetingFile, French) => {
            "impossible d’écrire dans son dossier. Vérifiez l’espace disque et vos droits sur ce dossier, puis réessayez."
        }
        (Failure::Confidential, English) => "this meeting is confidential, so nothing is sent.",
        (Failure::Confidential, French) => "cette réunion est confidentielle\u{a0}: rien n’est envoyé.",
        (Failure::NoneChosen, English) => "no model chosen. Choose one in Settings, then try again.",
        (Failure::NoneChosen, French) => "aucun modèle choisi. Choisissez-en un dans les Réglages, puis réessayez.",
        (Failure::NoTranscript, English) => "this meeting has no transcript yet. Transcribe it first.",
        (Failure::NoTranscript, French) => "cette réunion n’a pas encore de transcription. Transcrivez-la d’abord.",
        (Failure::ProviderChanged, English) => "the summary service changed. Review this page again.",
        (Failure::ProviderChanged, French) => "le service de compte rendu a changé. Vérifiez à nouveau cette page.",
        (Failure::ModelUnnamed, English) => "enter a model name, then try again.",
        (Failure::ModelUnnamed, French) => "indiquez un modèle, puis réessayez.",
        (Failure::ModelAbsent, English) => "this model isn’t installed. Download it in Settings.",
        (Failure::ModelAbsent, French) => "ce modèle n’est pas installé. Téléchargez-le dans les Réglages.",
        (Failure::BadAddress, English) => "invalid address. Enter the full address, starting with https://.",
        (Failure::BadAddress, French) => "adresse invalide. Saisissez l’adresse complète, commençant par https://.",
        (Failure::FolderUnnamed, English) => "choose a folder, then try again.",
        (Failure::FolderUnnamed, French) => "choisissez un dossier, puis réessayez.",
        (Failure::FolderNotWhole, English) => "enter the folder’s full path (starting with / or ~).",
        (Failure::FolderNotWhole, French) => "indiquez le chemin complet du dossier (commençant par / ou ~).",
        (Failure::FolderUnmade, English) => "can’t create this folder. Check that you have permission to write there.",
        (Failure::FolderUnmade, French) => "impossible de créer ce dossier. Vérifiez que vous avez le droit d’y écrire.",
        (Failure::Autostart, English) => "can’t change the start-at-login setting. Check that your home folder is writable.",
        (Failure::Autostart, French) => {
            "impossible de modifier le lancement automatique. Vérifiez que votre dossier personnel est accessible en écriture."
        }
        // Shows: under a command the window refused, after « not saved: », then the refusal.
        // When: the settings file cannot be read as settings. At most: no limit. Placeholders: {} the refusal.
        (Failure::SettingsUnread(why), English) => return format!("the settings file can’t be read. {}", refused(&why, language)),
        (Failure::SettingsUnread(why), French) => {
            return format!("le fichier de réglages est illisible. {}", refused(&why, language));
        }
        // Shows: alone on Settings, then the refusal. When: the settings file cannot be read.
        // At most: no limit. Placeholders: {} the refusal.
        (Failure::SettingsLocked(why), English) => {
            return format!(
                "The settings file can’t be read. Kitokey won’t change it until it’s fixed. {}",
                refused(&why, language)
            );
        }
        (Failure::SettingsLocked(why), French) => {
            return format!(
                "Le fichier de réglages est illisible\u{a0}: Kitokey n’y touchera pas tant qu’il n’est pas corrigé. {}",
                refused(&why, language)
            );
        }
        // Shows: under a command the window refused, after « not saved: », or alone where the page
        // shows it (the folder, the keyboard settings). When: the command named by its arm fails.
        // At most: no limit. Placeholders: none.
        (Failure::SettingsUnwritten, English) => "can’t save the settings file. Check disk space, then try again.",
        (Failure::SettingsUnwritten, French) => {
            "impossible d’enregistrer le fichier de réglages. Vérifiez l’espace disque, puis réessayez."
        }
        (Failure::FolderNotOpened, English) => "Couldn’t open the folder. Open it from your file manager.",
        (Failure::FolderNotOpened, French) => "Impossible d’ouvrir le dossier. Ouvrez-le depuis votre gestionnaire de fichiers.",
        (Failure::DatesNotSaved, English) => "the calendar file wasn’t written. Choose another folder, then try again.",
        (Failure::DatesNotSaved, French) => "le fichier d’agenda n’a pas pu être écrit. Choisissez un autre dossier, puis réessayez.",
        (Failure::LinkNotOpened, English) => "Couldn’t open your browser. Save the dates as a calendar file instead.",
        (Failure::LinkNotOpened, French) => "Impossible d’ouvrir le navigateur. Enregistrez plutôt les dates dans un fichier d’agenda.",
        (Failure::KeyboardSettings, English) => "Can’t open keyboard settings from here. Open them from your system settings.",
        (Failure::KeyboardSettings, French) => {
            "Impossible d’ouvrir les réglages du clavier d’ici. Ouvrez-les depuis les paramètres du système."
        }
        (Failure::Keyring, English) => "your system keychain isn’t responding. Unlock it, then try again.",
        (Failure::Keyring, French) => "le trousseau du système ne répond pas. Déverrouillez-le, puis réessayez.",
        (Failure::ProgramNotStarted, English) => "part of Kitokey couldn’t start. Reinstall Kitokey.",
        (Failure::ProgramNotStarted, French) => "un composant de Kitokey n’a pas pu démarrer. Réinstallez Kitokey.",
        (Failure::TestRefused, English) => "check the address, model name and key, then try again.",
        (Failure::TestRefused, French) => "vérifiez l’adresse, le nom du modèle et la clé, puis réessayez.",
        (Failure::DestinationUnnamed, English) => "give the destination a name of one line, at most 40 characters.",
        (Failure::DestinationUnnamed, French) => "donnez à la destination un nom d’une ligne, de 40 caractères au plus.",
        (Failure::DestinationNamed, English) => "a destination already has this name. Choose another.",
        (Failure::DestinationNamed, French) => "une destination porte déjà ce nom. Choisissez-en un autre.",
        (Failure::DestinationRefused, English) => {
            "this address is refused: paste the channel’s webhook address, starting with https://, not one on this computer or its network."
        }
        (Failure::DestinationRefused, French) => {
            "cette adresse est refusée\u{a0}: collez l’adresse du webhook du canal, qui commence par https://, et non une adresse de cet ordinateur ou de son réseau."
        }
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
    // Shows: the same, when the meeting's own model failed. When: its cause calls for another
    // model. At most: no limit. Placeholders: {} the model's name, as the page says it.
    use Language::{English, French};
    Some(match (cause, language) {
        (Cause::TooSlow, English) => {
            "{}, chosen for this meeting, took too long to respond. Try again later or choose a faster model in Redo…\u{a0}› Advanced."
        }
        (Cause::TooSlow, French) => {
            "{}, choisi pour cette réunion, n’a pas répondu à temps. Réessayez plus tard ou choisissez un modèle plus rapide dans Refaire…\u{a0}› Avancé."
        }
        (Cause::UnknownModel, English) => {
            "The service doesn’t recognize {}, chosen for this meeting. Choose another in Redo…\u{a0}› Advanced."
        }
        (Cause::UnknownModel, French) => {
            "Le service ne reconnaît pas {}, choisi pour cette réunion. Choisissez-en un autre dans Refaire…\u{a0}› Avancé."
        }
        (Cause::Down, English) => {
            "The service is having problems. Try again later. If {} runs on this computer, it may be too large for its memory — choose a smaller one in Redo…\u{a0}› Advanced."
        }
        (Cause::Down, French) => {
            "Le service rencontre un problème. Réessayez plus tard. Si {} tourne sur cet ordinateur, il est peut-être trop gros pour sa mémoire\u{a0}: choisissez-en un plus petit dans Refaire…\u{a0}› Avancé."
        }
        (Cause::CutShort, English) => {
            "{}, chosen for this meeting, stopped before finishing. Choose a model that handles longer answers in Redo…\u{a0}› Advanced."
        }
        (Cause::CutShort, French) => {
            "{}, choisi pour cette réunion, s’est arrêté avant la fin. Choisissez un modèle capable de réponses plus longues dans Refaire…\u{a0}› Avancé."
        }
        (Cause::NotMinutes, English) => {
            "{}, chosen for this meeting, didn’t produce a summary. Try again or choose another model in Redo…\u{a0}› Advanced."
        }
        (Cause::NotMinutes, French) => {
            "{}, choisi pour cette réunion, n’a pas produit de compte rendu. Réessayez ou choisissez un autre modèle dans Refaire…\u{a0}› Avancé."
        }
        _ => return None,
    })
}

/// Why minutes were not made, and what to do, as kitokey-summary coded it (summarising/v1).
pub fn cause(cause: Cause, language: Language) -> &'static str {
    // Shows: why minutes were not made, and the gesture, on a meeting's page and in a notification.
    // When: kitokey-summary names its cause. At most: no limit. Placeholders: none.
    use Language::{English, French};
    match (cause, language) {
        (Cause::Unreachable, English) => {
            "Can’t reach the service. Check your connection — or that it’s running, if it’s on this computer — then try again."
        }
        (Cause::Unreachable, French) => {
            "Impossible de joindre le service. Vérifiez votre connexion, ou qu’il est bien lancé s’il tourne sur cet ordinateur, puis réessayez."
        }
        (Cause::TooSlow, English) => "The service took too long to respond. Try again later or choose a faster model in Settings.",
        (Cause::TooSlow, French) => {
            "Le service n’a pas répondu à temps. Réessayez plus tard ou choisissez un modèle plus rapide dans les Réglages."
        }
        (Cause::RefusedKey, English) => "The service rejected the API key. Enter it again in Settings.",
        (Cause::RefusedKey, French) => "Le service a refusé la clé d’API. Saisissez-la à nouveau dans les Réglages.",
        (Cause::NoCredit, English) => "Not enough credit with the service. Top up your account, then try again.",
        (Cause::NoCredit, French) => "Crédit insuffisant chez le service. Rechargez votre compte, puis réessayez.",
        (Cause::UnknownModel, English) => "The service doesn’t recognize this model or address. Check them in Settings.",
        (Cause::UnknownModel, French) => "Le service ne reconnaît pas ce modèle ou cette adresse. Vérifiez-les dans les Réglages.",
        (Cause::TooMany, English) => "The service is limiting requests, or your quota is used up. Wait, or check your plan.",
        (Cause::TooMany, French) => {
            "Le service limite le nombre de demandes, ou votre quota est atteint. Patientez, ou vérifiez votre abonnement."
        }
        (Cause::Down, English) => {
            "The service is having problems. Try again later. If the model runs on this computer, it may be too large for its memory — choose a smaller one."
        }
        (Cause::Down, French) => {
            "Le service rencontre un problème. Réessayez plus tard. Si le modèle tourne sur cet ordinateur, il est peut-être trop gros pour sa mémoire\u{a0}: choisissez-en un plus petit."
        }
        (Cause::NoTimes, English) => "This model doesn’t provide timestamps. Choose one that does in Settings.",
        (Cause::NoTimes, French) => "Ce modèle ne fournit pas d’horodatage. Choisissez-en un qui en fournit dans les Réglages.",
        (Cause::Redirected, English) => "The service redirects to another address. Check the address in Settings.",
        (Cause::Redirected, French) => "Le service redirige vers une autre adresse. Vérifiez l’adresse dans les Réglages.",
        (Cause::ProviderFailed, English) => "The service couldn’t respond. Try again later.",
        (Cause::ProviderFailed, French) => "Le service n’a pas pu répondre. Réessayez plus tard.",
        (Cause::CutShort, English) => "The model stopped before finishing. Choose a model that handles longer answers in Settings.",
        (Cause::CutShort, French) => {
            "Le modèle s’est arrêté avant la fin. Choisissez un modèle capable de réponses plus longues dans les Réglages."
        }
        (Cause::NotMinutes, English) => "The model didn’t produce a summary. Try again or choose another model in Settings.",
        (Cause::NotMinutes, French) => {
            "Le modèle n’a pas produit de compte rendu. Réessayez ou choisissez un autre modèle dans les Réglages."
        }
        (Cause::TranscriptUnusable, English) => "The transcript can’t be read. Redo it first.",
        (Cause::TranscriptUnusable, French) => "La transcription est illisible. Refaites-la d’abord.",
        (Cause::NoSpeech, English) => "No speech in the transcript, so nothing to summarize. Check the recording.",
        (Cause::NoSpeech, French) => "Aucune parole dans la transcription\u{a0}: rien à résumer. Vérifiez l’enregistrement.",
        (Cause::Settings, English) => "The service’s address, model or key isn’t valid. Fix them in Settings.",
        (Cause::Settings, French) => "L’adresse, le modèle ou la clé du service ne sont pas valides. Corrigez-les dans les Réglages.",
        (Cause::Unwritable, English) => "Couldn’t save the summary in the meeting’s folder. Free up space or check the folder.",
        (Cause::Unwritable, French) => {
            "Impossible d’enregistrer le compte rendu dans le dossier de la réunion. Libérez de l’espace ou vérifiez le dossier."
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
        // Shows: a notification, its title then its body. When: the program that records cannot be
        // started. At most: the title, 50 characters; the body, no limit. Placeholders: none.
        (Trouble::NoRecorder(_), English) => (
            "Kitokey can’t record",
            "A recording component is missing or damaged. Reinstall Kitokey to fix it.",
        ),
        (Trouble::NoRecorder(_), French) => (
            "Kitokey ne peut pas enregistrer",
            "Un composant d’enregistrement est manquant ou endommagé. Réinstallez Kitokey pour le réparer.",
        ),
        // Shows: a notification at a start, its body then the refusal. When: the settings file cannot
        // be read. At most: no limit. Placeholders: {} the refusal.
        (Trouble::Settings(why), English) => {
            return said(
                "Couldn’t read your settings",
                format!(
                    "Kitokey is using default settings until the file is fixed. {}",
                    refused(why, language)
                ),
            );
        }
        (Trouble::Settings(why), French) => {
            return said(
                "Vos réglages n’ont pas pu être lus",
                format!(
                    "Kitokey utilise les réglages par défaut jusqu’à ce que le fichier soit corrigé. {}",
                    refused(why, language)
                ),
            );
        }
        // Shows: a notification, its title then its body. When: the shortcut is not bound, the model or
        // the program that hears is missing, the meetings' folder cannot be written, or no recording is
        // there to transcribe. At most: the title, 50 characters; the body, no limit. Placeholders:
        // {at} where the model is looked for, or the folder.
        (Trouble::Shortcut(_), English) => (
            "Keyboard shortcut unavailable",
            "You can still start and stop recordings from the Kitokey icon.",
        ),
        (Trouble::Shortcut(_), French) => (
            "Raccourci clavier indisponible",
            "Vous pouvez toujours démarrer et arrêter un enregistrement depuis l’icône Kitokey.",
        ),
        (Trouble::NoModel(at), English) => {
            return said(
                "Can’t transcribe: model missing",
                format!("The recording is saved. Put the Whisper model at {at}, then choose “Transcribe last recording”."),
            );
        }
        (Trouble::NoModel(at), French) => {
            return said(
                "Transcription impossible\u{a0}: modèle manquant",
                format!(
                    "L’enregistrement est conservé. Placez le modèle Whisper ici\u{a0}: {at}, puis choisissez «\u{a0}Transcrire le dernier enregistrement\u{a0}»."
                ),
            );
        }
        (Trouble::NoTranscriber(_), English) => (
            "Kitokey can’t transcribe",
            "A transcription component is missing or damaged. The recording is saved; reinstall Kitokey to fix it.",
        ),
        (Trouble::NoFolder(at), English) => {
            return said(
                "Kitokey can’t record",
                format!("Can’t write to the meetings folder: {at}. Choose another one in Settings."),
            );
        }
        (Trouble::NoFolder(at), French) => {
            return said(
                "Kitokey ne peut pas enregistrer",
                format!("Impossible d’écrire dans le dossier des réunions\u{a0}: {at}. Choisissez-en un autre dans les Réglages."),
            );
        }
        (Trouble::NoSession, English) => (
            "No recording to transcribe",
            "Record a meeting first — it’s transcribed when you stop.",
        ),
        (Trouble::NoSession, French) => (
            "Aucun enregistrement à transcrire",
            "Enregistrez d’abord une réunion\u{a0}: elle sera transcrite à l’arrêt.",
        ),
        (Trouble::NoTranscriber(_), French) => (
            "Kitokey ne peut pas transcrire",
            "Un composant de transcription est manquant ou endommagé. L’enregistrement est conservé\u{202f}; réinstallez Kitokey pour le réparer.",
        ),
    };
    said(title, body.to_string())
}

/// What the user is told of a transcript's end: there, with where it is — or not, with why,
/// and how to have it made again.
pub fn made(made: &Made, language: Language) -> Notice {
    // Shows: a notification, its title then its body. When: a transcript is made, or was not.
    // At most: the title, 50 characters; the body, no limit. Placeholders: none.
    use Language::{English, French};
    let (title, body) = match (made, language) {
        // A click opens the meeting in Kitokey where the desktop says it was clicked (D44).
        (Made::Written(_), English) => (
            "Transcript ready",
            if CLICKED {
                "Click to open it."
            } else {
                "Find it in the Kitokey window."
            },
        ),
        (Made::Written(_), French) => (
            "Transcription prête",
            if CLICKED {
                "Cliquez pour l’ouvrir."
            } else {
                "Retrouvez-la dans la fenêtre de Kitokey."
            },
        ),
        (Made::Failed(_), English) => (
            "Transcription failed",
            "The recording is safe. Choose “Transcribe last recording” in the Kitokey menu to resume where it stopped.",
        ),
        (Made::Failed(_), French) => (
            "La transcription a échoué",
            "L’enregistrement est intact. Choisissez «\u{a0}Transcrire le dernier enregistrement\u{a0}» dans le menu de Kitokey pour reprendre là où elle s’est arrêtée.",
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
        // Shows: a notification, its title then its body. When: the minutes are written. At most:
        // the title, 50 characters; the body, no limit. Placeholders: {name} the name the ones
        // replaced are kept under.
        (Minutes::Ready, English) => (
            "Summary ready",
            if CLICKED {
                "Click to open it."
            } else {
                "Find it in the Kitokey window, next to the transcript."
            }
            .to_string(),
        ),
        (Minutes::ReadyKeptBeside(name), English) => (
            "Summary ready",
            format!("The previous summary couldn’t be moved to the trash. It’s kept alongside as {name}."),
        ),
        (Minutes::ReadyKeptBeside(name), French) => (
            "Compte rendu prêt",
            format!("L’ancien compte rendu n’a pas pu aller à la corbeille\u{a0}: il est conservé à côté, sous le nom {name}."),
        ),
        (Minutes::Ready, French) => (
            "Compte rendu prêt",
            if CLICKED {
                "Cliquez pour l’ouvrir."
            } else {
                "Retrouvez-le dans la fenêtre de Kitokey, à côté de la transcription."
            }
            .to_string(),
        ),
        // Its cause named, the cause's gesture; else the menu's entry, which tries again.
        // Shows: a notification, its title then its body. When: the minutes, or their redo, were
        // not written. At most: the title, 50 characters; the body, no limit. Placeholders: {} the
        // cause's sentence, {entry} the menu's entry that makes them again.
        (Minutes::Failed(Some(cause)), English) => (
            "Couldn’t write the summary",
            format!("The transcript is unchanged. {}", self::cause(*cause, language)),
        ),
        (Minutes::Failed(Some(cause)), French) => (
            "Le compte rendu n’a pas pu être rédigé",
            format!("La transcription n’a pas changé. {}", self::cause(*cause, language)),
        ),
        (Minutes::Failed(None), English) => (
            "Couldn’t write the summary",
            format!("The transcript is unchanged. Choose “{entry}” in the Kitokey menu to try again."),
        ),
        (Minutes::Failed(None), French) => (
            "Le compte rendu n’a pas pu être rédigé",
            format!("La transcription n’a pas changé. Choisissez «\u{a0}{entry}\u{a0}» dans le menu de Kitokey pour réessayer."),
        ),
        (Minutes::RedoFailed(cause), English) => (
            "Couldn’t redo the summary",
            match cause {
                Some(cause) => format!("The previous one is kept. {}", self::cause(*cause, language)),
                None => "The previous ones stayed in place.".to_string(),
            },
        ),
        (Minutes::RedoFailed(cause), French) => (
            "Le compte rendu n’a pas pu être refait",
            match cause {
                Some(cause) => format!("L’ancien est conservé. {}", self::cause(*cause, language)),
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
            "Couldn’t write the summary",
            format!("The transcript is unchanged. {}", cause_said(*cause, Some(model), language)),
        ),
        (
            Minutes::FailedOwn {
                cause,
                model,
                again: false,
            },
            French,
        ) => (
            "Le compte rendu n’a pas pu être rédigé",
            format!("La transcription n’a pas changé. {}", cause_said(*cause, Some(model), language)),
        ),
        (Minutes::FailedOwn { cause, model, again: true }, English) => (
            "Couldn’t redo the summary",
            format!("The previous one is kept. {}", cause_said(*cause, Some(model), language)),
        ),
        (Minutes::FailedOwn { cause, model, again: true }, French) => (
            "Le compte rendu n’a pas pu être refait",
            format!("L’ancien est conservé. {}", cause_said(*cause, Some(model), language)),
        ),
        // Shows: a notification, its title then its body. When: the minutes wait for the user's
        // agreement to send. At most: the title, 50 characters; the body, no limit. Placeholders:
        // {host} where they would go.
        (Minutes::Waiting(host), English) => (
            "The summary needs your OK",
            format!("Nothing is sent to {host} without your OK. Give it in the Kitokey window."),
        ),
        (Minutes::Waiting(host), French) => (
            "Le compte rendu attend votre accord",
            format!("Rien n’est envoyé à {host} sans votre accord. Donnez-le dans la fenêtre de Kitokey."),
        ),
        // Shows: a notification, its title then its body. When: no one is chosen to write the
        // minutes, or the address chosen is none. At most: the title, 50 characters; the body, no
        // limit. Placeholders: {address} what the settings name.
        (Minutes::Unchosen, English) => ("No model chosen for summaries", "Choose one in Kitokey Settings.".to_string()),
        (Minutes::Unchosen, French) => (
            "Aucun modèle choisi pour les comptes rendus",
            "Choisissez-en un dans les Réglages de Kitokey.".to_string(),
        ),
        (Minutes::NoProvider(address), English) => (
            "Couldn’t write the summary",
            format!("“{address}” isn’t a valid service address. Choose the summary service in Settings."),
        ),
        (Minutes::NoProvider(address), French) => (
            "Le compte rendu n’a pas pu être rédigé",
            format!(
                "«\u{a0}{address}\u{a0}» n’est pas l’adresse d’un service valide. Choisissez le service de compte rendu dans les Réglages."
            ),
        ),
        // Shows: a notification, its title then its body. When: the minutes are asked for with no
        // whole transcript. At most: the title, 50 characters; the body, no limit. Placeholders: {}
        // the menu's entry that transcribes.
        (Minutes::NoTranscript, English) => (
            "No summary without a transcript",
            format!("The last recording has no complete transcript. Choose “{}” first.", menu(language).transcribe),
        ),
        (Minutes::NoTranscript, French) => (
            "Pas de compte rendu sans transcription",
            format!(
                "Le dernier enregistrement n’a pas de transcription complète. Choisissez d’abord «\u{a0}{}\u{a0}».",
                menu(language).transcribe
            ),
        ),
        // Shows: a notification, its title then its body. When: a confidential meeting's minutes
        // wait for Ollama. At most: the title, 50 characters; the body, no limit. Placeholders: {}
        // the model Kitokey's window fetches.
        (Minutes::OllamaWaits { silent: true }, English) => (
            "The summary is waiting for Ollama",
            "Confidential meeting: the summary must be written on this computer, but Ollama isn’t responding. Ollama is free at ollama.com.".to_string(),
        ),
        (Minutes::OllamaWaits { silent: true }, French) => (
            "Le compte rendu attend Ollama",
            "Réunion confidentielle\u{a0}: le compte rendu doit être rédigé sur cet ordinateur, mais Ollama ne répond pas. Ollama est gratuit, sur ollama.com.".to_string(),
        ),
        (Minutes::OllamaWaits { silent: false }, English) => (
            "The summary is waiting for Ollama",
            format!(
                "Confidential meeting: the summary must be written on this computer, but Ollama has no model. Download {} in one click in the Kitokey window.",
                crate::providers::RECOMMENDED
            ),
        ),
        (Minutes::OllamaWaits { silent: false }, French) => (
            "Le compte rendu attend Ollama",
            format!(
                "Réunion confidentielle\u{a0}: le compte rendu doit être rédigé sur cet ordinateur, mais Ollama n’a aucun modèle. Téléchargez {} en un clic dans la fenêtre de Kitokey.",
                crate::providers::RECOMMENDED
            ),
        ),
    };
    notice(title, body).expect("the minutes' end is always said")
}

/// Said when a transcript made again replaced one the trash refused: kept beside it; or, the
/// new one failed (`not_put_back`), when the one there could not be put back: where it is.
pub fn transcript_kept_beside(language: Language, name: &str, not_put_back: bool) -> Notice {
    // Shows: a notification, its title then its body. When: a transcript made again could not send
    // the old one to the trash, or put it back. At most: the title, 50 characters; the body, no
    // limit. Placeholders: {name} the name it is kept under.
    let (title, body) = match (language, not_put_back) {
        (Language::English, false) => (
            "Previous transcript kept",
            format!("It couldn’t be moved to the trash. It’s next to the new one, as {name}."),
        ),
        (Language::French, false) => (
            "Ancienne transcription conservée",
            format!("Elle n’a pas pu aller à la corbeille\u{a0}: elle est à côté de la nouvelle, sous le nom {name}."),
        ),
        (Language::English, true) => (
            "Couldn’t restore the transcript",
            format!("It’s in the meeting’s folder, as {name}."),
        ),
        (Language::French, true) => (
            "La transcription n’a pas pu être restaurée",
            format!("Elle se trouve dans le dossier de la réunion, sous le nom {name}."),
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
    // Shows: a notification, its title then its body. When: the microphone sends nothing at all.
    // At most: the title, 50 characters; the body, no limit. Placeholders: {name} the microphone.
    let body = match (name, language) {
        (Some(name), Language::English) => format!(
            "No signal from {name} for 10 seconds, not even background noise. Check it isn’t muted on the device and that it’s switched on. Recording continues."
        ),
        (None, Language::English) => "No signal from your microphone for 10 seconds, not even background noise. Check it isn’t muted on the device and that it’s switched on. Recording continues.".to_string(),
        (Some(name), Language::French) => format!(
            "Aucun signal du micro {name} depuis 10 secondes, pas même un bruit de fond. Vérifiez qu’il n’est pas coupé sur l’appareil et qu’il est allumé. L’enregistrement continue."
        ),
        (None, Language::French) => "Aucun signal de votre micro depuis 10 secondes, pas même un bruit de fond. Vérifiez qu’il n’est pas coupé sur l’appareil et qu’il est allumé. L’enregistrement continue.".to_string(),
    };
    let title = match language {
        Language::English => "Your microphone isn’t sending any signal",
        Language::French => "Votre micro ne transmet aucun signal",
    };
    notice(title, body)
}

/// A side with no device left while the other is still recorded.
fn lost_beside(source: Source, language: Language) -> Notice {
    // Shows: a notification, its title then its body. When: a side has no device left while the
    // other is still recorded. At most: the title, 50 characters; the body, no limit. Placeholders:
    // none.
    let (title, body) = match (source, language) {
        (Source::Me, Language::English) => ("Your microphone isn’t being recorded", "The meeting audio is still being recorded."),
        (Source::Me, Language::French) => (
            "Votre micro n’est pas enregistré",
            "Le son de la réunion, lui, est bien enregistré.",
        ),
        (Source::Others, Language::English) => (
            "Meeting audio is no longer being recorded",
            "Your computer has no audio output. Your microphone is still recorded; plug in an output to resume.",
        ),
        (Source::Others, Language::French) => (
            "Le son de la réunion n’est plus enregistré",
            "Votre ordinateur n’a plus de sortie audio. Votre micro reste enregistré\u{202f}; branchez une sortie pour reprendre.",
        ),
    };
    Notice {
        title: title.into(),
        body: body.into(),
    }
}

/// Said when the meeting's sound, with no device left, is recorded again, from this one (D31).
pub fn back(name: Option<&str>, language: Language) -> Notice {
    // Shows: a notification, its title then its body. When: the meeting's sound is recorded again.
    // At most: the title, 50 characters; the body, no limit. Placeholders: {name} the output now
    // recorded.
    let (title, body) = match language {
        Language::English => (
            "Meeting audio is being recorded again",
            name.map_or("From the current audio output.".into(), |name| format!("From {name}.")),
        ),
        Language::French => (
            "Le son de la réunion est de nouveau enregistré",
            name.map_or("Depuis la sortie audio actuelle.".into(), |name| format!("Depuis {name}.")),
        ),
    };
    Notice { title: title.into(), body }
}

/// The offer to record a call another application started (D30): the notice, naming the
/// application, and its three gestures — record, not this time, never offer.
pub fn offer(application: &str, language: Language) -> (Notice, [&'static str; 3]) {
    // Shows: a notification, its title then its body, then its three buttons. When: another
    // application takes a microphone. At most: the title, 50 characters; the body, no limit.
    // Placeholders: {application} that application.
    match language {
        Language::English => (
            Notice {
                title: format!("{application} is using your microphone"),
                body: "Record this call with Kitokey?".into(),
            },
            ["Record", "Not this time", "Don’t ask again"],
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
    // Shows: a notification's body, after a stop. When: the transcript follows without another
    // gesture. At most: no limit. Placeholders: none.
    match language {
        Language::English => "Transcribing now.",
        Language::French => "Transcription en cours.",
    }
}

/// The menu's entry that fetches the default model: what it is, how big, and from where — the
/// user's click on it is their agreement.
pub fn download(language: Language, size: &str, host: &str) -> String {
    // Shows: the Kitokey icon's menu, as a menu entry. When: the default model is not here.
    // At most: about 60 characters. Placeholders: {size} the model's size, {host} where it comes
    // from.
    match language {
        Language::English => format!("Download the model ({size}, {host})"),
        Language::French => format!("Télécharger le modèle ({size}, {host})"),
    }
}

/// The same entry while the model comes.
pub fn downloading(language: Language, percent: u64) -> String {
    // Shows: the same entry. When: the model comes. At most: about 40 characters. Placeholders:
    // {percent} the share come.
    match language {
        Language::English => format!("Downloading the model… {percent} %"),
        Language::French => format!("Téléchargement du modèle… {percent}\u{a0}%"),
    }
}

/// The entry while the model comes, from the bytes held: the share come — then, once every
/// byte is there, that its fingerprint is being checked.
pub fn fetching(language: Language, held: u64, bytes: u64) -> String {
    // Shows: the same entry. When: every byte is there and the fingerprint is checked. At most:
    // about 40 characters. Placeholders: none.
    if held >= bytes {
        return match language {
            Language::English => "Verifying the model…",
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
    // Shows: a notification, its title then its body. When: audio kept longer than the settings say
    // went to the trash, or could not. At most: the title, 50 characters; the body, no limit.
    // Placeholders: {meetings} how many meetings, {} how long the audio is kept.
    let (title, body) = match (language, outcome) {
        (Language::English, Ok((1, days))) => (
            "Audio moved to trash".to_string(),
            format!(
                "The audio of 1 meeting older than {} is in the trash; its transcript and summary remain.",
                english_days(*days)
            ),
        ),
        (Language::English, Ok((meetings, days))) => (
            "Audio moved to trash".to_string(),
            format!(
                "The audio of {meetings} meetings older than {} is in the trash; their transcripts and summaries remain.",
                english_days(*days)
            ),
        ),
        (Language::French, Ok((1, days))) => (
            "Audio déplacé dans la corbeille".to_string(),
            format!(
                "L’audio d’une réunion de plus {} est dans la corbeille\u{202f}; sa transcription et son compte rendu restent.",
                french_days(*days)
            ),
        ),
        (Language::French, Ok((meetings, days))) => (
            "Audio déplacé dans la corbeille".to_string(),
            format!(
                "L’audio de {meetings} réunions de plus {} est dans la corbeille\u{202f}; leurs transcriptions et comptes rendus restent.",
                french_days(*days)
            ),
        ),
        // The trash may have taken some before it refused: what is said holds either way.
        (Language::English, Err(refusal)) => (
            "Some audio couldn’t be moved to the trash".to_string(),
            format!(
                "{} The rest is untouched — nothing was deleted.",
                match refusal {
                    Refusal::NotAllowed => "The trash refused access.",
                    Refusal::NoSpace => "The trash is full.",
                    Refusal::Other => "The system trash refused these files.",
                }
            ),
        ),
        (Language::French, Err(refusal)) => (
            "Audio pas entièrement mis à la corbeille".to_string(),
            format!(
                "{} Le reste est intact\u{a0}: rien n’a été supprimé.",
                match refusal {
                    Refusal::NotAllowed => "La corbeille a refusé l’accès.",
                    Refusal::NoSpace => "La corbeille est pleine.",
                    Refusal::Other => "La corbeille du système a refusé ces fichiers.",
                }
            ),
        ),
    };
    Notice { title, body }
}

/// What is left of the trial, said in Settings (D83): its days and its meetings; the days alone once
/// the meetings are made, the meetings alone once the days have passed.
pub fn trial_left(left: Left, language: Language) -> String {
    // Shows: Settings, under the trial's name, as a status. When: a trial licence. At most: no
    // limit. Placeholders: {days} the days left, {meetings} the meetings left.
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
        (Language::English, ..) => format!("{days} and {meetings} left. The trial ends when both run out."),
        (Language::French, 0, _) => format!("Il reste {meetings}."),
        (Language::French, _, 0) => format!("Il reste {days}."),
        (Language::French, ..) => format!("Il reste {days} et {meetings}\u{202f}; l’essai se termine quand les deux sont épuisés."),
    }
}

fn english_days(days: u32) -> String {
    // Shows: inside audio_trashed's sentence. When: always. At most: about 10 characters.
    // Placeholders: {days} the days.
    if days == 1 { "1 day".to_string() } else { format!("{days} days") }
}

fn french_days(days: u32) -> String {
    // Shows: inside audio_trashed's sentence. When: always. At most: about 14 characters.
    // Placeholders: {days} the days.
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
    // Shows: a notification, its title then its body. When: the first start after Wholeheard became
    // Kitokey, something carried over. At most: the title, 50 characters; the body, no limit.
    // Placeholders: none.
    let (title, body) = match language {
        Language::English => (
            "Wholeheard is now Kitokey",
            "Your settings, meetings and transcription model have been carried over. You may be asked for the keyboard shortcut once more.",
        ),
        Language::French => (
            "Wholeheard s’appelle maintenant Kitokey",
            "Vos réglages, vos réunions et le modèle de transcription ont été repris. Le raccourci clavier pourra vous être redemandé une fois.",
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
    // Shows: a notification, its title then its body. When: what was held under the earlier name
    // was not all carried over. At most: the title, 50 characters; the body, no limit.
    // Placeholders: none.
    let (title, body) = match language {
        Language::English => (
            "Some Wholeheard settings weren’t carried over",
            "Nothing is lost — Kitokey will try again next time it starts.",
        ),
        Language::French => (
            "Réglages de Wholeheard en partie non repris",
            "Rien n’est perdu\u{a0}: Kitokey réessaiera au prochain démarrage.",
        ),
    };
    Notice {
        title: title.into(),
        body: body.into(),
    }
}

pub fn model_needed(language: Language, size: &str, host: &str) -> Notice {
    // Shows: a notification, its title then its body. When: a transcript needs the default model
    // and it is not here. At most: the title, 50 characters; the body, no limit. Placeholders:
    // {entry} the menu's entry that fetches it.
    let entry = download(language, size, host);
    let (title, body) = match language {
        Language::English => (
            "Transcription model needed",
            format!("It downloads once, with your OK: choose “{entry}” in the Kitokey menu. The recording is saved."),
        ),
        Language::French => (
            "Modèle de transcription requis",
            format!(
                "Il se télécharge une seule fois, avec votre accord\u{a0}: choisissez «\u{a0}{entry}\u{a0}» dans le menu de Kitokey. L’enregistrement est conservé."
            ),
        ),
    };
    notice(title, body).expect("always said")
}

/// Said when Ollama's pull of a model ended: there, offered first for the minutes — or not, and
/// offered again (D37).
pub fn pulled(outcome: &Result<(), String>, model: &str, language: Language) -> Notice {
    // Shows: a notification, its title then its body. When: Ollama's pull of a model ended.
    // At most: the title, 50 characters; the body, no limit. Placeholders: {model} the model.
    let (title, body) = match (outcome, language) {
        (Ok(()), Language::English) => (
            "Model ready",
            format!("{model} is installed in Ollama and suggested first for summaries."),
        ),
        (Ok(()), Language::French) => (
            "Modèle prêt",
            format!("{model} est installé dans Ollama et proposé en premier pour le compte rendu."),
        ),
        (Err(_), Language::English) => ("Model download failed", "Nothing was changed. You can try again.".into()),
        (Err(_), Language::French) => (
            "Échec du téléchargement du modèle",
            "Rien n’a été modifié. Vous pouvez réessayer.".into(),
        ),
    };
    notice(title, body).expect("always said")
}

/// What a model of Settings' list is good for, as the list says it (D37, D40).
pub fn offered_good(language: Language, good: Good) -> &'static str {
    // Shows: Settings' list of models to download, beside a model. When: always. At most: about 30
    // characters. Placeholders: none.
    match (language, good) {
        (Language::English, Good::Recommended) => "Recommended: most accurate, fast on a recent computer",
        (Language::French, Good::Recommended) => "Recommandé\u{a0}: le plus précis, rapide sur un ordinateur récent",
        (Language::English, Good::Faster) => "Faster, less accurate: for older computers",
        (Language::French, Good::Faster) => "Plus rapide, moins précis\u{a0}: pour un ordinateur plus ancien",
    }
}

/// Said when the model's fetch ended: ready, and the transcript begun — or not, with why, and
/// that nothing of it was used.
pub fn fetched(outcome: &Result<(), String>, chosen: bool, language: Language) -> Notice {
    // Shows: a notification, its title then its body. When: the model's fetch ended. At most: the
    // title, 50 characters; the body, no limit. Placeholders: none.
    use Language::{English, French};
    let (title, body) = match (outcome, chosen, language) {
        (Ok(()), true, English) => ("Model ready", "Model verified. Transcription is starting."),
        (Ok(()), true, French) => ("Modèle prêt", "Modèle vérifié. La transcription démarre."),
        // Another than the one chosen: the choice stays the user's (D37).
        (Ok(()), false, English) => ("Model ready", "Model verified. Choose it in Settings for your next transcripts."),
        (Ok(()), false, French) => (
            "Modèle prêt",
            "Modèle vérifié. Choisissez-le dans les Réglages pour vos prochaines transcriptions.",
        ),
        (Err(_), _, English) => ("Model download failed", "Nothing was changed. You can try again."),
        (Err(_), _, French) => ("Échec du téléchargement du modèle", "Rien n’a été modifié. Vous pouvez réessayer."),
    };
    notice(title, body.to_string()).expect("always said")
}

/// The sentence « Copier l'annonce » puts on the clipboard, for the call's chat (D50): in the
/// meeting's language when it says French or English, else in the window's.
pub fn announcement(meeting: Option<&str>, window: Language) -> &'static str {
    // Shows: the clipboard, for the call's chat. When: « Copier l’annonce » is pressed. At most: no
    // limit. Placeholders: none.
    let language = match meeting {
        Some("fr") => Language::French,
        Some("en") => Language::English,
        _ => window,
    };
    match language {
        Language::French => {
            "J’enregistre cette réunion pour en faire le compte rendu. Dites-le-moi si vous préférez que je ne le fasse pas."
        }
        Language::English => "I’m recording this meeting to write a summary. Let me know if you’d rather I didn’t.",
    }
}
