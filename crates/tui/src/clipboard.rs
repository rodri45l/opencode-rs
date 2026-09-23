//! Clipboard copy-command selection.
//!
//! Port of packages/tui/src/clipboard.ts `copyCommand` behaviour (upstream 18ef3cc).

/// Pick the native clipboard command for a platform, or `None` if unavailable.
pub fn copy_command(os: &str, wayland: bool, has: impl Fn(&str) -> bool) -> Option<Vec<String>> {
    if os == "darwin" && has("osascript") {
        return Some(vec!["osascript".to_string()]);
    }
    if os == "linux" && wayland && has("wl-copy") {
        return Some(vec!["wl-copy".to_string()]);
    }
    if os == "linux" && has("xclip") {
        return Some(vec![
            "xclip".to_string(),
            "-selection".to_string(),
            "clipboard".to_string(),
        ]);
    }
    if os == "linux" && has("xsel") {
        return Some(vec![
            "xsel".to_string(),
            "--clipboard".to_string(),
            "--input".to_string(),
        ]);
    }
    None
}
