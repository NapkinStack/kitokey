//! D117's `.ics`: one all-day `VEVENT` per date the shell hands over, as RFC 5545 says — lines ended by
//! CRLF and folded at 75 octets, text escaped as §3.3.11 says; read back by an independent parser
//! (icalendar's), and the raw lines checked against the RFC where a lenient parser would not tell.

use icalendar::parser::{Calendar, Component, read_calendar, unfold};
use kitokey_export::{Dated, ics};

/// 2026-11-24T19:05:09Z.
const STAMP: u64 = 1_795_547_109;

/// A meeting's name with commas, semicolons, a newline and non-Latin letters.
const NAME: &str = "Budget, 予算; Ωmega\nsuite \\ fin";

/// A property read back: its name, its value unescaped, its parameters.
type Property = (String, String, Vec<(String, String)>);
/// An event's properties, in their order.
type Event = Vec<Property>;

fn dated(id: &str, date: &str, what: &str, who: Option<&str>, passage: Option<&str>) -> Dated {
    Dated {
        id: id.into(),
        date: date.into(),
        what: what.into(),
        who: who.map(str::to_string),
        passage: passage.map(str::to_string),
    }
}

fn dates() -> Vec<Dated> {
    vec![
        dated(
            "a1",
            "2026-11-27",
            "Envoyer le devis, version 2; avec annexes",
            Some("Léa, Zoé"),
            Some("On l'envoie vendredi ;\nLéa s'en charge, avec Zoé."),
        ),
        dated("a3", "2026-12-31", "Clore l'exercice", None, None),
    ]
}

fn parsed(text: &str) -> (Vec<(String, String)>, Vec<Event>) {
    let unfolded = unfold(text);
    let calendar: Calendar = read_calendar(&unfolded).expect("an independent parser reads it");
    let properties = calendar
        .properties
        .iter()
        .map(|p| (p.name.as_str().to_string(), p.val.as_str().to_string()))
        .collect();
    let events = calendar
        .components
        .iter()
        .map(|c: &Component| {
            assert_eq!(c.name.as_str(), "VEVENT", "the calendar holds events only");
            assert!(c.components.is_empty(), "no alarm, nothing inside an event");
            c.properties
                .iter()
                .map(|p| {
                    let params = p
                        .params
                        .iter()
                        .map(|q| {
                            (
                                q.key.as_str().to_string(),
                                q.val.as_ref().map(|v| v.as_str().to_string()).unwrap_or_default(),
                            )
                        })
                        .collect();
                    (p.name.as_str().to_string(), p.val.as_str().to_string(), params)
                })
                .collect()
        })
        .collect();
    (properties, events)
}

fn value<'a>(event: &'a [Property], name: &str) -> &'a str {
    let found: Vec<_> = event.iter().filter(|(n, ..)| n == name).collect();
    assert_eq!(found.len(), 1, "{name} once in the event: {event:?}");
    &found[0].1
}

fn params<'a>(event: &'a [Property], name: &str) -> &'a [(String, String)] {
    &event.iter().find(|(n, ..)| n == name).unwrap().2
}

