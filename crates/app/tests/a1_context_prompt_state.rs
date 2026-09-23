//! Port of packages/app/src/context/prompt-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct PromptText {
    part_type: String,
    content: String,
    start: i64,
    end: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Model {
    provider_id: String,
    model_id: String,
    variant: Option<String>,
}

#[derive(Default)]
struct PromptState {
    current: Vec<PromptText>,
    cursor: Option<i64>,
    model: Option<Model>,
}

// Local stub (fast wave): real module lands later.
fn create_prompt_state(_prompt: Option<&str>, _model: Option<&Model>) -> PromptState {
    PromptState::default()
}

fn default_prompt() -> Vec<PromptText> {
    Vec::new()
}

#[test]
#[ignore = "porting: context/prompt-state not implemented"]
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
#[ignore = "porting: context/prompt-state not implemented"]
fn uses_the_default_prompt_without_initial_values() {
    let prompt = create_prompt_state(None, None);
    assert_eq!(prompt.current, default_prompt());
    assert_eq!(prompt.cursor, None);
    assert_eq!(prompt.model, None);
}
