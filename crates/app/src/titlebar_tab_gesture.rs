//! Titlebar tab gestures (port of packages/app/src/components/titlebar-tab-gesture.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub slot: Option<String>,
    pub parent: Option<Box<Element>>,
}

pub fn is_tab_close_target(element: &Element) -> bool {
    let mut current = Some(element);
    while let Some(node) = current {
        if node.slot.as_deref() == Some("tab-close") {
            return true;
        }
        current = node.parent.as_deref();
    }
    false
}

pub fn forward_tab_ref(element: &str) -> String {
    element.to_string()
}

pub fn can_open_tab_rename(editing: bool, busy: bool, save_pending: bool) -> bool {
    !editing && !busy && !save_pending
}

pub fn can_start_tab_drag(pointer_type: &str) -> bool {
    pointer_type != "touch"
}
