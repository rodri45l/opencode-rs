//! Port of packages/app/src/components/titlebar-tab-gesture.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::titlebar_tab_gesture::{
    can_open_tab_rename, can_start_tab_drag, forward_tab_ref, is_tab_close_target, Element,
};

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
fn excludes_close_controls_from_tab_gestures() {
    let close_element = close();
    let button = child(close_element.clone());
    let link_element = link();
    assert!(is_tab_close_target(&close_element));
    assert!(is_tab_close_target(&button));
    assert!(!is_tab_close_target(&link_element));
}

#[test]
fn forwards_component_refs() {
    let element = "element";
    assert_eq!(forward_tab_ref(element), element);
}

#[test]
fn does_not_reopen_rename_while_a_save_is_pending() {
    assert!(can_open_tab_rename(false, false, false));
    assert!(!can_open_tab_rename(false, false, true));
}

#[test]
fn preserves_native_panning_for_touch_pointers() {
    assert!(can_start_tab_drag("mouse"));
    assert!(can_start_tab_drag("pen"));
    assert!(!can_start_tab_drag("touch"));
}
