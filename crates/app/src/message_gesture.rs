//! Scroll gesture normalisation (port of packages/app/src/pages/session/message-gesture.ts).

pub fn normalize_wheel_delta(delta_y: i64, delta_mode: i64, root_height: i64) -> i64 {
    if delta_mode == 1 {
        return delta_y * 40;
    }
    if delta_mode == 2 {
        return delta_y * root_height;
    }
    delta_y
}

pub fn should_mark_boundary_gesture(
    delta: i64,
    scroll_top: i64,
    scroll_height: i64,
    client_height: i64,
) -> bool {
    let max = scroll_height - client_height;
    if max <= 1 {
        return true;
    }
    if delta == 0 {
        return false;
    }
    if delta < 0 {
        return scroll_top + delta <= 0;
    }
    let remaining = max - scroll_top;
    delta > remaining
}
