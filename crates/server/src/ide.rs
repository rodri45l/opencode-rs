//! IDE detection.
//!
//! Ports the observable behaviour of `packages/opencode/src/ide/index.ts`:
//! environment-driven IDE name detection and caller recognition.

/// A typed IDE error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdeError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for IdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for IdeError {}

const SUPPORTED_IDES: [&str; 5] = [
    "Windsurf",
    "Visual Studio Code - Insiders",
    "Visual Studio Code",
    "Cursor",
    "VSCodium",
];

/// Detect the running IDE from `TERM_PROGRAM` and `GIT_ASKPASS`.
pub fn ide(term_program: &str, git_askpass: &str) -> Result<String, IdeError> {
    if term_program == "vscode" {
        for name in SUPPORTED_IDES {
            if git_askpass.contains(name) {
                return Ok(name.to_string());
            }
        }
    }
    Ok("unknown".to_string())
}

/// Whether the caller is a supported VS Code variant.
pub fn already_installed(caller: &str) -> Result<bool, IdeError> {
    Ok(caller == "vscode" || caller == "vscode-insiders")
}
