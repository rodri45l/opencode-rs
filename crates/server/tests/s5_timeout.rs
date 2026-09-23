//! Port of packages/opencode/test/util/timeout.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `withTimeout` resolves with the value when the future
//! completes in time, and rejects with "Operation timed out after <ms>ms"
//! otherwise.
#![allow(dead_code)]

use std::time::Duration;

use opencode_server::timeout_util::with_timeout;

#[tokio::test]
async fn resolves_when_promise_completes_before_timeout() {
    let fast = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "fast".to_string()
    };

    assert_eq!(with_timeout(fast, 100).await.unwrap(), "fast");
}

#[tokio::test]
async fn rejects_when_promise_exceeds_timeout() {
    let slow = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        "slow".to_string()
    };

    let err = with_timeout(slow, 50).await.unwrap_err();
    assert!(err.to_string().contains("Operation timed out after 50ms"));
}
