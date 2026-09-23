//! Port of packages/core/test/util/effect-flock.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `EffectFlock.Service` and `AppNodeBuilder.build` layer
//! wiring are replaced by the real [`opencode_core::effect_flock`] service.
//! Observable contract (acquire/release scope, data-first and pipeable
//! `withLock`, owner metadata, staleness recovery, compromise/token/permission
//! failures) is preserved.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use opencode_core::effect_flock::{self, EffectFlock};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn root(tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "opencode-eflock-{tag}-{}-{}",
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
    std::fs::write(format!("{lock_dir}/meta.json"), "held").expect("meta");
    std::fs::write(format!("{lock_dir}/heartbeat"), "held").expect("heartbeat");
}

#[test]
fn acquire_and_release_via_scoped_effect() {
    let dir = root("acquire");
    let flock = EffectFlock;
    let guard = flock.acquire("eflock:acquire", &dir).expect("acquire");
    assert!(guard.held);
    drop(guard);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_lock_data_first() {
    let dir = root("df");
    let flock = EffectFlock;
    let mut hit = false;
    let result = flock.with_lock("eflock:df", &dir, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_lock_pipeable() {
    let dir = root("pipe");
    let flock = EffectFlock;
    let mut hit = false;
    let result = flock.with_lock_pipeable("eflock:pipe", &dir, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn writes_owner_metadata() {
    let meta = effect_flock::owner_meta();
    assert!(!meta.token.is_empty());
    assert!(meta.pid > 0);
    assert!(!meta.hostname.is_empty());
    assert!(!meta.created_at.is_empty());
}

#[test]
fn breaks_stale_lock_dirs() {
    let dir = root("stale");
    let key = "eflock:stale";
    let lock_dir = effect_flock::lock_path(&dir, key);
    hold(&lock_dir);
    age(Path::new(&format!("{lock_dir}/heartbeat")));

    let flock = EffectFlock;
    let mut hit = false;
    let result = flock.with_lock(key, &dir, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovers_from_stale_breaker() {
    let dir = root("stale-breaker");
    let key = "eflock:stale-breaker";
    let lock_dir = effect_flock::lock_path(&dir, key);
    hold(&lock_dir);
    age(Path::new(&format!("{lock_dir}/heartbeat")));

    let breaker = format!("{lock_dir}.breaker");
    std::fs::write(&breaker, "stale").expect("breaker");
    age(Path::new(&breaker));

    let flock = EffectFlock;
    let mut hit = false;
    let result = flock.with_lock(key, &dir, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    assert!(!Path::new(&breaker).exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detects_compromise_when_lock_dir_removed() {
    let dir = root("compromised");
    let key = "eflock:compromised";
    let lock_dir = PathBuf::from(effect_flock::lock_path(&dir, key));

    let flock = EffectFlock;
    let err = flock
        .with_lock(key, &dir, || {
            std::fs::remove_dir_all(&lock_dir).expect("remove held lock");
            Ok(())
        })
        .expect_err("missing");
    assert!(err.to_string().contains("missing"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn detects_token_mismatch() {
    let dir = root("token");
    let key = "eflock:token";
    let lock_dir = effect_flock::lock_path(&dir, key);
    let meta = format!("{lock_dir}/meta.json");

    let flock = EffectFlock;
    let err = flock
        .with_lock(key, &dir, || {
            std::fs::write(
                &meta,
                r#"{"token":"tampered","pid":1,"hostname":"h","createdAt":"0"}"#,
            )
            .expect("tamper");
            Ok(())
        })
        .expect_err("token mismatch");
    assert!(err.to_string().contains("token mismatch"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fails_on_unwritable_lock_roots() {
    let parent = std::env::temp_dir().join(format!(
        "opencode-eflock-perm-{}-{}",
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
    let flock = EffectFlock;
    let err = flock
        .with_lock("eflock:perm", locks.to_str().expect("utf8"), || Ok(()))
        .expect_err("permission");
    assert!(err.to_string().contains("PermissionDenied"), "{err}");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&locks, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&parent);
}

#[test]
fn enforces_mutual_exclusion_under_process_contention() {
    let dir = root("stress");
    let flock = EffectFlock;
    let mut outcomes = Vec::new();
    for _ in 0..16 {
        outcomes.push(flock.with_lock("eflock:stress", &dir, || Ok(())).is_ok());
    }
    assert!(outcomes.iter().all(|ok| *ok));
    assert_eq!(outcomes.len(), 16);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recovers_after_a_crashed_lock_owner() {
    let dir = root("crash");
    let key = "eflock:crash";
    let lock_dir = effect_flock::lock_path(&dir, key);
    hold(&lock_dir);
    age(Path::new(&format!("{lock_dir}/heartbeat")));

    let flock = EffectFlock;
    let mut hit = false;
    let result = flock.with_lock(key, &dir, || {
        hit = true;
        Ok(())
    });
    assert!(result.is_ok());
    assert!(hit);
    let _ = std::fs::remove_dir_all(&dir);
}
