//! « Auto » (D122, PDR-0008): the engine each job takes when Settings name none — the default
//! Whisper model to hear, Ollama to write —, on this computer only: never a service online nor one
//! that needs a key, which stay the user's explicit choice with its agreement. Settings' `None` is
//! « Auto »; a choice they hold is kept as it is, nothing migrated.

use serde::Serialize;

use crate::{confidential, model, providers};

/// Where « Auto »'s engine stands: usable now, its model to download, or its program to install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Ready,
    Download,
    Install,
}

/// Who « Auto » writes with: Ollama's address on this computer and a model's name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Writing {
    pub provider: String,
    pub model: String,
    pub state: State,
}

/// What « Auto » hears with: the default Whisper model, by its file's name, and its size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hearing {
    pub model: String,
    pub state: State,
    pub bytes: u64,
}

/// « Auto » to write: the model a confidential meeting is written with (`confidential::writer`,
/// D60) — the recommended one Ollama at `base` holds, else the first —; Ollama holding none, the
/// recommended to download; Ollama silent or elsewhere than this computer, Ollama to install.
pub fn writing(base: &str, held: impl FnOnce() -> Option<Vec<String>>) -> Writing {
    let address = providers::ollama_address(base);
    let (provider, model, state) = match confidential::writer(None, base, held) {
        Ok((provider, model)) => (provider, model, State::Ready),
        Err(confidential::Wait::Empty) => (address, providers::RECOMMENDED.to_string(), State::Download),
        Err(confidential::Wait::Silent) => (
            providers::ollama_address(providers::OLLAMA),
            providers::RECOMMENDED.to_string(),
            State::Install,
        ),
    };
    Writing { provider, model, state }
}

/// Who writes: the one Settings (or a meeting, or a Refaire…) `named`, kept as it is; none named,
/// « Auto » when it is ready — else no one, and the user is asked, as before « Auto ».
pub fn writer(named: Option<(String, String)>, auto: &Writing) -> Option<(String, String)> {
    named.or_else(|| (auto.state == State::Ready).then(|| (auto.provider.clone(), auto.model.clone())))
}

/// « Auto » to hear: the default Whisper model (`model::DEFAULT`), ready once its file is among
/// those `listed` here, else to download at its size.
pub fn hearing(listed: &[String]) -> Hearing {
    let file = model::file(&model::DEFAULT);
    let state = if listed.iter().any(|name| name == file) {
        State::Ready
    } else {
        State::Download
    };
    Hearing {
        model: file.to_string(),
        state,
        bytes: model::DEFAULT.bytes,
    }
}
