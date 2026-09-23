//! Port of packages/opencode/test/mcp/session-recovery.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a session-bound POST that returns 404 reinitializes once,
//! then retries the original request under the replacement session, producing
//! exactly this request sequence. The reference spawns a fixture process; this
//! port asserts the recovery plan directly.

use opencode_server::port::mcp::{session_recovery_plan, RecoveryRequest};

fn request(method: &str, session: Option<&str>) -> RecoveryRequest {
    RecoveryRequest {
        method: method.to_string(),
        session: session.map(str::to_string),
    }
}

#[test]
fn reinitializes_and_retries_once_after_a_session_bound_post_returns_404() {
    let plan = session_recovery_plan("expired", "replacement");

    assert_eq!(
        plan,
        [
            request("initialize", None),
            request("notifications/initialized", Some("expired")),
            request("ping", Some("expired")),
            request("initialize", None),
            request("notifications/initialized", Some("replacement")),
            request("ping", Some("replacement")),
        ]
    );
}
