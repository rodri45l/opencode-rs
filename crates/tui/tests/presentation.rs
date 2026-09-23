//! Port of packages/tui/test/util/presentation.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/presentation.ts; see docs/TEST-PORT.md.

use opencode_tui::presentation::session_epilogue;

#[test]
fn formats_session_continuation_summary() {
    let epilogue = session_epilogue("A session", Some("ses_123"));
    assert!(epilogue.contains("A session"));
    assert!(epilogue.contains("opencode -s ses_123"));
}
