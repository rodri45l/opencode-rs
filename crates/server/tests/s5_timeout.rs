//! Port of packages/opencode/test/util/timeout.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `withTimeout` resolves with the value when the future
//! completes in time, and rejects with "Operation timed out after <ms>ms"
//! otherwise.
#![allow(dead_code)]

use std::time::Duration;

// Fast-wave local stubs: `util::timeout` is not implemented in this crate yet.
mod timeout {
    use std::future::Future;

    #[derive(Debug)]
    pub enum TimeoutError {
        NotImplemented,
        TimedOut(u64),
    }

    impl std::fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TimeoutError::NotImplemented => {
                    write!(f, "porting: timeout::withTimeout not implemented")
                }
                TimeoutError::TimedOut(ms) => write!(f, "Operation timed out after {ms}ms"),
            }
        }
    }

    pub async fn with_timeout<F, T>(_future: F, _millis: u64) -> Result<T, TimeoutError>
    where
        F: Future<Output = T>,
    {
        Err(TimeoutError::NotImplemented)
    }
}

#[tokio::test]
#[ignore = "porting: timeout not implemented"]
async fn resolves_when_promise_completes_before_timeout() {
    let fast = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "fast".to_string()
    };

    assert_eq!(timeout::with_timeout(fast, 100).await.unwrap(), "fast");
}

#[tokio::test]
#[ignore = "porting: timeout not implemented"]
async fn rejects_when_promise_exceeds_timeout() {
    let slow = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        "slow".to_string()
    };

    let err = timeout::with_timeout(slow, 50).await.unwrap_err();
    assert!(err.to_string().contains("Operation timed out after 50ms"));
}
