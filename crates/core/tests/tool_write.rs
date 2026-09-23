//! Port of packages/core/test/tool-write.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool is registered as `write` with the locked schema
//! (`path`, `content`), creates or overwrites through the mutation layer,
//! reports `Created file successfully: <resource>` or
//! `Wrote file successfully: <resource>`, resolves absolute external paths with
//! a leading `external_directory` approval before `edit`, and reports
//! `Unable to write <path>` when denied. Re-derived: the `ToolRegistry`/
//! `FileMutation`/`Location`/`Permission` service wiring, BOM preservation, and
//! the source-docstring assertions are dropped.

use opencode_core::tool_write::WriteTool;

#[test]
fn registers_with_the_locked_write_schema() {
    let mut keys = WriteTool::schema_keys().unwrap();
    keys.sort_unstable();
    assert_eq!(keys, vec!["content", "path"]);
    assert_eq!(WriteTool::NAME, "write");
}

#[test]
fn reports_created_and_wrote_messages_from_the_mutation_result() {
    assert_eq!(
        WriteTool::success_message(false, "src/new.txt").unwrap(),
        "Created file successfully: src/new.txt"
    );
    assert_eq!(
        WriteTool::success_message(true, "existing.txt").unwrap(),
        "Wrote file successfully: existing.txt"
    );
}

#[test]
fn resolves_relative_and_absolute_internal_paths() {
    let relative = WriteTool::resolve("/project", "src/new.txt").unwrap();
    assert_eq!(relative.resource, "src/new.txt");
    assert!(!relative.external);
    assert!(relative.canonical.ends_with("src/new.txt"));

    let absolute = WriteTool::resolve("/project", "/project/absolute.txt").unwrap();
    assert_eq!(absolute.resource, "absolute.txt");
    assert!(!absolute.external);
}

#[test]
fn resolves_an_external_target_with_a_leading_external_directory_approval() {
    let external = WriteTool::resolve("/project", "/outside/external.txt").unwrap();
    assert!(external.external);
    assert_eq!(
        WriteTool::permission_actions(external.external).unwrap(),
        vec!["external_directory", "edit"]
    );
    assert_eq!(WriteTool::permission_actions(false).unwrap(), vec!["edit"]);
}

#[test]
fn reports_a_failure_for_the_requested_path() {
    assert_eq!(
        WriteTool::failure_message("denied.txt").unwrap(),
        "Unable to write denied.txt"
    );
}
