//! A model written from the home, `~/…`, in Summit's settings: carried over, it is named where it
//! went. Its own test binary: it points the home at a folder of its own, and the environment is the
//! process's — every test of `migrate.rs` reads the home, side by side with the others.

use std::fs;
use std::path::Path;

use kitokey_shell::migrate::{Places, carry_over};
use kitokey_shell::settings;

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn a_model_written_from_home_is_named_where_it_went() {
    // No test reads or writes the person's home: it is a folder of the test's own.
    let home = tempfile::tempdir().unwrap();
    // SAFETY: the only test of this binary, and nothing else runs in it.
    unsafe {
        std::env::set_var("HOME", home.path());
        for xdg in ["XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME"] {
            std::env::set_var(xdg, home.path().join(xdg));
        }
    }
    assert_eq!(dirs::home_dir().as_deref(), Some(home.path()));
    let root = tempfile::tempdir().unwrap();
    // Summit's data folder under the home, as the user wrote it: `~/…`.
    let data = home.path().join(".local/share");
    let here = Places {
        config: root.path().join("config"),
        data: data.clone(),
        documents: root.path().join("Documents"),
    };
    write(&here.data.join("summit/models/a.bin"), "a model");
    write(
        &here.config.join("summit/settings.toml"),
        "model = '~/.local/share/summit/models/a.bin'\n",
    );

    let carried = carry_over(&here).unwrap();

    assert!(carried.model_named, "{carried:?}");
    assert_eq!(
        settings::read(&here.config.join("kitokey/settings.toml")).unwrap().model,
        here.data.join("kitokey/models/a.bin")
    );
}
