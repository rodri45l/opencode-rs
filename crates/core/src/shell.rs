//! Shell name classification and argument construction.
//!
//! Ports the pure helpers of `packages/core/src/shell.ts`: normalize shell
//! names, classify login/posix families, and build the `-c` argument vector for
//! each family.

use crate::{CoreError, CoreResult};

/// Shell helpers.
#[derive(Debug, Default)]
pub struct Shell;

impl Shell {
    /// Normalize a shell path to its bare, lowercased name.
    pub fn name(_path: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented("shell::Shell::name"))
    }

    /// Whether the shell is a login shell.
    pub fn login(_path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("shell::Shell::login"))
    }

    /// Whether the shell is POSIX-compatible.
    pub fn posix(_path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("shell::Shell::posix"))
    }

    /// Build the argument vector used to run `command` in `cwd`.
    pub fn args(_shell: &str, _command: &str, _cwd: &str) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented("shell::Shell::args"))
    }
}
