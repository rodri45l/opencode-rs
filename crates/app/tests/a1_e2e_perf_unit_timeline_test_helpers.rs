//! Port of packages/app/e2e/performance/unit/timeline-test-helpers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_test_support as ts;

// Local stub (fast wave): real module lands later.
fn stress_session_href(_source_id: &str) -> String {
    String::new()
}

#[test]
#[ignore = "porting: e2e/performance/unit/timeline-test-helpers not implemented"]
fn builds_stress_session_links_for_the_benchmark_server() {
    let source_id = "session-source";
    assert_eq!(
        stress_session_href(source_id),
        format!(
            "/server/{}/session/{source_id}",
            ts::encode_b64url(b"http://127.0.0.1:4096")
        )
    );
}
