//! Port of packages/session-ui/src/components/part-default-open.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/part-default-open.ts; see docs/TEST-PORT.md.

use opencode_session_ui::part_default_open::part_default_open;
use serde_json::json;

#[test]
fn keeps_edited_files_expanded_when_enabled() {
    assert!(part_default_open(
        "edit",
        &json!({ "filediff": { "additions": 1, "deletions": 1 } }),
        false,
        true,
    ));
}

#[test]
fn collapses_deletion_only_edits_when_enabled() {
    assert!(!part_default_open(
        "edit",
        &json!({ "filediff": { "additions": 0, "deletions": 1200 } }),
        false,
        true,
    ));
}

#[test]
fn collapses_patches_containing_only_deleted_files_when_enabled() {
    assert!(!part_default_open(
        "apply_patch",
        &json!({
            "files": [
                { "filePath": "one.ts", "type": "delete" },
                { "filePath": "two.ts", "type": "delete" }
            ]
        }),
        false,
        true,
    ));
}

#[test]
fn keeps_mixed_patches_expanded_when_enabled() {
    assert!(part_default_open(
        "apply_patch",
        &json!({
            "files": [
                { "filePath": "one.ts", "type": "delete" },
                { "filePath": "two.ts", "type": "update" }
            ]
        }),
        false,
        true,
    ));
}

#[test]
fn preserves_shell_defaults() {
    assert!(part_default_open("shell", &json!({}), true, false));
}
