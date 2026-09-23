//! Port of packages/app/e2e/performance/unit/first-navigation-metrics.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sample {
    observed_at_ms: i64,
    source: bool,
    destination: bool,
    content: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Summary {
    samples: usize,
    first_destination_observed_ms: Option<i64>,
    stable_destination_observed_ms: Option<i64>,
    source_samples: usize,
    blank_samples: usize,
    unknown_samples: usize,
    destination_samples: usize,
}

// Local stub (fast wave): real module lands later.
fn summarize_first_navigation(_samples: &[Sample]) -> Summary {
    Summary {
        samples: 0,
        first_destination_observed_ms: None,
        stable_destination_observed_ms: None,
        source_samples: 0,
        blank_samples: 0,
        unknown_samples: 0,
        destination_samples: 0,
    }
}

fn sample(observed_at_ms: i64, source: bool, destination: bool, content: bool) -> Sample {
    Sample {
        observed_at_ms,
        source,
        destination,
        content,
    }
}

#[test]
#[ignore = "porting: e2e/performance/unit/first-navigation-metrics not implemented"]
fn reports_blank_frames_before_first_destination_and_stable_paint() {
    assert_eq!(
        summarize_first_navigation(&[
            sample(16, true, false, true),
            sample(32, false, false, false),
            sample(48, false, true, true),
            sample(64, false, true, true),
            sample(80, false, true, true),
        ]),
        Summary {
            samples: 5,
            first_destination_observed_ms: Some(48),
            stable_destination_observed_ms: Some(80),
            source_samples: 1,
            blank_samples: 1,
            unknown_samples: 0,
            destination_samples: 3,
        }
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit/first-navigation-metrics not implemented"]
fn does_not_report_stability_for_interrupted_destination_frames() {
    let summary = summarize_first_navigation(&[
        sample(16, false, true, true),
        sample(32, false, false, true),
        sample(48, false, true, true),
    ]);
    assert_eq!(summary.stable_destination_observed_ms, None);
}
