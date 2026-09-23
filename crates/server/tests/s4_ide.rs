//! Port of packages/opencode/test/ide/ide.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/ide.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure environment-driven detection `Ide.ide` and
//! `Ide.alreadyInstalled`.
//! Dropped: none — every upstream case is pure. The reference reads `process.env`;
//! the Rust port passes the two relevant variables explicitly.

use opencode_server::ide::{already_installed, ide};

const VSCODE_ASKPASS: &str =
    "/path/to/Visual Studio Code.app/Contents/Resources/app/extensions/git/dist/askpass.sh";

#[test]
fn detects_visual_studio_code() {
    assert_eq!(ide("vscode", VSCODE_ASKPASS).unwrap(), "Visual Studio Code");
}

#[test]
fn detects_visual_studio_code_insiders() {
    assert_eq!(
        ide(
            "vscode",
            "/Applications/Visual Studio Code - Insiders.app/Contents/Resources/app/extensions/git/dist/askpass.sh"
        )
        .unwrap(),
        "Visual Studio Code - Insiders"
    );
}

#[test]
fn detects_cursor() {
    assert_eq!(
        ide(
            "vscode",
            "/path/to/Cursor.app/Contents/Resources/app/extensions/git/dist/askpass.sh"
        )
        .unwrap(),
        "Cursor"
    );
}

#[test]
fn detects_vscodium() {
    assert_eq!(
        ide(
            "vscode",
            "/path/to/VSCodium.app/Contents/Resources/app/extensions/git/dist/askpass.sh"
        )
        .unwrap(),
        "VSCodium"
    );
}

#[test]
fn detects_windsurf() {
    assert_eq!(
        ide(
            "vscode",
            "/path/to/Windsurf.app/Contents/Resources/app/extensions/git/dist/askpass.sh"
        )
        .unwrap(),
        "Windsurf"
    );
}

#[test]
fn returns_unknown_when_term_program_is_not_vscode() {
    assert_eq!(
        ide(
            "iTerm2",
            "/Applications/Visual Studio Code - Insiders.app/Contents/Resources/app/extensions/git/dist/askpass.sh"
        )
        .unwrap(),
        "unknown"
    );
}

#[test]
fn returns_unknown_when_git_askpass_does_not_contain_ide_name() {
    assert_eq!(
        ide("vscode", "/path/to/unknown/askpass.sh").unwrap(),
        "unknown"
    );
}

#[test]
fn recognizes_vscode_insiders_opencode_caller() {
    assert!(already_installed("vscode-insiders").unwrap());
}

#[test]
fn recognizes_vscode_opencode_caller() {
    assert!(already_installed("vscode").unwrap());
}

#[test]
fn returns_false_for_unknown_opencode_caller() {
    assert!(!already_installed("unknown").unwrap());
}
