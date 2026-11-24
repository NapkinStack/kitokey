//! D115, criterion 2: an action's *by when* read against the meeting's day. An absolute date only for an
//! explicit date, a weekday, « demain », « dans N jours|semaines » and their English forms; « not said »
//! (`None`) for anything else; no procedural rule; the reference day the meeting's local start, as its
//! folder names it (session-audio/v3, *The folder*), never UTC.

use chrono::NaiveDate;
use kitokey_shell::dates::{reference_day, resolve};

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Tuesday 24 November 2026.
fn tuesday() -> NaiveDate {
    day(2026, 11, 24)
}

#[test]
fn an_explicit_date_with_its_year_is_that_date() {
    assert_eq!(resolve("12 décembre 2026", tuesday()), Some(day(2026, 12, 12)));
    assert_eq!(resolve("December 12, 2026", tuesday()), Some(day(2026, 12, 12)));
    assert_eq!(resolve("le 3 mars 2027", tuesday()), Some(day(2027, 3, 3)));
    // A year said is kept, even a later one than the next 12 December.
    assert_eq!(resolve("12 décembre 2027", tuesday()), Some(day(2027, 12, 12)));
    assert_eq!(resolve("2026-12-12", tuesday()), Some(day(2026, 12, 12)));
}

#[test]
fn an_explicit_date_without_its_year_is_the_next_one_on_or_after_the_day() {
    assert_eq!(resolve("12 décembre", tuesday()), Some(day(2026, 12, 12)));
    assert_eq!(resolve("le 1er décembre", tuesday()), Some(day(2026, 12, 1)));
    assert_eq!(resolve("24 novembre", tuesday()), Some(day(2026, 11, 24)));
    assert_eq!(resolve("3 mars", tuesday()), Some(day(2027, 3, 3)));
    assert_eq!(resolve("by March 3rd", tuesday()), Some(day(2027, 3, 3)));
    assert_eq!(resolve("3rd of March", tuesday()), Some(day(2027, 3, 3)));
    assert_eq!(resolve("12/12", tuesday()), Some(day(2026, 12, 12)));
    assert_eq!(resolve("15/01", tuesday()), Some(day(2027, 1, 15)));
}

#[test]
fn a_date_that_does_not_exist_or_reads_two_ways_is_not_said() {
    assert_eq!(resolve("31 février", tuesday()), None);
    assert_eq!(resolve("30/02", tuesday()), None);
    // 3 April or 4 March: which one is not said.
    assert_eq!(resolve("3/4", tuesday()), None);
}

#[test]
fn a_weekday_is_the_next_one_strictly_after_the_day() {
    assert_eq!(resolve("jeudi", tuesday()), Some(day(2026, 11, 26)));
    assert_eq!(resolve("d'ici lundi", tuesday()), Some(day(2026, 11, 30)));
    assert_eq!(resolve("by Friday", tuesday()), Some(day(2026, 11, 27)));
    assert_eq!(resolve("Monday", tuesday()), Some(day(2026, 11, 30)));
    // Said on that very weekday: this one or the next, not said.
    assert_eq!(resolve("mardi", tuesday()), None);
    assert_eq!(resolve("Tuesday", tuesday()), None);
}

#[test]
fn tomorrow_and_in_n_days_or_weeks() {
    assert_eq!(resolve("demain", tuesday()), Some(day(2026, 11, 25)));
    assert_eq!(resolve("Tomorrow", tuesday()), Some(day(2026, 11, 25)));
    assert_eq!(resolve("dans 3 jours", tuesday()), Some(day(2026, 11, 27)));
    assert_eq!(resolve("dans deux semaines", tuesday()), Some(day(2026, 12, 8)));
    assert_eq!(resolve("in 10 days", tuesday()), Some(day(2026, 12, 4)));
    assert_eq!(resolve("in two weeks", tuesday()), Some(day(2026, 12, 8)));
    assert_eq!(resolve("in a week", tuesday()), Some(day(2026, 12, 1)));
    // French counts « huit jours » a week and « quinze jours » two.
    assert_eq!(resolve("dans huit jours", tuesday()), Some(day(2026, 12, 1)));
    assert_eq!(resolve("dans quinze jours", tuesday()), Some(day(2026, 12, 8)));
}

#[test]
fn anything_else_is_not_said() {
    for when in [
        "fin du mois",
        "end of the month",
        "cet été",
        "this summer",
        "bientôt",
        "soon",
        "ASAP",
        "après-demain",
        "la semaine prochaine",
        "next week",
        "",
        "—",
        "non précisé",
        "not specified",
    ] {
        assert_eq!(resolve(when, tuesday()), None, "{when:?}");
    }
}

#[test]
fn no_procedural_rule_moves_a_date() {
    // Friday 27 November: tomorrow is a Saturday, and stays one.
    assert_eq!(resolve("demain", day(2026, 11, 27)), Some(day(2026, 11, 28)));
    assert_eq!(resolve("samedi", tuesday()), Some(day(2026, 11, 28)));
    // 25 December, a holiday, stays itself.
    assert_eq!(resolve("25 décembre", tuesday()), Some(day(2026, 12, 25)));
}

