//! The dates a summary's actions mention (D115): an action's *by when* read against the meeting's day.
//!
//! An absolute date only for what names one without a guess — an explicit date, a weekday, « demain »,
//! « dans N jours|semaines » and their English forms —; anything else is « not said » (`None`): a month's
//! end, a season, « bientôt », « la semaine prochaine », a weekday said on that very weekday, a numeric date
//! that reads two ways. No procedural rule moves a date (no weekend, no holiday): the user ticks what they
//! keep. Days are calendar days, so a change of daylight saving time moves nothing.

use chrono::{Datelike, Days, NaiveDate};

const MONTHS: [(&str, u32); 24] = [
    ("janvier", 1),
    ("fevrier", 2),
    ("mars", 3),
    ("avril", 4),
    ("mai", 5),
    ("juin", 6),
    ("juillet", 7),
    ("aout", 8),
    ("septembre", 9),
    ("octobre", 10),
    ("novembre", 11),
    ("decembre", 12),
    ("january", 1),
    ("february", 2),
    ("march", 3),
    ("april", 4),
    ("may", 5),
    ("june", 6),
    ("july", 7),
    ("august", 8),
    ("september", 9),
    ("october", 10),
    ("november", 11),
    ("december", 12),
];

const DAYS: [(&str, u32); 14] = [
    ("lundi", 0),
    ("mardi", 1),
    ("mercredi", 2),
    ("jeudi", 3),
    ("vendredi", 4),
    ("samedi", 5),
    ("dimanche", 6),
    ("monday", 0),
    ("tuesday", 1),
    ("wednesday", 2),
    ("thursday", 3),
    ("friday", 4),
    ("saturday", 5),
    ("sunday", 6),
];

const NUMBERS: [(&str, i64); 34] = [
    ("un", 1),
    ("une", 1),
    ("deux", 2),
    ("trois", 3),
    ("quatre", 4),
    ("cinq", 5),
    ("six", 6),
    ("sept", 7),
    ("huit", 8),
    ("neuf", 9),
    ("dix", 10),
    ("onze", 11),
    ("douze", 12),
    ("quinze", 15),
    ("vingt", 20),
    ("trente", 30),
    ("a", 1),
    ("an", 1),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("fourteen", 14),
    ("fifteen", 15),
    ("twenty", 20),
    ("thirty", 30),
];

/// Words that make a date a guess, a past, a range or a rule of its own — « pas demain », « jeudi dernier »,
/// « vendredi prochain » (this week's or the next?), « lundi ou mardi », « après demain », « jours ouvrés »
/// (a working-day rule): with one of them, nothing is said.
const GUESSES: [&str; 26] = [
    "pas",
    "not",
    "dernier",
    "derniere",
    "last",
    "passe",
    "passee",
    "past",
    "ago",
    "prochain",
    "prochaine",
    "next",
    "ou",
    "or",
    "apres",
    "after",
    "ouvre",
    "ouvres",
    "ouvrable",
    "ouvrables",
    "business",
    "working",
    "environ",
    "around",
    "about",
    "vers",
];

