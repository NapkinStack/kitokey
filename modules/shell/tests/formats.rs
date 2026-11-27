//! D130: the formats a person manages — made, duplicated, renamed, changed, deleted; the six built
//! in hidden and shown again, never changed nor deleted; one the default — and the file
//! `summarising/v1`'s `--format` reads, written from one. Names and words here are made up.

use kitokey_shell::formats::{self, BUILT_IN, Format, Formats, Refused, Section};

fn section(title: &str, about: &str) -> Section {
    Section {
        title: title.into(),
        about: about.into(),
    }
}

fn weekly() -> Vec<Section> {
    vec![
        section("Contexte", "Où en est le client, en deux phrases"),
        section("Demandes", "Ce que le client a demandé, une ligne chacune"),
        section("Prochaines étapes", "Qui fait quoi, pour quand"),
    ]
}

#[test]
fn the_six_built_in_are_offered_first_general_the_default_and_none_of_the_persons() {
    let formats = Formats::default();
    assert_eq!(
        BUILT_IN,
        ["general", "one_to_one", "stand_up", "client_call", "interview", "project_review"]
    );
    assert_eq!(formats.shown(), BUILT_IN.to_vec());
    assert_eq!(formats.default, "general");
    assert!(formats.own.is_empty());
}

#[test]
fn a_format_made_is_kept_under_a_name_of_its_own_offered_after_the_built_in() {
    let mut formats = Formats::default();
    let made = formats.made("Point client hebdo", weekly(), Some(2)).unwrap();
    // Its structure, as the minutes write it: the name in [a-z0-9_], its marks taken off.
    assert_eq!(made, "point_client_hebdo");
    assert_eq!(formats.shown().last(), Some(&"point_client_hebdo"));
    let kept = formats.own("point_client_hebdo").unwrap();
    assert_eq!(kept.name, "Point client hebdo");
    assert_eq!(kept.sections, weekly());
    assert_eq!(kept.actions, Some(2));
    // A second of the same name is refused, whatever its case or spaces; one whose structure would
    // be taken gets one of its own.
    assert_eq!(formats.made(" point CLIENT hebdo ", weekly(), None), Err(Refused::NameTaken));
    assert_eq!(formats.made("Point client hébdo", weekly(), None).unwrap(), "point_client_hebdo_2");
    // A name that would be a built-in's structure never takes it.
    assert_eq!(formats.made("General", weekly(), None).unwrap(), "general_2");
    // A name with no letter of [a-z0-9] still gets one.
    assert_eq!(formats.made("週報", weekly(), None).unwrap(), "format");
}

#[test]
fn a_format_is_refused_outside_minutes_v5_s_bounds() {
    let mut formats = Formats::default();
    let long = "é".repeat(201);
    let cases: Vec<(&str, Vec<Section>, Option<usize>, Refused)> = vec![
        ("", weekly(), None, Refused::Name),
        ("   ", weekly(), None, Refused::Name),
        ("a\nb", weekly(), None, Refused::Name),
        (&"x".repeat(41), weekly(), None, Refused::Name),
        ("Vide", vec![], None, Refused::Sections),
        ("Sept", (0..7).map(|n| section(&format!("S{n}"), "x")).collect(), None, Refused::Sections),
        ("Sans titre", vec![section(" ", "x")], None, Refused::Section(0)),
        ("Titre long", vec![section(&"t".repeat(41), "x")], None, Refused::Section(0)),
        ("Sans contenu", vec![section("A", "x"), section("B", "")], None, Refused::Section(1)),
        ("Deux lignes", vec![section("A", "x\ny")], None, Refused::Section(0)),
        ("Trop long", vec![section("A", &long)], None, Refused::Section(0)),
        // Two titles that would be one section's id.
        ("Deux fois", vec![section("Suites", "x"), section("suites", "y")], None, Refused::Section(1)),
        ("Actions", weekly(), Some(3), Refused::Actions),
    ];
    for (name, sections, actions, why) in cases {
        assert_eq!(formats.made(name, sections, actions), Err(why), "{name:?}");
    }
    assert!(formats.own.is_empty());
    // 200 characters, counted as characters, are within.
    assert!(formats.made("Long", vec![section("A", &"é".repeat(200))], None).is_ok());
}

