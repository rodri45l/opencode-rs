//! Port of packages/core/test/ripgrep.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `.git` metadata is never included in find or grep results,
//! `.opencode` files are not treated as ignored, and an oversized matched line
//! is previewed with an ellipsis without splitting a surrogate pair.
//! Re-derived: the live filesystem `find`/`grep` results and the gitignore
//! integration are dropped (they require a real git checkout); the always-ignored
//! `.git` rule and the pure preview truncation remain.

use opencode_core::ripgrep::Ripgrep;

const NOTE: &str = "porting: ripgrep not implemented";

#[test]
#[ignore = "porting: ripgrep not implemented"]
fn keeps_git_metadata_out_of_results_but_not_opencode_files() {
    assert!(Ripgrep::is_ignored(".git/config").expect(NOTE));
    assert!(Ripgrep::is_ignored(".git").expect(NOTE));
    assert!(!Ripgrep::is_ignored(".opencode/config").expect(NOTE));
    assert!(!Ripgrep::is_ignored("src/index.js").expect(NOTE));
}

#[test]
#[ignore = "porting: ripgrep not implemented"]
fn does_not_split_surrogate_pairs_in_oversized_line_previews() {
    let line = format!("needle{}😀", "x".repeat(1_993));
    assert_eq!(
        Ripgrep::preview_line(&line).expect(NOTE),
        format!("needle{}...", "x".repeat(1_993))
    );
}
