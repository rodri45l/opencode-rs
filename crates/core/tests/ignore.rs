//! Port of packages/core/test/filesystem/ignore.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the default ignore set matches `node_modules` at any depth,
//! with or without a trailing slash, including nested descendants.

use opencode_core::fs_ignore::Ignore;

const NOTE: &str = "porting: filesystem ignore not implemented";

#[test]
#[ignore = "porting: filesystem ignore not implemented"]
fn matches_nested_and_non_nested_node_modules() {
    assert!(Ignore::match_path("node_modules/index.js").expect(NOTE));
    assert!(Ignore::match_path("node_modules").expect(NOTE));
    assert!(Ignore::match_path("node_modules/").expect(NOTE));
    assert!(Ignore::match_path("node_modules/bar").expect(NOTE));
    assert!(Ignore::match_path("node_modules/bar/").expect(NOTE));
}
