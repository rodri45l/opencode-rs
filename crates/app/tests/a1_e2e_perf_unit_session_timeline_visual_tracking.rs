//! Port of packages/app/e2e/performance/unit/session-timeline-visual-tracking.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq)]
struct RawShift {
    start_time: i64,
    value: f64,
    had_recent_input: bool,
}

// Local stubs (fast wave): real module lands later.
fn layout_shift_value(_shift: RawShift, _window_start: i64) -> Option<f64> {
    None
}

fn remove_visible_row(_visible: &mut BTreeSet<String>, _row: &str) -> bool {
    false
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-timeline-visual-tracking not implemented"]
fn excludes_layout_shifts_before_the_probe_window_and_recent_input() {
    assert_eq!(
        layout_shift_value(
            RawShift {
                start_time: 9,
                value: 0.1,
                had_recent_input: false
            },
            10
        ),
        None
    );
    assert_eq!(
        layout_shift_value(
            RawShift {
                start_time: 10,
                value: 0.2,
                had_recent_input: true
            },
            10
        ),
        None
    );
    assert_eq!(
        layout_shift_value(
            RawShift {
                start_time: 11,
                value: 0.3,
                had_recent_input: false
            },
            10
        ),
        Some(0.3)
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-timeline-visual-tracking not implemented"]
fn classifies_removed_rows_from_their_last_painted_visibility() {
    let row = "row";
    let mut visible = BTreeSet::new();
    visible.insert(row.to_string());
    assert!(remove_visible_row(&mut visible, row));
    assert!(!remove_visible_row(&mut visible, row));
}
