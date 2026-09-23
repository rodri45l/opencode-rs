//! Local model selection helpers.
//!
//! Port of packages/tui/src/context/local.tsx `parseModel`/`recentModels`
//! behaviour (upstream 18ef3cc).

/// A provider/model pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRef {
    pub provider_id: String,
    pub model_id: String,
}

/// Parse a model identifier containing slashes.
pub fn parse_model(model: &str) -> ModelRef {
    match model.split_once('/') {
        Some((provider_id, model_id)) => ModelRef {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        },
        None => ModelRef {
            provider_id: model.to_string(),
            model_id: String::new(),
        },
    }
}

/// Move a model to the front of the recents, deduplicate, and cap at 10.
pub fn recent_models(model: ModelRef, recent: &[ModelRef]) -> Vec<ModelRef> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in std::iter::once(&model).chain(recent.iter()) {
        let key = format!("{}/{}", item.provider_id, item.model_id);
        if seen.insert(key) {
            result.push(item.clone());
        }
    }
    result.truncate(10);
    result
}
