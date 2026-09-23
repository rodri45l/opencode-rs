//! Port of packages/opencode/test/session/compaction.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `session.compaction.isOverflow` across context/output
//! limits, explicit input caps, cache-read accounting, the disabled-auto
//! switch, and the zero-context guard.
//!
//! Dropped (needs live services): `create`/`prune`/`process` cases require the
//! session database, event bridge, and a mock LLM stream.

use opencode_server::port::session_compaction::{is_overflow, CompactionModel, TokenCounts};

fn tokens(input: u64, output: u64, reasoning: u64, cache_read: u64) -> TokenCounts {
    TokenCounts {
        input,
        output,
        reasoning,
        cache_read,
        cache_write: 0,
    }
}

fn model(context: u64, input_limit: Option<u64>, output: u64) -> CompactionModel {
    CompactionModel {
        context,
        input_limit,
        output_limit: output,
    }
}

#[test]
fn returns_true_when_token_count_exceeds_usable_context() {
    let model = model(100_000, None, 32_000);
    assert!(is_overflow(tokens(75_000, 5_000, 0, 0), model, true));
}

#[test]
fn returns_false_when_token_count_within_usable_context() {
    let model = model(200_000, None, 32_000);
    assert!(!is_overflow(tokens(100_000, 10_000, 0, 0), model, true));
}

#[test]
fn includes_cache_read_in_token_count() {
    let model = model(100_000, None, 32_000);
    assert!(is_overflow(tokens(60_000, 10_000, 0, 10_000), model, true));
}

#[test]
fn respects_input_limit_for_input_caps() {
    let model = model(400_000, Some(272_000), 128_000);
    assert!(is_overflow(tokens(271_000, 1_000, 0, 2_000), model, true));
}

#[test]
fn returns_false_when_input_output_within_input_caps() {
    let model = model(400_000, Some(272_000), 128_000);
    assert!(!is_overflow(
        tokens(200_000, 20_000, 0, 10_000),
        model,
        true
    ));
}

#[test]
fn returns_false_when_output_within_limit_with_input_caps() {
    let model = model(200_000, Some(120_000), 10_000);
    assert!(!is_overflow(tokens(50_000, 9_999, 0, 0), model, true));
}

#[test]
fn reserves_headroom_when_input_limit_is_set() {
    // Regression: with an explicit input cap the next response still needs room.
    let model = model(200_000, Some(200_000), 32_000);
    assert!(is_overflow(tokens(180_000, 15_000, 0, 3_000), model, true));
}

#[test]
fn without_input_limit_same_count_triggers_compaction() {
    let model = model(200_000, None, 32_000);
    assert!(is_overflow(tokens(180_000, 15_000, 0, 3_000), model, true));
}

#[test]
fn input_limit_does_not_create_asymmetry() {
    let with_limit = model(200_000, Some(200_000), 32_000);
    let without_limit = model(200_000, None, 32_000);
    let usage = tokens(166_000, 10_000, 0, 5_000);
    assert!(is_overflow(usage, with_limit, true));
    assert!(is_overflow(usage, without_limit, true));
}

#[test]
fn returns_false_when_model_context_limit_is_zero() {
    let model = model(0, None, 32_000);
    assert!(!is_overflow(tokens(100_000, 10_000, 0, 0), model, true));
}

#[test]
fn returns_false_when_auto_is_disabled() {
    let model = model(100_000, None, 32_000);
    assert!(!is_overflow(tokens(75_000, 5_000, 0, 0), model, false));
}
