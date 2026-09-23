//! Port of packages/app/src/pages/session/session-panel-width.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::session_panel_width::{
    clamp_session_panel_width, session_panel_width_max, REVIEW_PANE_WIDTH_MIN,
    REVIEW_PANE_WIDTH_MIN_SPLIT, SESSION_PANEL_WIDTH_MIN,
};

#[test]
fn reserves_the_unified_review_pane_minimum() {
    assert_eq!(
        session_panel_width_max(1700, false),
        1700 - REVIEW_PANE_WIDTH_MIN
    );
}

#[test]
fn reserves_a_larger_minimum_for_split_diffs() {
    assert_eq!(
        session_panel_width_max(1700, true),
        1700 - REVIEW_PANE_WIDTH_MIN_SPLIT
    );
    const { assert!(REVIEW_PANE_WIDTH_MIN_SPLIT > REVIEW_PANE_WIDTH_MIN) };
}

#[test]
fn lets_the_chat_panel_take_everything_beyond_the_review_pane_minimum() {
    let available = 3440;
    assert!(session_panel_width_max(available, false) > (available as f64 * 0.45) as i64);
}

#[test]
fn never_drops_below_the_chat_panel_minimum_on_small_windows() {
    assert_eq!(session_panel_width_max(600, true), SESSION_PANEL_WIDTH_MIN);
    assert_eq!(session_panel_width_max(0, false), SESSION_PANEL_WIDTH_MIN);
}

#[test]
fn keeps_widths_already_within_the_limit() {
    assert_eq!(clamp_session_panel_width(800, Some(1700), false), 800);
}

#[test]
fn forces_the_width_down_when_the_window_shrinks() {
    assert_eq!(
        clamp_session_panel_width(1600, Some(1700), false),
        1700 - REVIEW_PANE_WIDTH_MIN
    );
    assert_eq!(
        clamp_session_panel_width(1600, Some(1700), true),
        1700 - REVIEW_PANE_WIDTH_MIN_SPLIT
    );
}

#[test]
fn holds_the_chat_panel_minimum_when_there_is_no_room_for_both() {
    assert_eq!(
        clamp_session_panel_width(1600, Some(700), true),
        SESSION_PANEL_WIDTH_MIN
    );
}

#[test]
fn skips_clamping_before_the_layout_is_measured() {
    assert_eq!(clamp_session_panel_width(1600, None, false), 1600);
}
