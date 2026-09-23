//! Port of packages/tui/test/prompt/traits.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/prompt/traits.ts; see docs/TEST-PORT.md.

use opencode_tui::prompt_traits::{compute_prompt_traits, Capture, PromptMode};

#[test]
fn normal_mode_without_autocomplete_only_captures_tab() {
    let traits = compute_prompt_traits(PromptMode::Normal, false);
    assert_eq!(traits.capture, Some(vec![Capture::Tab]));
    assert_eq!(traits.status, None);
}

#[test]
fn normal_mode_with_autocomplete_captures_navigation_keys() {
    let traits = compute_prompt_traits(PromptMode::Normal, true);
    assert_eq!(
        traits.capture,
        Some(vec![
            Capture::Escape,
            Capture::Navigate,
            Capture::Submit,
            Capture::Tab,
        ])
    );
    assert_eq!(traits.status, None);
}

#[test]
fn shell_mode_disables_capture_and_labels_the_prompt_without_suspending() {
    let traits = compute_prompt_traits(PromptMode::Shell, false);
    assert_eq!(traits.capture, None);
    assert_eq!(traits.status.as_deref(), Some("SHELL"));
}
