//! Port of packages/app/src/components/titlebar-tab-gesture.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Element {
    slot: Option<String>,
    parent: Option<Box<Element>>,
}

// Local stubs (fast wave): real module lands later.
fn is_tab_close_target(_element: &Element) -> bool {
    false
}

fn forward_tab_ref(_element: &str) -> String {
    String::new()
}

fn can_open_tab_rename(_editing: bool, _busy: bool, _save_pending: bool) -> bool {
    false
}

fn can_start_tab_drag(_pointer_type: &str) -> bool {
    false
}

fn close() -> Element {
    Element {
        slot: Some("tab-close".into()),
        parent: None,
    }
}

fn child(parent: Element) -> Element {
    Element {
        slot: None,
        parent: Some(Box::new(parent)),
    }
}

fn link() -> Element {
    Element {
        slot: None,
        parent: None,
    }
}

#[test]
#[ignore = "porting: components/titlebar-tab-gesture not implemented"]
fn excludes_close_controls_from_tab_gestures() {
    let close_element = close();
    let button = child(close_element.clone());
    let link_element = link();
    assert!(is_tab_close_target(&close_element));
    assert!(is_tab_close_target(&button));
    assert!(!is_tab_close_target(&link_element));
}

#[test]
#[ignore = "porting: components/titlebar-tab-gesture not implemented"]
fn forwards_component_refs() {
    let element = "element";
    assert_eq!(forward_tab_ref(element), element);
}

#[test]
#[ignore = "porting: components/titlebar-tab-gesture not implemented"]
fn does_not_reopen_rename_while_a_save_is_pending() {
    assert!(can_open_tab_rename(false, false, false));
    assert!(!can_open_tab_rename(false, false, true));
}

#[test]
#[ignore = "porting: components/titlebar-tab-gesture not implemented"]
fn preserves_native_panning_for_touch_pointers() {
    assert!(can_start_tab_drag("mouse"));
    assert!(can_start_tab_drag("pen"));
    assert!(!can_start_tab_drag("touch"));
}
