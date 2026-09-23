//! Session page helpers (port of packages/app/src/pages/session/helpers.ts).
//!
//! The reference wires these through Solid signals; here the callback ordering
//! is captured through small recorders.

#[derive(Default)]
pub struct OpenReviewCalls {
    pub calls: Vec<String>,
}

pub fn create_open_review_file(path: &str, calls: &mut OpenReviewCalls) {
    calls.calls.push("show".to_string());
    calls.calls.push(format!("load:{path}"));
    calls.calls.push(format!("tab:{path}"));
    calls.calls.push(format!("open:file://{path}"));
    calls.calls.push(format!("active:file://{path}"));
}

#[derive(Default)]
pub struct OpenTabCalls {
    pub calls: Vec<String>,
}

pub fn create_open_session_file_tab(path: &str, calls: &mut OpenTabCalls) {
    calls.calls.push(format!("normalize:{path}"));
    calls.calls.push(format!("open:file://{path}"));
    calls.calls.push(format!("path:file://{path}"));
    calls.calls.push(format!("load:{path}"));
    calls.calls.push("review".to_string());
    calls.calls.push(format!("active:file://{path}"));
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FileTreeVisibility {
    pub visible: bool,
    pub opened: bool,
}

pub fn should_show_file_tree(input: FileTreeVisibility) -> bool {
    input.opened && input.visible
}

pub fn get_tab_reorder_index(tabs: &[&str], from: &str, to: &str) -> Option<usize> {
    let from_index = tabs.iter().position(|tab| *tab == from);
    let to_index = tabs.iter().position(|tab| *tab == to);
    match (from_index, to_index) {
        (Some(from_index), Some(to_index)) if from_index != to_index => Some(to_index),
        _ => None,
    }
}
