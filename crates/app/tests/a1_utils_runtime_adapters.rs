//! Port of packages/app/src/utils/runtime-adapters.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::runtime_adapters::{
    dispose_if_disposable, get_hovered_link_text, get_speech_recognition_ctor, has_set_option,
    is_disposable, set_option_if_supported, DisposableProbe, HoveredLink, RuntimeValue, SpeechCtor,
};

#[test]
fn detects_and_disposes_disposable_values() {
    let probe = DisposableProbe::default();
    assert!(is_disposable(RuntimeValue::Disposable));
    dispose_if_disposable(RuntimeValue::Disposable, &probe);
    assert_eq!(probe.count(), 1);
}

#[test]
fn ignores_non_disposable_values() {
    assert!(!is_disposable(RuntimeValue::NonDisposable));
    let probe = DisposableProbe::default();
    dispose_if_disposable(RuntimeValue::NonDisposable, &probe);
    assert_eq!(probe.count(), 0);
}

#[test]
fn sets_options_only_when_setter_exists() {
    assert!(has_set_option(RuntimeValue::Setter));
    assert_eq!(
        set_option_if_supported(RuntimeValue::Setter, "fontFamily", "Berkeley Mono"),
        vec![("fontFamily".to_string(), "Berkeley Mono".to_string())]
    );
    assert!(set_option_if_supported(RuntimeValue::Plain, "fontFamily", "Berkeley Mono").is_empty());
}

#[test]
fn reads_hovered_link_text_safely() {
    assert_eq!(
        get_hovered_link_text(Some(HoveredLink {
            text: Some("https://example.com".into())
        })),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        get_hovered_link_text(Some(HoveredLink { text: None })),
        None
    );
    assert_eq!(get_hovered_link_text(None), None);
}

#[test]
fn resolves_speech_recognition_constructor_with_webkit_precedence() {
    assert_eq!(
        get_speech_recognition_ctor(Some(SpeechCtor::Speech), Some(SpeechCtor::Webkit)),
        Some(SpeechCtor::Webkit)
    );
}

#[test]
fn returns_undefined_when_no_valid_speech_constructor_exists() {
    assert_eq!(
        get_speech_recognition_ctor(Some(SpeechCtor::Speech), None),
        None
    );
    assert_eq!(get_speech_recognition_ctor(None, None), None);
}
