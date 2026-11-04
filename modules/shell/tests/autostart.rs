//! Kitokey started with the user's session (D32): an entry of the user's autostart folder, as
//! the XDG Autostart specification has it — the file is the setting, written once by itself.

use std::fs;

use kitokey_shell::{autostart, settings};

#[test]
fn the_entry_starts_the_tray_alone_and_nothing_once_the_package_is_gone() {
    let folder = tempfile::tempdir().unwrap();
    let entry = folder.path().join("autostart").join(autostart::NAME);
    assert_eq!(autostart::NAME, "com.napkinstack.Kitokey.desktop");
    assert!(!autostart::on(&entry));
    autostart::set(&entry, true).unwrap();
    assert!(autostart::on(&entry));
    let text = fs::read_to_string(&entry).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.first(), Some(&"[Desktop Entry]"), "{text}");
    // The tray, no window (D17); a removed package starts nothing.
    let wanted = ["Type=Application", "Exec=kitokey", "TryExec=kitokey"]; // gitleaks:allow: "kitokey" holds "key", no secret
    for line in wanted {
        assert!(lines.contains(&line), "{line} in {text}");
    }
    assert!(!text.contains("--window") && !text.contains("Hidden=true"), "{text}");
    autostart::set(&entry, false).unwrap();
    assert!(!entry.exists() && !autostart::on(&entry));
    // Off twice is off.
    autostart::set(&entry, false).unwrap();
}

#[test]
fn what_the_desktops_tools_turn_off_is_off() {
    let folder = tempfile::tempdir().unwrap();
    let entry = folder.path().join(autostart::NAME);
    for off in ["Hidden=true", "X-GNOME-Autostart-enabled=false", "Hidden = true"] {
        fs::write(&entry, format!("[Desktop Entry]\nType=Application\nExec=kitokey\n{off}\n")).unwrap();
        assert!(!autostart::on(&entry), "{off}");
    }
    fs::write(
        &entry,
        "[Desktop Entry]\nType=Application\nExec=kitokey\nHidden=false\nX-GNOME-Autostart-enabled=true\n",
    )
    .unwrap();
    assert!(autostart::on(&entry));
    // A key of another group is not the entry's.
    fs::write(&entry, "[Desktop Entry]\nExec=kitokey\n[Desktop Action other]\nHidden=true\n").unwrap();
    assert!(autostart::on(&entry));
}

#[test]
fn the_first_start_writes_it_once_and_never_again_once_turned_off() {
    let folder = tempfile::tempdir().unwrap();
    let entry = folder.path().join("autostart").join(autostart::NAME);
    let file = folder.path().join("kitokey").join("settings.toml");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "# mine\nreminder = false\n").unwrap();
    assert!(autostart::first(&entry, &file).unwrap());
    assert!(autostart::on(&entry));
    let written = fs::read_to_string(&file).unwrap();
    assert!(
        written.contains("# mine") && written.contains("reminder = false") && written.contains("autostart = true"),
        "{written}"
    );
    assert!(!autostart::first(&entry, &file).unwrap());
    // Turned off — by Settings or by the desktop's tools: the next starts leave it off.
    autostart::set(&entry, false).unwrap();
    assert!(!autostart::first(&entry, &file).unwrap() && !entry.exists());
    // No settings file yet is a first start too.
    let fresh = folder.path().join("fresh").join("settings.toml");
    let other = folder.path().join("other").join(autostart::NAME);
    assert!(autostart::first(&other, &fresh).unwrap() && autostart::on(&other));
    assert_eq!(settings::read(&fresh).unwrap().autostart, Some(true));
    // A settings file that cannot be read is never written over, and nothing starts by itself.
    let broken = folder.path().join("broken.toml");
    fs::write(&broken, "reminder = maybe\n").unwrap();
    let third = folder.path().join("third").join(autostart::NAME);
    assert!(autostart::first(&third, &broken).is_err() && !third.exists());
    assert_eq!(fs::read_to_string(&broken).unwrap(), "reminder = maybe\n");
}

#[test]
fn the_settings_file_keeps_whether_it_was_written() {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("settings.toml");
    assert_eq!(settings::read(&file).unwrap().autostart, None);
    fs::write(&file, "autostart = false\n").unwrap();
    assert_eq!(settings::read(&file).unwrap().autostart, Some(false));
    let mut changed = settings::read(&file).unwrap();
    changed.autostart = Some(true);
    settings::write(&file, &changed).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "autostart = true\n");
}

#[test]
fn wholeheards_entry_becomes_kitokeys_and_an_entry_turned_off_stays_off() {
    let folder = tempfile::tempdir().unwrap();
    let before = folder.path().join(autostart::NAME_BEFORE);
    let entry = folder.path().join(autostart::NAME);
    assert_eq!(autostart::NAME_BEFORE, "dev.wholeheard.Wholeheard.desktop");
    // On under Wholeheard: on under Kitokey, Wholeheard's gone — its program is, once the package is replaced.
    fs::write(&before, "[Desktop Entry]\nType=Application\nExec=wholeheard\nTryExec=wholeheard\n").unwrap();
    assert!(autostart::carry_over(&before, &entry).unwrap());
    assert!(autostart::on(&entry) && !before.exists());
    assert!(fs::read_to_string(&entry).unwrap().contains("Exec=kitokey"));
    // Nothing left to carry: nothing done, Kitokey's own kept.
    assert!(!autostart::carry_over(&before, &entry).unwrap() && autostart::on(&entry));
    // Turned off under Wholeheard: off under Kitokey too.
    fs::remove_file(&entry).unwrap();
    fs::write(&before, "[Desktop Entry]\nType=Application\nExec=wholeheard\nHidden=true\n").unwrap();
    assert!(autostart::carry_over(&before, &entry).unwrap());
    assert!(!before.exists() && !autostart::on(&entry));
}
