#![allow(dead_code)]

//! Port of packages/opencode/test/session/tools.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a running tool's `time.start` is preserved across repeated
//! metadata updates — two `ctx.metadata` calls must not reset the start time or
//! flip the part out of `running`. The reference wires this through the live
//! `SessionTools.resolve` effect; the pure state contract is re-derived here.
//! Stubs are local per the fast-wave protocol.

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ToolStatus {
    Running { start: f64 },
    Completed,
}

fn apply_metadata_updates(
    _start: f64,
    _outputs: &[&str],
) -> Result<(Vec<f64>, ToolStatus), S2Error> {
    Err(S2Error::NotImplemented("SessionTools.metadata"))
}

#[test]
#[ignore = "porting: SessionTools metadata updates not implemented"]
fn preserves_running_tool_start_time_across_metadata_updates() {
    let (observed, final_status) =
        apply_metadata_updates(100.0, &["first", "second"]).expect("metadata updates");

    assert_eq!(observed, vec![100.0, 100.0]);
    assert_eq!(final_status, ToolStatus::Running { start: 100.0 });
}
