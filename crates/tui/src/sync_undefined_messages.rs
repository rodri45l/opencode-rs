//! Session message hydration fallback.
//!
//! Port of packages/tui/src/cli/cmd/tui/sync-undefined-messages.test.tsx
//! behaviour (upstream 18ef3cc): a non-2xx messages response must not crash
//! sync, it resolves to no data.

use crate::sync_hydration::MessageInfo;

/// Load session messages, returning `None` for a failed response.
pub fn load_session_messages(response: Result<Vec<MessageInfo>, ()>) -> Option<Vec<MessageInfo>> {
    response.ok()
}