#[test]
fn a_format_is_duplicated_renamed_changed_and_deleted_its_structure_kept() {
    let mut formats = Formats::default();
    let made = formats.made("Point client", weekly(), Some(2)).unwrap();
    let copy = formats.own(&made).unwrap().clone();
    let twin = formats.duplicated(&copy, "Point client (copie)").unwrap();
    assert_eq!(twin, "point_client_copie");
    assert_eq!(formats.own(&twin).unwrap().sections, weekly());

    // Renamed: its name, never its structure — the minutes already written still name it.
    formats.renamed(&made, "Le point du lundi").unwrap();
    assert_eq!(formats.own(&made).unwrap().name, "Le point du lundi");
    assert_eq!(formats.renamed(&made, "Point client (copie)"), Err(Refused::NameTaken));
    assert_eq!(formats.renamed(&made, ""), Err(Refused::Name));

    // Changed: its sections and the one holding the actions, bounded as when made.
    let fewer = vec![section("Notes", "Tout ce qui a été dit d'important")];
    formats.changed(&made, fewer.clone(), None).unwrap();
    assert_eq!(formats.own(&made).unwrap().sections, fewer);
    assert_eq!(formats.own(&made).unwrap().actions, None);
    assert_eq!(formats.changed(&made, vec![], None), Err(Refused::Sections));
    assert_eq!(formats.own(&made).unwrap().sections, fewer);

    formats.deleted(&made).unwrap();
    assert!(formats.own(&made).is_none());
    assert_eq!(formats.deleted(&made), Err(Refused::Unknown));
}

#[test]
fn a_built_in_is_hidden_and_shown_again_never_changed_renamed_nor_deleted() {
    let mut formats = Formats::default();
    formats.hide("interview", true).unwrap();
    assert!(!formats.shown().contains(&"interview"));
    formats.hide("interview", false).unwrap();
    assert_eq!(formats.shown(), BUILT_IN.to_vec());
    for built in BUILT_IN {
        assert_eq!(formats.renamed(built, "Autre"), Err(Refused::BuiltIn));
        assert_eq!(formats.changed(built, weekly(), None), Err(Refused::BuiltIn));
        assert_eq!(formats.deleted(built), Err(Refused::BuiltIn));
    }
    // A person's own format is deleted, not hidden.
    let made = formats.made("Point", weekly(), None).unwrap();
    assert_eq!(formats.hide(&made, true), Err(Refused::NotBuiltIn));
    assert_eq!(formats.hide("unknown", true), Err(Refused::Unknown));
}

#[test]
fn one_is_the_default_always_one_offered() {
    let mut formats = Formats::default();
    let made = formats.made("Point", weekly(), None).unwrap();
    formats.default_to(&made).unwrap();
    assert_eq!(formats.default, made);
    // The default deleted, `general` is the default again.
    formats.deleted(&made).unwrap();
    assert_eq!(formats.default, "general");
    // The default is never hidden, nor a hidden one the default.
    assert_eq!(formats.hide("general", true), Err(Refused::Default));
    formats.hide("stand_up", true).unwrap();
    assert_eq!(formats.default_to("stand_up"), Err(Refused::Hidden));
    assert_eq!(formats.default_to("unknown"), Err(Refused::Unknown));
    // The default deleted while `general` is hidden: the first offered is.
    let made = formats.made("Point", weekly(), None).unwrap();
    formats.default_to(&made).unwrap();
    formats.hide("general", true).unwrap();
    formats.deleted(&made).unwrap();
    assert_eq!(formats.default, "one_to_one");
}

#[test]
fn the_file_given_to_kitokey_summary_is_summarising_v1_s_format() {
    let mut formats = Formats::default();
    let made = formats.made("Point client hebdo", weekly(), Some(2)).unwrap();
    let file = formats::file(formats.own(&made).unwrap());
    assert_eq!(
        file,
        serde_json::json!({
            "structure": "point_client_hebdo",
            "sections": [
                { "id": "contexte", "about": "Où en est le client, en deux phrases" },
                { "id": "demandes", "about": "Ce que le client a demandé, une ligne chacune" },
                { "id": "prochaines_etapes", "about": "Qui fait quoi, pour quand" },
            ],
            "actions": "prochaines_etapes",
        })
    );
    // No actions: no key. A title with no letter of [a-z0-9] still gets an id, its place.
    let other = Format {
        structure: "notes".into(),
        name: "Notes".into(),
        sections: vec![section("要点", "x"), section("Autre", "y")],
        actions: None,
    };
    assert_eq!(
        formats::file(&other),
        serde_json::json!({ "structure": "notes", "sections": [{ "id": "section_1", "about": "x" }, { "id": "autre", "about": "y" }] })
    );
}
