//! Port of packages/core/test/state.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: transforms layer over initial values, `reload` re-runs every
//! registered transform against fresh state, and disposing a transform removes it
//! (idempotently) and rebuilds. Re-derived: the Effect fiber/interruption and
//! automatic-batching cases are replaced by synchronous calls; the interrupted
//! commit and batch tests are noted as dropped.

use opencode_core::state::State;

const NOTE: &str = "porting: state not implemented";

#[test]
#[ignore = "porting: state not implemented"]
fn runs_the_registered_transforms() {
    let mut state = State::new();
    state
        .transform(|values| values.push("first".into()))
        .expect(NOTE);
    assert_eq!(state.values(), vec!["first".to_string()]);
}

#[test]
#[ignore = "porting: state not implemented"]
fn runs_effectful_transforms_during_every_reload() {
    let mut state = State::new();
    state
        .transform(|values| values.push("first".into()))
        .expect(NOTE);
    state.reload().expect(NOTE);
    assert_eq!(state.values(), vec!["first".to_string()]);
}

#[test]
#[ignore = "porting: state not implemented"]
fn disposes_a_transform_once_and_rebuilds_remaining_state() {
    let mut state = State::new();
    state
        .transform(|values| values.push("first".into()))
        .expect(NOTE);
    let registration = state
        .transform(|values| values.push("second".into()))
        .expect(NOTE);
    assert_eq!(
        state.values(),
        vec!["first".to_string(), "second".to_string()]
    );

    state.dispose(registration).expect(NOTE);
    assert_eq!(state.values(), vec!["first".to_string()]);

    state.dispose(registration).expect(NOTE);
    assert_eq!(state.values(), vec!["first".to_string()]);
}
