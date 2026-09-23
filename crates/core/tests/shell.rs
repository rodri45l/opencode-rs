//! Port of packages/core/test/shell.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: shell name normalization, login/posix classification, and
//! per-family argument construction. Dropped (re-derived): the environment- and
//! platform-dependent `preferred`/`acceptable` resolution cases and all
//! Windows-only branches.

use opencode_core::shell::Shell;

const NOTE: &str = "porting: shell not implemented";

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
#[ignore = "porting: shell not implemented"]
fn normalizes_shell_names() {
    assert_eq!(Shell::name("/bin/bash").expect(NOTE), "bash");
}

#[test]
#[ignore = "porting: shell not implemented"]
fn detects_login_shells() {
    assert!(Shell::login("/bin/bash").expect(NOTE));
    assert!(!Shell::login("C:/tools/pwsh.exe").expect(NOTE));
}

#[test]
#[ignore = "porting: shell not implemented"]
fn detects_posix_shells() {
    assert!(Shell::posix("/bin/bash").expect(NOTE));
    assert!(!Shell::posix("/bin/fish").expect(NOTE));
    assert!(!Shell::posix("C:/tools/pwsh.exe").expect(NOTE));
}

#[test]
#[ignore = "porting: shell not implemented"]
fn builds_command_args_per_shell_family() {
    assert_eq!(
        Shell::args("/bin/sh", "echo hi", "/tmp").expect(NOTE),
        strings(&["-c", "echo hi"])
    );
    assert_eq!(
        Shell::args("/usr/bin/fish", "echo hi", "/tmp").expect(NOTE),
        strings(&["-c", "echo hi"])
    );

    let zsh = Shell::args("/bin/zsh", "echo hi", "/tmp").expect(NOTE);
    assert_eq!(zsh.first().map(String::as_str), Some("-l"));
    assert_eq!(zsh.get(1).map(String::as_str), Some("-c"));
    assert_eq!(zsh.last().map(String::as_str), Some("/tmp"));
}
