//! Port of packages/session-ui/src/components/markdown-code-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-code-state.ts; see docs/TEST-PORT.md.

use opencode_session_ui::markdown_code_state::{should_reset_code_tokens, CodeTokenState};

fn previous() -> CodeTokenState {
    CodeTokenState {
        language: "ts".to_string(),
        generation: 1,
        stable_count: 3,
        unstable: Vec::new(),
        raw: "```ts\nconst x = 1\n```".to_string(),
    }
}

#[test]
fn resets_tokens_for_a_non_prefix_replacement_with_the_same_generation_and_token_count() {
    assert!(should_reset_code_tokens(
        &previous(),
        &CodeTokenState {
            language: "ts".to_string(),
            generation: 1,
            stable_count: 3,
            unstable: Vec::new(),
            raw: "```ts\nlet y = 2\n```".to_string(),
        },
    ));
}

#[test]
fn retains_tokens_for_an_append_only_streaming_update() {
    let previous = previous();
    assert!(!should_reset_code_tokens(
        &previous,
        &CodeTokenState {
            language: "ts".to_string(),
            generation: 1,
            stable_count: 4,
            unstable: Vec::new(),
            raw: format!("{}\nmore", previous.raw),
        },
    ));
}