#[test]
fn each_date_is_one_all_day_event_with_rfc_5545s_required_properties_read_back_by_a_parser() {
    let text = ics(&dates(), NAME, "2026-11-20_09-30", STAMP).unwrap();
    let (calendar, events) = parsed(&text);

    // §3.6: VERSION and PRODID required, once each.
    assert_eq!(calendar.iter().filter(|(n, _)| n == "VERSION").count(), 1);
    assert_eq!(calendar.iter().find(|(n, _)| n == "VERSION").unwrap().1, "2.0");
    assert_eq!(calendar.iter().filter(|(n, _)| n == "PRODID").count(), 1);
    assert_eq!(
        calendar.iter().find(|(n, _)| n == "PRODID").unwrap().1,
        "-//Kitokey//kitokey-export//EN"
    );

    assert_eq!(events.len(), 2, "one event per date, in their order");
    for event in &events {
        // §3.6.1: UID and DTSTAMP required, DTSTART required when no METHOD; all-day by VALUE=DATE.
        value(event, "UID");
        assert_eq!(value(event, "DTSTAMP"), "20261124T190509Z", "DTSTAMP in UTC");
        assert_eq!(params(event, "DTSTART"), [("VALUE".to_string(), "DATE".to_string())]);
        assert_eq!(params(event, "DTEND"), [("VALUE".to_string(), "DATE".to_string())]);
        let names: Vec<_> = event.iter().map(|(n, ..)| n.as_str()).collect();
        assert!(!names.contains(&"DURATION"), "DTEND, never DURATION beside it");
    }

    let first = &events[0];
    assert_eq!(value(first, "UID"), "a1-2026-11-20_09-30@kitokey");
    assert_eq!(value(first, "DTSTART"), "20261127");
    assert_eq!(
        value(first, "DTEND"),
        "20261128",
        "the next day: an all-day event's end is exclusive"
    );
    assert_eq!(value(first, "SUMMARY"), "Envoyer le devis, version 2; avec annexes — Léa, Zoé");
    assert_eq!(
        value(first, "DESCRIPTION"),
        format!("On l'envoie vendredi ;\nLéa s'en charge, avec Zoé.\n\n{NAME}")
    );

    let second = &events[1];
    assert_eq!(value(second, "UID"), "a3-2026-11-20_09-30@kitokey");
    assert_eq!((value(second, "DTSTART"), value(second, "DTEND")), ("20261231", "20270101"));
    assert_eq!(value(second, "SUMMARY"), "Clore l'exercice", "no who: the action's words alone");
    assert_eq!(value(second, "DESCRIPTION"), NAME, "no passage: the meeting's name alone");
}

#[test]
fn lines_end_with_crlf_and_are_folded_at_75_octets_never_inside_a_letter() {
    let long = "会議で決めたこと、".repeat(20) + &"é, ;".repeat(40);
    let text = ics(
        &[dated("a1", "2026-11-27", &long, Some(&long), Some(&long))],
        &long,
        "2026-11-20_09-30",
        STAMP,
    )
    .unwrap();
    let bytes = text.as_bytes();

    assert!(text.ends_with("\r\n"), "the last line ends with CRLF too");
    assert!(!text.replace("\r\n", "").contains(['\r', '\n']), "no bare CR or LF");
    let lines: Vec<&str> = text.split("\r\n").collect();
    for line in &lines[..lines.len() - 1] {
        assert!(line.len() <= 75, "{} octets: {line}", line.len());
    }
    assert!(lines.iter().any(|line| line.starts_with(' ')), "the long values are folded");
    assert!(
        lines.iter().filter(|line| line.len() > 70).count() > 10,
        "folded near 75, not early"
    );
    // A fold never splits a letter: each physical line is UTF-8 on its own.
    for raw in bytes.split(|b| *b == b'\n') {
        std::str::from_utf8(raw).expect("each physical line is whole UTF-8");
    }
    let (_, events) = parsed(&text);
    assert_eq!(value(&events[0], "SUMMARY"), format!("{long} — {long}"));
}

#[test]
fn text_is_escaped_as_section_3_3_11_says() {
    let text = ics(&dates(), NAME, "2026-11-20_09-30", STAMP).unwrap();
    let unfolded = text.replace("\r\n ", "");

    assert!(unfolded.contains("\r\nSUMMARY:Envoyer le devis\\, version 2\\; avec annexes — Léa\\, Zoé\r\n"));
    assert!(unfolded.contains(
        "\r\nDESCRIPTION:On l'envoie vendredi \\;\\nLéa s'en charge\\, avec Zoé.\\n\\nBudget\\, 予算\\; Ωmega\\nsuite \\\\ fin\r\n"
    ));
    // §3.3.11: a colon is not escaped.
    let text = ics(&[dated("a1", "2026-11-27", "Heure : 10:00", None, None)], "n", "f", STAMP).unwrap();
    assert!(text.contains("\r\nSUMMARY:Heure : 10:00\r\n"));
}

