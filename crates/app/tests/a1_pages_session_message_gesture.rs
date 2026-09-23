//! Port of packages/app/src/pages/session/message-gesture.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::message_gesture::{normalize_wheel_delta, should_mark_boundary_gesture};

#[test]
fn converts_line_mode_to_px() {
    assert_eq!(normalize_wheel_delta(3, 1, 500), 120);
}

#[test]
fn converts_page_mode_to_container_height() {
    assert_eq!(normalize_wheel_delta(-1, 2, 600), -600);
}

#[test]
fn keeps_pixel_mode_unchanged() {
    assert_eq!(normalize_wheel_delta(16, 0, 600), 16);
}

#[test]
fn marks_when_nested_scroller_cannot_scroll() {
    assert!(should_mark_boundary_gesture(20, 0, 300, 300));
}

#[test]
fn marks_when_scrolling_beyond_top_boundary() {
    assert!(should_mark_boundary_gesture(-40, 10, 1000, 400));
}

#[test]
fn marks_when_scrolling_beyond_bottom_boundary() {
    assert!(should_mark_boundary_gesture(50, 580, 1000, 400));
}

#[test]
fn does_not_mark_when_nested_scroller_can_consume_movement() {
    assert!(!should_mark_boundary_gesture(20, 200, 1000, 400));
}
