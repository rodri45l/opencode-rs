//! Session context metrics (port of packages/app/src/components/session/session-context-metrics.ts).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub provider_id: String,
    pub model_id: String,
    pub cost: f64,
    pub tokens: Option<Tokens>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelInfo {
    pub name: String,
    pub context_limit: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Provider {
    pub id: String,
    pub name: Option<String>,
    pub models: BTreeMap<String, ModelInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Context {
    pub message: Message,
    pub total: i64,
    pub input: i64,
    pub usage: Option<f64>,
    pub provider_label: String,
    pub model_label: String,
    pub limit: Option<i64>,
}

fn token_total(tokens: &Tokens) -> i64 {
    tokens.input + tokens.output + tokens.reasoning + tokens.cache_read + tokens.cache_write
}

pub fn get_session_context(messages: &[Message], providers: &[Provider]) -> Option<Context> {
    let message = messages.iter().rev().find(|message| {
        message.role == "assistant"
            && message
                .tokens
                .as_ref()
                .map(|tokens| token_total(tokens) > 0)
                .unwrap_or(false)
    })?;
    let tokens = message.tokens.as_ref()?;

    let provider = providers.iter().find(|item| item.id == message.provider_id);
    let model = provider.and_then(|provider| provider.models.get(&message.model_id));
    let limit = model.map(|model| model.context_limit);
    let total = token_total(tokens);
    let usage = match limit {
        Some(limit) if limit != 0 => Some((total as f64 / limit as f64 * 100.0).round()),
        _ => None,
    };

    Some(Context {
        message: message.clone(),
        total,
        input: tokens.input,
        usage,
        provider_label: provider
            .and_then(|provider| provider.name.clone())
            .unwrap_or_else(|| message.provider_id.clone()),
        model_label: model
            .map(|model| model.name.clone())
            .unwrap_or_else(|| message.model_id.clone()),
        limit,
    })
}
