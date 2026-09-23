//! Port of packages/app/e2e/performance/unit/session-timeline-stream-probe.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Progress {
    index: i64,
    phase: String,
}

// Local stub (fast wave): real module lands later.
fn stream_progress(_output: &str) -> Progress {
    Progress {
        index: 0,
        phase: String::new(),
    }
}

fn stream_chunk(index: i64, _total: i64) -> String {
    format!("before stream-{index} after stream-{}", index + 1)
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-timeline-stream-probe not implemented"]
fn classifies_emitted_stream_markers_using_the_fixture_cycle() {
    assert_eq!(
        stream_progress("before stream-17 after stream-18"),
        Progress {
            index: 18,
            phase: "boundary".into()
        }
    );
    assert_eq!(
        stream_progress("before stream-18 after stream-19"),
        Progress {
            index: 19,
            phase: "stream".into()
        }
    );
    assert_eq!(
        stream_progress("benchmark-complete stream-36"),
        Progress {
            index: 36,
            phase: "complete".into()
        }
    );
    assert_eq!(
        stream_progress("no marker"),
        Progress {
            index: -1,
            phase: "unknown".into()
        }
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit/session-timeline-stream-probe not implemented"]
fn emits_progress_markers_at_fixture_boundaries() {
    assert_eq!(
        stream_progress(&stream_chunk(18, 160)),
        Progress {
            index: 18,
            phase: "boundary".into()
        }
    );
}
