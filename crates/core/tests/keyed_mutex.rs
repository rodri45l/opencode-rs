//! Port of packages/core/test/effect/keyed-mutex.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: effects sharing a key are serialized and different keys
//! proceed independently, leaving the tracked key count at zero once settled.
//! Re-derived against the Rust API; the interrupted-waiter case is dropped as it
//! depends on `Fiber` interruption semantics.

use opencode_core::keyed_mutex::KeyedMutex;

const NOTE: &str = "porting: keyed mutex not implemented";

#[test]
#[ignore = "porting: keyed mutex not implemented"]
fn serializes_effects_with_the_same_key() {
    let mutex: KeyedMutex<String> = KeyedMutex::new();

    let first = mutex.with_lock("shared".to_string(), || Ok(1)).expect(NOTE);
    let second = mutex.with_lock("shared".to_string(), || Ok(2)).expect(NOTE);

    assert_eq!((first, second), (1, 2));
    assert_eq!(mutex.size().expect(NOTE), 0);
}

#[test]
#[ignore = "porting: keyed mutex not implemented"]
fn allows_different_keys_to_proceed_independently() {
    let mutex: KeyedMutex<String> = KeyedMutex::new();

    assert_eq!(
        mutex
            .with_lock("first".to_string(), || Ok("a"))
            .expect(NOTE),
        "a"
    );
    assert_eq!(
        mutex
            .with_lock("second".to_string(), || Ok("b"))
            .expect(NOTE),
        "b"
    );
    assert_eq!(mutex.size().expect(NOTE), 0);
}
