//! « Auto » (D122): which engine each job takes when Settings name none — on this computer only,
//! never a service online nor one that needs a key; and a choice Settings already hold kept as it is.

use kitokey_shell::{
    engines::{self, State},
    model, providers,
};

const OLLAMA: &str = "http://localhost:11434";

#[test]
fn auto_writes_with_the_recommended_model_ollama_holds() {
    let auto = engines::writing(OLLAMA, || Some(vec!["mistral".into(), providers::RECOMMENDED.into()]));
    assert_eq!(auto.state, State::Ready);
    assert_eq!(auto.provider, providers::ollama_address(OLLAMA));
    assert_eq!(auto.model, providers::RECOMMENDED);
}

#[test]
fn auto_writes_with_the_model_ollama_holds_when_it_holds_not_the_recommended() {
    let auto = engines::writing(OLLAMA, || Some(vec!["mistral".into()]));
    assert_eq!(auto.state, State::Ready);
    assert_eq!(auto.model, "mistral");
}

#[test]
fn auto_offers_the_recommended_model_to_download_when_ollama_holds_none() {
    let auto = engines::writing(OLLAMA, || Some(vec![]));
    assert_eq!(auto.state, State::Download);
    assert_eq!(auto.provider, providers::ollama_address(OLLAMA));
    assert_eq!(auto.model, providers::RECOMMENDED);
}

#[test]
fn auto_says_ollama_is_to_install_when_it_does_not_answer() {
    let auto = engines::writing(OLLAMA, || None);
    assert_eq!(auto.state, State::Install);
    assert!(providers::local(&auto.provider), "{}", auto.provider);
}

#[test]
fn auto_never_writes_online_even_with_ollama_elsewhere() {
    // An Ollama on another host is a service: Auto asks it nothing and takes it not.
    let auto = engines::writing("https://ollama.example.com", || panic!("asked"));
    assert_eq!(auto.state, State::Install);
    assert!(providers::local(&auto.provider), "{}", auto.provider);
}

#[test]
fn a_writer_settings_name_is_kept_even_online() {
    // No migration: what the user chose stays theirs, a service included — its agreement asked as before.
    let auto = engines::writing(OLLAMA, || Some(vec![providers::RECOMMENDED.into()]));
    let named = ("https://openrouter.ai/api/v1".to_string(), "some/model".to_string());
    assert_eq!(engines::writer(Some(named.clone()), &auto), Some(named));
}

#[test]
fn none_named_is_auto_when_it_is_ready() {
    let auto = engines::writing(OLLAMA, || Some(vec![providers::RECOMMENDED.into()]));
    assert_eq!(
        engines::writer(None, &auto),
        Some((providers::ollama_address(OLLAMA), providers::RECOMMENDED.to_string()))
    );
}

#[test]
fn none_named_and_auto_not_ready_writes_with_no_one() {
    // Nothing downloaded nor installed behind the user's back: they are asked, as before D122.
    for held in [None, Some(vec![])] {
        let auto = engines::writing(OLLAMA, || held);
        assert_eq!(engines::writer(None, &auto), None);
    }
}

#[test]
fn auto_hears_with_the_default_whisper_model_once_it_is_here() {
    let auto = engines::hearing(&[model::file(&model::DEFAULT).to_string()]);
    assert_eq!(auto.state, State::Ready);
    assert_eq!(auto.model, model::file(&model::DEFAULT));
    assert_eq!(auto.bytes, model::DEFAULT.bytes);
}

#[test]
fn auto_offers_the_default_whisper_model_to_download_with_its_size() {
    let auto = engines::hearing(&["ggml-small.bin".to_string()]);
    assert_eq!(auto.state, State::Download);
    assert_eq!(auto.model, model::file(&model::DEFAULT));
    assert_eq!(auto.bytes, 874_188_075);
}
