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
    assert_eq!(
        formats.made("Point client hebdo !", weekly(), None).unwrap(),
        "point_client_hebdo_2"
    );
    // A name that would be a built-in's structure never takes it.
    assert_eq!(formats.made("General.", weekly(), None).unwrap(), "general_2");
    // A name with no letter of [a-z0-9] still gets one.
    assert_eq!(formats.made("週報", weekly(), None).unwrap(), "format");
}

#[test]
fn a_format_is_refused_outside_minutes_v5_s_bounds() {
    let mut formats = Formats::default();
    let long = "é".repeat(201);
    let long_name = "x".repeat(41);
    let cases: Vec<(&str, Vec<Section>, Option<usize>, Refused)> = vec![
        ("", weekly(), None, Refused::Name),
        ("   ", weekly(), None, Refused::Name),
        ("a\nb", weekly(), None, Refused::Name),
        (&long_name, weekly(), None, Refused::Name),
        ("Vide", vec![], None, Refused::Sections),
        (
            "Sept",
            (0..7).map(|n| section(&format!("S{n}"), "x")).collect(),
            None,
            Refused::Sections,
        ),
        ("Sans titre", vec![section(" ", "x")], None, Refused::Section(0)),
        ("Titre long", vec![section(&"t".repeat(41), "x")], None, Refused::Section(0)),
        ("Sans contenu", vec![section("A", "x"), section("B", "")], None, Refused::Section(1)),
        ("Deux lignes", vec![section("A", "x\ny")], None, Refused::Section(0)),
        ("Trop long", vec![section("A", &long)], None, Refused::Section(0)),
        // Two titles that would be one section's id.
        (
            "Deux fois",
            vec![section("Suites", "x"), section("suites", "y")],
            None,
            Refused::Section(1),
        ),
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

#[test]
fn the_formats_are_kept_in_the_settings_file_and_read_back_as_they_were() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("settings.toml");
    let mut settings = kitokey_shell::settings::Settings::default();
    assert_eq!(settings.formats, Formats::default());
    let made = settings.formats.made("Point client hebdo", weekly(), Some(2)).unwrap();
    settings.formats.made("Notes", vec![section("Notes", "Tout")], None).unwrap();
    settings.formats.default_to(&made).unwrap();
    settings.formats.hide("interview", true).unwrap();
    kitokey_shell::settings::write(&path, &settings).unwrap();
    assert_eq!(kitokey_shell::settings::read(&path).unwrap().formats, settings.formats);
    // Deleted, it is gone from the file.
    settings.formats.deleted(&made).unwrap();
    kitokey_shell::settings::write(&path, &settings).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.contains("point_client_hebdo"), "{text}");
    assert_eq!(kitokey_shell::settings::read(&path).unwrap().formats, settings.formats);
}

#[test]
fn a_format_written_by_hand_outside_its_bounds_is_left_aside_the_rest_read() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("settings.toml");
    std::fs::write(
        &path,
        r#"format_default = "interview"
formats_hidden = ["interview", "one_to_one", "made_up"]

[[formats]]
structure = "point"
name = "Point"
actions = 0
sections = [{ title = "Suites", about = "Qui, quoi, quand" }]

[[formats]]
structure = "point"
name = "Le même"
sections = [{ title = "A", about = "x" }]

[[formats]]
structure = "general"
name = "Pris"
sections = [{ title = "A", about = "x" }]

[[formats]]
structure = "Capitales"
name = "Capitales"
sections = [{ title = "A", about = "x" }]

[[formats]]
structure = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
name = "Quarante et un"
sections = [{ title = "A", about = "x" }]

[[formats]]
structure = "vide"
name = "Vide"
sections = []

[[formats]]
structure = "actions"
name = "Actions"
actions = 4
sections = [{ title = "A", about = "x" }]
"#,
    )
    .unwrap();
    let formats = kitokey_shell::settings::read(&path).unwrap().formats;
    assert_eq!(formats.own.iter().map(|f| f.structure.as_str()).collect::<Vec<_>>(), ["point"]);
    assert_eq!(formats.own[0].actions, Some(0));
    // Only a built-in is hidden; the default, never: the one written is hidden, so `general` is.
    assert_eq!(formats.hidden, ["interview", "one_to_one"]);
    assert_eq!(formats.default, "general");
}

