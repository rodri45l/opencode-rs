//! Runtime capability adapters (port of packages/app/src/utils/runtime-adapters.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuntimeValue {
    Disposable,
    NonDisposable,
    Setter,
    Plain,
}

#[derive(Default)]
pub struct DisposableProbe {
    count: std::cell::Cell<usize>,
}

impl DisposableProbe {
    pub fn count(&self) -> usize {
        self.count.get()
    }
}

pub fn is_disposable(value: RuntimeValue) -> bool {
    matches!(value, RuntimeValue::Disposable)
}

pub fn dispose_if_disposable(value: RuntimeValue, probe: &DisposableProbe) {
    if is_disposable(value) {
        probe.count.set(probe.count.get() + 1);
    }
}

pub fn has_set_option(value: RuntimeValue) -> bool {
    matches!(value, RuntimeValue::Setter)
}

pub fn set_option_if_supported(
    value: RuntimeValue,
    key: &str,
    value_text: &str,
) -> Vec<(String, String)> {
    if has_set_option(value) {
        vec![(key.to_string(), value_text.to_string())]
    } else {
        Vec::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HoveredLink {
    pub text: Option<String>,
}

pub fn get_hovered_link_text(value: Option<HoveredLink>) -> Option<String> {
    value.and_then(|link| link.text)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpeechCtor {
    Speech,
    Webkit,
}

/// Resolve the speech recognition constructor. Webkit takes precedence; the
/// generic `SpeechRecognition` slot is only honoured when it is the webkit
/// variant (the port's placeholder for a usable constructor).
pub fn get_speech_recognition_ctor(
    speech: Option<SpeechCtor>,
    webkit: Option<SpeechCtor>,
) -> Option<SpeechCtor> {
    webkit
        .filter(|ctor| *ctor == SpeechCtor::Webkit)
        .or_else(|| speech.filter(|ctor| *ctor == SpeechCtor::Webkit))
}
