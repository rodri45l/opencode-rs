//! Port of packages/app/src/utils/runtime-adapters.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RuntimeValue {
    Disposable,
    NonDisposable,
    Setter,
    Plain,
}

#[derive(Default)]
struct DisposableProbe {
    count: Cell<usize>,
}

// Local stubs (fast wave): real module lands later.
fn is_disposable(_value: RuntimeValue) -> bool {
    false
}

fn dispose_if_disposable(_value: RuntimeValue, _probe: &DisposableProbe) {}

fn has_set_option(_value: RuntimeValue) -> bool {
    false
}

fn set_option_if_supported(
    _value: RuntimeValue,
    _key: &str,
    _value_text: &str,
) -> Vec<(String, String)> {
    Vec::new()
}

#[derive(Clone, Debug, PartialEq)]
struct HoveredLink {
    text: Option<String>,
}

fn get_hovered_link_text(_value: Option<HoveredLink>) -> Option<String> {
    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SpeechCtor {
    Speech,
    Webkit,
}

fn get_speech_recognition_ctor(
    _speech: Option<SpeechCtor>,
    _webkit: Option<SpeechCtor>,
) -> Option<SpeechCtor> {
    None
}

#[test]
#[ignore = "porting: utils/runtime-adapters not implemented"]
fn detects_and_disposes_disposable_values() {
    let probe = DisposableProbe::default();
    assert!(is_disposable(RuntimeValue::Disposable));
    dispose_if_disposable(RuntimeValue::Disposable, &probe);
    assert_eq!(probe.count.get(), 1);
}

#[test]
#[ignore = "porting: utils/runtime-adapters not implemented"]
fn ignores_non_disposable_values() {
    assert!(!is_disposable(RuntimeValue::NonDisposable));
    let probe = DisposableProbe::default();
    dispose_if_disposable(RuntimeValue::NonDisposable, &probe);
    assert_eq!(probe.count.get(), 0);
}

#[test]
#[ignore = "porting: utils/runtime-adapters not implemented"]
fn sets_options_only_when_setter_exists() {
    assert!(has_set_option(RuntimeValue::Setter));
    assert_eq!(
        set_option_if_supported(RuntimeValue::Setter, "fontFamily", "Berkeley Mono"),
        vec![("fontFamily".to_string(), "Berkeley Mono".to_string())]
    );
    assert!(set_option_if_supported(RuntimeValue::Plain, "fontFamily", "Berkeley Mono").is_empty());
}

#[test]
#[ignore = "porting: utils/runtime-adapters not implemented"]
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
#[ignore = "porting: utils/runtime-adapters not implemented"]
fn resolves_speech_recognition_constructor_with_webkit_precedence() {
    assert_eq!(
        get_speech_recognition_ctor(Some(SpeechCtor::Speech), Some(SpeechCtor::Webkit)),
        Some(SpeechCtor::Webkit)
    );
}

#[test]
#[ignore = "porting: utils/runtime-adapters not implemented"]
fn returns_undefined_when_no_valid_speech_constructor_exists() {
    assert_eq!(
        get_speech_recognition_ctor(Some(SpeechCtor::Speech), None),
        None
    );
    assert_eq!(get_speech_recognition_ctor(None, None), None);
}