/// lina's remark on #1324, the coordinator's decision of 2026-11-28: a name is taken by a built-in format's
/// name in either language Kitokey speaks, or by the person's own, whatever its case and its accents.
#[test]
fn a_name_a_built_in_format_or_another_has_is_taken_whatever_its_case_and_accents() {
    let mut formats = Formats::default();
    for taken in [
        "Général",
        "general",
        "GÉNÉRAL",
        "Entretien",
        "entretien",
        "Interview",
        "One-to-one",
        "revue de projet",
        "Project review",
    ] {
        assert_eq!(formats.made(taken, weekly(), None), Err(Refused::NameTaken), "{taken}");
    }
    // The same name typed with a straight apostrophe, or a no-break space (the coordinator, 2026-11-28).
    for taken in ["Point d'equipe", "Point\u{a0}d’équipe"] {
        assert_eq!(formats.made(taken, weekly(), None), Err(Refused::NameTaken), "{taken}");
    }
    let made = formats.made("Pôint client", weekly(), None).unwrap();
    assert_eq!(formats.made("point CLIENT", weekly(), None), Err(Refused::NameTaken));
    assert_eq!(formats.renamed(&made, "Appel client"), Err(Refused::NameTaken));
    assert_eq!(formats.own(&made).unwrap().name, "Pôint client");
    // Written by hand under a built-in's name: left aside, as any other out of bounds.
    let kept = Formats::kept(
        vec![Format {
            structure: "pris".into(),
            name: "Point d’équipe".into(),
            sections: weekly(),
            actions: None,
        }],
        vec![],
        None,
    );
    assert!(kept.own.is_empty(), "{kept:?}");
}

/// lina's survivors on #1324 (2026-11-28): the bounds as `format.schema.json` writes them, each pinned.
#[test]
fn a_carriage_return_is_refused_in_a_name_a_title_and_what_goes_in_a_section() {
    let mut formats = Formats::default();
    assert_eq!(formats.made("Point\rclient", weekly(), None), Err(Refused::Name));
    assert_eq!(
        formats.made("Point", vec![section("Con\rtexte", "x")], None),
        Err(Refused::Section(0))
    );
    assert_eq!(
        formats.made("Point", vec![section("Contexte", "Où en est\rle client")], None),
        Err(Refused::Section(0))
    );
    assert!(formats.own.is_empty());
}

#[test]
fn the_actions_are_the_section_chosen_wherever_it_stands() {
    let mut formats = Formats::default();
    for (at, id) in [(0, "contexte"), (1, "demandes")] {
        let made = formats.made(&format!("Point {at}"), weekly(), Some(at)).unwrap();
        assert_eq!(formats::file(formats.own(&made).unwrap())["actions"], id, "actions at {at}");
    }
}

#[test]
fn a_structure_taken_at_forty_characters_gets_one_of_forty_with_its_number() {
    let mut formats = Formats::default();
    let first = formats.made("Réunion hebdomadaire de l’équipe produit", weekly(), None).unwrap();
    assert_eq!(first, "reunion_hebdomadaire_de_l_equipe_produit");
    let second = formats.made("Reunion hebdomadaire de l equipe produit", weekly(), None).unwrap();
    assert_eq!(second, "reunion_hebdomadaire_de_l_equipe_produ_2");
    assert_eq!(second.chars().count(), 40);
}
#[test]
fn kitokey_summary_is_given_the_format_chosen_as_summarising_v1_says() {
    let folder = tempfile::tempdir().unwrap();
    let mut formats = Formats::default();
    let made = formats.made("Point client hebdo", weekly(), Some(2)).unwrap();
    // `general`: no structure, as before — minutes/v4, its sections the same.
    assert!(formats::argument(&formats, "general", folder.path()).unwrap().is_empty());
    // Another built-in: its name.
    assert_eq!(
        formats::argument(&formats, "stand_up", folder.path()).unwrap(),
        ["--structure", "stand_up"]
    );
    // A person's own: the file --format reads, written in the formats' folder.
    let given = formats::argument(&formats, &made, folder.path()).unwrap();
    let file = folder.path().join("point_client_hebdo.json");
    assert_eq!(given, ["--format".to_string(), file.to_string_lossy().into_owned()]);
    let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(written, formats::file(formats.own(&made).unwrap()));
    // One deleted, or hidden, since it was chosen: the default.
    formats.default_to("client_call").unwrap();
    formats.hide("interview", true).unwrap();
    for gone in ["interview", "deleted_since"] {
        assert_eq!(
            formats::argument(&formats, gone, folder.path()).unwrap(),
            ["--structure", "client_call"]
        );
    }
}

/// lina's point 3 on #1340: the page names the built-in formats in `formats::BUILT_IN`'s order, its own copy
/// the same — `words.format_names` follows that order.
#[test]
fn the_page_lists_the_built_in_formats_as_formats_rs_does() {
    let page = include_str!("../ui/window.js");
    let line = page
        .lines()
        .find(|line| line.starts_with("const BUILT_IN = ["))
        .expect("window.js's BUILT_IN");
    let listed: Vec<&str> = line.split('"').skip(1).step_by(2).collect();
    assert_eq!(listed, formats::BUILT_IN);
}
