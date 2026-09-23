//! Port of packages/session-ui/src/components/apply-patch-file.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/apply-patch-file.ts: server
//! patch metadata is parsed into a complete diff view, and legacy before/after payloads
//! keep working.
//! Red-first: the apply-patch file projection is not implemented.

use opencode_tui::apply_patch_file::{patch_files, text, PatchInput, NOTE};

fn server_patch(patch: &str) -> PatchInput {
    PatchInput {
        file_path: "/tmp/a.ts".into(),
        relative_path: "a.ts".into(),
        kind: "update".into(),
        patch: Some(patch.into()),
        before: None,
        after: None,
        additions: 1,
        deletions: 1,
    }
}

fn legacy_patch(before: &str, after: &str) -> PatchInput {
    PatchInput {
        file_path: "/tmp/a.ts".into(),
        relative_path: "a.ts".into(),
        kind: "update".into(),
        patch: None,
        before: Some(before.into()),
        after: Some(after.into()),
        additions: 1,
        deletions: 1,
    }
}

#[test]
fn parses_patch_metadata_from_the_server() {
    let files = patch_files(&[server_patch(
        "Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n@@ -1,2 +1,2 @@\n one\n-two\n+three\n",
    )])
    .expect(NOTE);
    let file = files.into_iter().next().expect("file");

    assert_eq!(file.view.file_diff.name, "a.ts");
    assert!(!file.view.file_diff.is_partial);
    assert_eq!(text(&file.view, "deletions").expect(NOTE), "one\ntwo\n");
    assert_eq!(text(&file.view, "additions").expect(NOTE), "one\nthree\n");
}

#[test]
fn keeps_legacy_before_and_after_payloads_working() {
    let files = patch_files(&[legacy_patch("one\n", "two\n")]).expect(NOTE);
    let file = files.into_iter().next().expect("file");

    assert_eq!(text(&file.view, "deletions").expect(NOTE), "one\n");
    assert_eq!(text(&file.view, "additions").expect(NOTE), "two\n");
}