/// The meeting's day, as its folder names it (session-audio/v3, *The folder*: `2026-10-14_14-05`, then
/// `-2` for a second session in the same minute): the local day the recording started.
pub fn reference_day(folder: &str) -> Option<NaiveDate> {
    let (date, time) = folder.split_once('_')?;
    let mut parts = time.split('-');
    let hour = parts.next().filter(|h| h.len() == 2)?.parse::<u32>().ok().filter(|h| *h < 24);
    let minute = parts.next().filter(|m| m.len() == 2)?.parse::<u32>().ok().filter(|m| *m < 60);
    let rest = parts.next().is_none_or(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
    if hour.is_none() || minute.is_none() || !rest || parts.next().is_some() {
        return None;
    }
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// The date an action's *by when* names against `day`, or `None` when it names none without a guess: exactly
/// one date said — an explicit one, a weekday, « demain », a count of days or weeks —, or an explicit date and
/// the weekday it falls on; never a word of `GUESSES`.
pub fn resolve(when: &str, day: NaiveDate) -> Option<NaiveDate> {
    let text = plain(when);
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric() && c != '/' && c != '-')
        .filter(|w| !w.is_empty())
        .collect();
    let guessed = |w: &&str| w.split('-').any(|part| GUESSES.contains(&part));
    if words.iter().any(guessed) {
        return None;
    }
    // Each date said, `None` for one said that does not exist or cannot be read: then nothing is.
    let mut explicit = iso(&words);
    explicit.extend(named_month(&words, day));
    explicit.extend(numeric(&words, day));
    let mut relative: Vec<Option<NaiveDate>> = words
        .iter()
        .filter(|w| matches!(**w, "demain" | "tomorrow"))
        .map(|_| day.succ_opt())
        .collect();
    relative.extend(within(&words, day));
    let mut weekdays: Vec<u32> = words
        .iter()
        .filter_map(|w| DAYS.iter().find(|(name, _)| name == w).map(|(_, d)| *d))
        .collect();
    weekdays.dedup();
    match (explicit.as_slice(), relative.as_slice(), weekdays.as_slice()) {
        ([date], [], []) | ([], [date], []) => *date,
        // A date and its weekday: said twice, the same day; two days that differ, which one is not said.
        ([Some(date)], [], [weekday]) => (date.weekday().num_days_from_monday() == *weekday).then_some(*date),
        ([], [], [weekday]) => next_weekday(*weekday, day),
        _ => None,
    }
}

/// Lower case, no accent, French elisions split off (« d'ici » is « d ici »).
fn plain(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            '\'' | '’' => ' ',
            other => other,
        })
        .collect()
}

fn ordinal(word: &str) -> Option<u32> {
    let digits = word.trim_end_matches(|c: char| c.is_alphabetic());
    let suffix = &word[digits.len()..];
    if digits.is_empty() || digits.len() > 2 || !["", "er", "st", "nd", "rd", "th"].contains(&suffix) {
        return None;
    }
    digits.parse().ok()
}

fn month(word: &str) -> Option<u32> {
    MONTHS.iter().find(|(name, _)| *name == word).map(|(_, m)| *m)
}

fn year(word: Option<&&str>) -> Option<i32> {
    word.filter(|w| w.len() == 4).and_then(|w| w.parse().ok())
}

/// A day and month with a year is that day; without, the next one that exists on or after `day` (a 29
/// February within the next eight years). `None`: it names a date that does not exist.
fn dated(d: u32, m: u32, y: Option<i32>, day: NaiveDate) -> Option<NaiveDate> {
    if let Some(y) = y {
        return NaiveDate::from_ymd_opt(y, m, d);
    }
    (day.year()..=day.year() + 8).find_map(|y| NaiveDate::from_ymd_opt(y, m, d).filter(|found| *found >= day))
}

fn iso(words: &[&str]) -> Vec<Option<NaiveDate>> {
    words
        .iter()
        .filter(|w| w.len() == 10 && w.as_bytes()[4] == b'-' && w.as_bytes()[7] == b'-')
        .map(|w| NaiveDate::parse_from_str(w, "%Y-%m-%d").ok())
        .collect()
}

/// « 12 décembre [2026] », « le 1er décembre », « December 12[, 2026] », « 3rd of March »; a month named
/// with no day is no date.
fn named_month(words: &[&str], day: NaiveDate) -> Vec<Option<NaiveDate>> {
    let mut found = Vec::new();
    for (at, word) in words.iter().enumerate() {
        let Some(m) = month(word) else { continue };
        let before = at.checked_sub(1).and_then(|i| words.get(i)).copied();
        let before = if before == Some("of") {
            at.checked_sub(2).and_then(|i| words.get(i)).copied()
        } else {
            before
        };
        if let Some(d) = before.and_then(ordinal) {
            found.push(dated(d, m, year(words.get(at + 1)), day));
        } else if let Some(d) = words.get(at + 1).and_then(|w| ordinal(w)) {
            found.push(dated(d, m, year(words.get(at + 2)), day));
        }
    }
    found
}

/// « 12/12 », « 15/01[/2027] »: day then month, read only when it cannot be month then day.
fn numeric(words: &[&str], day: NaiveDate) -> Vec<Option<NaiveDate>> {
    let mut found = Vec::new();
    for word in words.iter().filter(|w| w.contains('/')) {
        let parts: Vec<&str> = word.split('/').collect();
        let digits = parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 4 && p.chars().all(|c| c.is_ascii_digit()));
        if !(2..=3).contains(&parts.len()) || !digits {
            found.push(None);
            continue;
        }
        let (a, b): (u32, u32) = (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0));
        found.push(if a != b && a <= 12 && b <= 12 {
            None
        } else {
            dated(a, b, year(parts.get(2)), day)
        });
    }
    found
}

