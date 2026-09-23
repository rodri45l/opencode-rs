//! Port of packages/app/src/pages/session/message-gesture.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

// Local stubs (fast wave): real module lands later.
fn normalize_wheel_delta(_delta_y: i64, _delta_mode: i64, _root_height: i64) -> i64 {
    0
}

fn should_mark_boundary_gesture(
    _delta: i64,
    _scroll_top: i64,
    _scroll_height: i64,
    _client_height: i64,
) -> bool {
    false
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn converts_line_mode_to_px() {
    assert_eq!(normalize_wheel_delta(3, 1, 500), 120);
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn converts_page_mode_to_container_height() {
    assert_eq!(normalize_wheel_delta(-1, 2, 600), -600);
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn keeps_pixel_mode_unchanged() {
    assert_eq!(normalize_wheel_delta(16, 0, 600), 16);
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn marks_when_nested_scroller_cannot_scroll() {
    assert!(should_mark_boundary_gesture(20, 0, 300, 300));
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn marks_when_scrolling_beyond_top_boundary() {
    assert!(should_mark_boundary_gesture(-40, 10, 1000, 400));
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn marks_when_scrolling_beyond_bottom_boundary() {
    assert!(should_mark_boundary_gesture(50, 580, 1000, 400));
}

#[test]
#[ignore = "porting: pages/session/message-gesture not implemented"]
fn does_not_mark_when_nested_scroller_can_consume_movement() {
    assert!(!should_mark_boundary_gesture(20, 200, 1000, 400));
}
