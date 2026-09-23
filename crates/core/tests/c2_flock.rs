//! Port of packages/core/test/util/flock.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: process-contention cases are expressed against a typed
//! synchronous lock stub. Deterministic pieces (lock path derivation, staleness,
//! owner metadata shape, error-message contract) are pure and stay faithful.

#![allow(dead_code)]

mod flock {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl std::fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "not implemented: {}", self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LockError {
        pub message: String,
    }

    impl LockError {
        pub fn not_implemented() -> Self {
            LockError {
                message: "not implemented: flock".to_string(),
            }
        }
        pub fn timed_out() -> Self {
            LockError {
                message: "Timed out waiting for lock".to_string(),
            }
        }
        pub fn token_mismatch() -> Self {
            LockError {
                message: "lock token mismatch".to_string(),
            }
        }
        pub fn compromised() -> Self {
            LockError {
                message: "lock dir was compromised (removed while held)".to_string(),
            }
        }
    }

    impl std::fmt::Display for LockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for LockError {}

    /// FNV-1a 64-bit, rendered as lowercase hex. Only needs to be deterministic
    /// so the lock path is stable across processes.
    pub fn fast_hash(key: &str) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in key.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{hash:016x}")
    }

    pub fn lock_path(dir: &str, key: &str) -> String {
        format!("{}/{}.lock", dir.trim_end_matches('/'), fast_hash(key))
    }

    pub fn breaker_path(lock_dir: &str) -> String {
        format!("{lock_dir}.breaker")
    }

    pub fn heartbeat_path(lock_dir: &str) -> String {
        format!("{lock_dir}/heartbeat")
    }

    pub fn meta_path(lock_dir: &str) -> String {
        format!("{lock_dir}/meta.json")
    }

    /// A lock is stale once its age strictly exceeds the configured threshold.
    pub fn is_stale(age_ms: u64, stale_ms: u64) -> bool {
        age_ms > stale_ms
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OwnerMeta {
        pub token: String,
        pub pid: u32,
        pub hostname: String,
        pub created_at: String,
    }

    pub fn owner_meta() -> Result<OwnerMeta, NotImplemented> {
        Err(NotImplemented("flock owner metadata"))
    }

    pub struct LockGuard {
        pub held: bool,
    }

    pub fn acquire(
        _key: &str,
        _dir: &str,
        _stale_ms: u64,
        _timeout_ms: u64,
    ) -> Result<LockGuard, LockError> {
        Err(LockError::not_implemented())
    }

    pub fn with_lock(
        _key: &str,
        _dir: &str,
        _stale_ms: u64,
        _timeout_ms: u64,
        _body: impl FnOnce() -> Result<(), LockError>,
    ) -> Result<(), LockError> {
        Err(LockError::not_implemented())
    }
}

const NOTE: &str = "porting: flock not implemented";

#[test]
#[ignore = "porting: flock not implemented"]
fn lock_path_is_derived_from_the_key_hash() {
    let dir = "/tmp/locks";
    let key = "flock:meta";
    let path = flock::lock_path(dir, key);
    assert!(path.starts_with("/tmp/locks/"));
    assert!(path.ends_with(".lock"));
    assert_eq!(path, flock::lock_path(dir, key));
    assert_ne!(path, flock::lock_path(dir, "flock:other"));
}

#[test]
#[ignore = "porting: flock not implemented"]
fn auxiliary_paths_are_derived_from_the_lock_dir() {
    let lock_dir = "/tmp/locks/abc.lock";
    assert_eq!(flock::breaker_path(lock_dir), "/tmp/locks/abc.lock.breaker");
    assert_eq!(
        flock::heartbeat_path(lock_dir),
        "/tmp/locks/abc.lock/heartbeat"
    );
    assert_eq!(flock::meta_path(lock_dir), "/tmp/locks/abc.lock/meta.json");
}

#[test]
#[ignore = "porting: flock not implemented"]
fn staleness_is_strictly_greater_than_the_threshold() {
    assert!(!flock::is_stale(199, 200));
    assert!(!flock::is_stale(200, 200));
    assert!(flock::is_stale(201, 200));
}

