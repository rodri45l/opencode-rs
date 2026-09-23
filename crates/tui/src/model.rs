//! Model identifier parsing.
//!
//! Port of packages/tui/src/util/model.ts `parse` behaviour (upstream 18ef3cc).

/// A parsed provider/model pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModel {
    pub provider_id: String,
    pub model_id: String,
}

/// Split a `provider/org/model` identifier, keeping the nested remainder.
pub fn parse(value: &str) -> ParsedModel {
    match value.split_once('/') {
        Some((provider_id, model_id)) => ParsedModel {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
        },
        None => ParsedModel {
            provider_id: value.to_string(),
            model_id: String::new(),
        },
    }
}
