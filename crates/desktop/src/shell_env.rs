//! Shell environment probing.
//!
//! Port of `packages/desktop/src/main/shell-env.ts` (upstream 18ef3cc).

use std::collections::BTreeMap;

/// Parse null-delimited `KEY=VALUE` pairs, dropping invalid entries.
pub fn parse_shell_env(bytes: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(bytes);
    let mut env = BTreeMap::new();
    for line in text.split('\0') {
        if line.is_empty() {
            continue;
        }
        if let Some(index) = line.find('=') {
            if index == 0 {
                continue;
            }
            env.insert(line[..index].to_string(), line[index + 1..].to_string());
        }
    }
    env
}

/// Merge a shell snapshot with explicit overrides (overrides win).
pub fn merge_shell_env(
    shell: &BTreeMap<String, String>,
    overrides: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = shell.clone();
    for (key, value) in overrides {
        merged.insert(key.clone(), value.clone());
    }
    merged
}

/// Resolve the user shell: `$SHELL`, then the login shell, then `/bin/sh`.
pub fn resolve_user_shell(env_shell: Option<&str>, login_shell: Option<&str>) -> String {
    let env = env_shell.filter(|value| !value.is_empty());
    let login = login_shell.filter(|value| !value.is_empty() && *value != "unknown");
    env.or(login).unwrap_or("/bin/sh").to_string()
}

/// Whether the shell binary is nushell.
pub fn is_nushell(shell: &str) -> bool {
    let name = shell
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(shell)
        .to_ascii_lowercase();
    let raw = shell.to_ascii_lowercase();
    name == "nu" || name == "nu.exe" || raw.ends_with("\\nu.exe")
}
