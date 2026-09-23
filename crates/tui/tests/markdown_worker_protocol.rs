//! Port of packages/session-ui/src/components/markdown-worker-protocol.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-worker-protocol.ts:
//! stable worker tokens accumulate, the unstable tail is replaced, stale responses are
//! ignored, resets bump the generation, only the latest completed state is released, and
//! block keys are namespaced by the component owner.
//! Red-first: the worker protocol reducer is not implemented.

#[allow(dead_code)]
mod worker_protocol {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: session-ui markdown worker protocol not implemented";

    pub type Token = (String, String);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WorkerState {
        pub id: i64,
        pub generation: i64,
        pub language: String,
        pub stable: Vec<Token>,
        pub unstable: Vec<Token>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WorkerResponse {
        pub id: i64,
        pub key: String,
        pub language: String,
        pub reset: bool,
        pub stable: Vec<Token>,
        pub unstable: Vec<Token>,
    }

    pub fn apply_markdown_worker_response(
        _current: Option<&WorkerState>,
        _response: &WorkerResponse,
    ) -> PortResult<Option<WorkerState>> {
        Err(NotImplemented(NOTE))
    }

    pub fn should_release_markdown_worker_state(
        _released: bool,
        _current_id: i64,
        _completed_id: i64,
    ) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }

    pub fn markdown_block_key(
        _owner: &str,
        _key: Option<&str>,
        _index: i64,
        _kind: &str,
    ) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }
}

use worker_protocol::{
    apply_markdown_worker_response, markdown_block_key, should_release_markdown_worker_state,
    WorkerResponse, WorkerState, NOTE,
};

fn token(content: &str) -> (String, String) {
    (content.to_string(), String::new())
}

fn response(id: i64, reset: bool, stable: &[&str], unstable: &[&str]) -> WorkerResponse {
    WorkerResponse {
        id,
        key: "code".into(),
        language: "typescript".into(),
        reset,
        stable: stable.iter().map(|item| token(item)).collect(),
        unstable: unstable.iter().map(|item| token(item)).collect(),
    }
}

#[test]
#[ignore = "porting: session-ui markdown worker protocol not implemented"]
fn accumulates_stable_worker_tokens_and_replaces_the_unstable_tail() {
    let first = apply_markdown_worker_response(None, &response(1, true, &["one\n"], &["tw"]))
        .expect(NOTE)
        .expect("state");
    let second =
        apply_markdown_worker_response(Some(&first), &response(2, false, &["two\n"], &["three"]))
            .expect(NOTE)
            .expect("state");

    assert_eq!(
        second
            .stable
            .iter()
            .map(|item| item.0.as_str())
            .collect::<Vec<_>>(),
        vec!["one\n", "two\n"]
    );
    assert_eq!(
        second
            .unstable
            .iter()
            .map(|item| item.0.as_str())
            .collect::<Vec<_>>(),
        vec!["three"]
    );
    assert_eq!(second.language, "typescript");
}

#[test]
#[ignore = "porting: session-ui markdown worker protocol not implemented"]
fn increments_generation_only_when_the_worker_resets_token_identity() {
    let first = apply_markdown_worker_response(None, &response(1, true, &["const"], &[]))
        .expect(NOTE)
        .expect("state");
    let append = apply_markdown_worker_response(Some(&first), &response(2, false, &[" x"], &[]))
        .expect(NOTE)
        .expect("state");
    let replacement =
        apply_markdown_worker_response(Some(&append), &response(3, true, &["let y"], &[]))
            .expect(NOTE)
            .expect("state");

    assert_eq!(
        [first.generation, append.generation, replacement.generation],
        [1, 1, 2]
    );
}

#[test]
#[ignore = "porting: session-ui markdown worker protocol not implemented"]
fn ignores_stale_worker_responses_and_resets_replacement_streams() {
    let current = WorkerState {
        id: 2,
        generation: 1,
        language: "typescript".into(),
        stable: vec![token("current")],
        unstable: Vec::new(),
    };

    let stale =
        apply_markdown_worker_response(Some(&current), &response(1, false, &["stale"], &[]))
            .expect(NOTE);
    assert_eq!(stale.as_ref(), Some(&current));

    let replacement =
        apply_markdown_worker_response(Some(&current), &response(3, true, &["replacement"], &[]))
            .expect(NOTE)
            .expect("state");
    assert_eq!(
        replacement
            .stable
            .iter()
            .map(|item| item.0.as_str())
            .collect::<Vec<_>>(),
        vec!["replacement"]
    );
}

#[test]
#[ignore = "porting: session-ui markdown worker protocol not implemented"]
fn releases_only_the_latest_completed_worker_state() {
    assert!(should_release_markdown_worker_state(true, 4, 4).expect(NOTE));
    assert!(!should_release_markdown_worker_state(true, 5, 4).expect(NOTE));
    assert!(!should_release_markdown_worker_state(false, 4, 4).expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui markdown worker protocol not implemented"]
fn prefixes_pending_and_dispatched_block_keys_with_the_component_owner() {
    assert_eq!(
        markdown_block_key("owner", Some("message"), 2, "code").expect(NOTE),
        "owner:message:2:code"
    );
    assert_eq!(
        markdown_block_key("owner", None, 2, "code").expect(NOTE),
        "owner:block:2"
    );
}
