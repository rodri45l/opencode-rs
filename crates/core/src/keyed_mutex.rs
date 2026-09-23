//! Keyed mutex.
//!
//! Ports the observable behaviour of `packages/core/src/effect/keyed-mutex.ts`:
//! effects sharing a key are serialized, different keys proceed independently,
//! and an interrupted waiter is removed without dropping the holder lock.

use std::marker::PhantomData;

use crate::{CoreError, CoreResult};

/// A mutex keyed by an arbitrary key.
#[derive(Debug)]
pub struct KeyedMutex<K> {
    marker: PhantomData<fn(K)>,
}

impl<K> Default for KeyedMutex<K> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<K> KeyedMutex<K> {
    /// Create an empty keyed mutex.
    pub fn new() -> Self {
        Self::default()
    }

    /// Run `f` while holding the lock for `key`.
    pub fn with_lock<T, F>(&self, _key: K, _f: F) -> CoreResult<T>
    where
        F: FnOnce() -> CoreResult<T>,
    {
        Err(CoreError::NotImplemented(
            "keyed_mutex::KeyedMutex::with_lock",
        ))
    }

    /// The number of keys currently tracked.
    pub fn size(&self) -> CoreResult<usize> {
        Err(CoreError::NotImplemented("keyed_mutex::KeyedMutex::size"))
    }
}
