//! Port of packages/app/src/components/titlebar-tab-order.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::titlebar_tab_order::{adjacent_tab_key, merge_visible_tab_order};

#[test]
fn adjacent_tab_key_follows_the_visible_left_to_right_order() {
    assert_eq!(
        adjacent_tab_key(&["c", "a", "b"], "c", 1),
        Some("a".to_string())
    );
    assert_eq!(
        adjacent_tab_key(&["c", "a", "b"], "a", -1),
        Some("c".to_string())
    );
}

#[test]
fn adjacent_tab_key_skips_tabs_omitted_from_the_visible_order() {
    assert_eq!(adjacent_tab_key(&["a", "c"], "a", 1), Some("c".to_string()));
    assert_eq!(adjacent_tab_key(&["a", "c"], "c", 1), Some("a".to_string()));
}

#[test]
fn merges_reordered_visible_tabs_around_hidden_tabs() {
    assert_eq!(
        merge_visible_tab_order(
            &["a", "hidden", "b", "c"],
            &["a", "b", "c"],
            &["c", "a", "b"]
        ),
        vec![
            "c".to_string(),
            "hidden".to_string(),
            "a".to_string(),
            "b".to_string()
        ]
    );
}
