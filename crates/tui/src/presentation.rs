//! Session continuation presentation.
//!
//! Port of packages/tui/src/util/presentation.ts `sessionEpilogue` behaviour
//! (upstream 18ef3cc). Only the non-visual text content is ported; the ASCII
//! wordmark itself is human-verified.

/// Build the text shown when a session is exited.
pub fn session_epilogue(title: &str, session_id: Option<&str>) -> String {
    let continue_line = session_id
        .map(|id| format!("opencode -s {id}"))
        .unwrap_or_else(|| "opencode".to_string());
    format!("  Session  {title}\n  Continue {continue_line}\n")
}
