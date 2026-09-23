//! File tab list scroll (port of packages/app/src/pages/session/file-tab-scroll.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollInput {
    pub prev_scroll_width: i64,
    pub scroll_width: i64,
    pub client_width: i64,
    pub prev_context_open: bool,
    pub context_open: bool,
}

pub fn next_tab_list_scroll_left(input: ScrollInput) -> Option<i64> {
    if input.scroll_width <= input.prev_scroll_width {
        return None;
    }
    if !input.prev_context_open && input.context_open {
        return Some(0);
    }
    if input.scroll_width <= input.client_width {
        return None;
    }
    Some(input.scroll_width - input.client_width)
}
