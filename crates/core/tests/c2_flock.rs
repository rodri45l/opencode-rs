//! Port of packages/core/test/util/flock.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `EffectFlock`/`LayerNode` runtime is replaced by the
//! real [`opencode_core::flock`] file lock. Lock path derivation, staleness,
//! owner metadata, error-message contract, mutual exclusion, stale recovery,
//! compromise and permission failures are exercised against the filesystem.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use opencode_core::flock;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn root(tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "opencode-flock-{tag}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("root");
    dir.to_string_lossy().into_owned()
}

fn age(path: &Path) {
    let file = std::fs::File::open(path).expect("open");
    let past = SystemTime::now() - Duration::from_secs(30);
    file.set_times(std::fs::FileTimes::new().set_modified(past))
        .expect("set_times");
}

fn hold(lock_dir: &str) {
    std::fs::create_dir_all(lock_dir).expect("lock dir");
    std::fs::write(
        flock::meta_path(lock_dir),
        r#"{"token":"held","pid":1,"hostname":"h","createdAt":"0"}"#,
    )
    .expect("meta");
    std::fs::write(flock::heartbeat_path(lock_dir), "held").expect("heartbeat");
}

#[test]
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
fn staleness_is_strictly_greater_than_the_threshold() {
    assert!(!flock::is_stale(199, 200));
    assert!(!flock::is_stale(200, 200));
    assert!(flock::is_stale(201, 200));
}

#[test]
fn timeout_error_message_contract() {
    assert!(flock::LockError::timed_out()
        .to_string()
        .contains("Timed out waiting for lock"));
}

#[test]
fn token_mismatch_and_compromise_message_contract() {
    assert!(flock::LockError::token_mismatch()
        .to_string()
        .contains("token mismatch"));
    assert!(flock::LockError::compromised()
        .to_string()
        .contains("compromised"));
}

#[test]
fn enforces_mutual_exclusion_under_process_contention() {
    let dir = root("stress");
    let mut completed = Vec::new();
    for _ in 0..16 {
        let outcome = flock::with_lock("flock:stress", &dir, 1_000, 15_000, || Ok(()));
        completed.push(outcome.is_ok());
    }
    assert!(completed.iter().all(|ok| *ok));
    assert_eq!(completed.len(), 16);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn times_out_while_waiting_when_lock_is_still_healthy() {
    let dir = root("healthy");
    let key = "flock:timeout";
    let lock_dir = flock::lock_path(&dir, key);
    hold(&lock_dir);

    let err = flock::with_lock(key, &dir, 10_000, 500, || Ok(())).expect_err("timeout");
    assert!(err.to_string().contains("Timed out waiting for lock"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovers_after_a_crashed_lock_owner() {
    let dir = root("crash");
    let key = "flock:crash";
    let lock_dir = flock::lock_path(&dir, key);
    hold(&lock_dir);
    age(Path::new(&flock::heartbeat_path(&lock_dir)));

    let mut hit = false;
    let result = flock::with_lock(key, &dir, 500, 8_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn breaks_stale_lock_dirs_when_heartbeat_is_missing() {
    let dir = root("missing-heartbeat");
    let key = "flock:missing-heartbeat";
    let lock_dir = flock::lock_path(&dir, key);
    std::fs::create_dir_all(&lock_dir).expect("lock dir");
    age(Path::new(&lock_dir));

    let mut hit = false;
    let result = flock::with_lock(key, &dir, 200, 3_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovers_when_a_stale_breaker_claim_was_left_behind() {
    let dir = root("stale-breaker");
    let key = "flock:stale-breaker";
    let lock_dir = flock::lock_path(&dir, key);
    hold(&lock_dir);
    age(Path::new(&flock::heartbeat_path(&lock_dir)));

    let breaker = flock::breaker_path(&lock_dir);
    std::fs::write(&breaker, "stale").expect("breaker");
    age(Path::new(&breaker));

    let mut hit = false;
    let result = flock::with_lock(key, &dir, 200, 3_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    assert!(!Path::new(&breaker).exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fails_clearly_if_lock_dir_is_removed_while_held() {
    let dir = root("compromised");
    let key = "flock:compromised";
    let lock_dir = flock::lock_path(&dir, key);
    let target = PathBuf::from(&lock_dir);

    let err = flock::with_lock(key, &dir, 1_000, 3_000, || {
        std::fs::remove_dir_all(&target).expect("remove held lock");
        Ok(())
    })
    .expect_err("compromised");
    assert!(err.to_string().contains("compromised"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn writes_owner_metadata_while_lock_is_held() {
    let meta = flock::owner_meta();
    assert!(!meta.token.is_empty());
    assert!(meta.pid > 0);
    assert!(!meta.hostname.is_empty());
    assert!(!meta.created_at.is_empty());
}

#[test]
fn supports_acquire_with_await_using() {
    let dir = root("acquire");
    let guard = flock::acquire("flock:acquire", &dir, 1_000, 3_000).expect("acquire");
    assert!(guard.held);
    drop(guard);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refuses_token_mismatch_release_and_recovers_from_stale() {
    let dir = root("token");
    let key = "flock:token";
    let lock_dir = flock::lock_path(&dir, key);
    let meta = flock::meta_path(&lock_dir);

    let err = flock::with_lock(key, &dir, 500, 3_000, || {
        std::fs::write(
            &meta,
            r#"{"token":"tampered","pid":1,"hostname":"h","createdAt":"0"}"#,
        )
        .expect("tamper");
        Ok(())
    })
    .expect_err("token mismatch");
    assert!(err.to_string().contains("token mismatch"));

    let mut hit = false;
    let result = flock::with_lock(key, &dir, 500, 6_000, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fails_clearly_on_unwritable_lock_roots() {
    let parent = std::env::temp_dir().join(format!(
        "opencode-flock-perm-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&parent);
    let locks = parent.join("locks");
    std::fs::create_dir_all(&locks).expect("locks root");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&locks, std::fs::Permissions::from_mode(0o555)).expect("chmod");
    }
    let err = flock::with_lock(
        "flock:perm",
        locks.to_str().expect("utf8"),
        100,
        500,
        || Ok(()),
    )
    .expect_err("permission");
    let text = err.to_string();
    assert!(text.contains("EACCES") || text.contains("EPERM"), "{text}");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&locks, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&parent);
}
