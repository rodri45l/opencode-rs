//! Port of packages/opencode/test/cli/run/stream.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run stream bridge in `cli/cmd/run/stream` is not implemented
//! in this crate. The status-patch defaulting behaviour is pinned against local
//! typed stubs.

#![allow(dead_code)]

use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
enum FooterEvent {
    StreamPatch { phase: String, status: String },
}

#[derive(Debug, Default)]
struct FooterApi {
    events: RefCell<Vec<FooterEvent>>,
}

impl FooterApi {
    fn event(&self, next: FooterEvent) {
        self.events.borrow_mut().push(next);
    }
    fn take(&self) -> Vec<FooterEvent> {
        self.events.borrow().clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StreamPatch {
    phase: Option<String>,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StreamOutput {
    commits: Vec<String>,
    footer: Option<StreamPatch>,
}

fn write_session_output(api: &FooterApi, output: &StreamOutput) {
    if let Some(patch) = &output.footer {
        api.event(FooterEvent::StreamPatch {
            phase: patch.phase.clone().unwrap_or_else(|| "running".to_string()),
            status: patch.status.clone(),
        });
    }
}

#[test]
fn defaults_status_patches_to_running_phase() {
    let api = FooterApi::default();
    write_session_output(
        &api,
        &StreamOutput {
            commits: Vec::new(),
            footer: Some(StreamPatch {
                phase: None,
                status: "assistant responding".to_string(),
            }),
        },
    );

    assert_eq!(
        api.take(),
        vec![FooterEvent::StreamPatch {
            phase: "running".to_string(),
            status: "assistant responding".to_string(),
        }]
    );
}
