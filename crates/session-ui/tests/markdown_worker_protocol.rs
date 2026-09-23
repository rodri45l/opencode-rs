//! Port of packages/session-ui/src/components/markdown-worker-protocol.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-worker-protocol.ts; see docs/TEST-PORT.md.

use opencode_session_ui::markdown_worker_protocol::{
    apply_markdown_worker_response, markdown_block_key, should_release_markdown_worker_state,
    HighlightResponse, WorkerState,
};

fn token(content: &str) -> (String, String) {
    (content.to_string(), String::new())
}

fn response(
    id: u64,
    reset: bool,
    stable: Vec<(String, String)>,
    unstable: Vec<(String, String)>,
) -> HighlightResponse {
    HighlightResponse {
        id,
        key: "code".to_string(),
        language: "typescript".to_string(),
        reset,
        stable,
        unstable,
    }
}

#[test]
fn accumulates_stable_worker_tokens_and_replaces_the_unstable_tail() {
    let first = apply_markdown_worker_response(
        None,
        response(1, true, vec![token("one\n")], vec![token("tw")]),
    );
    let second = apply_markdown_worker_response(
        Some(first),
        response(2, false, vec![token("two\n")], vec![token("three")]),
    );

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
fn increments_generation_only_when_the_worker_resets_token_identity() {
    let first =
        apply_markdown_worker_response(None, response(1, true, vec![token("const")], vec![]));
    let append = apply_markdown_worker_response(
        Some(first.clone()),
        response(2, false, vec![token(" x")], vec![]),
    );
    let replacement = apply_markdown_worker_response(
        Some(append.clone()),
        response(3, true, vec![token("let y")], vec![]),
    );

    assert_eq!(
        [first.generation, append.generation, replacement.generation],
        [1, 1, 2]
    );
}

#[test]
fn ignores_stale_worker_responses_and_resets_replacement_streams() {
    let current = WorkerState {
        id: 2,
        generation: 1,
        language: "typescript".to_string(),
        stable: vec![token("current")],
        unstable: Vec::new(),
    };
    assert_eq!(
        apply_markdown_worker_response(
            Some(current.clone()),
            response(1, false, vec![token("stale")], vec![])
        ),
        current
    );

    let replaced = apply_markdown_worker_response(
        Some(current),
        response(3, true, vec![token("replacement")], vec![]),
    );
    assert_eq!(
        replaced
            .stable
            .iter()
            .map(|item| item.0.as_str())
            .collect::<Vec<_>>(),
        vec!["replacement"]
    );
}

#[test]
fn releases_only_the_latest_completed_worker_state() {
    assert!(should_release_markdown_worker_state(true, 4, 4));
    assert!(!should_release_markdown_worker_state(true, 5, 4));
    assert!(!should_release_markdown_worker_state(false, 4, 4));
}

#[test]
fn prefixes_pending_and_dispatched_block_keys_with_the_component_owner() {
    assert_eq!(
        markdown_block_key("owner", Some("message"), 2, "code"),
        "owner:message:2:code"
    );
    assert_eq!(
        markdown_block_key("owner", None, 2, "code"),
        "owner:block:2"
    );
}
