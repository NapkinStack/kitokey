//! The `.ics` of the dates a meeting's actions mention (D117), as RFC 5545 writes a calendar: one all-day
//! `VEVENT` per date, lines ended by CRLF and folded at 75 octets (§3.1), text escaped (§3.3.11). The
//! dates come resolved from the shell: a day of the calendar, never moved here.

use std::collections::HashSet;

/// A date the shell hands over, as `exporting/v1`'s input carries it.
#[derive(Clone, Debug)]
pub struct Dated {
    /// The action's key within the meeting, stable: the UID is made of it.
    pub id: String,
    /// `YYYY-MM-DD`, a day of the calendar.
    pub date: String,
    pub what: String,
    pub who: Option<String>,
    pub passage: Option<String>,
}

/// The calendar of `dates`, in their order: `name` the meeting's, `folder` its session's folder's name —
/// with each `id`, a UID stable for the meeting and the item —, `stamp` the moment written, in seconds
/// since 1970 (UTC). Refused — and why — when there is no date, an `id` twice, or a day no calendar has.
pub fn ics(dates: &[Dated], name: &str, folder: &str, stamp: u64) -> Result<String, String> {
    if dates.is_empty() {
        return Err("no date to write".into());
    }
    let mut ids = HashSet::new();
    let mut out = String::new();
    line(&mut out, "BEGIN:VCALENDAR");
    line(&mut out, "VERSION:2.0");
    line(&mut out, "PRODID:-//Kitokey//kitokey-export//EN");
    let (day, time) = (stamp / 86_400, stamp % 86_400);
    let (y, m, d) = civil(day as i64);
    let dtstamp = format!("{y:04}{m:02}{d:02}T{:02}{:02}{:02}Z", time / 3600, time % 3600 / 60, time % 60);
    for dated in dates {
        if !ids.insert(dated.id.as_str()) {
            return Err("an item's id given twice".into());
        }
        let start = day_of(&dated.date).ok_or("a date no calendar has")?;
        let (sy, sm, sd) = civil(start);
        let (ey, em, ed) = civil(start + 1);
        let summary = match &dated.who {
            Some(who) => format!("{} — {who}", dated.what),
            None => dated.what.clone(),
        };
        let description = match &dated.passage {
            Some(passage) => format!("{passage}\n\n{name}"),
            None => name.to_string(),
        };
        line(&mut out, "BEGIN:VEVENT");
        line(&mut out, &format!("UID:{}", text(&format!("{}-{folder}@kitokey", dated.id))));
        line(&mut out, &format!("DTSTAMP:{dtstamp}"));
        line(&mut out, &format!("DTSTART;VALUE=DATE:{sy:04}{sm:02}{sd:02}"));
        line(&mut out, &format!("DTEND;VALUE=DATE:{ey:04}{em:02}{ed:02}"));
        line(&mut out, &format!("SUMMARY:{}", text(&summary)));
        line(&mut out, &format!("DESCRIPTION:{}", text(&description)));
        // A date said is no meeting: the day stays free, where RFC 5545's default would make it busy.
        line(&mut out, "TRANSP:TRANSPARENT");
        line(&mut out, "END:VEVENT");
    }
    line(&mut out, "END:VCALENDAR");
    Ok(out)
}

/// A content line, folded as §3.1 says: at most 75 octets a line, CRLF excluded, the next one opened by a
/// space; never inside a letter's UTF-8.
fn line(out: &mut String, content: &str) {
    let mut width = 0;
    for letter in content.chars() {
        if width + letter.len_utf8() > 75 {
            out.push_str("\r\n ");
            width = 1;
        }
        out.push(letter);
        width += letter.len_utf8();
    }
    out.push_str("\r\n");
}

/// A TEXT value as §3.3.11 escapes it: a backslash, a semicolon and a comma behind a backslash, a line's
/// end — CRLF, LF or CR — as `\n`. A control character other than a tab, which TEXT does not allow, is a
/// space.
fn text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut letters = value.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' | ';' | ',' => {
                out.push('\\');
                out.push(letter);
            }
            '\r' => {
                letters.next_if_eq(&'\n');
                out.push_str("\\n");
            }
            '\n' => out.push_str("\\n"),
            '\t' => out.push('\t'),
            control if control.is_control() => out.push(' '),
            _ => out.push(letter),
        }
    }
    out
}

/// The day `YYYY-MM-DD` names, counted from 1970-01-01; none for a day no Gregorian calendar has.
fn day_of(date: &str) -> Option<i64> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let digits = |range: std::ops::Range<usize>| -> Option<i64> {
        date[range]
            .bytes()
            .try_fold(0, |n, b| b.is_ascii_digit().then(|| n * 10 + i64::from(b - b'0')))
    };
    let (y, m, d) = (digits(0..4)?, digits(5..7)?, digits(8..10)?);
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let last = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    (1..=last).contains(&d).then(|| days(y, m, d))
}

/// Days from 1970-01-01 to a day of the proleptic Gregorian calendar (H. Hinnant, *chrono-Compatible
/// Low-Level Date Algorithms*, `days_from_civil`).
fn days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The day of the calendar `days` after 1970-01-01 (the same paper's `civil_from_days`).
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}
