//! D110, PDR-0007 §1: a destination a summary is sent to — its kind, its name, its address shown
//! only as its host and the end of its path once kept; a name a person reads, never a path or markup.
//! Addresses here are made up.

use kitokey_shell::destinations::{self, Kind};

#[test]
fn an_address_is_shown_as_its_host_and_the_end_of_its_path_never_whole() {
    let address = "https://hooks.example/services/T00/B00/made-up-secret-7f3a";
    let shown = destinations::shown(address);
    assert_eq!(shown, "hooks.example/…7f3a");
    assert!(!shown.contains("made-up-secret"), "{shown}");
    // A short path keeps only its end, never more than its last four characters.
    assert_eq!(
        destinations::shown("https://discord.example/api/webhooks/1/ab"),
        "discord.example/…/ab"
    );
    // No path: the host alone; a query or a fragment is never shown.
    assert_eq!(destinations::shown("https://hooks.example"), "hooks.example");
    assert_eq!(destinations::shown("https://hooks.example/x?token=made-up#y"), "hooks.example/…/x");
    // Something that is no address shows nothing of itself.
    assert_eq!(destinations::shown("not an address made-up-secret"), "");
}

#[test]
fn a_destination_is_named_on_one_line_of_at_most_forty_characters() {
    assert_eq!(destinations::name("  #équipe  "), Some("#équipe".to_string()));
    assert_eq!(destinations::name(""), None);
    assert_eq!(destinations::name("   "), None);
    assert_eq!(destinations::name("a\nb"), None);
    assert_eq!(destinations::name(&"x".repeat(41)), None);
    assert_eq!(destinations::name(&"é".repeat(40)), Some("é".repeat(40)));
}

#[test]
fn its_kind_is_one_the_posting_contract_names() {
    assert_eq!(Kind::named("webhook"), Some(Kind::Webhook));
    assert_eq!(Kind::named("discord"), Some(Kind::Discord));
    assert_eq!(Kind::named("slack"), Some(Kind::Slack));
    assert_eq!(Kind::named("teams"), None);
    assert_eq!(Kind::Slack.as_str(), "slack");
}

#[test]
fn its_address_is_kept_in_the_keyring_by_its_name_never_by_a_host() {
    keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
    destinations::keep("#équipe", "https://hooks.example/made-up").unwrap();
    assert_eq!(
        destinations::address("#équipe").unwrap().as_deref(),
        Some("https://hooks.example/made-up")
    );
    // Filed apart from a provider's key: a destination named as a host is not that host's key.
    assert_eq!(kitokey_shell::keys::key("#équipe"), Ok(None));
    destinations::forget("#équipe").unwrap();
    assert_eq!(destinations::address("#équipe"), Ok(None));
}

#[test]
fn the_settings_file_keeps_each_destination_by_name_and_kind_and_never_its_address() {
    use kitokey_shell::destinations::Destination;
    use kitokey_shell::settings;
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("settings.toml");
    std::fs::write(&file, "# mine\nafter = \"nothing\"\n").unwrap();
    let mut chosen = settings::read(&file).unwrap();
    chosen.destinations.push(Destination {
        name: "#équipe".into(),
        kind: Kind::Slack,
        shown: destinations::shown("https://hooks.example/services/made-up-secret-7f3a"),
    });
    settings::write(&file, &chosen).unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("# mine") && text.contains("[[destinations]]") && !text.contains("made-up-secret"),
        "{text}"
    );
    assert_eq!(settings::read(&file).unwrap().destinations, chosen.destinations);
    // Written by hand: a name that is no name, or the same one twice, is left aside.
    std::fs::write(
        &file,
        "[[destinations]]\nname = \"  \"\nkind = \"webhook\"\nshown = \"\"\n\n[[destinations]]\nname = \"#a\"\nkind = \"discord\"\nshown = \"d\"\n\n[[destinations]]\nname = \"#a\"\nkind = \"slack\"\nshown = \"s\"\n",
    )
    .unwrap();
    let read = settings::read(&file).unwrap().destinations;
    assert_eq!(read.len(), 1);
    assert_eq!((read[0].name.as_str(), read[0].kind), ("#a", Kind::Discord));
    // Forgotten, the last one takes its key out of the file.
    let mut none = settings::read(&file).unwrap();
    none.destinations.clear();
    settings::write(&file, &none).unwrap();
    assert!(!std::fs::read_to_string(&file).unwrap().contains("destinations"));
}

#[test]
fn a_name_already_kept_is_refused_its_address_untouched_and_one_removed_leaves_the_keyring_first() {
    // #1129: settings and keyring through the same two functions as the window's commands.
    use kitokey_shell::destinations::{Destination, NotAdded};
    keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
    let one = |name: &str| Destination {
        name: name.into(),
        kind: Kind::Discord,
        shown: "hooks.example/…7f3a".into(),
    };
    let mut kept = Vec::new();
    destinations::add(&mut kept, one("#kept"), "https://hooks.example/first").unwrap();
    assert_eq!(
        destinations::add(&mut kept, one("#kept"), "https://hooks.example/second"),
        Err(NotAdded::Named)
    );
    assert_eq!(
        destinations::address("#kept").unwrap().as_deref(),
        Some("https://hooks.example/first")
    );
    assert_eq!(kept.len(), 1);
    // The keyring refusing the removal: still listed, to be removed again; never a secret left unseen.
    destinations::add(&mut kept, one("#stuck"), "https://hooks.example/stuck").unwrap();
    let stuck = keyring_core::Entry::new("com.napkinstack.Kitokey", "destination:#stuck").unwrap();
    let mock: &keyring_core::mock::Cred = stuck.as_any().downcast_ref().unwrap();
    mock.set_error(keyring_core::Error::NoStorageAccess("locked".into()));
    assert!(destinations::remove(&mut kept, "#stuck").is_err());
    assert_eq!(kept.iter().map(|one| one.name.as_str()).collect::<Vec<_>>(), ["#kept", "#stuck"]);
    destinations::remove(&mut kept, "#kept").unwrap();
    destinations::remove(&mut kept, "#stuck").unwrap();
    assert_eq!(
        (kept.len(), destinations::address("#kept"), destinations::address("#stuck")),
        (0, Ok(None), Ok(None))
    );
}
