//! Bash-tool settlement model (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/bash.ts`: the tool
//! schema hides internal fields, a run settles structured `{ exit, truncated }`
//! plus content `[output, "Command exited with code N."]`, a relative `workdir`
//! resolves against the active location, an explicit external workdir is
//! approved as `external_directory` before `bash`, denial prevents execution,
//! external command arguments produce advisory warnings, and truncation and
//! timeouts are surfaced. The `ToolRegistry`/`PermissionV2`/`AppProcess`/`Config`
//! layer graph and the live shell are replaced by pure settlement helpers.

use std::path::Path;

/// Maximum bytes captured from a process.
pub const MAX_CAPTURE_BYTES: usize = 1_048_576;

/// A finished bash run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BashRunResult {
    /// Captured output.
    pub output: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Whether the output capture was truncated.
    pub output_truncated: bool,
    /// Whether the process timed out.
    pub timed_out: bool,
}

/// A settled bash result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    /// Content parts shown to the model.
    pub content: Vec<String>,
    /// Structured exit code.
    pub structured_exit: i32,
    /// Structured truncation flag.
    pub structured_truncated: bool,
    /// Structured timeout flag.
    pub structured_timeout: bool,
}

/// Advisory information about external command arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalArgs {
    /// Whether the workdir is external.
    pub workdir_external: bool,
    /// Advisory warning lines.
    pub advisories: Vec<String>,
}

/// The bash-tool settlement model.
pub struct BashTool;

impl BashTool {
    /// The schema keys exposed to the model.
    pub fn schema_keys() -> Vec<String> {
        vec!["timeout".to_string(), "workdir".to_string()]
    }

    /// Settle a finished run into content plus structured fields.
    pub fn settle(run: &BashRunResult) -> Settlement {
        let output = if run.output_truncated {
            format!("{}\n[output capture truncated]", run.output)
        } else {
            run.output.clone()
        };
        let status = if run.timed_out {
            "Command timed out.".to_string()
        } else {
            format!("Command exited with code {}.", run.exit_code)
        };
        Settlement {
            content: vec![output, status],
            structured_exit: run.exit_code,
            structured_truncated: run.output_truncated,
            structured_timeout: run.timed_out,
        }
    }

    /// Resolve a workdir against the active location.
    pub fn resolve_workdir(location: &str, workdir: Option<&str>) -> String {
        match workdir.filter(|workdir| !workdir.is_empty()) {
            Some(workdir) => Path::new(location)
                .join(workdir)
                .to_string_lossy()
                .replace('\\', "/"),
            None => location.to_string(),
        }
    }

    /// The permission actions required to run a command.
    pub fn permission_actions(workdir_external: bool, denied: Option<&str>) -> Vec<String> {
        if let Some(denied) = denied {
            return vec![denied.to_string()];
        }
        if workdir_external {
            vec!["external_directory".to_string(), "bash".to_string()]
        } else {
            vec!["bash".to_string()]
        }
    }

    /// Advisory warnings for external command arguments.
    pub fn external_advisories(command: &str, external_root: &str) -> ExternalArgs {
        let mut advisories = Vec::new();
        for token in command.split_whitespace() {
            if token.len() > external_root.len() && token.contains(external_root) {
                advisories.push(format!(
                    "Command argument {token} references an external path outside {external_root}"
                ));
            }
        }
        ExternalArgs {
            workdir_external: false,
            advisories,
        }
    }
}