#[test]
fn the_reference_day_is_the_folders_local_day() {
    assert_eq!(reference_day("2026-11-24_14-05"), Some(tuesday()));
    assert_eq!(reference_day("2026-11-24_14-05-2"), Some(tuesday()));
    // Started at 23:30 local: its day, whatever UTC says.
    let late = reference_day("2026-11-24_23-30").unwrap();
    assert_eq!(late, tuesday());
    assert_eq!(resolve("demain", late), Some(day(2026, 11, 25)));
    // Across a daylight saving change (Europe, 25 October 2026 at 03:00): calendar days, not 24-hour spans.
    let changed = reference_day("2026-10-25_01-30").unwrap();
    assert_eq!(resolve("demain", changed), Some(day(2026, 10, 26)));
    assert_eq!(resolve("dans une semaine", changed), Some(day(2026, 11, 1)));
    assert_eq!(reference_day("notes"), None);
    assert_eq!(reference_day("2026-13-01_10-00"), None);
}

// What the verifier found at fb5cd4f (marc, #1187): a date's guess, never a date.

#[test]
fn a_count_out_of_reach_is_not_said_and_never_panics() {
    for when in [
        "dans 999999999999 jours",
        "in 9999999999999999 weeks",
        "dans 9223372036854775807 semaines",
        "dans 99999999999999999999999 jours",
        "dans 0 jours",
        "dans -3 jours",
        "in -2 weeks",
    ] {
        assert_eq!(resolve(when, tuesday()), None, "{when:?}");
    }
}

#[test]
fn a_past_a_negation_a_range_or_a_rule_is_not_said() {
    for when in [
        "après demain",
        "apres demain",
        "pas demain",
        "jeudi dernier",
        "last Friday",
        "dans 2 jours ouvrés",
        "in 2 business days",
        "lundi ou mardi",
        "Monday or Tuesday",
        "demain ou jeudi",
        "vendredi prochain",
        "next Friday",
        "dans 1 semaine et 2 jours",
        "2 jours",
        "dans les 2 jours",
        "12 décembre, 15 décembre",
        "lundi, mardi",
        "lundi et mardi",
    ] {
        assert_eq!(resolve(when, tuesday()), None, "{when:?}");
    }
}

#[test]
fn a_weekday_and_a_date_that_disagree_are_not_said() {
    assert_eq!(resolve("jeudi 27 novembre", tuesday()), None);
    assert_eq!(resolve("lundi 25 décembre", tuesday()), None);
    assert_eq!(resolve("vendredi 27 novembre", tuesday()), Some(day(2026, 11, 27)));
    assert_eq!(resolve("Friday, December 25", tuesday()), Some(day(2026, 12, 25)));
}

#[test]
fn n_in_digits_is_taken_as_written() {
    assert_eq!(resolve("dans 15 jours", tuesday()), Some(day(2026, 12, 9)));
    assert_eq!(resolve("dans 8 jours", tuesday()), Some(day(2026, 12, 2)));
}

#[test]
fn the_next_29_february_is_the_next_one_that_exists() {
    assert_eq!(resolve("29 février", tuesday()), Some(day(2028, 2, 29)));
}

#[test]
fn a_folder_whose_hour_is_not_one_names_no_day() {
    for folder in [
        "2026-11-24_24-00",
        "2026-11-24_99-00",
        "2026-11-24_14-75",
        "2026-11-24_1430",
        "2026-11-24_14-30x",
        "2026-11-24_",
    ] {
        assert_eq!(reference_day(folder), None, "{folder:?}");
    }
}

use kitokey_shell::dates::{Event, google};

fn event<'a>(date: NaiveDate, who: Option<&'a str>, passage: Option<&'a str>) -> Event<'a> {
    Event {
        date,
        what: "Envoyer le devis & la grille #2",
        who,
        passage,
        meeting: "Point client — Dupont",
    }
}

#[test]
fn a_google_link_is_an_all_day_event_pre_filled_unsaved() {
    let link = google(&event(day(2026, 11, 26), Some("Léa"), Some("on se dit jeudi\npour le devis")));
    assert_eq!(
        link,
        "https://calendar.google.com/calendar/render?action=TEMPLATE\
         &text=Envoyer%20le%20devis%20%26%20la%20grille%20%232%20%E2%80%94%20L%C3%A9a\
         &dates=20261126/20261127\
         &details=on%20se%20dit%20jeudi%0Apour%20le%20devis%0A%0APoint%20client%20%E2%80%94%20Dupont"
    );
}

#[test]
fn a_google_link_ends_the_next_day_across_a_year_and_says_no_one_unnamed() {
    let link = google(&event(day(2026, 12, 31), None, None));
    assert!(link.contains("&dates=20261231/20270101&"), "{link}");
    assert!(link.contains("&text=Envoyer%20le%20devis%20%26%20la%20grille%20%232&"), "{link}");
    assert!(link.ends_with("&details=Point%20client%20%E2%80%94%20Dupont"), "{link}");
    // Every value percent-encoded: nothing but unreserved characters and escapes after each `=`.
    for value in link
        .split('?')
        .nth(1)
        .unwrap()
        .split('&')
        .map(|pair| pair.split_once('=').unwrap().1)
    {
        assert!(value.chars().all(|c| c.is_ascii_alphanumeric() || "-._~%/".contains(c)), "{value}");
    }
}
