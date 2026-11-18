//! A model written from the home, `~/…`, in Summit's settings: carried over, it is named where it
//! went. Its own test binary: it points the home at a folder of its own, and the environment is the
//! process's — every test of `migrate.rs` reads the home, side by side with the others. Windows reads
//! the home from its known folder, never from the environment: there the folder is made inside the
//! person's home, and removed when the test ends.

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
    #[cfg(not(windows))]
    let (_home, data) = {
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
        // Summit's data folder under the home.
        let data = home.path().join(".local/share");
        (home, data)
    };
    #[cfg(windows)]
    let (_home, data) = {
        let home = tempfile::tempdir_in(dirs::home_dir().unwrap()).unwrap();
        let data = home.path().to_path_buf();
        (home, data)
    };
    let root = tempfile::tempdir().unwrap();
    // As the user wrote it: `~/…`.
    let written = format!(
        "~/{}/summit/models/a.bin",
        data.strip_prefix(dirs::home_dir().unwrap()).unwrap().display()
    );
    let here = Places {
        config: root.path().join("config"),
        data: data.clone(),
        documents: root.path().join("Documents"),
    };
    write(&here.data.join("summit/models/a.bin"), "a model");
    write(&here.config.join("summit/settings.toml"), &format!("model = '{written}'\n"));

    let carried = carry_over(&here).unwrap();

    assert!(carried.model_named, "{carried:?}");
    assert_eq!(
        settings::read(&here.config.join("kitokey/settings.toml")).unwrap().model,
        here.data.join("kitokey/models/a.bin")
    );
}
