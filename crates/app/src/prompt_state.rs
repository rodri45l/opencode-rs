//! Prompt state initialisation (port of packages/app/src/context/prompt-state.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct PromptText {
    pub part_type: String,
    pub content: String,
    pub start: i64,
    pub end: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub provider_id: String,
    pub model_id: String,
    pub variant: Option<String>,
}

#[derive(Default)]
pub struct PromptState {
    pub current: Vec<PromptText>,
    pub cursor: Option<i64>,
    pub model: Option<Model>,
}

pub fn default_prompt() -> Vec<PromptText> {
    Vec::new()
}

pub fn create_prompt_state(prompt: Option<&str>, model: Option<&Model>) -> PromptState {
    match prompt {
        None => PromptState {
            current: default_prompt(),
            cursor: None,
            model: model.cloned(),
        },
        Some(text) => PromptState {
            current: vec![PromptText {
                part_type: "text".to_string(),
                content: text.to_string(),
                start: 0,
                end: text.chars().count() as i64,
            }],
            cursor: Some(text.chars().count() as i64),
            model: model.cloned(),
        },
    }
}
