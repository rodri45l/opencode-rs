//! Port of packages/app/src/pages/session/v2/review-diff-kinds.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct ReviewDiff {
    file: String,
    additions: i64,
    deletions: i64,
    status: Option<String>,
    patch: Option<String>,
}

// Local stubs (fast wave): real module lands later.
fn review_diff_kinds(_diffs: &[ReviewDiff]) -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn filter_review_files(_files: &[&str], _query: &str) -> Vec<String> {
    Vec::new()
}

fn review_diff_needs_load(_diff: &ReviewDiff) -> bool {
    false
}

fn review_diff_directory(_root: &str, _file: &str) -> String {
    String::new()
}

fn diff(file: &str, additions: i64, deletions: i64, status: &str) -> ReviewDiff {
    ReviewDiff {
        file: file.into(),
        additions,
        deletions,
        status: Some(status.into()),
        patch: None,
    }
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn maps_file_and_directory_kinds() {
    let kinds = review_diff_kinds(&[
        diff("src/a.ts", 1, 0, "added"),
        diff("src/b.ts", 0, 2, "deleted"),
    ]);
    assert_eq!(kinds.get("src/a.ts"), Some(&"add".to_string()));
    assert_eq!(kinds.get("src/b.ts"), Some(&"del".to_string()));
    assert_eq!(kinds.get("src"), Some(&"mix".to_string()));
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn normalizes_file_and_directory_paths() {
    let kinds = review_diff_kinds(&[diff("\\src//lib/a.ts/", 1, 1, "modified")]);
    assert_eq!(kinds.get("src/lib/a.ts"), Some(&"mix".to_string()));
    assert_eq!(kinds.get("src/lib"), Some(&"mix".to_string()));
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn filters_by_path_substring() {
    let files = ["src/a.ts", "src/b.ts", "lib/c.ts"];
    assert_eq!(
        filter_review_files(&files, "b.ts"),
        vec!["src/b.ts".to_string()]
    );
    assert_eq!(
        filter_review_files(&files, ""),
        vec![
            "src/a.ts".to_string(),
            "src/b.ts".to_string(),
            "lib/c.ts".to_string()
        ]
    );
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn loads_changed_files_whose_aggregate_patch_has_no_hunks() {
    let mut review = diff("src/a.ts", 1, 0, "modified");
    review.patch = Some("diff --git a/src/a.ts b/src/a.ts\n--- a/src/a.ts\n+++ b/src/a.ts".into());
    assert!(review_diff_needs_load(&review));
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn keeps_complete_patches_and_empty_changes() {
    let mut complete = diff("src/a.ts", 1, 0, "modified");
    complete.patch = Some("@@ -0,0 +1 @@\n+value".into());
    assert!(!review_diff_needs_load(&complete));

    let empty = diff("empty.txt", 0, 0, "modified");
    assert!(!review_diff_needs_load(&empty));
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn scopes_nested_files_to_their_parent_directory() {
    assert_eq!(
        review_diff_directory("/repo", "src/lib/a.ts"),
        "/repo/src/lib"
    );
    assert_eq!(
        review_diff_directory("C:\\repo", "src/lib/a.ts"),
        "C:\\repo\\src\\lib"
    );
}

#[test]
#[ignore = "porting: pages/session/v2/review-diff-kinds not implemented"]
fn does_not_rescope_root_files() {
    assert_eq!(review_diff_directory("/repo/", "README.md"), "/repo");
    assert_eq!(review_diff_directory("/", "README.md"), "/");
    assert_eq!(review_diff_directory("C:\\", "README.md"), "C:\\");
    assert_eq!(review_diff_directory("/", "src/a.ts"), "/src");
    assert_eq!(review_diff_directory("C:\\", "src/a.ts"), "C:\\src");
}
