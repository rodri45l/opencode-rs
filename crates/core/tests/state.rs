//! Port of packages/core/test/state.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: transforms layer over initial values, `reload` re-runs every
//! registered transform against fresh state, and disposing a transform removes it
//! (idempotently) and rebuilds. Re-derived: the Effect fiber/interruption and
//! automatic-batching cases are replaced by synchronous calls; the interrupted
//! commit and batch tests are noted as dropped.

use opencode_core::state::State;

#[test]
fn runs_the_registered_transforms() {
    let mut state = State::new();
    state
        .transform(|values| values.push("first".into()))
        .unwrap();
    assert_eq!(state.values(), vec!["first".to_string()]);
}

#[test]
fn runs_effectful_transforms_during_every_reload() {
    let mut state = State::new();
    state
        .transform(|values| values.push("first".into()))
        .unwrap();
    state.reload().unwrap();
    assert_eq!(state.values(), vec!["first".to_string()]);
}

#[test]
fn disposes_a_transform_once_and_rebuilds_remaining_state() {
    let mut state = State::new();
    state
        .transform(|values| values.push("first".into()))
        .unwrap();
    let registration = state
        .transform(|values| values.push("second".into()))
        .unwrap();
    assert_eq!(
        state.values(),
        vec!["first".to_string(), "second".to_string()]
    );

    state.dispose(registration).unwrap();
    assert_eq!(state.values(), vec!["first".to_string()]);

    state.dispose(registration).unwrap();
    assert_eq!(state.values(), vec!["first".to_string()]);
}
