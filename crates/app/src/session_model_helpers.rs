//! Session model selection helpers
//! (port of packages/app/src/pages/session/session-model-helpers.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub provider_id: String,
    pub model_id: String,
    pub variant: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserMessage {
    pub agent: String,
    pub model: Model,
}

#[derive(Default)]
pub struct SessionState {
    pub restore_calls: Vec<UserMessage>,
    pub reset_calls: usize,
}

#[derive(Default)]
pub struct PromptModel {
    pub set_calls: Vec<Model>,
    pub variant_calls: Vec<Option<String>>,
    pub current: Option<Model>,
}

pub fn sync_session_model(session: &mut SessionState, message: &UserMessage) {
    session.restore_calls.push(message.clone());
}

pub fn reset_session_model(session: &mut SessionState) {
    session.reset_calls += 1;
}

pub fn sync_prompt_model(session_model: &Model, prompt: &mut PromptModel) {
    let next = session_model.clone();
    if let Some(current) = &prompt.current {
        if current.provider_id == next.provider_id
            && current.model_id == next.model_id
            && current.variant == next.variant
        {
            return;
        }
    }
    prompt.current = Some(next.clone());
    prompt.set_calls.push(next);
}

pub fn restore_prompt_model(session: &mut PromptModel, prompt: &PromptModel) -> bool {
    let model = match &prompt.current {
        Some(model) => model,
        None => return false,
    };
    if let Some(current) = &session.current {
        if current.provider_id == model.provider_id
            && current.model_id == model.model_id
            && current.variant == model.variant
        {
            return true;
        }
    }
    session.current = Some(Model {
        provider_id: model.provider_id.clone(),
        model_id: model.model_id.clone(),
        variant: None,
    });
    session.set_calls.push(Model {
        provider_id: model.provider_id.clone(),
        model_id: model.model_id.clone(),
        variant: None,
    });
    session.variant_calls.push(model.variant.clone());
    true
}
