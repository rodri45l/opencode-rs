//! Port of packages/core/test/tool-apply-patch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool is registered as `apply_patch`, parses a
//! `*** Begin Patch` document into add/update/delete operations, applies them
//! sequentially, summarizes a batch as `Applied patch sequentially:` followed by
//! `A`/`M`/`D` lines, rejects moves before applying any hunk, and reports
//! `Unable to apply patch at <path>` for a failing hunk. Re-derived: the
//! `ToolRegistry`/`FileMutation`/`Permission` wiring, external-directory
//! approval, deferred remove blocking, interruption, and filesystem effects are
//! dropped; the pure parse/format decisions remain.

use opencode_core::tool_apply_patch::{ApplyPatchTool, ParsedPatch, PatchOperation};

const NOTE: &str = "porting: apply-patch tool not implemented";

const PATCH: &str = "*** Begin Patch\n*** Add File: nested/new.txt\n+created\n*** Update File: update.txt\n@@\n-before\n+after\n*** Delete File: remove.txt\n*** End Patch";

#[test]
#[ignore = "porting: apply-patch tool not implemented"]
fn registers_as_apply_patch() {
    assert_eq!(ApplyPatchTool::NAME, "apply_patch");
}

#[test]
#[ignore = "porting: apply-patch tool not implemented"]
fn parses_add_update_and_delete_hunks_in_order() {
    let parsed = ApplyPatchTool::parse(PATCH).expect(NOTE);
    assert_eq!(
        parsed.operations,
        vec![
            PatchOperation::Add {
                path: "nested/new.txt".into()
            },
            PatchOperation::Update {
                path: "update.txt".into()
            },
            PatchOperation::Delete {
                path: "remove.txt".into()
            },
        ]
    );
}

#[test]
#[ignore = "porting: apply-patch tool not implemented"]
fn summarizes_an_applied_batch_with_lettered_lines() {
    let parsed = ParsedPatch {
        operations: vec![
            PatchOperation::Add {
                path: "nested/new.txt".into(),
            },
            PatchOperation::Update {
                path: "update.txt".into(),
            },
            PatchOperation::Delete {
                path: "remove.txt".into(),
            },
        ],
    };
    assert_eq!(
        ApplyPatchTool::summary(&parsed).expect(NOTE),
        "Applied patch sequentially:\nA nested/new.txt\nM update.txt\nD remove.txt"
    );
}

#[test]
#[ignore = "porting: apply-patch tool not implemented"]
fn rejects_move_hunks_before_applying_anything() {
    let error = ApplyPatchTool::parse(
        "*** Begin Patch\n*** Add File: created.txt\n+created\n*** Update File: old.txt\n*** Move to: moved.txt\n@@\n-before\n+after\n*** End Patch",
    )
    .expect_err(NOTE);
    assert_eq!(error.to_string(), ApplyPatchTool::MOVE_ERROR);
}

#[test]
#[ignore = "porting: apply-patch tool not implemented"]
fn reports_a_failure_for_the_requested_hunk() {
    assert_eq!(
        ApplyPatchTool::failure_message("missing.txt").expect(NOTE),
        "Unable to apply patch at missing.txt"
    );
}
