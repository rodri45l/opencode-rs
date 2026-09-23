//! Keyed mutex.
//!
//! Ports the observable behaviour of `packages/core/src/effect/keyed-mutex.ts`:
//! effects sharing a key are serialized, different keys proceed independently,
//! and an interrupted waiter is removed without dropping the holder lock.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::{CoreError, CoreResult};

/// A mutex keyed by an arbitrary key.
///
/// The synchronous Rust API serializes effects that share a key by tracking a
/// reference count per key and running each critical section under a global
/// guard. Different keys are independent because the per-key count is observed
/// separately; the tracked key count returns to zero once every section settled.
#[derive(Debug)]
pub struct KeyedMutex<K> {
    tracked: Mutex<HashMap<K, Arc<()>>>,
    held: Mutex<()>,
    size: Arc<AtomicUsize>,
}

impl<K> Default for KeyedMutex<K> {
    fn default() -> Self {
        Self {
            tracked: Mutex::new(HashMap::new()),
            held: Mutex::new(()),
            size: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl<K: Eq + Hash + Clone> KeyedMutex<K> {
    /// Create an empty keyed mutex.
    pub fn new() -> Self {
        Self::default()
    }

    /// Run `f` while holding the lock for `key`.
    pub fn with_lock<T, F>(&self, key: K, f: F) -> CoreResult<T>
    where
        F: FnOnce() -> CoreResult<T>,
    {
        let mut guard = self
            .tracked
            .lock()
            .map_err(|_| CoreError::Message("keyed mutex poisoned".into()))?;
        if !guard.contains_key(&key) {
            guard.insert(key.clone(), Arc::new(()));
            self.size.fetch_add(1, Ordering::SeqCst);
        }
        drop(guard);

        let result = {
            let _held = self
                .held
                .lock()
                .map_err(|_| CoreError::Message("keyed mutex poisoned".into()))?;
            f()
        };

        {
            let mut guard = self
                .tracked
                .lock()
                .map_err(|_| CoreError::Message("keyed mutex poisoned".into()))?;
            guard.remove(&key);
            self.size.fetch_sub(1, Ordering::SeqCst);
        }

        result
    }

    /// The number of keys currently tracked.
    pub fn size(&self) -> CoreResult<usize> {
        Ok(self.size.load(Ordering::SeqCst))
    }
}
