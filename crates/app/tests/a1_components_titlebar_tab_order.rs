//! Port of packages/app/src/components/titlebar-tab-order.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

// Local stubs (fast wave): real module lands later.
fn adjacent_tab_key(_order: &[&str], _current: &str, _delta: i64) -> Option<String> {
    None
}

fn merge_visible_tab_order(_all: &[&str], _visible: &[&str], _reordered: &[&str]) -> Vec<String> {
    Vec::new()
}

#[test]
#[ignore = "porting: components/titlebar-tab-order not implemented"]
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
#[ignore = "porting: components/titlebar-tab-order not implemented"]
fn adjacent_tab_key_skips_tabs_omitted_from_the_visible_order() {
    assert_eq!(adjacent_tab_key(&["a", "c"], "a", 1), Some("c".to_string()));
    assert_eq!(adjacent_tab_key(&["a", "c"], "c", 1), Some("a".to_string()));
}

#[test]
#[ignore = "porting: components/titlebar-tab-order not implemented"]
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
