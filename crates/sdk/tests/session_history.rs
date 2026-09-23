//! Port of packages/sdk/js/test/session-history.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the generated `V2SessionHistoryData` type; see docs/TEST-PORT.md.

use opencode_sdk::{V2SessionHistoryData, V2SessionHistoryPath, V2SessionHistoryQuery};

#[test]
fn uses_numeric_session_history_positions() {
    let input = V2SessionHistoryData {
        path: V2SessionHistoryPath {
            session_id: "ses_test".to_string(),
        },
        query: V2SessionHistoryQuery {
            after: 1,
            limit: 50,
        },
        url: "/api/session/{sessionID}/history".to_string(),
    };

    assert_eq!(input.query.after, 1);
}
