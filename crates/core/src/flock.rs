//! Advisory file locks.
//!
//! Ports the observable behaviour of `packages/core/src/util/flock.ts`: a lock
//! is a directory derived from a stable hash of the key, guarded by an atomic
//! `mkdir`, with owner metadata and a heartbeat for staleness detection. Stale
//! locks (and stale breaker claims) are reclaimed; a lock removed while held or
//! whose metadata no longer matches is reported as compromised/token-mismatch.

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::json;

/// Error raised by the lock service.
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

    /// A timeout waiting for a healthy lock.
    pub fn timed_out() -> Self {
        LockError::new("Timed out waiting for lock")
    }

    /// The lock metadata token did not match the holder.
    pub fn token_mismatch() -> Self {
        LockError::new("lock token mismatch")
    }

    /// The lock directory disappeared while held.
    pub fn compromised() -> Self {
        LockError::new("lock dir was compromised (removed while held)")
    }

    fn from_io(error: &std::io::Error) -> Self {
        match error.raw_os_error() {
            Some(13) => LockError::new("EACCES: permission denied"),
            Some(1) => LockError::new("EPERM: operation not permitted"),
            _ => LockError::new(error.to_string()),
        }
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

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// FNV-1a 64-bit, rendered as lowercase hex. Deterministic across processes.
pub fn fast_hash(key: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The lock directory path for `key` under `dir`.
pub fn lock_path(dir: &str, key: &str) -> String {
    format!("{}/{}.lock", dir.trim_end_matches('/'), fast_hash(key))
}

/// The breaker claim path for a lock directory.
pub fn breaker_path(lock_dir: &str) -> String {
    format!("{lock_dir}.breaker")
}

/// The heartbeat file path inside a lock directory.
pub fn heartbeat_path(lock_dir: &str) -> String {
    format!("{lock_dir}/heartbeat")
}

/// The owner metadata path inside a lock directory.
pub fn meta_path(lock_dir: &str) -> String {
    format!("{lock_dir}/meta.json")
}

/// A lock is stale once its age strictly exceeds the configured threshold.
pub fn is_stale(age_ms: u64, stale_ms: u64) -> bool {
    age_ms > stale_ms
}

/// Generate fresh owner metadata.
pub fn owner_meta() -> OwnerMeta {
    OwnerMeta {
        token: new_token(),
        pid: std::process::id(),
        hostname: hostname(),
        created_at: now_millis().to_string(),
    }
}

/// A held lock. Releases on drop unless explicitly released.
pub struct LockGuard {
    /// Whether the lock is currently held.
    pub held: bool,
    token: String,
    lock_dir: PathBuf,
    released: bool,
}

impl LockGuard {
    /// Release the lock, validating that it is still ours.
    pub fn release(&mut self) -> Result<(), LockError> {
        if self.released {
            return Ok(());
        }
        self.released = true;
        self.held = false;
        if !self.lock_dir.exists() {
            return Err(LockError::compromised());
        }
        if let Some(meta) = read_meta(&self.lock_dir) {
            if meta.token != self.token {
                return Err(LockError::token_mismatch());
            }
        }
        fs::remove_dir_all(&self.lock_dir).map_err(|error| LockError::from_io(&error))?;
        Ok(())
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

/// Acquire the lock for `key`, reclaiming stale locks and waiting up to
/// `timeout_ms`.
pub fn acquire(
    key: &str,
    dir: &str,
    stale_ms: u64,
    timeout_ms: u64,
) -> Result<LockGuard, LockError> {
    acquire_at(PathBuf::from(lock_path(dir, key)), stale_ms, timeout_ms)
}

/// Acquire a lock at an explicit directory path.
pub(crate) fn acquire_at(
    lock_dir: PathBuf,
    stale_ms: u64,
    timeout_ms: u64,
) -> Result<LockGuard, LockError> {
    let start = Instant::now();
    loop {
        match fs::create_dir(&lock_dir) {
            Ok(()) => {
                let meta = owner_meta();
                write_meta(&lock_dir, &meta)?;
                return Ok(LockGuard {
                    held: true,
                    token: meta.token,
                    lock_dir,
                    released: false,
                });
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                if should_break(&lock_dir, stale_ms) {
                    break_lock(&lock_dir, stale_ms)?;
                    continue;
                }
            }
            Err(error) => return Err(LockError::from_io(&error)),
        }
        if start.elapsed() >= Duration::from_millis(timeout_ms) {
            return Err(LockError::timed_out());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Run `body` while holding the lock for `key`.
pub fn with_lock(
    key: &str,
    dir: &str,
    stale_ms: u64,
    timeout_ms: u64,
    body: impl FnOnce() -> Result<(), LockError>,
) -> Result<(), LockError> {
    let mut guard = acquire(key, dir, stale_ms, timeout_ms)?;
    let body_result = body();
    let release_result = guard.release();
    match release_result {
        Ok(()) => body_result,
        Err(error) => Err(error),
    }
}

fn should_break(lock_dir: &Path, stale_ms: u64) -> bool {
    match lock_age_ms(lock_dir) {
        Some(age) => is_stale(age, stale_ms),
        None => true,
    }
}

fn break_lock(lock_dir: &Path, stale_ms: u64) -> Result<(), LockError> {
    let breaker = PathBuf::from(format!("{}.breaker", lock_dir.display()));
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&breaker)
    {
        Ok(mut file) => {
            let _ = file.write_all(now_millis().to_string().as_bytes());
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            let stale = lock_age_ms(&breaker)
                .map(|age| is_stale(age, stale_ms))
                .unwrap_or(true);
            if stale {
                let _ = fs::remove_file(&breaker);
                let _ = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&breaker);
            } else {
                return Ok(());
            }
        }
        Err(error) => return Err(LockError::from_io(&error)),
    }
    let _ = fs::remove_dir_all(lock_dir);
    let _ = fs::remove_file(&breaker);
    Ok(())
}

fn lock_age_ms(path: &Path) -> Option<u64> {
    let heartbeat = PathBuf::from(heartbeat_path(&path.to_string_lossy()));
    let meta = PathBuf::from(meta_path(&path.to_string_lossy()));
    let target = if heartbeat.exists() {
        heartbeat
    } else if meta.exists() {
        meta
    } else {
        path.to_path_buf()
    };
    let modified = fs::metadata(&target).ok()?.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    Some(age.as_millis() as u64)
}

fn write_meta(lock_dir: &Path, meta: &OwnerMeta) -> Result<(), LockError> {
    let value = json!({
        "token": meta.token,
        "pid": meta.pid,
        "hostname": meta.hostname,
        "createdAt": meta.created_at,
    });
    fs::write(meta_path(&lock_dir.to_string_lossy()), value.to_string())
        .map_err(|error| LockError::from_io(&error))?;
    let heartbeat = heartbeat_path(&lock_dir.to_string_lossy());
    let mut file = fs::File::create(heartbeat).map_err(|error| LockError::from_io(&error))?;
    file.write_all(meta.created_at.as_bytes())
        .map_err(|error| LockError::from_io(&error))?;
    Ok(())
}

fn read_meta(lock_dir: &Path) -> Option<OwnerMeta> {
    let text = fs::read_to_string(meta_path(&lock_dir.to_string_lossy())).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(OwnerMeta {
        token: value.get("token")?.as_str()?.to_string(),
        pid: value.get("pid")?.as_u64()? as u32,
        hostname: value.get("hostname")?.as_str()?.to_string(),
        created_at: value.get("createdAt")?.as_str()?.to_string(),
    })
}

fn new_token() -> String {
    let nanos = now_millis();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{counter}", std::process::id())
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn hostname() -> String {
    if let Ok(name) = fs::read_to_string("/etc/hostname") {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string())
}
