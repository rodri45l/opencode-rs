//! Edit tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/edit.ts`: the tool
//! is registered as `edit`, accepts `path`, `oldString`, `newString`, and
//! `replaceAll`, rejects no-op/empty/missing/ambiguous exact replacements with
//! fixed messages, replaces one or all exact occurrences, reports replacements
//! and a diff, and returns `Unable to edit <path>` when approval is denied. The
//! `ToolRegistry`, `FileMutation`, BOM/CRLF preservation, and conditional-commit
//! wiring are dropped; the pure replacement decisions remain.

use crate::{CoreError, CoreResult};

/// The outcome of an exact replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditOutcome {
    /// Number of replacements performed.
    pub replacements: usize,
    /// The resulting content.
    pub content: String,
}

/// The edit tool.
#[derive(Debug, Default)]
pub struct EditTool;

impl EditTool {
    /// The tool name.
    pub const NAME: &'static str = "edit";

    /// The locked input schema property names.
    pub fn schema_keys() -> CoreResult<Vec<&'static str>> {
        Ok(vec!["path", "oldString", "newString", "replaceAll"])
    }

    /// Apply an exact replacement, returning the count and new content.
    pub fn apply(
        content: &str,
        old_string: &str,
        new_string: &str,
        replace_all: bool,
    ) -> CoreResult<EditOutcome> {
        if old_string == new_string {
            return Err(CoreError::Invalid(
                "No changes to apply: oldString and newString are identical.".into(),
            ));
        }
        if old_string.is_empty() {
            return Err(CoreError::Invalid(
                "oldString must not be empty. Use write to create or overwrite a file.".into(),
            ));
        }
        let occurrences = count_occurrences(content, old_string);
        if occurrences == 0 {
            return Err(CoreError::Invalid(
                "Could not find oldString in the file. It must match exactly, including whitespace and indentation."
                    .into(),
            ));
        }
        if occurrences > 1 && !replace_all {
            return Err(CoreError::Invalid(
                "Found multiple exact matches for oldString. Provide more surrounding context or set replaceAll to true."
                    .into(),
            ));
        }
        let replacements = if replace_all { occurrences } else { 1 };
        let content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };
        Ok(EditOutcome {
            replacements,
            content,
        })
    }

    /// The success message for an edit.
    pub fn success_message(resource: &str, replacements: usize, diff: &str) -> CoreResult<String> {
        Ok(format!(
            "Edited file successfully: {resource}\nReplacements: {replacements}\n```diff\n{diff}\n```"
        ))
    }

    /// The failure message when an edit is denied or fails.
    pub fn failure_message(path: &str) -> CoreResult<String> {
        Ok(format!("Unable to edit {path}"))
    }
}

fn count_occurrences(content: &str, search: &str) -> usize {
    if search.is_empty() {
        return content.len() + 1;
    }
    let mut count = 0;
    let mut offset = 0;
    while let Some(index) = content[offset..].find(search) {
        count += 1;
        offset += index + search.len();
    }
    count
}
