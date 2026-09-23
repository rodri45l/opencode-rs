//! Port of packages/opencode/test/patch/patch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned (pure surface only): `parsePatch` parses add/delete/update
//! and move hunks and rejects malformed text, and `maybeParseApplyPatch`
//! recognises direct/`applypatch`/bash-heredoc invocations.
//!
//! The reference file's `applyPatch` filesystem suites are tracked as skipped
//! (see PORT-STATUS.s5.json).
#![allow(dead_code)]

use opencode_server::patch;

#[test]
fn should_parse_simple_add_file_patch() {
    let patch_text = "*** Begin Patch
*** Add File: test.txt
+Hello World
*** End Patch";

    let result = patch::parse_patch(patch_text).unwrap();
    assert_eq!(result.hunks.len(), 1);
    assert_eq!(
        result.hunks[0],
        patch::Hunk::Add {
            path: "test.txt".to_string(),
            contents: "Hello World".to_string(),
        }
    );
}

#[test]
fn should_parse_delete_file_patch() {
    let patch_text = "*** Begin Patch
*** Delete File: old.txt
*** End Patch";

    let result = patch::parse_patch(patch_text).unwrap();
    assert_eq!(result.hunks.len(), 1);
    assert!(matches!(
        &result.hunks[0],
        patch::Hunk::Delete { path } if path == "old.txt"
    ));
}

#[test]
fn should_parse_patch_with_multiple_hunks() {
    let patch_text = "*** Begin Patch
*** Add File: new.txt
+This is a new file
*** Update File: existing.txt
@@
 old line
-new line
+updated line
*** End Patch";

    let result = patch::parse_patch(patch_text).unwrap();
    assert_eq!(result.hunks.len(), 2);
    assert!(matches!(result.hunks[0], patch::Hunk::Add { .. }));
    assert!(matches!(result.hunks[1], patch::Hunk::Update { .. }));
}

#[test]
fn should_parse_file_move_operation() {
    let patch_text = "*** Begin Patch
*** Update File: old-name.txt
*** Move to: new-name.txt
@@
-Old content
+New content
*** End Patch";

    let result = patch::parse_patch(patch_text).unwrap();
    assert_eq!(result.hunks.len(), 1);
    match &result.hunks[0] {
        patch::Hunk::Update { path, move_path } => {
            assert_eq!(path, "old-name.txt");
            assert_eq!(move_path.as_deref(), Some("new-name.txt"));
        }
        other => panic!("expected update hunk, got {other:?}"),
    }
}

#[test]
fn should_throw_error_for_invalid_patch_format() {
    let invalid_patch = "This is not a valid patch";
    let err = patch::parse_patch(invalid_patch).unwrap_err();
    assert!(err.contains("Invalid patch format"));
}

#[test]
fn should_parse_direct_apply_patch_command() {
    let patch_text = "*** Begin Patch
*** Add File: test.txt
+Content
*** End Patch";

    match patch::maybe_parse_apply_patch(&["apply_patch", patch_text]).unwrap() {
        patch::MaybeApplyPatch::Body { patch, hunks } => {
            assert_eq!(patch, patch_text);
            assert_eq!(hunks.len(), 1);
        }
        patch::MaybeApplyPatch::NotApplyPatch => panic!("expected apply_patch body"),
    }
}

#[test]
fn should_parse_applypatch_command() {
    let patch_text = "*** Begin Patch
*** Add File: test.txt
+Content
*** End Patch";

    assert!(matches!(
        patch::maybe_parse_apply_patch(&["applypatch", patch_text]).unwrap(),
        patch::MaybeApplyPatch::Body { .. }
    ));
}

#[test]
fn should_handle_bash_heredoc_format() {
    let script = "apply_patch <<'PATCH'
*** Begin Patch
*** Add File: test.txt
+Content
*** End Patch
PATCH";

    match patch::maybe_parse_apply_patch(&["bash", "-lc", script]).unwrap() {
        patch::MaybeApplyPatch::Body { hunks, .. } => assert_eq!(hunks.len(), 1),
        patch::MaybeApplyPatch::NotApplyPatch => panic!("expected heredoc body"),
    }
}

#[test]
fn should_return_not_apply_patch_for_non_patch_commands() {
    assert_eq!(
        patch::maybe_parse_apply_patch(&["echo", "hello"]).unwrap(),
        patch::MaybeApplyPatch::NotApplyPatch
    );
}
