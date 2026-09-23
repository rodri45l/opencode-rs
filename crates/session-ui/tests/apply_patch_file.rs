//! Port of packages/session-ui/src/components/apply-patch-file.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/apply-patch-file.ts; see docs/TEST-PORT.md.

use opencode_session_ui::apply_patch_file::{patch_files, ApplyPatchRaw};
use opencode_session_ui::session_diff::{text, DiffSide};

#[test]
fn parses_patch_metadata_from_the_server() {
    let file = patch_files(&[ApplyPatchRaw {
        file_path: Some("/tmp/a.ts".to_string()),
        relative_path: Some("a.ts".to_string()),
        kind: Some("update".to_string()),
        patch: Some(
            "Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n@@ -1,2 +1,2 @@\n one\n-two\n+three\n"
                .to_string(),
        ),
        additions: Some(1),
        deletions: Some(1),
        ..Default::default()
    }]);

    let file = file.first().expect("file");
    assert_eq!(file.view.file_diff.name, "a.ts");
    assert!(!file.view.file_diff.is_partial);
    assert_eq!(text(&file.view, DiffSide::Deletions), "one\ntwo\n");
    assert_eq!(text(&file.view, DiffSide::Additions), "one\nthree\n");
}

#[test]
fn keeps_legacy_before_and_after_payloads_working() {
    let file = patch_files(&[ApplyPatchRaw {
        file_path: Some("/tmp/a.ts".to_string()),
        relative_path: Some("a.ts".to_string()),
        kind: Some("update".to_string()),
        before: Some("one\n".to_string()),
        after: Some("two\n".to_string()),
        additions: Some(1),
        deletions: Some(1),
        ..Default::default()
    }]);

    let file = file.first().expect("file");
    assert_eq!(text(&file.view, DiffSide::Deletions), "one\n");
    assert_eq!(text(&file.view, DiffSide::Additions), "two\n");
}
