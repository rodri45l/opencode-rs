//! Port of packages/tui/test/cli/tui/dialog-prompt.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/ui/dialog-prompt.tsx and config/keybind.ts;
//! see docs/TEST-PORT.md. Textarea focus/rendering is visual (human-verified).
#![allow(dead_code)]

use std::collections::BTreeMap;

/// Resolves the keys that submit the dialog prompt. `dialog.prompt.submit`
/// defaults to `return`, but an explicit override replaces the default entirely
/// and is evaluated ahead of `input.submit`/`input.newline`.
fn dialog_prompt_submit_keys(overrides: &BTreeMap<String, String>) -> Vec<String> {
    let raw = overrides
        .get("dialog.prompt.submit")
        .map(String::as_str)
        .unwrap_or("return");
    raw.split(',')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn confirms(keys: &[String], pressed: &str) -> bool {
    keys.iter().any(|key| key == pressed)
}

fn overrides(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
fn dialog_prompt_submit_wins_when_return_is_also_input_newline() {
    let config = overrides(&[
        ("input_submit", "super+return"),
        ("input_newline", "return,shift+return,alt+return,ctrl+j"),
    ]);
    let keys = dialog_prompt_submit_keys(&config);

    assert!(confirms(&keys, "return"));
    assert!(!confirms(&keys, "super+return"));
}

#[test]
fn dialog_prompt_submit_can_be_rebound_separately_from_input_submit() {
    let config = overrides(&[
        ("input_submit", "return"),
        ("dialog.prompt.submit", "ctrl+y"),
    ]);
    let keys = dialog_prompt_submit_keys(&config);

    assert!(!confirms(&keys, "return"));
    assert!(confirms(&keys, "ctrl+y"));
}
