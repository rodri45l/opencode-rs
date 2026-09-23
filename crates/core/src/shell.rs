//! Shell name classification and argument construction.
//!
//! Ports the pure helpers of `packages/core/src/shell.ts`: normalize shell
//! names, classify login/posix families, and build the `-c` argument vector for
//! each family.

use crate::CoreResult;

/// Shell helpers.
#[derive(Debug, Default)]
pub struct Shell;

fn meta(name: &str) -> Option<(bool, bool, bool)> {
    // (login, posix, powershell)
    match name {
        "bash" | "dash" | "ksh" | "sh" | "zsh" => Some((true, true, false)),
        "fish" => Some((true, false, false)),
        "powershell" | "pwsh" => Some((false, false, true)),
        "cmd" => Some((false, false, false)),
        _ => None,
    }
}

impl Shell {
    /// Normalize a shell path to its bare, lowercased name.
    pub fn name(path: &str) -> CoreResult<String> {
        let basename = path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(path)
            .to_ascii_lowercase();
        let basename = basename.strip_suffix(".exe").unwrap_or(&basename);
        Ok(basename.to_string())
    }

    /// Whether the shell is a login shell.
    pub fn login(path: &str) -> CoreResult<bool> {
        Ok(meta(&Self::name(path)?)
            .map(|value| value.0)
            .unwrap_or(false))
    }

    /// Whether the shell is POSIX-compatible.
    pub fn posix(path: &str) -> CoreResult<bool> {
        Ok(meta(&Self::name(path)?)
            .map(|value| value.1)
            .unwrap_or(false))
    }

    /// Build the argument vector used to run `command` in `cwd`.
    pub fn args(shell: &str, command: &str, cwd: &str) -> CoreResult<Vec<String>> {
        let name = Self::name(shell)?;
        let args = match name.as_str() {
            "nu" | "fish" => vec!["-c".to_string(), command.to_string()],
            "zsh" => vec![
                "-l".to_string(),
                "-c".to_string(),
                format!(
                    "[[ -f ~/.zshenv ]] && source ~/.zshenv >/dev/null 2>&1 || true\ncd -- \"$1\"\neval {}\n",
                    quote(command)
                ),
                "opencode".to_string(),
                cwd.to_string(),
            ],
            "bash" => vec![
                "-l".to_string(),
                "-c".to_string(),
                format!(
                    "shopt -s expand_aliases\ncd -- \"$1\"\neval {}\n",
                    quote(command)
                ),
                "opencode".to_string(),
                cwd.to_string(),
            ],
            "cmd" => vec!["/c".to_string(), command.to_string()],
            _ if meta(&name).map(|value| value.2).unwrap_or(false) => {
                vec!["-NoProfile".to_string(), "-Command".to_string(), command.to_string()]
            }
            _ => vec!["-c".to_string(), command.to_string()],
        };
        Ok(args)
    }
}

fn quote(command: &str) -> String {
    serde_json::to_string(command).unwrap_or_else(|_| format!("\"{command}\""))
}
