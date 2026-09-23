//! Session panel width constraints (port of
//! packages/app/src/pages/session/session-panel-width.ts).

pub const SESSION_PANEL_WIDTH_MIN: i64 = 320;
pub const REVIEW_PANE_WIDTH_MIN: i64 = 420;
pub const REVIEW_PANE_WIDTH_MIN_SPLIT: i64 = 640;

pub fn session_panel_width_max(available: i64, split: bool) -> i64 {
    let pane = if split {
        REVIEW_PANE_WIDTH_MIN_SPLIT
    } else {
        REVIEW_PANE_WIDTH_MIN
    };
    std::cmp::max(SESSION_PANEL_WIDTH_MIN, available - pane)
}

pub fn clamp_session_panel_width(width: i64, available: Option<i64>, split: bool) -> i64 {
    match available {
        None => width,
        Some(available) => std::cmp::min(width, session_panel_width_max(available, split)),
    }
}
