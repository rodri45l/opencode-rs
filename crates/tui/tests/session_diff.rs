//! Port of packages/session-ui/src/components/session-diff.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/session-diff.ts: whole-file
//! unified/VCS patches render as complete diffs, ordinary tool patches stay partial,
//! separated hunks keep their collapse gap, headerless and legacy patches work, and
//! malformed persisted patches are ignored.
//! Re-derived: the Solid `View` wrapper is represented as a plain value.
//! Red-first: the session-diff projection is not implemented.

use opencode_tui::session_diff::{
    normalize, resolve_file_diff, text, DiffInput, FileDiffInput, NOTE,
};

fn modified(patch: &str) -> DiffInput {
    DiffInput {
        file: "a.ts".into(),
        patch: Some(patch.into()),
        before: None,
        after: None,
        additions: 1,
        deletions: 1,
        status: "modified".into(),
    }
}

fn file_diff(patch: &str) -> FileDiffInput {
    FileDiffInput {
        file: "a.ts".into(),
        patch: patch.into(),
    }
}

#[test]
fn renders_whole_file_unified_patches_as_complete_diffs() {
    let view = normalize(&modified(
        "Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n@@ -1,2 +1,2 @@\n one\n-two\n+three\n",
    ))
    .expect(NOTE);

    assert_eq!(view.file_diff.name, "a.ts");
    assert!(!view.file_diff.is_partial);
    assert_eq!(text(&view, "deletions").expect(NOTE), "one\ntwo\n");
    assert_eq!(text(&view, "additions").expect(NOTE), "one\nthree\n");
}

#[test]
fn keeps_missing_final_newlines_from_unified_patches() {
    let view = normalize(&modified(
        "Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n@@ -1,2 +1,2 @@\n one\n-two\n\\ No newline at end of file\n+three\n\\ No newline at end of file\n",
    ))
    .expect(NOTE);

    assert_eq!(text(&view, "deletions").expect(NOTE), "one\ntwo");
    assert_eq!(text(&view, "additions").expect(NOTE), "one\nthree");
}

#[test]
fn renders_whole_file_vcs_patches_as_complete_diffs() {
    let diff = resolve_file_diff(&file_diff(
        "diff --git a/a.ts b/a.ts\nindex 1a2b3c4..5d6e7f8 100644\n--- a/a.ts\n+++ b/a.ts\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n",
    ))
    .expect(NOTE);

    assert!(!diff.is_partial);
    assert_eq!(
        diff.addition_lines,
        vec!["one\n".to_string(), "new\n".to_string()]
    );
}

#[test]
fn keeps_ordinary_leading_tool_patches_partial() {
    let diff = resolve_file_diff(&file_diff(
        "Index: a.ts\n===================================================================\n--- a.ts\n+++ a.ts\n@@ -1,5 +1,5 @@\n-old\n+new\n two\n three\n four\n five\n",
    ))
    .expect(NOTE);

    assert!(diff.is_partial);
    assert_eq!(
        diff.addition_lines,
        vec![
            "new\n".to_string(),
            "two\n".to_string(),
            "three\n".to_string(),
            "four\n".to_string(),
            "five\n".to_string()
        ]
    );
}

#[test]
fn keeps_separated_patch_hunks_partial_without_complete_file_contents() {
    let diff = resolve_file_diff(&FileDiffInput {
        file: "project.ts".into(),
        patch: "Index: project.ts\n===================================================================\n--- project.ts\t\n+++ project.ts\t\n@@ -1,3 +1,2 @@\n import { and } from \"drizzle-orm\"\n-import { sql } from \"drizzle-orm\"\n import { ProjectTable } from \"./project.sql\"\n@@ -346,3 +345,3 @@\n import { Database } from \"@/storage/db\"\n-import { ProjectTable } from \"./project.sql\"\n+import { ProjectTable } from \"../project/project.sql\"\n import { SessionTable } from \"../session/session.sql\"\n".into(),
    })
    .expect(NOTE);

    assert!(diff.is_partial);
    assert_eq!(diff.hunks.len(), 2);
    assert!(diff.hunks[1].collapsed_before > 0);
}

#[test]
fn renders_headerless_persisted_patches() {
    let view = normalize(&modified("@@ -1 +1 @@\n-old\n+new\n")).expect(NOTE);

    assert_eq!(view.file_diff.name, "a.ts");
    assert!(view.file_diff.is_partial);
    assert_eq!(text(&view, "deletions").expect(NOTE), "old\n");
    assert_eq!(text(&view, "additions").expect(NOTE), "new\n");
}

#[test]
fn does_not_share_headerless_patch_metadata_between_files() {
    let patch = "@@ -1 +1 @@\n-old\n+new\n";

    assert_eq!(
        resolve_file_diff(&FileDiffInput {
            file: "a.ts".into(),
            patch: patch.into()
        })
        .expect(NOTE)
        .name,
        "a.ts"
    );
    assert_eq!(
        resolve_file_diff(&FileDiffInput {
            file: "b.ts".into(),
            patch: patch.into()
        })
        .expect(NOTE)
        .name,
        "b.ts"
    );
}

#[test]
fn keeps_capped_header_only_patches_partial() {
    let diff = resolve_file_diff(&file_diff(
        "Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n",
    ))
    .expect(NOTE);

    assert_eq!(diff.name, "a.ts");
    assert!(diff.is_partial);
    assert!(diff.hunks.is_empty());
}

#[test]
fn keeps_full_legacy_content_as_a_complete_diff() {
    let view = normalize(&DiffInput {
        file: "a.ts".into(),
        patch: None,
        before: Some("one\n".into()),
        after: Some("two\n".into()),
        additions: 1,
        deletions: 1,
        status: "modified".into(),
    })
    .expect(NOTE);

    assert!(!view.file_diff.is_partial);
    assert_eq!(text(&view, "deletions").expect(NOTE), "one\n");
    assert_eq!(text(&view, "additions").expect(NOTE), "two\n");
}

#[test]
fn ignores_malformed_persisted_patches() {
    let view = normalize(&modified(
        "diff --git a/a.ts b/a.ts\nindex ff4ceb2..65a1de0 100644\n--- a/a.ts\n+++ b/a.ts\n@@ -1,3 +1,3 @@\n keep\n+add\n same\r",
    ))
    .expect(NOTE);

    assert_eq!(text(&view, "deletions").expect(NOTE), "");
    assert_eq!(text(&view, "additions").expect(NOTE), "");
}
