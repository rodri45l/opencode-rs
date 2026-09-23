//! Effect-style file locks.
//!
//! Ports the observable behaviour of `packages/core/src/util/effect-flock.ts`:
//! a scoped acquire/release, a data-first and a pipeable `withLock`, owner
//! metadata, staleness recovery, and compromise/token/permission failures. The
//! Effect `Scope`/`Layer` wiring is replaced by a synchronous service.

use std::path::PathBuf;

use crate::flock::{self, LockError as FlockError};

/// Error raised by the effect-style lock service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockError {
    /// Human-readable message.
    pub message: String,
}

impl LockError {
    fn new(message: impl Into<String>) -> Self {
        LockError {
            message: message.into(),
        }
    }

    /// A missing lock directory.
    pub fn missing() -> Self {
        LockError::new("lock dir missing")
    }

    /// A metadata token mismatch.
    pub fn token_mismatch() -> Self {
        LockError::new("lock token mismatch")
    }

    /// A permission failure.
    pub fn permission_denied() -> Self {
        LockError::new("PermissionDenied")
    }
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LockError {}

/// Owner metadata written while a lock is held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerMeta {
    /// Random lock token.
    pub token: String,
    /// Process id of the owner.
    pub pid: u32,
    /// Hostname of the owner.
    pub hostname: String,
    /// Creation timestamp.
    pub created_at: String,
}

/// The lock directory path for `key` under `dir`.
pub fn lock_path(dir: &str, key: &str) -> String {
    format!(
        "{}/{}.lock",
        dir.trim_end_matches('/'),
        key.replace(':', "-")
    )
}

/// Generate fresh owner metadata.
pub fn owner_meta() -> OwnerMeta {
    let meta = flock::owner_meta();
    OwnerMeta {
        token: meta.token,
        pid: meta.pid,
        hostname: meta.hostname,
        created_at: meta.created_at,
    }
}

/// A held lock. Releases on drop.
pub struct LockGuard {
    /// Whether the lock is currently held.
    pub held: bool,
    inner: flock::LockGuard,
}

impl LockGuard {
    /// Release the lock, validating that it is still ours.
    pub fn release(&mut self) -> Result<(), LockError> {
        self.held = false;
        self.inner.release().map_err(translate)
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// The effect-style lock service.
#[derive(Debug, Clone, Copy, Default)]
pub struct EffectFlock;

impl EffectFlock {
    /// Acquire the lock for `key`.
    pub fn acquire(&self, key: &str, dir: &str) -> Result<LockGuard, LockError> {
        let path = PathBuf::from(lock_path(dir, key));
        let inner = flock::acquire_at(path, 1_000, 5_000).map_err(translate)?;
        Ok(LockGuard { held: true, inner })
    }

    /// Run `body` while holding the lock (data-first form).
    pub fn with_lock(
        &self,
        key: &str,
        dir: &str,
        body: impl FnOnce() -> Result<(), LockError>,
    ) -> Result<(), LockError> {
        let mut guard = self.acquire(key, dir)?;
        let body_result = body();
        let release_result = guard.release();
        match release_result {
            Ok(()) => body_result,
            Err(error) => Err(error),
        }
    }

    /// Run `body` while holding the lock (pipeable form).
    pub fn with_lock_pipeable(
        &self,
        key: &str,
        dir: &str,
        body: impl FnOnce() -> Result<(), LockError>,
    ) -> Result<(), LockError> {
        self.with_lock(key, dir, body)
    }
}

fn translate(error: FlockError) -> LockError {
    if error.message.contains("token mismatch") {
        LockError::token_mismatch()
    } else if error.message.contains("EACCES") || error.message.contains("EPERM") {
        LockError::permission_denied()
    } else if error.message.contains("compromised") {
        LockError::missing()
    } else {
        LockError::new(error.message)
    }
}
