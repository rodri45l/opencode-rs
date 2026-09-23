//! Port of packages/app/src/context/command-keybind.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::keybind::{format_keybind, match_keybind, parse_keybind, KeyEvent};

fn event(key: &str, ctrl: bool, meta: bool, shift: bool, alt: bool) -> KeyEvent {
    KeyEvent {
        key: key.into(),
        ctrl,
        meta,
        shift,
        alt,
    }
}

#[test]
fn parse_keybind_handles_aliases_and_multiple_combos() {
    let keybinds = parse_keybind("control+option+k, mod+shift+comma");
    assert_eq!(keybinds.len(), 2);
    assert_eq!(
        keybinds[0],
        opencode_app::keybind::Keybind {
            key: "k".into(),
            ctrl: true,
            meta: false,
            shift: false,
            alt: true
        }
    );
    assert!(keybinds[1].shift);
    assert_eq!(keybinds[1].key, "comma");
    assert!(keybinds[1].ctrl || keybinds[1].meta);
}

#[test]
fn parse_keybind_treats_none_and_empty_as_disabled() {
    assert!(parse_keybind("none").is_empty());
    assert!(parse_keybind("").is_empty());
}

#[test]
fn match_keybind_normalizes_punctuation_keys() {
    let keybinds = parse_keybind("ctrl+comma, shift+plus, meta+space");
    assert!(match_keybind(
        &keybinds,
        &event(",", true, false, false, false)
    ));
    assert!(match_keybind(
        &keybinds,
        &event("+", false, false, true, false)
    ));
    assert!(match_keybind(
        &keybinds,
        &event(" ", false, true, false, false)
    ));
    assert!(!match_keybind(
        &keybinds,
        &event(",", true, false, false, true)
    ));
}

#[test]
fn match_keybind_supports_bracket_keys() {
    let keybinds = parse_keybind("mod+alt+[, mod+alt+]");
    let prev = keybinds[0].clone();
    let next = keybinds[1].clone();
    assert!(match_keybind(
        &keybinds,
        &event("[", prev.ctrl, prev.meta, false, true)
    ));
    assert!(match_keybind(
        &keybinds,
        &event("]", next.ctrl, next.meta, false, true)
    ));
}

#[test]
fn format_keybind_returns_human_readable_output() {
    let display = format_keybind("ctrl+alt+arrowup");
    assert!(display.contains('↑'));
    assert!(display.contains("Ctrl") || display.contains('⌃'));
    assert!(display.contains("Alt") || display.contains('⌥'));
    assert_eq!(format_keybind("none"), "");
}

#[test]
fn format_keybind_prefers_the_first_combo() {
    let display = format_keybind("mod+k,mod+p");
    assert!(display.contains('K') || display.contains('k'));
    assert!(!(display.contains('P') || display.contains('p')));
}
