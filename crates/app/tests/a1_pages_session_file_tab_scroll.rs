//! Port of packages/app/src/pages/session/file-tab-scroll.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollInput {
    prev_scroll_width: i64,
    scroll_width: i64,
    client_width: i64,
    prev_context_open: bool,
    context_open: bool,
}

// Local stub (fast wave): real module lands later.
fn next_tab_list_scroll_left(_input: ScrollInput) -> Option<i64> {
    None
}

#[test]
#[ignore = "porting: pages/session/file-tab-scroll not implemented"]
fn does_not_scroll_when_width_shrinks() {
    assert_eq!(
        next_tab_list_scroll_left(ScrollInput {
            prev_scroll_width: 500,
            scroll_width: 420,
            client_width: 300,
            prev_context_open: false,
            context_open: false
        }),
        None
    );
}

#[test]
#[ignore = "porting: pages/session/file-tab-scroll not implemented"]
fn scrolls_to_start_when_context_tab_opens() {
    assert_eq!(
        next_tab_list_scroll_left(ScrollInput {
            prev_scroll_width: 400,
            scroll_width: 500,
            client_width: 320,
            prev_context_open: false,
            context_open: true
        }),
        Some(0)
    );
}

#[test]
#[ignore = "porting: pages/session/file-tab-scroll not implemented"]
fn scrolls_to_right_edge_for_new_file_tabs() {
    assert_eq!(
        next_tab_list_scroll_left(ScrollInput {
            prev_scroll_width: 500,
            scroll_width: 780,
            client_width: 300,
            prev_context_open: true,
            context_open: true
        }),
        Some(480)
    );
}
