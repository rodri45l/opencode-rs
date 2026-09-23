//! Port of packages/app/src/context/prompt-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::prompt_state::{create_prompt_state, Model, PromptText};

fn default_prompt() -> Vec<PromptText> {
    Vec::new()
}

#[test]
fn initializes_prompt_text_cursor_and_model_together() {
    let model = Model {
        provider_id: "anthropic".into(),
        model_id: "claude".into(),
        variant: Some("high".into()),
    };
    let prompt = create_prompt_state(Some("hello"), Some(&model));
    assert_eq!(
        prompt.current,
        vec![PromptText {
            part_type: "text".into(),
            content: "hello".into(),
            start: 0,
            end: 5
        }]
    );
    assert_eq!(prompt.cursor, Some(5));
    assert_eq!(prompt.model, Some(model));
}

#[test]
fn uses_the_default_prompt_without_initial_values() {
    let prompt = create_prompt_state(None, None);
    assert_eq!(prompt.current, default_prompt());
    assert_eq!(prompt.cursor, None);
    assert_eq!(prompt.model, None);
}
