//! Port of packages/tui/test/util/revert-diff.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/revert-diff.ts; see docs/TEST-PORT.md.

use opencode_tui::revert_diff::{get_revert_diff_files, RevertDiffFile};

#[test]
fn prefers_the_actual_file_path_over_dev_null_for_added_and_deleted_files() {
    let files = get_revert_diff_files(
        "diff --git a/new.txt b/new.txt\nnew file mode 100644\nindex 0000000..3b18e51\n--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1 @@\n+new content\ndiff --git a/old.txt b/old.txt\ndeleted file mode 100644\nindex 3b18e51..0000000\n--- a/old.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n-old content\n",
    );

    assert_eq!(
        files,
        vec![
            RevertDiffFile {
                filename: "new.txt".to_string(),
                additions: 1,
                deletions: 0,
            },
            RevertDiffFile {
                filename: "old.txt".to_string(),
                additions: 0,
                deletions: 1,
            },
        ]
    );
}
