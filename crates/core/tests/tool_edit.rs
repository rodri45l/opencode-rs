//! Port of packages/core/test/tool-edit.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool is registered as `edit` with the locked schema
//! (`path`, `oldString`, `newString`, `replaceAll`), rejects no-op, empty,
//! missing, and ambiguous exact replacements with fixed messages, replaces one
//! or every exact occurrence, reports the replacement count, and returns
//! `Unable to edit <path>` when denied. Re-derived: the `ToolRegistry`/
//! `FileMutation`/`Permission` service wiring, BOM/CRLF preservation,
//! conditional-commit races, and the source-docstring assertions are dropped.

use opencode_core::tool_edit::EditTool;

const NOTE: &str = "porting: edit tool not implemented";

#[test]
#[ignore = "porting: edit tool not implemented"]
fn registers_with_the_locked_edit_schema() {
    let mut keys = EditTool::schema_keys().expect(NOTE);
    keys.sort_unstable();
    assert_eq!(keys, vec!["newString", "oldString", "path", "replaceAll"]);
    assert_eq!(EditTool::NAME, "edit");
}

#[test]
#[ignore = "porting: edit tool not implemented"]
fn replaces_a_single_exact_occurrence() {
    let outcome = EditTool::apply("before\nrest\n", "before", "after", false).expect(NOTE);
    assert_eq!(outcome.replacements, 1);
    assert_eq!(outcome.content, "after\nrest\n");
}

#[test]
#[ignore = "porting: edit tool not implemented"]
fn replaces_every_exact_occurrence_when_replace_all_is_true() {
    let outcome = EditTool::apply("same same same", "same", "after", true).expect(NOTE);
    assert_eq!(outcome.replacements, 3);
    assert_eq!(outcome.content, "after after after");
}

#[test]
#[ignore = "porting: edit tool not implemented"]
fn rejects_noop_and_empty_replacements() {
    assert_eq!(
        EditTool::apply("same same", "same", "same", false)
            .expect_err(NOTE)
            .to_string(),
        "No changes to apply: oldString and newString are identical."
    );
    assert_eq!(
        EditTool::apply("same same", "", "after", false)
            .expect_err(NOTE)
            .to_string(),
        "oldString must not be empty. Use write to create or overwrite a file."
    );
}

#[test]
#[ignore = "porting: edit tool not implemented"]
fn rejects_missing_and_ambiguous_exact_replacements() {
    assert_eq!(
        EditTool::apply("same same", "missing", "after", false)
            .expect_err(NOTE)
            .to_string(),
        "Could not find oldString in the file. It must match exactly, including whitespace and indentation."
    );
    assert_eq!(
        EditTool::apply("same same", "same", "after", false)
            .expect_err(NOTE)
            .to_string(),
        "Found multiple exact matches for oldString. Provide more surrounding context or set replaceAll to true."
    );
}

#[test]
#[ignore = "porting: edit tool not implemented"]
fn reports_a_failure_for_the_requested_path() {
    assert_eq!(
        EditTool::failure_message("secret.txt").expect(NOTE),
        "Unable to edit secret.txt"
    );
}
