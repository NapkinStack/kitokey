//! A provider's key, kept in the system's keyring — the Secret Service under a Linux desktop, the
//! Keychain on macOS, the Credential Manager on Windows — under Kitokey's name and the provider's
//! host. Never in a file, never in what Kitokey says.

use keyring_core::{Entry, Error};

/// What Kitokey's keys are filed under, in the keyring.
const SERVICE: &str = "com.napkinstack.Kitokey";
/// What they were filed under before, the latest first: before D86, when the product was named
/// Wholeheard; before D28, Summit.
const SERVICES_BEFORE: [&str; 2] = ["dev.wholeheard.Wholeheard", "dev.summit.Summit"];

/// Opens the system's keyring for this process, once, before a key is read or kept. Refused —
/// no keyring service on this desktop — the sentence says why. With `KITOKEY_KEYRING=memory`, in
/// a build for trying Kitokey only, a keyring that lives as long as the program instead: a scenario
/// never reads nor writes the user's, and a release never keeps a key where it does not last.
pub fn open_system_keyring() -> Result<(), String> {
    if cfg!(debug_assertions) && std::env::var("KITOKEY_KEYRING").as_deref() == Ok("memory") {
        keyring_core::set_default_store(keyring_core::mock::Store::new().map_err(|error| error.to_string())?);
        return Ok(());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    let store = zbus_secret_service_keyring_store::Store::new();
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new();
    #[cfg(windows)]
    let store = windows_native_keyring_store::Store::new();
    keyring_core::set_default_store(store.map_err(|error| error.to_string())?);
    Ok(())
}

fn entry(host: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, host).map_err(|error| error.to_string())
}

/// Keeps a provider's key, replacing the one kept for that host.
pub fn keep(host: &str, key: &str) -> Result<(), String> {
    entry(host)?.set_password(key).map_err(|error| error.to_string())
}

/// The key kept for a provider's host; none when none is. One kept under an earlier name is moved
/// under Kitokey's as it is read: kept anew, then forgotten there.
pub fn key(host: &str) -> Result<Option<String>, String> {
    match entry(host)?.get_password() {
        Ok(key) => return Ok(Some(key)),
        Err(Error::NoEntry) => {}
        Err(error) => return Err(error.to_string()),
    }
    for service in SERVICES_BEFORE {
        let before = Entry::new(service, host).map_err(|error| error.to_string())?;
        match before.get_password() {
            // Kept anew, then forgotten there; kept or not, the key read is the key.
            Ok(key) => {
                if keep(host, &key).is_ok() {
                    let _ = before.delete_credential();
                }
                return Ok(Some(key));
            }
            Err(Error::NoEntry) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(None)
}

/// Forgets the key kept for a provider's host — under the earlier names too, so that it never
/// comes back; nothing kept is nothing to forget.
pub fn forget(host: &str) -> Result<(), String> {
    for service in SERVICES_BEFORE {
        if let Ok(before) = Entry::new(service, host) {
            let _ = before.delete_credential();
        }
    }
    match entry(host)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