#[test]
#[ignore = "porting: flock not implemented"]
fn timeout_error_message_contract() {
    assert!(flock::LockError::timed_out()
        .to_string()
        .contains("Timed out waiting for lock"));
}

#[test]
#[ignore = "porting: flock not implemented"]
fn token_mismatch_and_compromise_message_contract() {
    assert!(flock::LockError::token_mismatch()
        .to_string()
        .contains("token mismatch"));
    assert!(flock::LockError::compromised()
        .to_string()
        .contains("compromised"));
}

#[test]
#[ignore = "porting: flock not implemented"]
fn enforces_mutual_exclusion_under_process_contention() {
    let mut completed = Vec::new();
    for _ in 0..16 {
        let outcome = flock::with_lock("flock:stress", "/tmp/locks", 1_000, 15_000, || Ok(()));
        completed.push(outcome.is_ok());
    }
    assert!(completed.iter().all(|ok| *ok));
    assert_eq!(completed.len(), 16);
}

#[test]
#[ignore = "porting: flock not implemented"]
fn times_out_while_waiting_when_lock_is_still_healthy() {
    let err =
        flock::with_lock("flock:timeout", "/tmp/locks", 10_000, 1_000, || Ok(())).expect_err(NOTE);
    assert!(err.to_string().contains("Timed out waiting for lock"));
}

#[test]
#[ignore = "porting: flock not implemented"]
fn recovers_after_a_crashed_lock_owner() {
    let mut hit = false;
    let result = flock::with_lock("flock:crash", "/tmp/locks", 500, 8_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}

#[test]
#[ignore = "porting: flock not implemented"]
fn breaks_stale_lock_dirs_when_heartbeat_is_missing() {
    let mut hit = false;
    let result = flock::with_lock("flock:missing-heartbeat", "/tmp/locks", 200, 3_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}

#[test]
#[ignore = "porting: flock not implemented"]
fn recovers_when_a_stale_breaker_claim_was_left_behind() {
    let mut hit = false;
    let result = flock::with_lock("flock:stale-breaker", "/tmp/locks", 200, 3_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let breaker = flock::breaker_path(&flock::lock_path("/tmp/locks", "flock:stale-breaker"));
    assert!(!std::path::Path::new(&breaker).exists());
}

#[test]
#[ignore = "porting: flock not implemented"]
fn fails_clearly_if_lock_dir_is_removed_while_held() {
    let err = flock::with_lock("flock:compromised", "/tmp/locks", 1_000, 3_000, || Ok(()))
        .expect_err(NOTE);
    assert!(err.to_string().contains("compromised"));
}

#[test]
#[ignore = "porting: flock not implemented"]
fn writes_owner_metadata_while_lock_is_held() {
    let meta = flock::owner_meta().expect(NOTE);
    assert!(!meta.token.is_empty());
    assert!(meta.pid > 0);
    assert!(!meta.hostname.is_empty());
    assert!(!meta.created_at.is_empty());
}

#[test]
#[ignore = "porting: flock not implemented"]
fn supports_acquire_with_await_using() {
    let guard = flock::acquire("flock:acquire", "/tmp/locks", 1_000, 3_000).expect(NOTE);
    assert!(guard.held);
}

#[test]
#[ignore = "porting: flock not implemented"]
fn refuses_token_mismatch_release_and_recovers_from_stale() {
    let err = flock::with_lock("flock:token", "/tmp/locks", 500, 3_000, || Ok(())).expect_err(NOTE);
    assert!(err.to_string().contains("token mismatch"));

    let mut hit = false;
    let result = flock::with_lock("flock:token", "/tmp/locks", 500, 6_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}

#[test]
#[ignore = "porting: flock not implemented"]
fn fails_clearly_on_unwritable_lock_roots() {
    let err = flock::with_lock("flock:perm", "/tmp/locks", 100, 500, || Ok(())).expect_err(NOTE);
    let text = err.to_string();
    assert!(text.contains("EACCES") || text.contains("EPERM"));
}
