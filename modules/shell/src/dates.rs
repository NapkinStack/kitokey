//! The dates a summary's actions mention (D115): an action's *by when* read against the meeting's day.
//!
//! An absolute date only for what names one without a guess — an explicit date, a weekday, « demain »,
//! « dans N jours|semaines » and their English forms —; anything else is « not said » (`None`): a month's
//! end, a season, « bientôt », « la semaine prochaine », a weekday said on that very weekday, a numeric date
//! that reads two ways. No procedural rule moves a date (no weekend, no holiday): the user ticks what they
//! keep. Days are calendar days, so a change of daylight saving time moves nothing.

use chrono::{Datelike, Duration, NaiveDate};

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

/// The meeting's day, as its folder names it (session-audio/v3, *The folder*: `2026-10-14_14-05`, then
/// `-2` for a second session in the same minute): the local day the recording started.
pub fn reference_day(folder: &str) -> Option<NaiveDate> {
    let (date, time) = folder.split_once('_')?;
    let time_ok = time.len() >= 5 && time.as_bytes()[2] == b'-' && time[..2].parse::<u32>().is_ok_and(|h| h < 24);
    if !time_ok {
        return None;
    }
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// The date an action's *by when* names against `day`, or `None` when it names none without a guess.
pub fn resolve(when: &str, day: NaiveDate) -> Option<NaiveDate> {
    let text = plain(when);
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric() && c != '/' && c != '-')
        .filter(|w| !w.is_empty())
        .collect();
    if let Some(found) = iso(&words) {
        return found;
    }
    if let Some(found) = named_month(&words, day) {
        return found;
    }
    if let Some(found) = numeric(&words, day) {
        return found;
    }
    if words.iter().any(|w| *w == "demain" || *w == "tomorrow") {
        return day.checked_add_signed(Duration::days(1));
    }
    if let Some(found) = within(&words, day) {
        return found;
    }
    weekday(&words, day)
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

/// A day and month with a year is that day; without, the next one on or after `day`. `Some(None)`: it
/// names a date that does not exist.
fn dated(d: u32, m: u32, y: Option<i32>, day: NaiveDate) -> Option<NaiveDate> {
    if let Some(y) = y {
        return NaiveDate::from_ymd_opt(y, m, d);
    }
    match NaiveDate::from_ymd_opt(day.year(), m, d) {
        Some(found) if found >= day => Some(found),
        _ => NaiveDate::from_ymd_opt(day.year() + 1, m, d),
    }
}

fn iso(words: &[&str]) -> Option<Option<NaiveDate>> {
    let word = words
        .iter()
        .find(|w| w.len() == 10 && w.as_bytes()[4] == b'-' && w.as_bytes()[7] == b'-')?;
    Some(NaiveDate::parse_from_str(word, "%Y-%m-%d").ok())
}

/// « 12 décembre [2026] », « le 1er décembre », « December 12[, 2026] », « 3rd of March ».
fn named_month(words: &[&str], day: NaiveDate) -> Option<Option<NaiveDate>> {
    for (at, word) in words.iter().enumerate() {
        let Some(m) = month(word) else { continue };
        let before = at.checked_sub(1).and_then(|i| words.get(i)).copied();
        let before = if before == Some("of") {
            at.checked_sub(2).and_then(|i| words.get(i)).copied()
        } else {
            before
        };
        if let Some(d) = before.and_then(ordinal) {
            return Some(dated(d, m, year(words.get(at + 1)), day));
        }
        if let Some(d) = words.get(at + 1).and_then(|w| ordinal(w)) {
            return Some(dated(d, m, year(words.get(at + 2)), day));
        }
    }
    None
}

/// « 12/12 », « 15/01[/2027] »: day then month, read only when it cannot be month then day.
fn numeric(words: &[&str], day: NaiveDate) -> Option<Option<NaiveDate>> {
    let word = words.iter().find(|w| w.contains('/'))?;
    let parts: Vec<&str> = word.split('/').collect();
    if !(2..=3).contains(&parts.len()) || parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let (a, b): (u32, u32) = (parts[0].parse().ok()?, parts[1].parse().ok()?);
    if a != b && a <= 12 && b <= 12 {
        return Some(None);
    }
    Some(dated(a, b, year(parts.get(2)), day))
}

/// « dans N jours|semaines », « in N days|weeks »; French counts « huit jours » a week, « quinze » two.
fn within(words: &[&str], day: NaiveDate) -> Option<Option<NaiveDate>> {
    for at in 0..words.len() {
        if !matches!(words[at], "dans" | "in") {
            continue;
        }
        let (Some(count), Some(unit)) = (words.get(at + 1), words.get(at + 2)) else {
            continue;
        };
        let n = count
            .parse::<i64>()
            .ok()
            .or_else(|| NUMBERS.iter().find(|(w, _)| w == count).map(|(_, n)| *n));
        let Some(n) = n else { continue };
        let days = match *unit {
            "jour" | "jours" if n == 8 || n == 15 => n - 1,
            "jour" | "jours" | "day" | "days" => n,
            "semaine" | "semaines" | "week" | "weeks" => n * 7,
            _ => continue,
        };
        return Some(day.checked_add_signed(Duration::days(days)));
    }
    None
}

/// A weekday: the next one strictly after `day`; said on that very weekday, not said.
fn weekday(words: &[&str], day: NaiveDate) -> Option<NaiveDate> {
    let target = words
        .iter()
        .find_map(|w| DAYS.iter().find(|(name, _)| name == w).map(|(_, d)| *d))?;
    let ahead = (7 + target - day.weekday().num_days_from_monday()) % 7;
    if ahead == 0 {
        return None;
    }
    day.checked_add_signed(Duration::days(i64::from(ahead)))
}

