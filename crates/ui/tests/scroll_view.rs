//! Port of packages/ui/src/components/scroll-view.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/ui/src/components/scroll-view.ts; see docs/TEST-PORT.md.

use opencode_ui::scroll_view::{
    can_scroll_key, scroll_key, scroll_top_from_thumb_pointer, ScrollDirection, ThumbPointerInput,
};

fn dir(key: &str, shift: bool) -> Option<&'static str> {
    scroll_key(key, false, false, false, shift).map(ScrollDirection::as_str)
}

#[test]
fn maps_plain_navigation_keys() {
    assert_eq!(dir("PageDown", false), Some("page-down"));
    assert_eq!(dir("ArrowUp", false), Some("up"));
}

#[test]
fn ignores_modified_keybinds() {
    assert_eq!(scroll_key("ArrowDown", false, false, true, false), None);
    assert_eq!(scroll_key("PageUp", false, true, false, false), None);
    assert_eq!(scroll_key("End", false, false, false, true), None);
}

#[test]
fn maps_space_and_shift_space_directions() {
    assert_eq!(dir(" ", false), Some("page-down"));
    assert_eq!(dir(" ", true), Some("page-up"));
}

#[test]
fn owns_upward_keys_only_above_the_top_boundary() {
    assert!(can_scroll_key(50.0, 100.0, 300.0, ScrollDirection::PageUp));
    assert!(!can_scroll_key(0.0, 100.0, 300.0, ScrollDirection::PageUp));
}

#[test]
fn owns_downward_keys_only_before_the_bottom_boundary() {
    assert!(can_scroll_key(
        50.0,
        100.0,
        300.0,
        ScrollDirection::PageDown
    ));
    assert!(!can_scroll_key(
        200.0,
        100.0,
        300.0,
        ScrollDirection::PageDown
    ));
    assert!(!can_scroll_key(
        0.0,
        100.0,
        100.0,
        ScrollDirection::PageDown
    ));
}

#[test]
fn keeps_downward_thumb_movement_monotonic_when_content_height_changes() {
    let first = scroll_top_from_thumb_pointer(ThumbPointerInput {
        pointer: 300.0,
        viewport_top: 100.0,
        grab_offset: 12.0,
        client_height: 600.0,
        scroll_client_height: None,
        scroll_height: 6_000.0,
        thumb_height: 60.0,
    });
    let second = scroll_top_from_thumb_pointer(ThumbPointerInput {
        pointer: 320.0,
        viewport_top: 100.0,
        grab_offset: 12.0,
        client_height: 600.0,
        scroll_client_height: None,
        scroll_height: 60_000.0,
        thumb_height: 32.0,
    });
    assert!(second > first);
}

#[test]
fn clamps_pointer_positions_to_the_scroll_range() {
    let input = ThumbPointerInput {
        viewport_top: 100.0,
        grab_offset: 12.0,
        client_height: 600.0,
        scroll_client_height: None,
        scroll_height: 6_000.0,
        thumb_height: 60.0,
        pointer: 0.0,
    };
    assert_eq!(
        scroll_top_from_thumb_pointer(ThumbPointerInput {
            pointer: 0.0,
            ..input
        }),
        0.0
    );
    assert_eq!(
        scroll_top_from_thumb_pointer(ThumbPointerInput {
            pointer: 1_000.0,
            ..input
        }),
        5_400.0
    );
}

#[test]
fn uses_scroll_client_height_when_the_thumb_track_differs_from_the_viewport() {
    let value = scroll_top_from_thumb_pointer(ThumbPointerInput {
        pointer: 400.0,
        viewport_top: 100.0,
        grab_offset: 0.0,
        client_height: 400.0,
        scroll_client_height: Some(800.0),
        scroll_height: 8_000.0,
        thumb_height: 40.0,
    });
    let expected = (292.0 / 344.0) * 7_200.0;
    assert!((value - expected).abs() < 1e-9, "{value} != {expected}");
}
