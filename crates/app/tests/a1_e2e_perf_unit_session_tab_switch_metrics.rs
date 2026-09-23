//! Port of packages/app/e2e/performance/unit/session-tab-switch-metrics.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Sample {
    observed_at_ms: i64,
    destination: Vec<String>,
    source: Vec<String>,
    has_visible_rows: bool,
    last: bool,
    required_part_visible: Option<bool>,
    bottom_anchor_required: Option<bool>,
    bottom_error_px: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Classification {
    blank_samples: usize,
    source_samples: usize,
    unknown_samples: usize,
    first_destination_observed_ms: Option<i64>,
    stable_observed_ms: Option<i64>,
    first_correct_observed_ms: Option<i64>,
}

// Local stub (fast wave): real module lands later.
fn classify_session_switch(_samples: &[Sample]) -> Classification {
    Classification {
        blank_samples: 0,
        source_samples: 0,
        unknown_samples: 0,
        first_destination_observed_ms: None,
        stable_observed_ms: None,
        first_correct_observed_ms: None,
    }
}

fn sample(
    observed_at_ms: i64,
    destination: &[&str],
    source: &[&str],
    has_visible_rows: bool,
    last: bool,
) -> Sample {
    Sample {
        observed_at_ms,
        destination: destination.iter().map(|s| (*s).to_string()).collect(),
        source: source.iter().map(|s| (*s).to_string()).collect(),
        has_visible_rows,
        last,
        required_part_visible: None,
        bottom_anchor_required: None,
        bottom_error_px: None,
    }
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-switch-metrics not implemented"]
fn counts_source_and_blank_samples_before_the_destination_is_observed() {
    let result = classify_session_switch(&[
        sample(16, &[], &["source"], true, false),
        sample(32, &[], &[], false, false),
        sample(48, &["destination"], &[], true, true),
        sample(64, &["destination"], &[], true, true),
        sample(80, &["destination"], &[], true, true),
    ]);
    assert_eq!(result.blank_samples, 1);
    assert_eq!(result.source_samples, 1);
    assert_eq!(result.unknown_samples, 0);
    assert_eq!(result.first_destination_observed_ms, Some(48));
    assert_eq!(result.stable_observed_ms, Some(80));
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-switch-metrics not implemented"]
fn does_not_classify_mixed_source_and_destination_content_as_correct() {
    let result = classify_session_switch(&[
        sample(16, &["destination"], &["source"], true, true),
        sample(32, &["destination"], &[], true, true),
        sample(48, &["destination"], &[], true, true),
        sample(64, &["destination"], &[], true, true),
    ]);
    assert_eq!(result.first_correct_observed_ms, Some(32));
    assert_eq!(result.stable_observed_ms, Some(64));
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-switch-metrics not implemented"]
fn reports_missing_correctness_without_throwing() {
    let result = classify_session_switch(&[sample(16, &["destination"], &["source"], true, true)]);
    assert_eq!(result.first_destination_observed_ms, Some(16));
    assert_eq!(result.first_correct_observed_ms, None);
    assert_eq!(result.stable_observed_ms, None);
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-switch-metrics not implemented"]
fn requires_an_explicitly_tracked_part_to_be_visible() {
    let mut s = sample(16, &["destination"], &[], true, true);
    s.required_part_visible = Some(false);
    s.bottom_error_px = Some(0);
    assert_eq!(
        classify_session_switch(&[s]).first_correct_observed_ms,
        None
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-switch-metrics not implemented"]
fn can_measure_content_correctness_without_requiring_a_bottom_anchor() {
    let mut s = sample(16, &["destination"], &[], true, true);
    s.required_part_visible = Some(true);
    s.bottom_anchor_required = Some(false);
    assert_eq!(
        classify_session_switch(&[s]).first_correct_observed_ms,
        Some(16)
    );
}
