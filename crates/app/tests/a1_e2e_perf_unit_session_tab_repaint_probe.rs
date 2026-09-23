//! Port of packages/app/e2e/performance/unit/session-tab-repaint-probe.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Shift {
    occurred_at_ms: i64,
    value: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct RawShift {
    start_time: i64,
    value: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct RepaintState {
    scroll_top: i64,
    rows: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct RawSample {
    observed_at_ms: i64,
    state: RepaintState,
    destination: Vec<String>,
    source: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct CompressedGroup {
    observed_at_ms: Vec<i64>,
    state: RepaintState,
}

#[derive(Clone, Debug, PartialEq)]
struct CompressedTrace {
    samples: Vec<CompressedGroup>,
    mutations: Vec<String>,
    shifts: Vec<Shift>,
}

// Local stubs (fast wave): real module lands later.
fn compress_cached_repaint_trace(
    _samples: Vec<RawSample>,
    _mutations: Vec<String>,
    _shifts: Vec<Shift>,
) -> CompressedTrace {
    CompressedTrace {
        samples: Vec::new(),
        mutations: Vec::new(),
        shifts: Vec::new(),
    }
}

fn layout_shift_sample(_shift: RawShift, _window_start: i64) -> Option<Shift> {
    None
}

fn state(scroll_top: i64) -> RepaintState {
    RepaintState {
        scroll_top,
        rows: vec!["row".into()],
    }
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-repaint-probe not implemented"]
fn compresses_repeated_repaint_states_without_losing_frame_samples() {
    let trace = compress_cached_repaint_trace(
        vec![
            RawSample {
                observed_at_ms: 16,
                state: state(10),
                destination: vec!["target".into()],
                source: Vec::new(),
            },
            RawSample {
                observed_at_ms: 32,
                state: state(10),
                destination: vec!["target".into()],
                source: Vec::new(),
            },
            RawSample {
                observed_at_ms: 48,
                state: state(11),
                destination: vec!["target".into()],
                source: Vec::new(),
            },
        ],
        vec!["mutation".into()],
        vec![Shift {
            occurred_at_ms: 24,
            value: 0.1,
        }],
    );

    let samples: Vec<i64> = trace
        .samples
        .iter()
        .flat_map(|group| group.observed_at_ms.clone())
        .collect();
    assert_eq!(samples, vec![16, 32, 48]);
    assert_eq!(trace.mutations, vec!["mutation".to_string()]);
    assert_eq!(
        trace.shifts,
        vec![Shift {
            occurred_at_ms: 24,
            value: 0.1
        }]
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-tab-repaint-probe not implemented"]
fn records_layout_shifts_at_occurrence_time_within_the_probe_window() {
    assert_eq!(
        layout_shift_sample(
            RawShift {
                start_time: 99,
                value: 0.1
            },
            100
        ),
        None
    );
    assert_eq!(
        layout_shift_sample(
            RawShift {
                start_time: 124,
                value: 0.2
            },
            100
        ),
        Some(Shift {
            occurred_at_ms: 24,
            value: 0.2
        })
    );
}
