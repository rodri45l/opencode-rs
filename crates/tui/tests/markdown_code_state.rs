//! Port of packages/session-ui/src/components/markdown-code-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-code-state.ts:
//! token state resets on a non-prefix replacement within the same generation and
//! token count, and is retained for an append-only streaming update.
//! Red-first: the code-token reset rule is not implemented.

use opencode_tui::markdown_code_state::{should_reset_code_tokens, CodeTokenState, NOTE};

fn previous() -> CodeTokenState {
    CodeTokenState {
        language: "ts".into(),
        generation: 1,
        stable_count: 3,
        unstable: Vec::new(),
        raw: "```ts\nconst x = 1\n```".into(),
    }
}

#[test]
fn resets_tokens_for_a_non_prefix_replacement_with_the_same_generation_and_token_count() {
    let next = CodeTokenState {
        language: "ts".into(),
        generation: 1,
        stable_count: 3,
        unstable: Vec::new(),
        raw: "```ts\nlet y = 2\n```".into(),
    };

    assert!(should_reset_code_tokens(&previous(), &next).expect(NOTE));
}

#[test]
fn retains_tokens_for_an_append_only_streaming_update() {
    let mut next = previous();
    next.stable_count = 4;
    next.raw = format!("{}\nmore", previous().raw);

    assert!(!should_reset_code_tokens(&previous(), &next).expect(NOTE));
}
