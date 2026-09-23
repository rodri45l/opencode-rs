//! Scroll-view key resolution and thumb-drag math.
//!
//! Port of packages/ui/src/components/scroll-view.ts behaviour (upstream 18ef3cc).
//! Layout/rendering of the scroll container itself is visual and human-verified.

/// The direction a scroll key maps to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    PageUp,
    PageDown,
}

impl ScrollDirection {
    /// The wire string used by the reference (`"up"`, `"page-down"`, ...).
    pub fn as_str(self) -> &'static str {
        match self {
            ScrollDirection::Up => "up",
            ScrollDirection::Down => "down",
            ScrollDirection::PageUp => "page-up",
            ScrollDirection::PageDown => "page-down",
        }
    }
}

/// Resolve a keyboard event to a scroll direction.
///
/// Modified keys (alt/ctrl/meta) never scroll; shift is only meaningful for the
/// space key, where it reverses the page direction.
pub fn scroll_key(
    key: &str,
    alt: bool,
    ctrl: bool,
    meta: bool,
    shift: bool,
) -> Option<ScrollDirection> {
    if alt || ctrl || meta {
        return None;
    }
    match key {
        " " => Some(if shift {
            ScrollDirection::PageUp
        } else {
            ScrollDirection::PageDown
        }),
        "PageDown" if !shift => Some(ScrollDirection::PageDown),
        "PageUp" if !shift => Some(ScrollDirection::PageUp),
        "ArrowUp" if !shift => Some(ScrollDirection::Up),
        "ArrowDown" if !shift => Some(ScrollDirection::Down),
        "Home" if !shift => Some(ScrollDirection::PageUp),
        "End" if !shift => Some(ScrollDirection::PageDown),
        _ => None,
    }
}

/// Whether a scroll element can still move in the given direction.
pub fn can_scroll_key(
    scroll_top: f64,
    client_height: f64,
    scroll_height: f64,
    direction: ScrollDirection,
) -> bool {
    match direction {
        ScrollDirection::Up | ScrollDirection::PageUp => scroll_top > 0.0,
        ScrollDirection::Down | ScrollDirection::PageDown => {
            scroll_top < scroll_height - client_height
        }
    }
}

/// Inputs for [`scroll_top_from_thumb_pointer`].
#[derive(Debug, Clone, Copy)]
pub struct ThumbPointerInput {
    pub pointer: f64,
    pub viewport_top: f64,
    pub grab_offset: f64,
    pub client_height: f64,
    /// Height of the scroll client when it differs from the thumb track.
    pub scroll_client_height: Option<f64>,
    pub scroll_height: f64,
    pub thumb_height: f64,
}

const THUMB_TRACK_PADDING: f64 = 8.0;

/// Map an absolute pointer position to a clamped scroll offset.
pub fn scroll_top_from_thumb_pointer(input: ThumbPointerInput) -> f64 {
    let client = input.scroll_client_height.unwrap_or(input.client_height);
    let max_scroll = (input.scroll_height - client).max(0.0);
    let track = (input.client_height - 2.0 * THUMB_TRACK_PADDING - input.thumb_height).max(0.0);
    if track <= 0.0 {
        return 0.0;
    }
    let thumb_top = (input.pointer - input.viewport_top - input.grab_offset - THUMB_TRACK_PADDING)
        .clamp(0.0, track);
    (thumb_top / track * max_scroll).clamp(0.0, max_scroll)
}
