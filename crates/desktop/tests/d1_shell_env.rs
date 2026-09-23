//! Port of packages/desktop/src/main/shell-env.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/shell-env.ts: null-delimited
//! `KEY=VALUE` pairs are parsed (invalid entries dropped), explicit overrides
//! win over the shell snapshot, the login shell is the fallback before
//! `/bin/sh`, and nushell is detected by binary name or path on either platform.

#[allow(dead_code)]
mod shell_env {
    use std::collections::BTreeMap;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop shell-env helpers not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    pub fn parse_shell_env(_bytes: &[u8]) -> PortResult<BTreeMap<String, String>> {
        stub()
    }

    pub fn merge_shell_env(
        _shell: &BTreeMap<String, String>,
        _overrides: &BTreeMap<String, String>,
    ) -> PortResult<BTreeMap<String, String>> {
        stub()
    }

    pub fn resolve_user_shell(
        _env_shell: Option<&str>,
        _login_shell: Option<&str>,
    ) -> PortResult<String> {
        stub()
    }

    pub fn is_nushell(_path: &str) -> PortResult<bool> {
        stub()
    }
}

use std::collections::BTreeMap;

use shell_env::{is_nushell, merge_shell_env, parse_shell_env, resolve_user_shell, NOTE};

fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
#[ignore = "porting: desktop shell-env helpers not implemented"]
fn parse_shell_env_supports_null_delimited_pairs() {
    let env = parse_shell_env(b"PATH=/usr/bin:/bin\0FOO=bar=baz\0\0").expect(NOTE);
    assert_eq!(env.get("PATH").map(String::as_str), Some("/usr/bin:/bin"));
    assert_eq!(env.get("FOO").map(String::as_str), Some("bar=baz"));
}

#[test]
#[ignore = "porting: desktop shell-env helpers not implemented"]
fn parse_shell_env_ignores_invalid_entries() {
    let env = parse_shell_env(b"INVALID\0=empty\0OK=1\0").expect(NOTE);
    assert_eq!(env.len(), 1);
    assert_eq!(env.get("OK").map(String::as_str), Some("1"));
}

#[test]
#[ignore = "porting: desktop shell-env helpers not implemented"]
fn merge_shell_env_keeps_explicit_overrides() {
    let env = merge_shell_env(
        &map(&[("PATH", "/shell/path"), ("HOME", "/tmp/home")]),
        &map(&[("PATH", "/desktop/path"), ("OPENCODE_CLIENT", "desktop")]),
    )
    .expect(NOTE);

    assert_eq!(env.get("PATH").map(String::as_str), Some("/desktop/path"));
    assert_eq!(env.get("HOME").map(String::as_str), Some("/tmp/home"));
    assert_eq!(
        env.get("OPENCODE_CLIENT").map(String::as_str),
        Some("desktop")
    );
}

#[test]
#[ignore = "porting: desktop shell-env helpers not implemented"]
fn resolve_user_shell_falls_back_to_the_login_shell_before_bin_sh() {
    assert_eq!(
        resolve_user_shell(Some("/custom/env-shell"), Some("/bin/zsh")).expect(NOTE),
        "/custom/env-shell"
    );
    assert_eq!(
        resolve_user_shell(None, Some("/bin/zsh")).expect(NOTE),
        "/bin/zsh"
    );
    assert_eq!(
        resolve_user_shell(None, Some("unknown")).expect(NOTE),
        "/bin/sh"
    );
    assert_eq!(resolve_user_shell(None, None).expect(NOTE), "/bin/sh");
}

#[test]
#[ignore = "porting: desktop shell-env helpers not implemented"]
fn is_nushell_handles_path_and_binary_name() {
    assert!(is_nushell("nu").expect(NOTE));
    assert!(is_nushell("/opt/homebrew/bin/nu").expect(NOTE));
    assert!(is_nushell("C:\\Program Files\\nu.exe").expect(NOTE));
    assert!(!is_nushell("/bin/zsh").expect(NOTE));
}
