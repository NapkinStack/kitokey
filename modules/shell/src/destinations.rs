//! Where a meeting's summary can be sent (D110, PDR-0007): a destination the user set up in Settings
//! — its kind, as `posting/v1` names it, and a name they read, such as « #équipe ». Its address is a
//! secret, whoever holds it posts to the channel: it is kept in the system's keyring under Kitokey's
//! name and the destination's, never in the settings' file nor in a log, and once kept it is shown
//! only as its host and the end of its path.

use keyring_core::{Entry, Error};

/// Kitokey's keys and destinations are filed under the same service; a destination's account says
/// it is one, so that no provider's host is ever read as a destination, nor the other way round.
const SERVICE: &str = "com.napkinstack.Kitokey";

/// What the address is, and so what it is sent (`posting/v1`'s `--kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Webhook,
    Discord,
    Slack,
}

impl Kind {
    pub fn named(word: &str) -> Option<Self> {
        match word {
            "webhook" => Some(Self::Webhook),
            "discord" => Some(Self::Discord),
            "slack" => Some(Self::Slack),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Webhook => "webhook",
            Self::Discord => "discord",
            Self::Slack => "slack",
        }
    }
}

/// A destination as the settings keep it: never its address.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Destination {
    pub name: String,
    pub kind: Kind,
    /// Its address as it is shown: `shown`'s.
    pub shown: String,
}

/// A destination's name as the user typed it: one line, trimmed, 1 to 40 characters.
pub fn name(typed: &str) -> Option<String> {
    let name = typed.trim();
    (!name.is_empty() && !name.contains(['\n', '\r']) && name.chars().count() <= 40).then(|| name.to_string())
}

/// An address as it is shown once kept: its host, then the end of its path — the last four
/// characters of its last segment, or the whole segment when it is that short —, never its query
/// nor its fragment. Nothing that is no address shows anything of itself.
pub fn shown(address: &str) -> String {
    let Some((_, rest)) = address.split_once("://") else {
        return String::new();
    };
    let rest = rest.split(['?', '#']).next().unwrap_or_default();
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    let host = host.rsplit('@').next().unwrap_or_default();
    let last = path.trim_end_matches('/').rsplit('/').next().unwrap_or_default();
    let count = last.chars().count();
    match count {
        0 => host.to_string(),
        1..=4 => format!("{host}/…/{last}"),
        _ => format!("{host}/…{}", last.chars().skip(count - 4).collect::<String>()),
    }
}

fn entry(name: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &format!("destination:{name}")).map_err(|error| error.to_string())
}

/// Keeps a destination's address, replacing the one kept under that name.
pub fn keep(name: &str, address: &str) -> Result<(), String> {
    entry(name)?.set_password(address).map_err(|error| error.to_string())
}

/// The address kept for a destination; none when none is.
pub fn address(name: &str) -> Result<Option<String>, String> {
    match entry(name)?.get_password() {
        Ok(address) => Ok(Some(address)),
        Err(Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

/// Forgets a destination's address; nothing kept is nothing to forget.
pub fn forget(name: &str) -> Result<(), String> {
    match entry(name)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Why a destination was not added.
#[derive(Debug, PartialEq, Eq)]
pub enum NotAdded {
    /// A destination already has its name.
    Named,
    /// The keyring refused the address.
    Keyring(String),
}

/// Adds a destination to those kept, its address to the keyring. Refused, the keyring untouched,
/// when a destination already has its name: never written over another's address (#1129).
pub fn add(kept: &mut Vec<Destination>, destination: Destination, address: &str) -> Result<(), NotAdded> {
    if kept.iter().any(|one| one.name == destination.name) {
        return Err(NotAdded::Named);
    }
    keep(&destination.name, address).map_err(NotAdded::Keyring)?;
    kept.push(destination);
    Ok(())
}

/// Forgets a destination: its address out of the keyring first, then out of those kept. The
/// keyring refusing, the destination stays listed, to be removed again: no secret left unseen (#1129).
pub fn remove(kept: &mut Vec<Destination>, name: &str) -> Result<(), String> {
    forget(name)?;
    kept.retain(|one| one.name != name);
    Ok(())
}
