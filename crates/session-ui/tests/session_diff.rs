//! Port of packages/session-ui/src/components/session-diff.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/session-diff.ts; see docs/TEST-PORT.md.

use opencode_session_ui::session_diff::{normalize, resolve_file_diff, text, DiffSide, DiffSource};

fn source(
    file: &str,
    patch: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
) -> DiffSource {
    DiffSource {
        file: file.to_string(),
        patch: patch.map(str::to_string),
        before: before.map(str::to_string),
        after: after.map(str::to_string),
        additions: 1,
        deletions: 1,
        status: Some("modified".to_string()),
    }
}

#[test]
fn renders_whole_file_unified_patches_as_complete_diffs() {
    let diff = source(
        "a.ts",
        Some("Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n@@ -1,2 +1,2 @@\n one\n-two\n+three\n"),
        None,
        None,
    );
    let view = normalize(&diff);

    assert_eq!(view.file_diff.name, "a.ts");
    assert!(!view.file_diff.is_partial);
    assert_eq!(text(&view, DiffSide::Deletions), "one\ntwo\n");
    assert_eq!(text(&view, DiffSide::Additions), "one\nthree\n");
}

#[test]
fn keeps_missing_final_newlines_from_unified_patches() {
    let diff = source(
        "a.ts",
        Some("Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n@@ -1,2 +1,2 @@\n one\n-two\n\\ No newline at end of file\n+three\n\\ No newline at end of file\n"),
        None,
        None,
    );
    let view = normalize(&diff);

    assert_eq!(text(&view, DiffSide::Deletions), "one\ntwo");
    assert_eq!(text(&view, DiffSide::Additions), "one\nthree");
}

#[test]
fn renders_whole_file_vcs_patches_as_complete_diffs() {
    let file_diff = resolve_file_diff(&source(
        "a.ts",
        Some("diff --git a/a.ts b/a.ts\nindex 1a2b3c4..5d6e7f8 100644\n--- a/a.ts\n+++ b/a.ts\n@@ -1,2 +1,2 @@\n one\n-old\n+new\n"),
        None,
        None,
    ));

    assert!(!file_diff.is_partial);
    assert_eq!(file_diff.addition_lines, vec!["one\n", "new\n"]);
}

#[test]
fn keeps_ordinary_leading_tool_patches_partial() {
    let file_diff = resolve_file_diff(&source(
        "a.ts",
        Some("Index: a.ts\n===================================================================\n--- a.ts\n+++ a.ts\n@@ -1,5 +1,5 @@\n-old\n+new\n two\n three\n four\n five\n"),
        None,
        None,
    ));

    assert!(file_diff.is_partial);
    assert_eq!(
        file_diff.addition_lines,
        vec!["new\n", "two\n", "three\n", "four\n", "five\n"]
    );
}

#[test]
fn keeps_separated_patch_hunks_partial_without_complete_file_contents() {
    let file_diff = resolve_file_diff(&source(
        "project.ts",
        Some("Index: project.ts\n===================================================================\n--- project.ts\t\n+++ project.ts\t\n@@ -1,3 +1,2 @@\n import { and } from \"drizzle-orm\"\n-import { sql } from \"drizzle-orm\"\n import { ProjectTable } from \"./project.sql\"\n@@ -346,3 +345,3 @@\n import { Database } from \"@/storage/db\"\n-import { ProjectTable } from \"./project.sql\"\n+import { ProjectTable } from \"../project/project.sql\"\n import { SessionTable } from \"../session/session.sql\"\n"),
        None,
        None,
    ));

    assert!(file_diff.is_partial);
    assert_eq!(file_diff.hunks.len(), 2);
    assert!(file_diff.hunks[1].collapsed_before > 0);
}

#[test]
fn renders_headerless_persisted_patches() {
    let view = normalize(&source(
        "a.ts",
        Some("@@ -1 +1 @@\n-old\n+new\n"),
        None,
        None,
    ));

    assert_eq!(view.file_diff.name, "a.ts");
    assert!(view.file_diff.is_partial);
    assert_eq!(text(&view, DiffSide::Deletions), "old\n");
    assert_eq!(text(&view, DiffSide::Additions), "new\n");
}

#[test]
fn does_not_share_headerless_patch_metadata_between_files() {
    let patch = "@@ -1 +1 @@\n-old\n+new\n";

    assert_eq!(
        resolve_file_diff(&source("a.ts", Some(patch), None, None)).name,
        "a.ts"
    );
    assert_eq!(
        resolve_file_diff(&source("b.ts", Some(patch), None, None)).name,
        "b.ts"
    );
}

#[test]
fn keeps_capped_header_only_patches_partial() {
    let file_diff = resolve_file_diff(&source(
        "a.ts",
        Some("Index: a.ts\n===================================================================\n--- a.ts\t\n+++ a.ts\t\n"),
        None,
        None,
    ));

    assert_eq!(file_diff.name, "a.ts");
    assert!(file_diff.is_partial);
    assert!(file_diff.hunks.is_empty());
}

#[test]
fn keeps_full_legacy_content_as_a_complete_diff() {
    let view = normalize(&source("a.ts", None, Some("one\n"), Some("two\n")));

    assert!(!view.file_diff.is_partial);
    assert_eq!(text(&view, DiffSide::Deletions), "one\n");
    assert_eq!(text(&view, DiffSide::Additions), "two\n");
}

#[test]
fn ignores_malformed_persisted_patches() {
    let view = normalize(&source(
        "a.ts",
        Some("diff --git a/a.ts b/a.ts\nindex ff4ceb2..65a1de0 100644\n--- a/a.ts\n+++ b/a.ts\n@@ -1,3 +1,3 @@\n keep\n+add\n same\r"),
        None,
        None,
    ));

    assert_eq!(text(&view, DiffSide::Deletions), "");
    assert_eq!(text(&view, DiffSide::Additions), "");
}
