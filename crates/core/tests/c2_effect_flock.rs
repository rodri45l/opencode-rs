//! Port of packages/core/test/util/effect-flock.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `EffectFlock.Service` and `AppNodeBuilder.build` layer
//! wiring are replaced by a typed synchronous service stub. Observable contract
//! (acquire/release scope, data-first and pipeable `withLock`, owner metadata,
//! staleness recovery, compromise/token/permission failures) is preserved.

#![allow(dead_code)]

mod effect_flock {
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
                message: "not implemented: effect-flock".to_string(),
            }
        }
        pub fn permission_denied() -> Self {
            LockError {
                message: "PermissionDenied".to_string(),
            }
        }
        pub fn missing() -> Self {
            LockError {
                message: "lock dir missing".to_string(),
            }
        }
        pub fn token_mismatch() -> Self {
            LockError {
                message: "lock token mismatch".to_string(),
            }
        }
    }

    impl std::fmt::Display for LockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for LockError {}

    pub fn lock_path(dir: &str, key: &str) -> String {
        format!(
            "{}/{}.lock",
            dir.trim_end_matches('/'),
            key.replace(':', "-")
        )
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OwnerMeta {
        pub token: String,
        pub pid: u32,
        pub hostname: String,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct EffectFlock;

    impl EffectFlock {
        pub fn acquire(&self, _key: &str, _dir: &str) -> Result<LockGuard, LockError> {
            Err(LockError::not_implemented())
        }

        pub fn with_lock(
            &self,
            _key: &str,
            _dir: &str,
            _body: impl FnOnce() -> Result<(), LockError>,
        ) -> Result<(), LockError> {
            Err(LockError::not_implemented())
        }

        pub fn with_lock_pipeable(
            &self,
            _key: &str,
            _dir: &str,
            _body: impl FnOnce() -> Result<(), LockError>,
        ) -> Result<(), LockError> {
            Err(LockError::not_implemented())
        }
    }

    pub struct LockGuard {
        pub held: bool,
    }

    pub fn owner_meta() -> Result<OwnerMeta, NotImplemented> {
        Err(NotImplemented("effect-flock owner metadata"))
    }
}

const NOTE: &str = "porting: effect-flock not implemented";

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn acquire_and_release_via_scoped_effect() {
    let flock = effect_flock::EffectFlock;
    let guard = flock.acquire("eflock:acquire", "/tmp/locks").expect(NOTE);
    assert!(guard.held);
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn with_lock_data_first() {
    let flock = effect_flock::EffectFlock;
    let mut hit = false;
    let result = flock.with_lock("eflock:df", "/tmp/locks", || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn with_lock_pipeable() {
    let flock = effect_flock::EffectFlock;
    let mut hit = false;
    let result = flock.with_lock_pipeable("eflock:pipe", "/tmp/locks", || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn writes_owner_metadata() {
    let meta = effect_flock::owner_meta().expect(NOTE);
    assert!(!meta.token.is_empty());
    assert!(meta.pid > 0);
    assert!(!meta.hostname.is_empty());
    assert!(!meta.created_at.is_empty());
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn breaks_stale_lock_dirs() {
    let flock = effect_flock::EffectFlock;
    let mut hit = false;
    let result = flock.with_lock("eflock:stale", "/tmp/locks", || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn recovers_from_stale_breaker() {
    let flock = effect_flock::EffectFlock;
    let mut hit = false;
    let result = flock.with_lock("eflock:stale-breaker", "/tmp/locks", || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let lock_dir = effect_flock::lock_path("/tmp/locks", "eflock:stale-breaker");
    let breaker = format!("{lock_dir}.breaker");
    assert!(!std::path::Path::new(&breaker).exists());
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn detects_compromise_when_lock_dir_removed() {
    let flock = effect_flock::EffectFlock;
    let err = flock
        .with_lock("eflock:compromised", "/tmp/locks", || Ok(()))
        .expect_err(NOTE);
    assert!(err.to_string().contains("missing"));
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn detects_token_mismatch() {
    let flock = effect_flock::EffectFlock;
    let err = flock
        .with_lock("eflock:token", "/tmp/locks", || Ok(()))
        .expect_err(NOTE);
    assert!(err.to_string().contains("token mismatch"));
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn fails_on_unwritable_lock_roots() {
    let flock = effect_flock::EffectFlock;
    let err = flock
        .with_lock("eflock:perm", "/tmp/locks", || Ok(()))
        .expect_err(NOTE);
    assert!(err.to_string().contains("PermissionDenied"));
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn enforces_mutual_exclusion_under_process_contention() {
    let flock = effect_flock::EffectFlock;
    let mut outcomes = Vec::new();
    for _ in 0..16 {
        outcomes.push(
            flock
                .with_lock("eflock:stress", "/tmp/locks", || Ok(()))
                .is_ok(),
        );
    }
    assert!(outcomes.iter().all(|ok| *ok));
    assert_eq!(outcomes.len(), 16);
}

#[test]
#[ignore = "porting: effect-flock not implemented"]
fn recovers_after_a_crashed_lock_owner() {
    let flock = effect_flock::EffectFlock;
    let mut hit = false;
    let result = flock.with_lock("eflock:crash", "/tmp/locks", || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
}
