//! Port of packages/tui/test/clipboard.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/clipboard.ts; see docs/TEST-PORT.md.

use opencode_tui::clipboard::copy_command;

fn has<'a>(names: &'a [&'static str]) -> impl Fn(&str) -> bool + 'a {
    move |name: &str| names.contains(&name)
}

#[test]
fn prefers_wayland_clipboard_when_available() {
    assert_eq!(
        copy_command("linux", true, has(&["wl-copy"])),
        Some(vec!["wl-copy".to_string()])
    );
}

#[test]
fn uses_osascript_on_macos() {
    assert_eq!(
        copy_command("darwin", false, has(&["osascript"])),
        Some(vec!["osascript".to_string()])
    );
}

#[test]
fn falls_back_through_x11_clipboard_commands() {
    assert_eq!(
        copy_command("linux", true, has(&["xclip"])),
        Some(vec![
            "xclip".to_string(),
            "-selection".to_string(),
            "clipboard".to_string()
        ])
    );
    assert_eq!(
        copy_command("linux", false, has(&["xsel"])),
        Some(vec![
            "xsel".to_string(),
            "--clipboard".to_string(),
            "--input".to_string()
        ])
    );
}

#[test]
fn returns_undefined_when_native_clipboard_is_unavailable() {
    assert_eq!(copy_command("linux", false, |_| false), None);
}
