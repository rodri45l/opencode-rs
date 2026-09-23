//! Port of packages/app/src/pages/session/helpers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//! DOM-only cases (focusTerminalById) are marked n/a in PORT-STATUS.a1.json.

use opencode_app::pages_session_helpers::{
    create_open_review_file, create_open_session_file_tab, get_tab_reorder_index,
    should_show_file_tree, FileTreeVisibility, OpenReviewCalls, OpenTabCalls,
};

#[test]
fn does_not_reserve_space_for_a_disabled_file_tree() {
    assert!(!should_show_file_tree(FileTreeVisibility {
        visible: false,
        opened: true
    }));
    assert!(should_show_file_tree(FileTreeVisibility {
        visible: true,
        opened: true
    }));
}

#[test]
fn opens_and_loads_selected_review_file() {
    let mut calls = OpenReviewCalls::default();
    create_open_review_file("src/a.ts", &mut calls);
    assert_eq!(
        calls.calls,
        vec![
            "show".to_string(),
            "load:src/a.ts".to_string(),
            "tab:src/a.ts".to_string(),
            "open:file://src/a.ts".to_string(),
            "active:file://src/a.ts".to_string()
        ]
    );
}

#[test]
fn activates_the_opened_file_tab() {
    let mut calls = OpenTabCalls::default();
    create_open_session_file_tab("src/a.ts", &mut calls);
    assert_eq!(
        calls.calls,
        vec![
            "normalize:src/a.ts".to_string(),
            "open:file://src/a.ts".to_string(),
            "path:file://src/a.ts".to_string(),
            "load:src/a.ts".to_string(),
            "review".to_string(),
            "active:file://src/a.ts".to_string()
        ]
    );
}

#[test]
fn returns_target_index_for_valid_drag_reorder() {
    assert_eq!(get_tab_reorder_index(&["a", "b", "c"], "a", "c"), Some(2));
}

#[test]
fn returns_undefined_for_unknown_droppable_id() {
    assert_eq!(
        get_tab_reorder_index(&["a", "b", "c"], "a", "missing"),
        None
    );
}