#[test]
fn a_carriage_return_or_a_control_character_never_reaches_the_file_raw() {
    let text = ics(
        &[dated("a1", "2026-11-27", "a", None, Some("un\r\ndeux\rtrois\u{7}\tfin"))],
        "n",
        "f",
        STAMP,
    )
    .unwrap();
    let unfolded = text.replace("\r\n ", "");

    assert!(unfolded.contains("\r\nDESCRIPTION:un\\ndeux\\ntrois \tfin\\n\\nn\r\n"));
}

#[test]
fn the_next_day_crosses_months_years_and_leap_days() {
    for (date, end) in [
        ("2026-01-31", "20260201"),
        ("2026-02-28", "20260301"),
        ("2028-02-28", "20280229"),
        ("2028-02-29", "20280301"),
        ("2100-02-28", "21000301"),
        ("2000-02-28", "20000229"),
        ("2026-04-30", "20260501"),
        ("2026-12-31", "20270101"),
    ] {
        let text = ics(&[dated("a1", date, "a", None, None)], "n", "f", STAMP).unwrap();
        let (_, events) = parsed(&text);
        assert_eq!(value(&events[0], "DTEND"), end, "after {date}");
    }
}

#[test]
fn a_uid_is_stable_for_the_meeting_and_the_item() {
    let once = ics(&dates(), NAME, "2026-11-20_09-30", STAMP).unwrap();
    let again = ics(&dates(), "another name", "2026-11-20_09-30", STAMP + 86_400).unwrap();
    let uids = |text: &str| -> Vec<String> { parsed(text).1.iter().map(|event| value(event, "UID").to_string()).collect() };

    assert_eq!(
        uids(&once),
        uids(&again),
        "the same meeting and items: the same UIDs, whenever written"
    );
    let elsewhere = ics(&dates(), NAME, "2026-11-21_14-00", STAMP).unwrap();
    assert!(
        uids(&once).iter().all(|uid| !uids(&elsewhere).contains(uid)),
        "another meeting: other UIDs"
    );
}

#[test]
fn dtstamp_is_the_moment_written_in_utc() {
    for (stamp, said) in [
        (0, "19700101T000000Z"),
        (951_782_400, "20000229T000000Z"),
        (4_102_444_799, "20991231T235959Z"),
    ] {
        let text = ics(&[dated("a1", "2026-11-27", "a", None, None)], "n", "f", stamp).unwrap();
        assert_eq!(value(&parsed(&text).1[0], "DTSTAMP"), said);
    }
}

#[test]
fn an_impossible_date_or_an_id_given_twice_is_refused() {
    for date in ["2028-02-29", "2000-02-29", "2026-11-30", "2026-12-31"] {
        assert!(
            ics(&[dated("a1", date, "a", None, None)], "n", "f", STAMP).is_ok(),
            "{date} is a day"
        );
    }
    for date in [
        "2026-02-29",
        "2100-02-29",
        "1900-02-29",
        "2026-02-30",
        "2026-13-01",
        "2026-00-10",
        "2026-04-31",
        "2026-11-00",
        "26-11-27",
        "2026-11-27T10:00",
    ] {
        assert!(ics(&[dated("a1", date, "a", None, None)], "n", "f", STAMP).is_err(), "{date}");
    }
    let twice = [
        dated("a1", "2026-11-27", "a", None, None),
        dated("a1", "2026-11-28", "b", None, None),
    ];
    assert!(ics(&twice, "n", "f", STAMP).is_err());
    assert!(ics(&[], "n", "f", STAMP).is_err(), "no date: nothing to write");
}
