//! Port of packages/app/src/components/command-tooltip-keybind.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::command_tooltip_keybind::{
    new_tab_tooltip_keybind, review_tooltip_keybind, Command,
};

#[test]
fn keeps_localized_review_shortcut_modifiers() {
    let command = Command {
        parts: vec!["Ctrl".into(), "Maj".into(), "R".into()],
    };
    assert_eq!(
        review_tooltip_keybind(&command),
        vec!["Ctrl".to_string(), "Maj".to_string(), "R".to_string()]
    );
}

#[test]
fn uses_the_configured_new_tab_shortcut() {
    let command = Command {
        parts: vec!["Alt".into(), "N".into()],
    };
    assert_eq!(
        new_tab_tooltip_keybind(&command),
        vec!["Alt".to_string(), "N".to_string()]
    );
}
