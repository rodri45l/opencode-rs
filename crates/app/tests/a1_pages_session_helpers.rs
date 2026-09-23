//! Port of packages/app/src/pages/session/helpers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//! DOM-only cases (focusTerminalById) are marked n/a in PORT-STATUS.a1.json.
#![allow(dead_code)]

const SESSION_OPEN_FILE_TAB: &str = "open-file";

#[derive(Clone, Copy, Debug, PartialEq)]
struct FileTreeVisibility {
    visible: bool,
    opened: bool,
}

// Local stubs (fast wave): real module lands later.
fn should_show_file_tree(_input: FileTreeVisibility) -> bool {
    false
}

#[derive(Default)]
struct OpenReviewCalls {
    calls: Vec<String>,
}

fn create_open_review_file(_path: &str, _calls: &mut OpenReviewCalls) {}

#[derive(Default)]
struct OpenTabCalls {
    calls: Vec<String>,
}

fn create_open_session_file_tab(_path: &str, _calls: &mut OpenTabCalls) {}

fn get_tab_reorder_index(_tabs: &[&str], _dragged: &str, _droppable: &str) -> Option<usize> {
    None
}

#[test]
#[ignore = "porting: pages/session/helpers not implemented"]
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
#[ignore = "porting: pages/session/helpers not implemented"]
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
#[ignore = "porting: pages/session/helpers not implemented"]
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
#[ignore = "porting: pages/session/helpers not implemented"]
fn returns_target_index_for_valid_drag_reorder() {
    assert_eq!(get_tab_reorder_index(&["a", "b", "c"], "a", "c"), Some(2));
}

#[test]
#[ignore = "porting: pages/session/helpers not implemented"]
fn returns_undefined_for_unknown_droppable_id() {
    assert_eq!(
        get_tab_reorder_index(&["a", "b", "c"], "a", "missing"),
        None
    );
}
