//! Port of packages/app/src/pages/session/session-model-helpers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::session_model_helpers::{
    reset_session_model, restore_prompt_model, sync_prompt_model, sync_session_model, Model,
    PromptModel, SessionState, UserMessage,
};

fn message(model: Model) -> UserMessage {
    UserMessage {
        agent: "build".into(),
        model,
    }
}

#[test]
fn restores_the_last_message_through_session_state() {
    let mut session = SessionState::default();
    let msg = message(Model {
        provider_id: "anthropic".into(),
        model_id: "claude-sonnet-4".into(),
        variant: Some("high".into()),
    });
    sync_session_model(&mut session, &msg);
    assert_eq!(session.restore_calls, vec![msg]);
}

#[test]
fn reset_session_model_clears_draft_session_state() {
    let mut session = SessionState::default();
    reset_session_model(&mut session);
    assert_eq!(session.reset_calls, 1);
}

#[test]
fn sync_prompt_model_stores_the_effective_session_model_in_prompt_state() {
    let session_model = Model {
        provider_id: "anthropic".into(),
        model_id: "claude-sonnet-4".into(),
        variant: Some("high".into()),
    };
    let mut prompt = PromptModel::default();
    sync_prompt_model(&session_model, &mut prompt);
    assert_eq!(prompt.set_calls, vec![session_model]);
}

#[test]
fn sync_prompt_model_does_not_rewrite_an_unchanged_prompt_model() {
    let model = Model {
        provider_id: "anthropic".into(),
        model_id: "claude-sonnet-4".into(),
        variant: Some("high".into()),
    };
    let mut prompt = PromptModel {
        current: Some(model.clone()),
        ..PromptModel::default()
    };
    sync_prompt_model(&model, &mut prompt);
    assert!(prompt.set_calls.is_empty());
}

#[test]
fn restore_prompt_model_restores_the_persisted_prompt_model_into_session_selection() {
    let mut session = PromptModel::default();
    let prompt = PromptModel {
        current: Some(Model {
            provider_id: "anthropic".into(),
            model_id: "claude".into(),
            variant: Some("high".into()),
        }),
        ..PromptModel::default()
    };
    let restored = restore_prompt_model(&mut session, &prompt);
    assert!(restored);
    assert_eq!(
        session.set_calls,
        vec![Model {
            provider_id: "anthropic".into(),
            model_id: "claude".into(),
            variant: None
        }]
    );
    assert_eq!(session.variant_calls, vec![Some("high".to_string())]);
}

#[test]
fn restore_prompt_model_does_nothing_without_a_persisted_prompt_model() {
    let mut session = PromptModel::default();
    let prompt = PromptModel::default();
    let restored = restore_prompt_model(&mut session, &prompt);
    assert!(!restored);
    assert!(session.set_calls.is_empty());
    assert!(session.variant_calls.is_empty());
}
