//! Tool display metadata projection.
//!
//! Port of packages/tui/src/util/tool-display.ts (upstream 18ef3cc).

use serde_json::Value;

/// The human label for a web-search provider.
pub fn web_search_provider_label(provider: Option<&Value>) -> &'static str {
    match provider.and_then(Value::as_str) {
        Some("parallel") => "Parallel Web Search",
        Some("exa") => "Exa Web Search",
        _ => "Web Search",
    }
}

/// The structured metadata to display for a tool state, or an empty object.
pub fn tool_display_metadata(state: Option<&Value>) -> Value {
    let Some(state) = state else {
        return Value::Object(Default::default());
    };
    let Some(object) = state.as_object() else {
        return Value::Object(Default::default());
    };
    if object.get("status").and_then(Value::as_str) == Some("pending") {
        return Value::Object(Default::default());
    }
    match object.get("structured") {
        Some(Value::Object(map)) => Value::Object(map.clone()),
        _ => Value::Object(Default::default()),
    }
}
