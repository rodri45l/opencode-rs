//! Port of packages/tui/test/cli/cmd/tui/sync-undefined-messages.test.tsx
//! (upstream 18ef3cc). Behaviour pinned by packages/tui/src/cli/cmd/tui/sync.tsx;
//! see docs/TEST-PORT.md.

use opencode_tui::sync_hydration::MessageInfo;
use opencode_tui::sync_undefined_messages::load_session_messages;

fn message(id: &str) -> MessageInfo {
    MessageInfo {
        id: id.to_string(),
        session_id: "ses_undef".to_string(),
        created: 0,
    }
}

#[test]
fn entering_a_session_whose_messages_endpoint_errors_does_not_crash_sync() {
    assert_eq!(load_session_messages(Err(())), None);
}

#[test]
fn loads_messages_when_the_endpoint_succeeds() {
    assert_eq!(
        load_session_messages(Ok(vec![message("msg_1")])),
        Some(vec![message("msg_1")])
    );
}
