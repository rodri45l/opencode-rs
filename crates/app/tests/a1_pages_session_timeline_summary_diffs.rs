//! Port of packages/app/src/pages/session/timeline/summary-diffs.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::summary_diffs::{unique_summary_diffs, Diff};

fn diff(file: &str, additions: i64) -> Diff {
    Diff {
        file: Some(file.into()),
        additions,
        deletions: 0,
    }
}

#[test]
fn drops_entries_without_files_and_preserves_unique_input() {
    let alpha = diff("alpha.ts", 1);
    let beta = diff("beta.ts", 1);
    let invalid = Diff {
        file: None,
        additions: 1,
        deletions: 0,
    };

    assert!(unique_summary_diffs(None).is_empty());
    assert!(unique_summary_diffs(Some(Vec::new())).is_empty());
    assert!(unique_summary_diffs(Some(vec![invalid.clone()])).is_empty());

    let result = unique_summary_diffs(Some(vec![alpha.clone(), invalid, beta.clone()]));
    assert_eq!(result, vec![alpha, beta]);
}

#[test]
fn keeps_the_last_diff_per_file_in_the_legacy_display_order() {
    let old_alpha = diff("alpha.ts", 1);
    let old_beta = diff("beta.ts", 1);
    let new_alpha = diff("alpha.ts", 2);
    let charlie = diff("charlie.ts", 1);
    let new_beta = diff("beta.ts", 2);

    let result = unique_summary_diffs(Some(vec![
        old_alpha,
        old_beta,
        new_alpha.clone(),
        charlie.clone(),
        new_beta.clone(),
    ]));
    assert_eq!(result, vec![new_alpha, charlie, new_beta]);
}