/// « dans N jours|semaines », « in N days|weeks », N one or more, its date within the calendar's reach; French counts « huit jours »
/// a week and « quinze jours » two when said in words, as written when in digits. A count of days or weeks with
/// no « dans » or « in » before it (« et 2 jours », « dans les 2 jours ») is not read: `None`.
fn within(words: &[&str], day: NaiveDate) -> Vec<Option<NaiveDate>> {
    let mut found = Vec::new();
    for (at, unit) in words.iter().enumerate() {
        let week = match *unit {
            "jour" | "jours" | "day" | "days" => false,
            "semaine" | "semaines" | "week" | "weeks" => true,
            _ => continue,
        };
        let Some(count) = at.checked_sub(1).and_then(|i| words.get(i)) else {
            continue;
        };
        let worded = NUMBERS.iter().find(|(w, _)| w == count).map(|(_, n)| *n);
        let n = worded.or_else(|| {
            count
                .chars()
                .all(|c| c.is_ascii_digit())
                .then(|| count.parse::<i64>().ok())
                .flatten()
        });
        if n.is_none() && !count.chars().all(|c| c.is_ascii_digit()) {
            // « une semaine » with no count word before it is no count: « la semaine prochaine » is left to GUESSES.
            continue;
        }
        let said = at
            .checked_sub(2)
            .and_then(|i| words.get(i))
            .is_some_and(|w| matches!(*w, "dans" | "in"));
        let days = n.filter(|_| said).and_then(|n| match (week, worded) {
            (true, _) => n.checked_mul(7),
            (false, Some(8 | 15)) => Some(n - 1),
            (false, _) => Some(n),
        });
        found.push(days.filter(|d| *d >= 1).and_then(|d| day.checked_add_days(Days::new(d as u64))));
    }
    found
}

/// A weekday: the next one strictly after `day`; said on that very weekday, not said.
fn next_weekday(target: u32, day: NaiveDate) -> Option<NaiveDate> {
    let ahead = (7 + target - day.weekday().num_days_from_monday()) % 7;
    if ahead == 0 {
        return None;
    }
    day.checked_add_days(Days::new(u64::from(ahead)))
}

/// One ticked item, as a calendar shows it: its day, all day long; its title the action's what, then « — who »
/// when someone is named; its notes the passage, a blank line, then the meeting's name — as `exporting/v1`'s
/// `.ics` writes SUMMARY and DESCRIPTION, so a link and the file say the same.
pub struct Event<'a> {
    pub date: NaiveDate,
    pub what: &'a str,
    pub who: Option<&'a str>,
    pub passage: Option<&'a str>,
    pub meeting: &'a str,
}

impl Event<'_> {
    fn title(&self) -> String {
        match self.who.map(str::trim).filter(|who| !who.is_empty()) {
            Some(who) => format!("{} — {who}", self.what),
            None => self.what.to_string(),
        }
    }

    fn notes(&self) -> String {
        match self.passage.map(str::trim).filter(|passage| !passage.is_empty()) {
            Some(passage) => format!("{passage}\n\n{}", self.meeting),
            None => self.meeting.to_string(),
        }
    }

    /// The day after, an all-day event's end as Google's link and RFC 5545 count it.
    fn end(&self) -> NaiveDate {
        self.date.succ_opt().unwrap_or(self.date)
    }
}

/// Google Calendar's own screen, the event pre-filled and unsaved: `render?action=TEMPLATE`, `text`, `dates` as
/// `YYYYMMDD/YYYYMMDD` ending the next day for an all-day event, `details`. Google publishes no reference page
/// for this link; the form followed is the Interaction Design Foundation's add-event-to-calendar-docs
/// (`services/google.md`), the one calendar buttons use.
pub fn google(event: &Event) -> String {
    format!(
        "https://calendar.google.com/calendar/render?action=TEMPLATE&text={}&dates={}/{}&details={}",
        encoded(&event.title()),
        event.date.format("%Y%m%d"),
        event.end().format("%Y%m%d"),
        encoded(&event.notes()),
    )
}

/// Percent-encoded as RFC 3986 says: its unreserved characters kept, every other byte of its UTF-8 escaped.
fn encoded(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
